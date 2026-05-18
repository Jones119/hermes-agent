
use crate::agent::Agent;
use crate::llm::ToolCall;
use crate::message_sanitization::sanitize_tool_arguments;
use tokio::time::{timeout, Duration};
use tracing::{info, warn};

const MAX_TOOL_WORKERS: usize = 8;
const DEFAULT_TOOL_TIMEOUT_SECS: u64 = 120;

pub struct ToolExecutionResult {
    pub tool_call_id: String,
    pub function_name: String,
    pub result: String,
    pub success: bool,
}

impl ToolExecutionResult {
    pub fn success(tool_call_id: String, function_name: String, result: String) -> Self {
        ToolExecutionResult {
            tool_call_id,
            function_name,
            result,
            success: true,
        }
    }

    pub fn failure(tool_call_id: String, function_name: String, error: String) -> Self {
        ToolExecutionResult {
            tool_call_id,
            function_name,
            result: error,
            success: false,
        }
    }
}

pub async fn execute_tool_calls_sequential(
    agent: &Agent,
    tool_calls: &[ToolCall],
) -> Vec<ToolExecutionResult> {
    let mut results = Vec::with_capacity(tool_calls.len());

    for tool_call in tool_calls {
        let result = execute_single_tool(agent, tool_call).await;
        results.push(result);
    }

    results
}

pub async fn execute_tool_calls_concurrent(
    agent: &Agent,
    tool_calls: &[ToolCall],
) -> Vec<ToolExecutionResult> {
    if tool_calls.is_empty() {
        return Vec::new();
    }

    if tool_calls.len() == 1 {
        return vec![execute_single_tool(agent, &tool_calls[0]).await];
    }

    let mut all_results = Vec::with_capacity(tool_calls.len());

    let futures: Vec<_> = tool_calls.iter().map(|tc| execute_single_tool(agent, tc)).collect();
    let batch_results = futures::future::join_all(futures).await;
    all_results.extend(batch_results);

    all_results
}

async fn execute_single_tool(agent: &Agent, tool_call: &ToolCall) -> ToolExecutionResult {
    let function_name = tool_call.function.name.clone();
    let tool_call_id = tool_call.id.clone();

    info!("Executing tool: {} (id={})", function_name, tool_call_id);

    let repaired_args = sanitize_tool_arguments(&tool_call.function.arguments, &function_name);

    let args: serde_json::Value = match serde_json::from_str(&repaired_args) {
        Ok(v) => v,
        Err(e) => {
            warn!("Failed to parse tool arguments for {}: {}", function_name, e);
            return ToolExecutionResult::failure(
                tool_call_id,
                function_name,
                format!("Invalid JSON arguments: {}", e),
            );
        }
    };

    let tool_timeout = Duration::from_secs(DEFAULT_TOOL_TIMEOUT_SECS);

    match timeout(tool_timeout, agent.tools().execute_tool(&function_name, args)).await {
        Ok(Ok(result)) => {
            info!("Tool {} executed successfully", function_name);
            ToolExecutionResult::success(tool_call_id, function_name, result)
        }
        Ok(Err(e)) => {
            warn!("Tool {} execution failed: {}", function_name, e);
            ToolExecutionResult::failure(tool_call_id, function_name, e.to_string())
        }
        Err(_) => {
            warn!("Tool {} timed out after {}s", function_name, DEFAULT_TOOL_TIMEOUT_SECS);
            ToolExecutionResult::failure(
                tool_call_id,
                function_name,
                format!("Tool execution timed out after {}s", DEFAULT_TOOL_TIMEOUT_SECS),
            )
        }
    }
}
