
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    
    #[error("LLM client error: {0}")]
    Llm(String),
    
    #[error("Tool execution error: {0}")]
    ToolExecution(String),
    
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
    
    #[error("Agent state error: {0}")]
    State(String),
    
    #[error("Other error: {0}")]
    Other(String),
}
