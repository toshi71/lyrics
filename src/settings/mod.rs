use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub target_directory: String,
    pub classical_composer_hierarchy: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            target_directory: String::new(),
            classical_composer_hierarchy: false,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let settings_path = Self::get_settings_file_path();
        if let Ok(contents) = fs::read_to_string(&settings_path) {
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            Settings::default()
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = Self::get_settings_file_path();
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&settings_path, json)?;
        Ok(())
    }

    fn get_settings_file_path() -> PathBuf {
        let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        path.push("settings.json");
        path
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.target_directory.is_empty() {
            let path = PathBuf::from(&self.target_directory);
            if !path.exists() {
                return Err("Target directory does not exist".to_string());
            }
            if !path.is_dir() {
                return Err("Target path is not a directory".to_string());
            }
        }
        Ok(())
    }
}