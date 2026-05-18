
use crate::error::Result;
use crate::llm::{ChatRequest, LlmClient, Message};
use tracing::{info, warn};

const COMPRESSION_PROMPT: &str = "You are a conversation summarizer. Summarize the following conversation history concisely, preserving key facts, decisions, and context. Focus on information that would be needed for future turns. Do not include pleasantries or filler.";

const DEFAULT_COMPRESSION_THRESHOLD: usize = 40;
const MIN_MESSAGES_TO_COMPRESS: usize = 10;

pub struct CompressionConfig {
    pub enabled: bool,
    pub threshold: usize,
    pub min_messages: usize,
    pub max_summary_tokens: u32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        CompressionConfig {
            enabled: true,
            threshold: DEFAULT_COMPRESSION_THRESHOLD,
            min_messages: MIN_MESSAGES_TO_COMPRESS,
            max_summary_tokens: 1024,
        }
    }
}

pub struct ContextCompressor {
    config: CompressionConfig,
}

impl ContextCompressor {
    pub fn new(config: CompressionConfig) -> Self {
        ContextCompressor { config }
    }

    pub fn should_compress(&self, message_count: usize) -> bool {
        if !self.config.enabled {
            return false;
        }
        message_count >= self.config.threshold
    }

    pub async fn compress(
        &self,
        messages: &[Message],
        client: &dyn LlmClient,
        model: &str,
    ) -> Result<Vec<Message>> {
        if messages.len() < self.config.min_messages {
            info!(
                "Skipping compression: only {} messages (min: {})",
                messages.len(),
                self.config.min_messages
            );
            return Ok(messages.to_vec());
        }

        info!(
            "Compressing {} messages (threshold: {})",
            messages.len(),
            self.config.threshold
        );

        let conversation_text = format_messages_for_compression(messages);

        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![
                Message {
                    role: "system".into(),
                    content: COMPRESSION_PROMPT.to_string(),
                    tool_calls: None,
                },
                Message {
                    role: "user".into(),
                    content: format!(
                        "Summarize this conversation:\n\n{}",
                        conversation_text
                    ),
                    tool_calls: None,
                },
            ],
            tools: None,
            temperature: 0.3,
            max_tokens: self.config.max_summary_tokens,
        };

        match client.chat(request).await {
            Ok(response) => {
                if let Some(choice) = response.choices.first() {
                    let summary = &choice.message.content;
                    info!("Generated compression summary ({} chars)", summary.len());

                    let mut compressed = Vec::new();
                    compressed.push(Message {
                        role: "system".into(),
                        content: format!(
                            "[Previous conversation summary]\n{}\n[End of summary]",
                            summary
                        ),
                        tool_calls: None,
                    });

                    let keep_recent = std::cmp::min(5, messages.len());
                    let recent_start = messages.len().saturating_sub(keep_recent);
                    compressed.extend(messages[recent_start..].to_vec());

                    return Ok(compressed);
                }
            }
            Err(e) => {
                warn!("Compression failed: {}, keeping original messages", e);
            }
        }

        Ok(messages.to_vec())
    }
}

fn format_messages_for_compression(messages: &[Message]) -> String {
    let mut output = String::new();
    for msg in messages {
        let role = &msg.role;
        let content = if msg.content.len() > 500 {
            format!("{}... [truncated]", &msg.content[..500])
        } else {
            msg.content.clone()
        };
        output.push_str(&format!("[{}]: {}\n", role, content));
    }
    output
}

pub fn strip_historical_media(messages: &[Message]) -> Vec<Message> {
    if messages.is_empty() {
        return messages.to_vec();
    }

    let anchor = messages.iter().rposition(|m| {
        m.role == "user" && m.content.contains("[Attached image")
    });

    match anchor {
        Some(0) | None => messages.to_vec(),
        Some(idx) => {
            messages.iter().enumerate().map(|(i, msg)| {
                if i < idx && msg.content.contains("[Attached image") {
                    Message {
                        role: msg.role.clone(),
                        content: "[Attached image — stripped after compression]".to_string(),
                        tool_calls: msg.tool_calls.clone(),
                    }
                } else {
                    msg.clone()
                }
            }).collect()
        }
    }
}

impl Default for ContextCompressor {
    fn default() -> Self {
        Self::new(CompressionConfig::default())
    }
}
