use super::{Playlist, PlaylistManager};
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PlaylistsData {
    playlists: Vec<Playlist>,
    active_playlist_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_playing_playlist_id: Option<String>,
}

impl PlaylistManager {
    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let data = PlaylistsData {
            playlists: self.playlists.clone(),
            active_playlist_id: self.active_playlist_id.clone(),
            current_playing_playlist_id: self.current_playing_playlist_id.clone(),
        };
        
        // Step 4-2: JSON生成のエラーハンドリング
        let json_data = match serde_json::to_string_pretty(&data) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Error: Failed to serialize playlist data: {}", e);
                return Err(e.into());
            }
        };
        
        // ディレクトリが存在しない場合は作成
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Error: Failed to create directory '{}': {}", parent.display(), e);
                return Err(e.into());
            }
        }
        
        // 一時ファイルに書き込んでから置換（原子的操作）
        let temp_path = format!("{}.tmp", path.display());
        if let Err(e) = fs::write(&temp_path, &json_data) {
            eprintln!("Error: Failed to write temporary file '{}': {}", temp_path, e);
            return Err(e.into());
        }

        // 一時ファイルを目的ファイルに移動
        if let Err(e) = fs::rename(&temp_path, path) {
            eprintln!("Error: Failed to move temporary file to '{}': {}", path.display(), e);
            // クリーンアップ
            let _ = fs::remove_file(&temp_path);
            return Err(e.into());
        }
        
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            // ファイルが存在しない場合はデフォルトのPlaylistManagerを返す
            return Ok(PlaylistManager::new());
        }
        
        // Step 4-2: より堅牢なファイル読み込み処理
        let json_data = match fs::read_to_string(path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Warning: Failed to read playlist file '{}': {}. Using default playlists.", 
                         path.display(), e);
                return Ok(PlaylistManager::new());
            }
        };

        // 空ファイルチェック
        if json_data.trim().is_empty() {
            eprintln!("Warning: Playlist file '{}' is empty. Using default playlists.", path.display());
            return Ok(PlaylistManager::new());
        }

        // JSON解析の試行
        let data: PlaylistsData = match serde_json::from_str(&json_data) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Warning: Failed to parse playlist file '{}': {}. Using default playlists.", 
                         path.display(), e);
                
                // バックアップファイルを作成
                let backup_path = format!("{}.backup", path.display());
                if let Err(backup_err) = fs::write(&backup_path, &json_data) {
                    eprintln!("Warning: Failed to create backup file '{}': {}", backup_path, backup_err);
                } else {
                    eprintln!("Corrupted file backed up to: {}", backup_path);
                }
                
                return Ok(PlaylistManager::new());
            }
        };
        
        // Step 4-2: データの検証とクリーンアップ
        if data.playlists.is_empty() {
            // プレイリストが空の場合はデフォルトを作成
            eprintln!("Warning: No playlists found in file. Using default playlists.");
            return Ok(PlaylistManager::new());
        }

        // プレイリストの検証とクリーンアップ
        let mut playlists = Vec::new();
        let mut invalid_playlists = Vec::new();

        for mut playlist in data.playlists {
            // プレイリスト名の検証
            if playlist.name.trim().is_empty() {
                playlist.name = format!("Unnamed Playlist {}", playlists.len() + 1);
                eprintln!("Warning: Found playlist with empty name. Renamed to '{}'", playlist.name);
            }

            // プレイリストIDの検証
            if playlist.id.trim().is_empty() {
                invalid_playlists.push(playlist.name.clone());
                continue;
            }

            // Step 4-3: 楽曲ファイルの存在チェックとクリーンアップ（バッチ最適化）
            let original_count = playlist.tracks.len();
            
            // 大量ファイルの場合は遅延チェック（起動時のパフォーマンス向上）
            if original_count > 1000 {
                eprintln!("Info: Playlist '{}' has {} tracks. File existence will be checked lazily.", 
                         playlist.name, original_count);
            } else {
                // 通常の数の場合は即座にチェック
                playlist.tracks.retain(|track| {
                    if track.path.exists() {
                        true
                    } else {
                        eprintln!("Warning: Removing missing track from playlist '{}': {}", 
                                 playlist.name, track.path.display());
                        false
                    }
                });

                let removed_count = original_count - playlist.tracks.len();
                if removed_count > 0 {
                    eprintln!("Info: Removed {} missing track(s) from playlist '{}'", 
                             removed_count, playlist.name);
                }
            }

            playlists.push(playlist);
        }

        if !invalid_playlists.is_empty() {
            eprintln!("Warning: Skipped {} playlist(s) with invalid IDs: {:?}", 
                     invalid_playlists.len(), invalid_playlists);
        }

        // 重複IDチェック
        let mut seen_ids = std::collections::HashSet::new();
        let mut unique_playlists = Vec::new();
        
        for mut playlist in playlists {
            if seen_ids.contains(&playlist.id) {
                // 重複IDの処理
                let original_id = playlist.id.clone();
                let mut counter = 1;
                loop {
                    let new_id = format!("{}_{}", original_id, counter);
                    if !seen_ids.contains(&new_id) {
                        playlist.id = new_id;
                        break;
                    }
                    counter += 1;
                }
                eprintln!("Warning: Resolved duplicate playlist ID '{}' to '{}'", 
                         original_id, playlist.id);
            }
            seen_ids.insert(playlist.id.clone());
            unique_playlists.push(playlist);
        }
        let mut playlists = unique_playlists;

        // デフォルトプレイリストの存在確認
        let has_default = playlists.iter().any(|p| p.id == "default");
        
        if !has_default {
            // デフォルトプレイリストが存在しない場合は先頭に追加
            let default_playlist = Playlist::new("default".to_string(), "デフォルト".to_string());
            playlists.insert(0, default_playlist);
            eprintln!("Info: Added missing default playlist.");
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
        
        // 現在再生中のプレイリストIDを復元（存在チェック）
        if let Some(playing_id) = data.current_playing_playlist_id {
            if manager.playlists.iter().any(|p| p.id == playing_id) {
                manager.current_playing_playlist_id = Some(playing_id);
            }
        }
        
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