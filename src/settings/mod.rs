use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub target_directory: String,
    pub classical_composer_hierarchy: bool,
    
    // Step 4-1: プレイリスト関連設定
    pub last_used_playlist_id: Option<String>,
    pub playlist_display_order: Vec<String>,
    pub default_playlist_settings: DefaultPlaylistSettings,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DefaultPlaylistSettings {
    pub auto_add_new_tracks: bool,
    pub clear_on_startup: bool,
    pub max_tracks: Option<usize>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            target_directory: String::new(),
            classical_composer_hierarchy: false,
            last_used_playlist_id: None,
            playlist_display_order: vec!["default".to_string()],
            default_playlist_settings: DefaultPlaylistSettings::default(),
        }
    }
}

impl Default for DefaultPlaylistSettings {
    fn default() -> Self {
        Self {
            auto_add_new_tracks: false,
            clear_on_startup: false,
            max_tracks: None,
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

    // Step 4-1: プレイリスト設定管理メソッド
    pub fn set_last_used_playlist(&mut self, playlist_id: String) {
        self.last_used_playlist_id = Some(playlist_id);
    }

    pub fn get_last_used_playlist_id(&self) -> Option<&str> {
        self.last_used_playlist_id.as_deref()
    }

    pub fn update_playlist_display_order(&mut self, order: Vec<String>) {
        self.playlist_display_order = order;
    }

    pub fn get_playlist_display_order(&self) -> &Vec<String> {
        &self.playlist_display_order
    }

    pub fn add_to_display_order(&mut self, playlist_id: String) {
        if !self.playlist_display_order.contains(&playlist_id) {
            self.playlist_display_order.push(playlist_id);
        }
    }

    pub fn remove_from_display_order(&mut self, playlist_id: &str) {
        self.playlist_display_order.retain(|id| id != playlist_id);
    }
}