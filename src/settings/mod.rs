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
    
    // フォント設定
    pub selected_font: String,
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
            selected_font: "Meiryo".to_string(),
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

    // フォント設定メソッド
    pub fn set_selected_font(&mut self, font_name: String) {
        self.selected_font = font_name;
    }

    pub fn get_available_fonts() -> Vec<String> {
        use font_kit::source::SystemSource;
        use font_kit::family_name::FamilyName;
        use font_kit::properties::Properties;
        
        let system_source = SystemSource::new();
        let mut available_fonts = Vec::new();
        
        // よく使われるCJK対応フォントのリスト（優先して表示）
        let preferred_fonts = vec![
            "Meiryo",
            "Yu Gothic UI",
            "Yu Gothic",
            "MS UI Gothic",
            "MS Gothic",
            "Microsoft YaHei UI",
            "Microsoft YaHei",
            "Malgun Gothic",
            "Noto Sans CJK JP",
            "Noto Sans CJK",
            "Source Han Sans",
            "Source Han Sans JP",
        ];
        
        // 優先フォントから利用可能なものを追加
        for font_name in &preferred_fonts {
            if let Ok(_) = system_source.select_best_match(&[FamilyName::Title(font_name.to_string())], &Properties::new()) {
                if !available_fonts.contains(&font_name.to_string()) {
                    available_fonts.push(font_name.to_string());
                }
            }
        }
        
        // システムの全フォントファミリーを取得してCJK関連を追加
        if let Ok(families) = system_source.all_families() {
            for family in families {
                let family_str = family.to_string();
                
                // CJK関連のキーワードを含むフォントを優先
                if family_str.contains("CJK") || 
                   family_str.contains("Japanese") || 
                   family_str.contains("Korean") || 
                   family_str.contains("Chinese") ||
                   family_str.contains("Sans") ||
                   family_str.contains("Serif") ||
                   family_str.contains("Gothic") ||
                   family_str.contains("Mincho") ||
                   family_str.contains("Noto") ||
                   family_str.contains("Source") {
                    if !available_fonts.contains(&family_str) {
                        available_fonts.push(family_str);
                    }
                }
            }
        }
        
        // フォント名でソート
        available_fonts.sort();
        
        // 利用できるフォントが見つからない場合は最低限のフォントを提供
        if available_fonts.is_empty() {
            available_fonts.push("Meiryo".to_string());
        }
        
        available_fonts
    }
}