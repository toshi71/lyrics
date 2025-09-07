use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeekPoint {
    pub id: String,              // 一意識別子
    pub name: String,            // ユーザー定義名（"サビ開始", "間奏終了"等）
    pub position_ms: u64,        // ミリ秒単位の位置
    pub color: Option<String>,   // UI表示用の色（将来拡張）
    pub created_at: SystemTime,  // 作成日時
}

impl SeekPoint {
    #[allow(dead_code)]
    pub fn new(name: String, position_ms: u64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            position_ms,
            color: None,
            created_at: SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackSeekPoints {
    pub track_path: PathBuf,     // 楽曲ファイルパス
    pub seek_points: Vec<SeekPoint>, // シークポイント一覧（位置順ソート）
    pub last_updated: SystemTime,   // 最終更新日時
}

impl TrackSeekPoints {
    #[allow(dead_code)]
    pub fn new(track_path: PathBuf) -> Self {
        Self {
            track_path,
            seek_points: Vec::new(),
            last_updated: SystemTime::now(),
        }
    }

    #[allow(dead_code)]
    pub fn add_seek_point(&mut self, name: String, position_ms: u64) -> String {
        let seek_point = SeekPoint::new(name, position_ms);
        let id = seek_point.id.clone();
        
        self.seek_points.push(seek_point);
        self.sort_seek_points();
        self.last_updated = SystemTime::now();
        
        id
    }

    #[allow(dead_code)]
    pub fn remove_seek_point(&mut self, seek_point_id: &str) -> bool {
        if let Some(index) = self.seek_points.iter().position(|sp| sp.id == seek_point_id) {
            self.seek_points.remove(index);
            self.last_updated = SystemTime::now();
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn get_seek_point(&self, seek_point_id: &str) -> Option<&SeekPoint> {
        self.seek_points.iter().find(|sp| sp.id == seek_point_id)
    }

    #[allow(dead_code)]
    fn sort_seek_points(&mut self) {
        self.seek_points.sort_by_key(|sp| sp.position_ms);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeekPointsData {
    pub version: String,
    pub tracks: std::collections::HashMap<PathBuf, Vec<SeekPoint>>,
}

impl Default for SeekPointsData {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            tracks: std::collections::HashMap::new(),
        }
    }
}