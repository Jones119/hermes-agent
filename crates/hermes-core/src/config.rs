
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
    pub x_search: XSearchConfig,
    pub compression: CompressionConfigSection,
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
    #[serde(default)]
    pub compression_enabled: bool,
    #[serde(default = "default_compression_threshold")]
    pub compression_threshold: usize,
}

fn default_compression_threshold() -> usize {
    40
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XSearchConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_x_search_model")]
    pub model: String,
    #[serde(default = "default_x_search_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_x_search_retries")]
    pub retries: u32,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: String,
}

fn default_x_search_model() -> String {
    "grok-4.20-reasoning".into()
}

fn default_x_search_timeout() -> u64 {
    180
}

fn default_x_search_retries() -> u32 {
    2
}

impl Default for XSearchConfig {
    fn default() -> Self {
        XSearchConfig {
            enabled: false,
            model: default_x_search_model(),
            timeout_seconds: default_x_search_timeout(),
            retries: default_x_search_retries(),
            api_key: String::new(),
            base_url: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfigSection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_compression_threshold")]
    pub threshold: usize,
    #[serde(default = "default_min_messages")]
    pub min_messages: usize,
    #[serde(default = "default_max_summary_tokens")]
    pub max_summary_tokens: u32,
}

fn default_true() -> bool {
    true
}

fn default_min_messages() -> usize {
    10
}

fn default_max_summary_tokens() -> u32 {
    1024
}

impl Default for CompressionConfigSection {
    fn default() -> Self {
        CompressionConfigSection {
            enabled: true,
            threshold: default_compression_threshold(),
            min_messages: default_min_messages(),
            max_summary_tokens: default_max_summary_tokens(),
        }
    }
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
                compression_enabled: true,
                compression_threshold: 40,
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
            x_search: XSearchConfig::default(),
            compression: CompressionConfigSection::default(),
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
