use super::{Playlist, PlaylistManager};
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PlaylistsData {
    playlists: Vec<Playlist>,
    active_playlist_id: String,
}

impl PlaylistManager {
    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let data = PlaylistsData {
            playlists: self.playlists.clone(),
            active_playlist_id: self.active_playlist_id.clone(),
        };
        
        let json_data = serde_json::to_string_pretty(&data)?;
        
        // ディレクトリが存在しない場合は作成
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, json_data)?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            // ファイルが存在しない場合はデフォルトのPlaylistManagerを返す
            return Ok(PlaylistManager::new());
        }
        
        let json_data = fs::read_to_string(path)?;
        let data: PlaylistsData = serde_json::from_str(&json_data)?;
        
        // データの検証
        if data.playlists.is_empty() {
            // プレイリストが空の場合はデフォルトを作成
            return Ok(PlaylistManager::new());
        }
        
        // デフォルトプレイリストの存在確認
        let has_default = data.playlists.iter().any(|p| p.id == "default");
        let mut playlists = data.playlists;
        
        if !has_default {
            // デフォルトプレイリストが存在しない場合は先頭に追加
            let default_playlist = Playlist::new("default".to_string(), "デフォルト".to_string());
            playlists.insert(0, default_playlist);
        }
        
        // アクティブプレイリストIDの検証
        let active_playlist_id = if playlists.iter().any(|p| p.id == data.active_playlist_id) {
            data.active_playlist_id
        } else {
            "default".to_string()
        };
        
        let mut manager = PlaylistManager::new();
        manager.playlists = playlists;
        manager.active_playlist_id = active_playlist_id;
        Ok(manager)
    }
    
    pub fn get_default_playlist_file_path() -> std::path::PathBuf {
        // 設定ディレクトリと同じ場所にプレイリストファイルを保存
        let mut path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        path.push("playlists.json");
        path
    }
    
    pub fn auto_save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_default_playlist_file_path();
        self.save_to_file(&path)
    }
    
    pub fn auto_load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::get_default_playlist_file_path();
        Self::load_from_file(&path)
    }
}