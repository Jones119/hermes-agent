
use crate::llm
use crate::llm::Message;
use serde::{Deserialize,
use crate::llm::Message;
use serde::{Deserialize, Serialize};
use std::collections::
use crate::llm::Message;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub
use crate::llm::Message;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    messages: VecDe
use crate::llm::Message;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    messages: VecDeque<Message>,
    max_size: usize,
}

impl ContextManager {
    pub
use crate::llm::Message;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    messages: VecDeque<Message>,
    max_size: usize,
}

impl ContextManager {
    pub fn new(max_size: usize) -> Self {
        ContextManager {
            messages: Vec
use crate::llm::Message;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    messages: VecDeque<Message>,
    max_size: usize,
}

impl ContextManager {
    pub fn new(max_size: usize) -> Self {
        ContextManager {
            messages: VecDeque::with_capacity(max_size),

use crate::llm::Message;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    messages: VecDeque<Message>,
    max_size: usize,
}

impl ContextManager {
    pub fn new(max_size: usize) -> Self {
        ContextManager {
            messages: VecDeque::with_capacity(max_size),
            max_size,
        }
    }
    
    pub fn add_message(&mut self,
use crate::llm::Message;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    messages: VecDeque<Message>,
    max_size: usize,
}

impl ContextManager {
    pub fn new(max_size: usize) -> Self {
        ContextManager {
            messages: VecDeque::with_capacity(max_size),
            max_size,
        }
    }
    
    pub fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);
        if self.messages.len()
use crate::llm::Message;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    messages: VecDeque<Message>,
    max_size: usize,
}

impl ContextManager {
    pub fn new(max_size: usize) -> Self {
        ContextManager {
            messages: VecDeque::with_capacity(max_size),
            max_size,
        }
    }
    
    pub fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);
        if self.messages.len() > self.max_size {
            self.messages.pop_front();
        }
    }
    
    pub fn add_user_message(&mut self,
use crate::llm::Message;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    messages: VecDeque<Message>,
    max_size: usize,
}

impl ContextManager {
    pub fn new(max_size: usize) -> Self {
        ContextManager {
            messages: VecDeque::with_capacity(max_size),
            max_size,
        }
    }
    
    pub fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);
        if self.messages.len() > self.max_size {
            self.messages.pop_front();
        }
    }
    
    pub fn add_user_message(&mut self, content: String) {
        self.add_message(
use crate::llm::Message;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    messages: VecDeque<Message>,
    max_size: usize,
}

impl ContextManager {
    pub fn new(max_size: usize) -> Self {
        ContextManager {
            messages: VecDeque::with_capacity(max_size),
            max_size,
        }
    }
    
    pub fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);
        if self.messages.len() > self.max_size {
            self.messages.pop_front();
        }
    }
    
    pub fn add_user_message(&mut self, content: String) {
        self.add_message(Message {
            role: "user".into(),
            content,
            tool_calls