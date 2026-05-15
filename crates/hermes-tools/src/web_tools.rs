
use async_trait::async_trait;
use serde_json::json;
use hermes_core::error::{Error, Result};
use hermes_core::tool::Tool;

#[derive(Clone)]
pub struct HttpGetTool {
    client: reqwest::Client,
}

impl HttpGetTool {
    pub fn new() -> Self {
        HttpGetTool {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Tool for HttpGetTool {
    fn name(&self) -> &str {
        "http_get"
    }
    
    fn description(&self) -> &str {
        "Make an HTTP GET request"
    }
    
    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch"
                },
                "headers": {
                    "type": "object",
                    "description": "Optional HTTP headers",
                    "default": {}
                }
            },
            "required": ["url"]
        })
    }
    
    async fn execute(&self, arguments: serde_json::Value) -> Result<String> {
        let url = arguments.get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| Error::InvalidArguments("Missing 'url' parameter".into()))?;
        
        let mut request = self.client.get(url);
        
        if let Some(headers) = arguments.get("headers").and_then(|h| h.as_object()) {
            for (key, value) in headers {
                if let Some(val_str) = value.as_str() {
                    request = request.header(key.as_str(), val_str);
                }
            }
        }
        
        let response = request.send().await
            .map_err(|e| Error::ToolExecution(format!("HTTP request failed: {}", e)))?;
        
        let status = response.status();
        let body = response.text().await
            .map_err(|e| Error::ToolExecution(format!("Failed to read response: {}", e)))?;
        
        Ok(format!("Status: {}\n\n{}", status, body))
    }
}

#[derive(Clone)]
pub struct HttpPostTool {
    client: reqwest::Client,
}

impl HttpPostTool {
    pub fn new() -> Self {
        HttpPostTool {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Tool for HttpPostTool {
    fn name(&self) -> &str {
        "http_post"
    }
    
    fn description(&self) -> &str {
        "Make an HTTP POST request"
    }
    
    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to post to"
                },
                "body": {
                    "type": "string",
                    "description": "The request body"
                },
                "content_type": {
                    "type": "string",
                    "description": "Content type",
                    "default": "application/json"
                }
            },
            "required": ["url", "body"]
        })
    }
    
    async fn execute(&self, arguments: serde_json::Value) -> Result<String> {
        let url = arguments.get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| Error::InvalidArguments("Missing 'url' parameter".into()))?;
        
        let body = arguments.get("body")
            .and_then(|b| b.as_str())
            .ok_or_else(|| Error::InvalidArguments("Missing 'body' parameter".into()))?;
        
        let content_type = arguments.get("content_type")
            .and_then(|c| c.as_str())
            .unwrap_or("application/json");
        
        let response = self.client
            .post(url)
            .header("Content-Type", content_type)
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| Error::ToolExecution(format!("HTTP request failed: {}", e)))?;
        
        let status = response.status();
        let response_body = response.text().await
            .map_err(|e| Error::ToolExecution(format!("Failed to read response: {}", e)))?;
        
        Ok(format!("Status: {}\n\n{}", status, response_body))
    }
}

#[derive(Clone)]
pub struct SearchTool {
    client: reqwest::Client,
}

impl SearchTool {
    pub fn new() -> Self {
        SearchTool {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Tool for SearchTool {
    fn name(&self) -> &str {
        "web_search"
    }
    
    fn description(&self) -> &str {
        "Search the web for information"
    }
    
    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum number of results",
                    "default": 5
                }
            },
            "required": ["query"]
        })
    }
    
    async fn execute(&self, arguments: serde_json::Value) -> Result<String> {
        let query = arguments.get("query")
            .and_then(|q| q.as_str())
            .ok_or_else(|| Error::InvalidArguments("Missing 'query' parameter".into()))?;
        
        let limit = arguments.get("limit")
            .and_then(|l| l.as_u64())
            .unwrap_or(5) as usize;
        
        let encoded_query = urlencoding::encode(query);
        let search_url = format!(
            "https://duckduckgo.com/html/?q={}",
            encoded_query
        );
        
        let response = self.client
            .get(&search_url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|e| Error::ToolExecution(format!("Search failed: {}", e)))?;
        
        let html = response.text().await
            .map_err(|e| Error::ToolExecution(format!("Failed to read response: {}", e)))?;
        
        let mut results = Vec::new();
        let mut current_pos = 0;
        
        for _ in 0..limit {
            if let Some(title_start) = html[current_pos..].find("<a class=\"result__a\" href=\"") {
                let pos = current_pos + title_start;
                if let Some(title_end) = html[pos..].find("</a>") {
                    let title = &html[pos..pos + title_end];
                    let title = title.split('>').nth(1).unwrap_or(title);
                    results.push(title);
                    current_pos = pos + title_end;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if results.is_empty() {
            return Ok("No search results found".to_string());
        }
        
        let mut output = format!("Search results for '{}':\n\n", query);
        for (i, result) in results.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, result));
        }
        
        Ok(output)
    }
}
