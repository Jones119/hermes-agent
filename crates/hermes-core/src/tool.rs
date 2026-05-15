
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry {
            tools: HashMap::new(),
        }
    }
    
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }
    
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }
    
    pub fn list(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
    
    pub fn get_definitions(&self) -> Vec<crate::llm::ToolDefinition> {
        self.tools.values()
            .map(|t| t.to_definition())
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        ToolRegistry::new()
    }
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;
    
    fn to_definition(&self) -> crate::llm::ToolDefinition {
        crate::llm::ToolDefinition {
            r#type: "function".into(),
            function: crate::llm::FunctionDefinition {
                name: self.name().into(),
                description: self.description().into(),
                parameters: self.parameters(),
            },
        }
    }
    
    async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value>;
}
