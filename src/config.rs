use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub last_used_model: Option<String>,
}

impl AppConfig {
    pub fn load() -> Self {
        match Self::config_file_path() {
            Some(config_path) => {
                if config_path.exists() {
                    match fs::read_to_string(&config_path) {
                        Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
                            Ok(config) => {
                                log::debug!("Loaded config from: {:?}", config_path);
                                return config;
                            }
                            Err(e) => {
                                log::warn!("Failed to parse config file: {}", e);
                            }
                        },
                        Err(e) => {
                            log::warn!("Failed to read config file: {}", e);
                        }
                    }
                }
            }
            None => {
                log::warn!("Could not determine config directory");
            }
        }

        log::debug!("Using default config");
        Self::default()
    }

    pub fn save(&self) {
        if let Some(config_path) = Self::config_file_path() {
            // Create parent directory if it doesn't exist
            if let Some(parent_dir) = config_path.parent() {
                if let Err(e) = fs::create_dir_all(parent_dir) {
                    log::error!("Failed to create config directory: {}", e);
                    return;
                }
            }

            match serde_json::to_string_pretty(self) {
                Ok(content) => {
                    if let Err(e) = fs::write(&config_path, content) {
                        log::error!("Failed to write config file: {}", e);
                    } else {
                        log::debug!("Saved config to: {:?}", config_path);
                    }
                }
                Err(e) => {
                    log::error!("Failed to serialize config: {}", e);
                }
            }
        } else {
            log::error!("Could not determine config file path");
        }
    }

    pub fn get_model(&self, env_model: Option<&str>, default: &str) -> String {
        // Priority: saved config > environment variable > default
        if let Some(saved_model) = &self.last_used_model {
            saved_model.clone()
        } else if let Some(env_model) = env_model {
            env_model.to_string()
        } else {
            default.to_string()
        }
    }

    pub fn set_last_used_model(&mut self, model: String) {
        self.last_used_model = Some(model);
    }

    pub fn get_all_settings(&self) -> HashMap<String, String> {
        let mut settings = HashMap::new();

        // Model settings
        if let Some(model) = &self.last_used_model {
            settings.insert("Last Used Model".to_string(), format!("{} [config]", model));
        }

        // Environment variables
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let masked_key = format!("{}...{}", &api_key[..8], &api_key[api_key.len()-8..]);
            settings.insert("OpenAI API Key".to_string(), format!("{} [env]", masked_key));
        }

        if let Ok(env_model) = std::env::var("OPENAI_MODEL") {
            settings.insert("OpenAI Model (env)".to_string(), format!("{} [env]", env_model));
        } else {
            settings.insert("OpenAI Model (env)".to_string(), "Not set".to_string());
        }

        if let Ok(prompt) = std::env::var("PROMPT") {
            let short_prompt = if prompt.len() > 50 {
                format!("{}..." , &prompt[..47])
            } else {
                prompt
            };
            settings.insert("System Prompt".to_string(), format!("{} [env]", short_prompt));
        } else {
            settings.insert("System Prompt".to_string(), "Using default [default]".to_string());
        }

        if let Ok(engine_url) = std::env::var("VOICEVOX_ENGINE_URL") {
            settings.insert("VOICEVOX Engine URL".to_string(), format!("{} [env]", engine_url));
        } else {
            settings.insert("VOICEVOX Engine URL".to_string(), "http://localhost:50021 [default]".to_string());
        }

        // Config file location
        if let Some(config_path) = Self::config_file_path() {
            let path_str = config_path.to_string_lossy();
            settings.insert("Config File Path".to_string(), path_str.to_string());
        } else {
            settings.insert("Config File Path".to_string(), "Could not determine".to_string());
        }

        // Available models
        settings.insert("Available Models".to_string(), "gpt-5, gpt-5-mini, gpt-5-nano".to_string());

        settings
    }

    fn config_file_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "voicevox", "voicevox_chat")
            .map(|project_dirs| project_dirs.config_dir().join("config.json"))
    }
}
