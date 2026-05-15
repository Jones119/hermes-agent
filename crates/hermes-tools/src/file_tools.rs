
use async_trait::async_trait;
use serde_json::json;
use tokio::fs;
use hermes_core::error::{Error, Result};
use hermes_core::tool::Tool;

#[derive(Clone)]
pub struct ReadFileTool;

impl ReadFileTool {
    pub fn new() -> Self {
        ReadFileTool
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }
    
    fn description(&self) -> &str {
        "Read the contents of a file"
    }
    
    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to read"
                },
                "offset": {
                    "type": "number",
                    "description": "Starting line offset (0-based)",
                    "default": 0
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum lines to read",
                    "default": 100
                }
            },
            "required": ["path"]
        })
    }
    
    async fn execute(&self, arguments: serde_json::Value) -> Result<String> {
        let path_str = arguments.get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| Error::InvalidArguments("Missing 'path' parameter".into()))?;
        
        let offset = arguments.get("offset")
            .and_then(|o| o.as_u64())
            .unwrap_or(0) as usize;
        
        let limit = arguments.get("limit")
            .and_then(|l| l.as_u64())
            .unwrap_or(100) as usize;
        
        let content = fs::read_to_string(path_str).await
            .map_err(|e| Error::ToolExecution(format!("Failed to read file: {}", e)))?;
        
        let lines: Vec<&str> = content.lines().collect();
        let start = std::cmp::min(offset, lines.len());
        let end = std::cmp::min(start + limit, lines.len());
        
        Ok(lines[start..end].join("\n"))
    }
}

#[derive(Clone)]
pub struct WriteFileTool;

impl WriteFileTool {
    pub fn new() -> Self {
        WriteFileTool
    }
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }
    
    fn description(&self) -> &str {
        "Write content to a file"
    }
    
    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                },
                "append": {
                    "type": "boolean",
                    "description": "Whether to append to the file instead of overwriting",
                    "default": false
                }
            },
            "required": ["path", "content"]
        })
    }
    
    async fn execute(&self, arguments: serde_json::Value) -> Result<String> {
        let path_str = arguments.get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| Error::InvalidArguments("Missing 'path' parameter".into()))?;
        
        let content = arguments.get("content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| Error::InvalidArguments("Missing 'content' parameter".into()))?;
        
        let append = arguments.get("append")
            .and_then(|a| a.as_bool())
            .unwrap_or(false);
        
        if append {
            use tokio::io::AsyncWriteExt;
            
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path_str)
                .await
                .map_err(|e| Error::ToolExecution(format!("Failed to open file: {}", e)))?;
            
            file.write_all(content.as_bytes()).await
                .map_err(|e| Error::ToolExecution(format!("Failed to write file: {}", e)))?;
        } else {
            fs::write(path_str, content).await
                .map_err(|e| Error::ToolExecution(format!("Failed to write file: {}", e)))?;
        }
        
        Ok("File written successfully".into())
    }
}
