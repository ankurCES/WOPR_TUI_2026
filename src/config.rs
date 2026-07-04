use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_provider")]
    pub provider: String,
    pub api_key: Option<String>,
    pub model: Option<String>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_token_budget")]
    pub token_budget: u32,
    #[serde(default = "default_true")]
    pub cache_enabled: bool,
}

fn default_provider() -> String { "stub".into() }
fn default_temperature() -> f32 { 0.8 }
fn default_max_tokens() -> u32 { 2048 }
fn default_token_budget() -> u32 { 100_000 }
fn default_true() -> bool { true }

impl Default for Settings {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            api_key: None,
            model: None,
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            token_budget: default_token_budget(),
            cache_enabled: default_true(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = config_path();
        match std::fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => {
                let settings = Self::default();
                // ponytail: best-effort write, don't crash if dir doesn't exist
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&path, serde_json::to_string_pretty(&settings).unwrap());
                settings
            }
        }
    }
}

fn config_path() -> PathBuf {
    dirs_next().join("settings.json")
}

fn dirs_next() -> PathBuf {
    // ponytail: ~/.wopr/ — simple, no XDG overhead
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".wopr")
}
