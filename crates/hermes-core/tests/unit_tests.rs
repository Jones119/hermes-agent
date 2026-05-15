
#[cfg(test)]
mod tests {
    use hermes_core::agent::Agent;
    use hermes_core::config::Config;
    use hermes_core::llm::{ChatRequest, ChatResponse, Choice, LlmClient, Message, MockLlmClient, Usage};
    use hermes_core::tool::EchoTool;
    use hermes_core::tool::Tool;
    use serde_json;

    #[tokio::test]
    async fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.llm.model, "gpt-4o");
        assert_eq!(config.agent.max_iterations, 20);
    }

    #[tokio::test]
    async fn test_echo_tool() {
        let tool = EchoTool::new();
        assert_eq!(tool.name(), "echo");
        assert_eq!(tool.description(), "Echo back the input message");

        let args = serde_json::json!({"message": "Hello"});
        let result: Result<String, hermes_core::error::Error> = tool.execute(args).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello");
    }

    #[tokio::test]
    async fn test_mock_llm_client() {
        let mut mock = MockLlmClient::new();
        
        let response = ChatResponse {
            id: "test-1".into(),
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: "assistant".into(),
                    content: "Test response".into(),
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
        
        mock.add_response(response);
        
        let request = ChatRequest {
            model: "gpt-4".into(),
            messages: vec![],
            tools: None,
            temperature: 0.7,
            max_tokens: 100,
        };
        
        let result: Result<ChatResponse, hermes_core::error::Error> = mock.chat(request).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().choices[0].message.content, "Test response");
    }

    #[tokio::test]
    async fn test_agent_basic_run() {
        let config = Config::default();
        let mut mock = MockLlmClient::new();
        
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
        
        mock.add_response(response);
        
        let mut agent = Agent::new(config, Box::new(mock));
        agent.register_default_tools().await;
        
        let result = agent.run("Hi").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello!");
    }
}
