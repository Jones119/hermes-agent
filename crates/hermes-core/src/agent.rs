
use tracing::{info, warn};
use crate::config::Config;
use crate::context::AgentContext;
use crate::error::{Error, Result};
use crate::llm::{ChatRequest, ChatResponse, LlmClient, Message, ToolDefinition};
use crate::prompt;
use crate::tool::ToolRegistry;
use crate::memory::MemoryStore;
use crate::tool_executor;
use crate::chat_completion_helpers::{self, ChatCompletionConfig};
use crate::context_compressor::{ContextCompressor, CompressionConfig};

pub struct Agent {
    config: Config,
    context: AgentContext,
    llm: Box<dyn LlmClient>,
    tools: ToolRegistry,
    memory: MemoryStore,
    compressor: ContextCompressor,
    iteration_count: u32,
}

impl Agent {
    pub fn new(config: Config, llm: Box<dyn LlmClient>) -> Self {
        let compression_config = CompressionConfig {
            enabled: config.agent.compression_enabled,
            threshold: config.agent.compression_threshold,
            ..Default::default()
        };
        Agent {
            config,
            context: AgentContext::default(),
            llm,
            tools: ToolRegistry::default(),
            memory: MemoryStore::default(),
            compressor: ContextCompressor::new(compression_config),
            iteration_count: 0,
        }
    }

    pub fn with_context(mut self, context: AgentContext) -> Self {
        self.context = context;
        self
    }

    pub fn tools(&self) -> &ToolRegistry {
        &self.tools
    }

    pub fn tools_mut(&mut self) -> &mut ToolRegistry {
        &mut self.tools
    }

    pub async fn register_default_tools(&mut self) {
        use crate::tool::EchoTool;
        self.tools.register(EchoTool::new()).await;
    }

    pub async fn run(&mut self, user_input: &str) -> Result<String> {
        info!("Received user input: {}", user_input);

        let user_msg = prompt::build_user_message(user_input);
        self.context.conversation.add_message(user_msg);

        let mut iterations = 0;
        let max_iterations = self.config.agent.max_iterations;

        while iterations < max_iterations {
            iterations += 1;
            self.iteration_count += 1;
            info!("Iteration {}/{}", iterations, max_iterations);

            if self.compressor.should_compress(self.context.conversation.len()) {
                info!("Context threshold reached, compressing...");
                let messages = self.context.conversation.get_messages();
                let compressed = self.compressor
                    .compress(&messages, self.llm.as_ref(), &self.config.llm.model)
                    .await
                    .unwrap_or(messages);
                self.context.conversation.clear();
                for msg in compressed {
                    self.context.conversation.add_message(msg);
                }
            }

            let response = self.call_llm().await?;

            if let Some(choice) = response.choices.first() {
                self.context.conversation.add_message(choice.message.clone());

                if let Some(tool_calls) = &choice.message.tool_calls {
                    info!("Processing {} tool calls", tool_calls.len());

                    let results = if tool_calls.len() > 1 {
                        tool_executor::execute_tool_calls_concurrent(self, tool_calls).await
                    } else {
                        tool_executor::execute_tool_calls_sequential(self, tool_calls).await
                    };

                    for result in results {
                        let content = if result.success {
                            result.result
                        } else {
                            format!("[Tool error: {}]", result.result)
                        };
                        let tool_response = Message {
                            role: "tool".into(),
                            content,
                            tool_calls: None,
                        };
                        self.context.conversation.add_message(tool_response);
                    }
                }

                if choice.finish_reason == "stop" {
                    info!("Agent finished successfully");
                    return Ok(choice.message.content.clone());
                }
            }
        }

        warn!("Max iterations reached");
        Err(Error::State("Max iterations reached".into()))
    }

    async fn call_llm(&self) -> Result<ChatResponse> {
        let mut messages = Vec::new();

        if let Some(system_prompt) = &self.context.system_prompt {
            messages.push(prompt::build_system_message(system_prompt));
        } else {
            messages.push(prompt::build_system_message(prompt::default_system_prompt()));
        }

        messages.extend(self.context.conversation.get_messages());

        let tools = self.tools.list_tools().await;
        let tool_definitions: Vec<ToolDefinition> = tools
            .into_iter()
            .map(|t| ToolDefinition {
                r#type: "function".into(),
                function: crate::llm::FunctionDefinition {
                    name: t.name,
                    description: t.description,
                    parameters: t.parameters,
                },
            })
            .collect();

        let request = ChatRequest {
            model: self.config.llm.model.clone(),
            messages,
            tools: if tool_definitions.is_empty() { None } else { Some(tool_definitions) },
            temperature: self.config.llm.temperature,
            max_tokens: self.config.llm.max_tokens,
        };

        info!("Calling LLM with model: {}", self.config.llm.model);
        let completion_config = ChatCompletionConfig::default();
        chat_completion_helpers::chat_with_retry(self.llm.as_ref(), request, &completion_config).await
    }

    pub async fn clear_conversation(&mut self) {
        self.context.conversation.clear();
    }

    pub fn get_context(&self) -> &AgentContext {
        &self.context
    }

    pub fn iteration_count(&self) -> u32 {
        self.iteration_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{MockLlmClient, ChatResponse, Choice, Message, Usage};

    #[tokio::test]
    async fn test_agent_basic() {
        let config = Config::default();
        let mut mock_llm = MockLlmClient::new();

        let response = ChatResponse {
            id: "test-1".into(),
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: "assistant".into(),
                    content: "Hello!".into(),
                    tool_calls: None,
                },
                finish_reason: "stop".into(),
            }],
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        };

        mock_llm.add_response(response);

        let mut agent = Agent::new(config, Box::new(mock_llm));
        let result = agent.run("Hi").await;
        assert!(result.is_ok());
    }
}
