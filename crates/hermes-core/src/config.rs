
use anyhow::Result;
use config::Config as ConfigLoader;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub llm: LlmConfig,
    pub agent: AgentConfig,
    pub web: WebConfig,
    pub skill: SkillConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_iterations: u32,
    pub max_tool_calls: u32,
    pub timeout_secs: u64,
    pub memory_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub host: String,
    pub port: u16,
    pub enable_cors: bool,
    pub static_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfig {
    pub auto_create: bool,
    pub skill_dir: String,
    pub review_interval_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_approval: bool,
    pub command_whitelist: Vec<String>,
    pub path_whitelist: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            llm: LlmConfig {
                provider: "openai".into(),
                model: "gpt-4o".into(),
                api_key: "".into(),
                base_url: None,
                temperature: 0.7,
                max_tokens: 4096,
            },
            agent: AgentConfig {
                max_iterations: 20,
                max_tool_calls: 50,
                timeout_secs: 600,
                memory_limit: 100,
            },
            web: WebConfig {
                host: "127.0.0.1".into(),
                port: 3000,
                enable_cors: true,
                static_dir: Some("./web/dist".into()),
            },
            skill: SkillConfig {
                auto_create: true,
                skill_dir: "./skills".into(),
                review_interval_hours: 168,
            },
            security: SecurityConfig {
                enable_approval: true,
                command_whitelist: vec!["ls".into(), "cd".into(), "pwd".into()],
                path_whitelist: vec!["/workspace".into()],
            },
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = ConfigLoader::builder()
            .add_source(config::File::with_name(path.as_ref().to_str().unwrap()))
            .add_source(config::Environment::with_prefix("HERMES"))
            .build()?;
        
        Ok(config.try_deserialize()?)
    }
    
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    pub fn from_env_or_default() -> Self {
        Config::load("config.toml").unwrap_or_default()
    }
}
