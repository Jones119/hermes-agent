
use async_trait::async_trait;
use serde_json::json;
use hermes_core::error::{Error, Result};
use hermes_core::tool::Tool;

const DEFAULT_XAI_BASE_URL: &str = "https://api.x.ai/v1";
const MAX_HANDLES: usize = 10;

#[derive(Clone)]
pub struct XSearchTool {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    timeout_seconds: u64,
    retries: u32,
}

impl XSearchTool {
    pub fn new(api_key: String) -> Self {
        XSearchTool {
            client: reqwest::Client::new(),
            api_key,
            base_url: DEFAULT_XAI_BASE_URL.to_string(),
            model: "grok-4.20-reasoning".to_string(),
            timeout_seconds: 180,
            retries: 2,
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        if !base_url.is_empty() {
            self.base_url = base_url.trim_end_matches('/').to_string();
        }
        self
    }

    pub fn with_model(mut self, model: String) -> Self {
        if !model.is_empty() {
            self.model = model;
        }
        self
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds.max(30);
        self
    }

    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    pub fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

fn normalize_handles(handles: &[String], field_name: &str) -> std::result::Result<Vec<String>, String> {
    let mut cleaned = Vec::new();
    for handle in handles {
        let normalized = handle.trim().trim_start_matches('@').to_string();
        if !normalized.is_empty() {
            cleaned.push(normalized);
        }
    }
    if cleaned.len() > MAX_HANDLES {
        return Err(format!("{} supports at most {} handles", field_name, MAX_HANDLES));
    }
    Ok(cleaned)
}

#[async_trait]
impl Tool for XSearchTool {
    fn name(&self) -> &str {
        "x_search"
    }

    fn description(&self) -> &str {
        "Search X/Twitter using xAI's built-in x_search tool. Requires XAI_API_KEY or xAI OAuth credentials."
    }

    fn toolset(&self) -> &str {
        "xai"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query for X/Twitter"
                },
                "from_handles": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Filter results to these X handles (max 10)",
                    "maxItems": 10
                },
                "to_handles": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Filter results to replies to these X handles (max 10)",
                    "maxItems": 10
                },
                "max_results": {
                    "type": "number",
                    "description": "Maximum number of results to return",
                    "default": 10
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<String> {
        if !self.is_available() {
            return Err(Error::ToolExecution(
                "X Search unavailable: no xAI credentials configured. Set XAI_API_KEY or configure xai-oauth.".into(),
            ));
        }

        let query = arguments.get("query")
            .and_then(|q| q.as_str())
            .ok_or_else(|| Error::InvalidArguments("Missing 'query' parameter".into()))?;

        let mut from_handles: Vec<String> = Vec::new();
        if let Some(handles) = arguments.get("from_handles").and_then(|h| h.as_array()) {
            for h in handles {
                if let Some(s) = h.as_str() {
                    from_handles.push(s.to_string());
                }
            }
        }
        from_handles = normalize_handles(&from_handles, "from_handles")
            .map_err(|e| Error::InvalidArguments(e))?;

        let mut to_handles: Vec<String> = Vec::new();
        if let Some(handles) = arguments.get("to_handles").and_then(|h| h.as_array()) {
            for h in handles {
                if let Some(s) = h.as_str() {
                    to_handles.push(s.to_string());
                }
            }
        }
        to_handles = normalize_handles(&to_handles, "to_handles")
            .map_err(|e| Error::InvalidArguments(e))?;

        let mut tools = vec![json!({
            "type": "x_search",
            "x_search": {
                "query": query,
                "max_results": arguments.get("max_results")
                    .and_then(|m| m.as_u64())
                    .unwrap_or(10)
            }
        })];

        if let Some(tool) = tools.first_mut() {
            let search_obj = tool.get_mut("x_search").unwrap();
            if !from_handles.is_empty() {
                search_obj["from_handles"] = json!(from_handles);
            }
            if !to_handles.is_empty() {
                search_obj["to_handles"] = json!(to_handles);
            }
        }

        let request_body = json!({
            "model": self.model,
            "tools": tools,
            "tool_choice": {"type": "x_search"},
        });

        let mut last_error = None;

        for attempt in 0..=self.retries {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
            }

            let response = tokio::time::timeout(
                std::time::Duration::from_secs(self.timeout_seconds),
                self.client
                    .post(format!("{}/responses", self.base_url))
                    .bearer_auth(&self.api_key)
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send(),
            ).await;

            match response {
                Ok(Ok(resp)) => {
                    let status = resp.status();
                    if status.is_success() {
                        match resp.json::<serde_json::Value>().await {
                            Ok(payload) => {
                                let output_text = extract_response_text(&payload);
                                if !output_text.is_empty() {
                                    return Ok(output_text);
                                }
                                let citations = extract_citations(&payload);
                                if !citations.is_empty() {
                                    return Ok(serde_json::to_string_pretty(&citations)
                                        .unwrap_or_else(|_| "Search completed but no text output".into()));
                                }
                                return Ok("X search completed with no results".into());
                            }
                            Err(e) => {
                                last_error = Some(format!("Failed to parse response: {}", e));
                            }
                        }
                    } else {
                        let body = resp.text().await.unwrap_or_default();
                        last_error = Some(format!("API error (status {}): {}", status, body));
                        if status.as_u16() == 401 || status.as_u16() == 403 {
                            break;
                        }
                    }
                }
                Ok(Err(e)) => {
                    last_error = Some(format!("Request failed: {}", e));
                }
                Err(_) => {
                    last_error = Some(format!("Request timed out after {}s", self.timeout_seconds));
                }
            }
        }

        Err(Error::ToolExecution(last_error.unwrap_or_else(|| "X search failed".into())))
    }
}

fn extract_response_text(payload: &serde_json::Value) -> String {
    if let Some(text) = payload.get("output_text").and_then(|t| t.as_str()) {
        if !text.trim().is_empty() {
            return text.to_string();
        }
    }

    let mut parts = Vec::new();
    if let Some(output) = payload.get("output").and_then(|o| o.as_array()) {
        for item in output {
            if item.get("type").and_then(|t| t.as_str()) != Some("message") {
                continue;
            }
            if let Some(content) = item.get("content").and_then(|c| c.as_array()) {
                for part in content {
                    let ctype = part.get("type").and_then(|t| t.as_str());
                    if ctype == Some("output_text") || ctype == Some("text") {
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            if !text.trim().is_empty() {
                                parts.push(text.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    parts.join("\n\n")
}

fn extract_citations(payload: &serde_json::Value) -> Vec<serde_json::Value> {
    let mut citations = Vec::new();
    if let Some(output) = payload.get("output").and_then(|o| o.as_array()) {
        for item in output {
            if item.get("type").and_then(|t| t.as_str()) != Some("message") {
                continue;
            }
            if let Some(content) = item.get("content").and_then(|c| c.as_array()) {
                for part in content {
                    if let Some(annotations) = part.get("annotations").and_then(|a| a.as_array()) {
                        for annotation in annotations {
                            if annotation.get("type").and_then(|t| t.as_str()) == Some("url_citation") {
                                citations.push(annotation.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    citations
}
