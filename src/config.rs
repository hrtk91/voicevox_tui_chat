use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
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

    fn config_file_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "voicevox", "voicevox_chat")
            .map(|project_dirs| project_dirs.config_dir().join("config.json"))
    }
}
