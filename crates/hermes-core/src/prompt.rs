
use crate::llm::Message;

pub fn build_system_prompt(base_prompt: &str, additional_info: &[&str]) -> String {
    let mut prompt = base_prompt.to_string();
    
    if !additional_info.is_empty() {
        prompt.push_str("\n\nAdditional Information:\n");
        for info in additional_info {
            prompt.push_str(&format!("- {}\n", info));
        }
    }
    
    prompt
}

pub fn build_message(role: &str, content: &str) -> Message {
    Message {
        role: role.into(),
        content: content.into(),
        tool_calls: None,
    }
}

pub fn build_user_message(content: &str) -> Message {
    build_message("user", content)
}

pub fn build_assistant_message(content: &str) -> Message {
    build_message("assistant", content)
}

pub fn build_system_message(content: &str) -> Message {
    build_message("system", content)
}

pub fn default_system_prompt() -> &'static str {
    "You are Hermes, a helpful AI assistant with access to powerful tools. \
    Use the provided tools to accomplish the user's tasks effectively. \
    Always explain your thought process and actions clearly."
}
