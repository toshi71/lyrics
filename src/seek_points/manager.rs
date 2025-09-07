use super::data::{SeekPoint, SeekPointsData};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct SeekPointManager {
    track_seek_points: HashMap<PathBuf, Vec<SeekPoint>>, // メモリ常駐データ
    seek_points_file: PathBuf,  // 単一JSONファイルパス
}

impl SeekPointManager {
    pub fn new() -> Self {
        Self {
            track_seek_points: HashMap::new(),
            seek_points_file: Self::get_seek_points_file_path(),
        }
    }

    // データ管理
    pub fn add_seek_point(&mut self, track_path: &Path, name: String, position_ms: u64) -> Result<String, String> {
        let seek_point = SeekPoint::new(name, position_ms);
        let id = seek_point.id.clone();

        let track_path = track_path.to_path_buf();
        let seek_points = self.track_seek_points.entry(track_path).or_insert_with(Vec::new);
        
        seek_points.push(seek_point);
        seek_points.sort_by_key(|sp| sp.position_ms);

        Ok(id)
    }

    #[allow(dead_code)]
    pub fn remove_seek_point(&mut self, track_path: &Path, seek_point_id: &str) -> Result<(), String> {
        if let Some(seek_points) = self.track_seek_points.get_mut(track_path) {
            if let Some(index) = seek_points.iter().position(|sp| sp.id == seek_point_id) {
                seek_points.remove(index);
                Ok(())
            } else {
                Err(format!("Seek point with id '{}' not found", seek_point_id))
            }
        } else {
            Err(format!("No seek points found for track: {}", track_path.display()))
        }
    }

    pub fn update_seek_point_name(&mut self, track_path: &Path, seek_point_id: &str, new_name: String) -> Result<(), String> {
        if let Some(seek_points) = self.track_seek_points.get_mut(track_path) {
            if let Some(seek_point) = seek_points.iter_mut().find(|sp| sp.id == seek_point_id) {
                seek_point.name = new_name;
                Ok(())
            } else {
                Err(format!("Seek point with id '{}' not found", seek_point_id))
            }
        } else {
            Err(format!("No seek points found for track: {}", track_path.display()))
        }
    }

    pub fn get_seek_points(&self, track_path: &Path) -> Option<&Vec<SeekPoint>> {
        self.track_seek_points.get(track_path)
    }

    #[allow(dead_code)]
    pub fn get_seek_point(&self, track_path: &Path, seek_point_id: &str) -> Option<&SeekPoint> {
        self.track_seek_points
            .get(track_path)?
            .iter()
            .find(|sp| sp.id == seek_point_id)
    }

    // ナビゲーション
    #[allow(dead_code)]
    pub fn find_next_seek_point(&self, track_path: &Path, current_ms: u64) -> Option<&SeekPoint> {
        self.track_seek_points
            .get(track_path)?
            .iter()
            .find(|sp| sp.position_ms > current_ms)
    }

    #[allow(dead_code)]
    pub fn find_previous_seek_point(&self, track_path: &Path, current_ms: u64) -> Option<&SeekPoint> {
        self.track_seek_points
            .get(track_path)?
            .iter()
            .rev()
            .find(|sp| sp.position_ms < current_ms)
    }

    // 永続化（単一JSONファイル + メモリ常駐）
    pub fn save_to_file(&self) -> Result<(), String> {
        let data = SeekPointsData {
            version: "1.0".to_string(),
            tracks: self.track_seek_points.clone(),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize seek points data: {}", e))?;

        std::fs::write(&self.seek_points_file, json)
            .map_err(|e| format!("Failed to write seek points file: {}", e))?;

        Ok(())
    }

    pub fn load_from_file(&mut self) -> Result<(), String> {
        if !self.seek_points_file.exists() {
            // ファイルが存在しない場合は空のデータで初期化
            self.track_seek_points = HashMap::new();
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.seek_points_file)
            .map_err(|e| format!("Failed to read seek points file: {}", e))?;

        let data: SeekPointsData = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse seek points data: {}", e))?;

        self.track_seek_points = data.tracks;
        Ok(())
    }

    // ファイルパス管理
    fn get_seek_points_file_path() -> PathBuf {
        let mut path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."))
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();
        
        path.push("seek_points.json");
        path
    }

    // 統計・ヘルパー
    #[allow(dead_code)]
    pub fn get_track_count(&self) -> usize {
        self.track_seek_points.len()
    }

    #[allow(dead_code)]
    pub fn get_total_seek_points_count(&self) -> usize {
        self.track_seek_points.values().map(|sp| sp.len()).sum()
    }

    #[allow(dead_code)]
    pub fn clear_track_seek_points(&mut self, track_path: &Path) {
        self.track_seek_points.remove(track_path);
    }
}