
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
}

pub struct OpenAIClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAIClient {
    pub fn new(api_key: String, base_url: Option<String>, model: String) -> Self {
        OpenAIClient {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".into()),
            model,
        }
    }
}

#[async_trait]
impl LlmClient for OpenAIClient {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("Request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Llm(format!("API error: {} - {}", status, text)));
        }
        
        let chat_response: ChatResponse = response.json()
            .await
            .map_err(|e| Error::Llm(format!("Parse failed: {}", e)))?;
        
        Ok(chat_response)
    }
}

pub struct MockLlmClient {
    responses: Vec<ChatResponse>,
}

impl MockLlmClient {
    pub fn new() -> Self {
        MockLlmClient {
            responses: Vec::new(),
        }
    }
    
    pub fn add_response(&mut self, response: ChatResponse) {
        self.responses.push(response);
    }
}

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
        if let Some(resp) = self.responses.first() {
            Ok(resp.clone())
        } else {
            Err(Error::Llm("No mock response available".into()))
        }
    }
}

pub fn create_client(provider: &str, api_key: String, base_url: Option<String>, model: String) -> Box<dyn LlmClient> {
    match provider.to_lowercase().as_str() {
        "openai" | "anthropic" => Box::new(OpenAIClient::new(api_key, base_url, model)),
        _ => Box::new(OpenAIClient::new(api_key, base_url, model)),
    }
}
