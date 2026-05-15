
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use hermes_core::error::{Error, Result};
use hermes_core::tool::Tool;

#[derive(Clone)]
pub struct TerminalTool {
    pub sandbox: SandboxConfig,
}

#[derive(Clone, Default)]
pub struct SandboxConfig {
    pub allowed_commands: Vec<String>,
    pub timeout_secs: u64,
    pub work_dir: Option<String>,
}

impl TerminalTool {
    pub fn new() -> Self {
        TerminalTool {
            sandbox: SandboxConfig::default(),
        }
    }
    
    pub fn with_sandbox(mut self, sandbox: SandboxConfig) -> Self {
        self.sandbox = sandbox;
        self
    }
}

#[async_trait]
impl Tool for TerminalTool {
    fn name(&self) -> &str {
        "terminal"
    }
    
    fn description(&self) -> &str {
        "Execute shell commands in the terminal"
    }
    
    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                },
                "timeout": {
                    "type": "number",
                    "description": "Timeout in seconds",
                    "default": 30
                }
            },
            "required": ["command"]
        })
    }
    
    async fn execute(&self, arguments: serde_json::Value) -> Result<String> {
        let command = arguments.get("command")
            .and_then(|c| c.as_str())
            .ok_or_else(|| Error::InvalidArguments("Missing 'command' parameter".into()))?;
        
        let timeout = arguments.get("timeout")
            .and_then(|t| t.as_u64())
            .unwrap_or(30);
        
        let timeout = std::time::Duration::from_secs(timeout);
        
        let mut cmd = Command::new("bash");
        cmd.arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        if let Some(ref work_dir) = self.sandbox.work_dir {
            cmd.current_dir(work_dir);
        }
        
        let mut child = cmd.spawn()
            .map_err(|e| Error::ToolExecution(format!("Failed to spawn command: {}", e)))?;
        
        let stdout = child.stdout.take()
            .ok_or_else(|| Error::ToolExecution("Failed to capture stdout".into()))?;
        
        let stderr = child.stderr.take()
            .ok_or_else(|| Error::ToolExecution("Failed to capture stderr".into()))?;
        
        let output = tokio::time::timeout(timeout, async {
            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();
            
            let mut stdout_output = Vec::new();
            let mut stderr_output = Vec::new();
            
            loop {
                tokio::select! {
                    line = stdout_reader.next_line() => {
                        match line {
                            Ok(Some(line)) => {
                                stdout_output.push(line);
                            }
                            Ok(None) => break,
                            Err(e) => {
                                stderr_output.push(format!("stdout error: {}", e));
                                break;
                            }
                        }
                    }
                    line = stderr_reader.next_line() => {
                        if let Ok(Some(line)) = line {
                            stderr_output.push(line);
                        }
                    }
                }
            }
            
            let status = child.wait().await
                .map_err(|e| Error::ToolExecution(format!("Failed to wait for process: {}", e)))?;
            
            Ok::<_, Error>((status, stdout_output, stderr_output))
        }).await
        .map_err(|_| Error::ToolExecution("Command timed out".into()))??;
        
        let (status, stdout_output, stderr_output) = output;
        
        let mut result = String::new();
        
        if !stdout_output.is_empty() {
            result.push_str("=== STDOUT ===\n");
            for line in stdout_output {
                result.push_str(&line);
                result.push('\n');
            }
        }
        
        if !stderr_output.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str("=== STDERR ===\n");
            for line in stderr_output {
                result.push_str(&line);
                result.push('\n');
            }
        }
        
        if !result.is_empty() {
            result.push('\n');
        }
        
        result.push_str(&format!("Exit code: {:?}", status.code()));
        
        Ok(result)
    }
}

#[derive(Clone)]
pub struct GrepTool;

impl GrepTool {
    pub fn new() -> Self {
        GrepTool
    }
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }
    
    fn description(&self) -> &str {
        "Search for patterns in files"
    }
    
    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "The directory or file to search in"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Search recursively",
                    "default": false
                }
            },
            "required": ["pattern", "path"]
        })
    }
    
    async fn execute(&self, arguments: serde_json::Value) -> Result<String> {
        let pattern = arguments.get("pattern")
            .and_then(|p| p.as_str())
            .ok_or_else(|| Error::InvalidArguments("Missing 'pattern' parameter".into()))?;
        
        let path = arguments.get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| Error::InvalidArguments("Missing 'path' parameter".into()))?;
        
        let recursive = arguments.get("recursive")
            .and_then(|r| r.as_bool())
            .unwrap_or(false);
        
        let mut cmd = Command::new("grep");
        cmd.arg("-n");
        
        if recursive {
            cmd.arg("-r");
        }
        
        cmd.arg(pattern)
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        let output = cmd.output().await
            .map_err(|e| Error::ToolExecution(format!("Failed to execute grep: {}", e)))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let mut result = String::new();
        
        if !stdout.is_empty() {
            result.push_str(&stdout);
        }
        
        if !stderr.is_empty() && result.is_empty() {
            result.push_str(&stderr);
        }
        
        if result.is_empty() {
            result.push_str("No matches found");
        }
        
        Ok(result)
    }
}

#[derive(Clone)]
pub struct ListDirTool;

impl ListDirTool {
    pub fn new() -> Self {
        ListDirTool
    }
}

#[async_trait]
impl Tool for ListDirTool {
    fn name(&self) -> &str {
        "list_dir"
    }
    
    fn description(&self) -> &str {
        "List directory contents"
    }
    
    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The directory to list"
                }
            },
            "required": ["path"]
        })
    }
    
    async fn execute(&self, arguments: serde_json::Value) -> Result<String> {
        let path = arguments.get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| Error::InvalidArguments("Missing 'path' parameter".into()))?;
        
        let output = Command::new("ls")
            .arg("-la")
            .arg(path)
            .output()
            .await
            .map_err(|e| Error::ToolExecution(format!("Failed to list directory: {}", e)))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if !stdout.is_empty() {
            Ok(stdout.to_string())
        } else if !stderr.is_empty() {
            Err(Error::ToolExecution(stderr.to_string()))
        } else {
            Ok(String::from("Directory is empty or inaccessible"))
        }
    }
}
