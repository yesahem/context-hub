use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub endpoint: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: usize,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            model: "llama3.2".to_string(),
            temperature: 0.3,
            max_tokens: 2048,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub default_commit_range: usize,
    pub max_tokens_per_commit: usize,
    pub global_retention_days: i32,
    pub ttl_days: i32,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            default_commit_range: 10,
            max_tokens_per_commit: 1000,
            global_retention_days: -1,
            ttl_days: 7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub auto_sync: bool,
    pub hook_enabled: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            auto_sync: false,
            hook_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "tokyo-night".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ollama: OllamaConfig,
    pub context: ContextConfig,
    pub git: GitConfig,
    pub ui: UiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ollama: OllamaConfig::default(),
            context: ContextConfig::default(),
            git: GitConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Config {
    pub fn load(repo_path: &PathBuf) -> anyhow::Result<Self> {
        let config_path = repo_path.join(".contexthub/config.json");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self, repo_path: &PathBuf) -> anyhow::Result<()> {
        let config_path = repo_path.join(".contexthub/config.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    pub fn set_model(&mut self, model: String) {
        self.ollama.model = model;
    }

    pub fn set_ollama_url(&mut self, url: String) {
        self.ollama.endpoint = url;
    }

    pub fn set_ttl_days(&mut self, days: i32) {
        self.context.ttl_days = days;
    }
}
