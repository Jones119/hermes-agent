
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::llm::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    messages: VecDeque<Message>,
    max_messages: usize,
}

impl ConversationContext {
    pub fn new(max_messages: usize) -> Self {
        ConversationContext {
            messages: VecDeque::with_capacity(max_messages),
            max_messages,
        }
    }
    
    pub fn add_message(&mut self, message: Message) {
        if self.messages.len() >= self.max_messages {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }
    
    pub fn get_messages(&self) -> Vec<Message> {
        self.messages.iter().cloned().collect()
    }
    
    pub fn clear(&mut self) {
        self.messages.clear();
    }
    
    pub fn len(&self) -> usize {
        self.messages.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self::new(50)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub conversation: ConversationContext,
    pub system_prompt: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
}

impl AgentContext {
    pub fn new() -> Self {
        AgentContext {
            conversation: ConversationContext::default(),
            system_prompt: None,
            user_id: None,
            session_id: None,
        }
    }
    
    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = Some(prompt);
        self
    }
    
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

impl Default for AgentContext {
    fn default() -> Self {
        Self::new()
    }
}
