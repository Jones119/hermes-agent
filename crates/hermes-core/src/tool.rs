
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub toolset: String,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;
    fn toolset(&self) -> &str {
        "builtin"
    }
    async fn execute(&self, arguments: serde_json::Value) -> Result<String>;

    fn to_definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().into(),
            description: self.description().into(),
            parameters: self.parameters(),
            toolset: self.toolset().into(),
        }
    }
}

pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, (Arc<dyn Tool + Send + Sync>, String)>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register<T: Tool + 'static>(&self, tool: T) {
        self.register_with_toolset(tool, "builtin").await;
    }

    pub async fn register_with_toolset<T: Tool + 'static>(&self, tool: T, toolset: &str) {
        let mut tools = self.tools.write().await;
        let name = tool.name().to_string();
        if let Some((_, existing_toolset)) = tools.get(&name) {
            if existing_toolset != toolset {
                tracing::warn!(
                    "Tool '{}' from toolset '{}' shadows existing toolset '{}'",
                    name, toolset, existing_toolset
                );
            }
        }
        tools.insert(name, (Arc::new(tool), toolset.to_string()));
    }

    pub async fn register_override<T: Tool + 'static>(&self, tool: T, toolset: &str) {
        let mut tools = self.tools.write().await;
        let name = tool.name().to_string();
        if let Some((_, existing_toolset)) = tools.get(&name) {
            tracing::info!(
                "Tool '{}' from toolset '{}' overriding existing toolset '{}' (override=True)",
                name, toolset, existing_toolset
            );
        }
        tools.insert(name, (Arc::new(tool), toolset.to_string()));
    }

    pub async fn deregister(&self, name: &str) -> bool {
        let mut tools = self.tools.write().await;
        tools.remove(name).is_some()
    }

    pub async fn execute_tool(&self, name: &str, arguments: serde_json::Value) -> Result<String> {
        let tool = {
            let tools = self.tools.read().await;
            tools.get(name)
                .ok_or_else(|| Error::ToolNotFound(name.into()))?
                .0
                .clone()
        };
        match tool.execute(arguments).await {
            Ok(result) => Ok(result),
            Err(e) => {
                let sanitized = sanitize_tool_error(&e.to_string());
                Err(Error::ToolExecution(sanitized))
            }
        }
    }

    pub async fn list_tools(&self) -> Vec<ToolDefinition> {
        let tools = self.tools.read().await;
        tools.values()
            .map(|(t, _)| t.to_definition())
            .collect()
    }

    pub async fn has_tool(&self, name: &str) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn sanitize_tool_error(error_msg: &str) -> String {
    let dangerous = ["```", "<![CDATA[", "||", "{{", "}}"];
    let mut result = error_msg.to_string();
    for pattern in dangerous {
        result = result.replace(pattern, "");
    }
    if result.len() > 2000 {
        result.truncate(2000);
        result.push_str("... [truncated]");
    }
    result
}

#[derive(Clone)]
pub struct EchoTool;

impl EchoTool {
    pub fn new() -> Self {
        EchoTool
    }
}

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echo back the input message"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "The message to echo"
                }
            },
            "required": ["message"]
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<String> {
        let message = arguments.get("message")
            .and_then(|m| m.as_str())
            .ok_or_else(|| Error::InvalidArguments("Missing 'message' parameter".into()))?;

        Ok(message.to_string())
    }
}
