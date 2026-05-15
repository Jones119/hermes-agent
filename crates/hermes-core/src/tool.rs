
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
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;
    async fn execute(&self, arguments: serde_json::Value) -> Result<String>;
    
    fn to_definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().into(),
            description: self.description().into(),
            parameters: self.parameters(),
        }
    }
}

pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool + Send + Sync>>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn register<T: Tool + 'static>(&self, tool: T) {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name().into(), Arc::new(tool));
    }
    
    pub async fn execute_tool(&self, name: &str, arguments: serde_json::Value) -> Result<String> {
        let tool = {
            let tools = self.tools.read().await;
            tools.get(name)
                .ok_or_else(|| Error::ToolNotFound(name.into()))?
                .clone()
        };
        tool.execute(arguments).await
    }
    
    pub async fn list_tools(&self) -> Vec<ToolDefinition> {
        let tools = self.tools.read().await;
        tools.values()
            .map(|t| t.to_definition())
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
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
