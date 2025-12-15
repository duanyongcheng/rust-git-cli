use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub ai: AIConfig,
    pub commit: CommitConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIConfig {
    pub provider: String,
    pub model: String,
    pub api_key_env: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_max_tokens() -> u32 {
    2000
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommitConfig {
    pub format: String,
    pub include_emoji: bool,
    pub max_diff_size: usize,
    pub auto_stage: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ai: AIConfig {
                provider: "openai".to_string(),
                model: "gpt-4.1".to_string(),
                api_key_env: "OPENAI_API_KEY".to_string(),
                api_key: None,
                base_url: None,
                max_tokens: 2000,
            },
            commit: CommitConfig {
                format: "conventional".to_string(),
                include_emoji: false,
                max_diff_size: 10000,
                auto_stage: false,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try to load from multiple locations
        let mut config_paths = vec![PathBuf::from(".rust-commit.toml")];

        // Only add home directory paths if home_dir is available
        if let Some(home) = dirs::home_dir() {
            config_paths.push(home.join(".config/rust-commit/config.toml"));
            config_paths.push(home.join(".rust-commit.toml"));
        }

        for path in config_paths {
            if path.exists() && path.is_file() {
                let content = fs::read_to_string(&path)
                    .context(format!("Failed to read config from {:?}", path))?;
                let config: Config = toml::from_str(&content)
                    .context(format!("Failed to parse config from {:?}", path))?;
                return Ok(config);
            }
        }

        // Return default if no config file found
        Ok(Self::default())
    }

    pub fn get_api_key(&self) -> Option<String> {
        // First check if api_key is directly set
        if let Some(key) = &self.ai.api_key {
            return Some(key.clone());
        }

        // Then check environment variable
        std::env::var(&self.ai.api_key_env).ok()
    }

    pub fn init(local: bool, force: bool) -> Result<PathBuf> {
        let path = if local {
            PathBuf::from(".rust-commit.toml")
        } else {
            // Use ~/.config/rust-commit/config.toml
            dirs::home_dir()
                .map(|p| p.join(".config/rust-commit/config.toml"))
                .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        };

        // Check if file already exists
        if path.exists() && !force {
            anyhow::bail!(
                "Config file already exists at {:?}. Use --force to overwrite.",
                path
            );
        }

        // Create default config with helpful comments
        let config_content = r#"# Rust Commit Configuration File
# This file configures the rust-commit tool for AI-powered commit message generation

[ai]
# AI provider: "openai" or "anthropic"
provider = "openai"

# Model to use for generation
# OpenAI: "gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"
# Anthropic: "claude-3-opus", "claude-3-sonnet", "claude-3-haiku"
model = "gpt-4"

# Environment variable containing the API key
# For OpenAI: typically "OPENAI_API_KEY"
# For Anthropic: typically "ANTHROPIC_API_KEY"
api_key_env = "OPENAI_API_KEY"

# Direct API key (not recommended for security reasons)
# Uncomment and set your API key here if you prefer not to use environment variables
# api_key = "your-api-key-here"

# Custom API endpoint (optional - uncomment and modify if using a proxy or alternative API)
# For OpenAI-compatible APIs (e.g., Azure OpenAI, local LLMs, proxies):
# base_url = "https://api.openai.com/v1"
# For Anthropic-compatible APIs:
# base_url = "https://api.anthropic.com"
# Examples:
# base_url = "https://your-proxy.com/v1"  # For API proxies
# base_url = "http://localhost:8080/v1"   # For local LLMs

# Maximum tokens for AI response (default: 2000)
# This is the initial max_tokens value. If the response is truncated,
# it will automatically double up to 4000 tokens.
max_tokens = 2000

[commit]
# Commit message format: "conventional" (follows Conventional Commits spec)
format = "conventional"

# Whether to include emoji in commit messages
include_emoji = false

# Maximum diff size in characters to send to AI
max_diff_size = 4000

# Whether to automatically stage all changes before committing
auto_stage = false
"#;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .context(format!("Failed to create directory {:?}", parent))?;
            }
        }

        // Write config file
        fs::write(&path, config_content)
            .context(format!("Failed to write config to {:?}", path))?;

        // Set appropriate file permissions on Unix-like systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&path)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(&path, permissions)
                .context("Failed to set config file permissions")?;
        }

        Ok(path)
    }
}
