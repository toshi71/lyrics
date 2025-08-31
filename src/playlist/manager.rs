use std::collections::HashSet;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::music::TrackInfo;
use crate::settings::RepeatMode;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub tracks: Vec<TrackInfo>,
    pub created_at: SystemTime,
    pub modified_at: SystemTime,
}

impl Playlist {
    pub fn new(id: String, name: String) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            name,
            tracks: Vec::new(),
            created_at: now,
            modified_at: now,
        }
    }

    pub fn add_track(&mut self, track: TrackInfo) {
        self.tracks.push(track);
        self.modified_at = SystemTime::now();
    }

    pub fn remove_track(&mut self, index: usize) -> Option<TrackInfo> {
        if index < self.tracks.len() {
            self.modified_at = SystemTime::now();
            Some(self.tracks.remove(index))
        } else {
            None
        }
    }

    pub fn move_track(&mut self, from: usize, to: usize) -> bool {
        if from < self.tracks.len() && to < self.tracks.len() {
            let track = self.tracks.remove(from);
            self.tracks.insert(to, track);
            self.modified_at = SystemTime::now();
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.modified_at = SystemTime::now();
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn get_track(&self, index: usize) -> Option<&TrackInfo> {
        self.tracks.get(index)
    }

    pub fn get_tracks(&self) -> &Vec<TrackInfo> {
        &self.tracks
    }

    /// 指定された楽曲がプレイリストに既に存在するかチェック
    pub fn contains_track(&self, track: &TrackInfo) -> bool {
        self.tracks.iter().any(|existing_track| existing_track.is_same_track(track))
    }

    /// 複数の楽曲のうち、プレイリストに既に存在するものを返す
    pub fn get_duplicate_tracks<'a>(&self, tracks: &'a [TrackInfo]) -> Vec<&'a TrackInfo> {
        tracks.iter().filter(|track| self.contains_track(track)).collect()
    }
}

#[derive(Debug)]
pub struct PlaylistManager {
    pub(crate) playlists: Vec<Playlist>,
    pub(crate) active_playlist_id: String,
    selected_indices: HashSet<usize>,
    current_playing_index: Option<usize>,
    pub(crate) current_playing_playlist_id: Option<String>, // 現在再生中の楽曲があるプレイリスト
    shuffle_order: Vec<usize>, // シャッフル時の再生順序
    shuffle_position: Option<usize>, // シャッフル順序内での現在位置
    last_selected_index: Option<usize>, // 範囲選択用の最後に選択されたインデックス
}

impl PlaylistManager {
    pub fn new() -> Self {
        let default_playlist = Playlist::new("default".to_string(), "デフォルト".to_string());
        let active_playlist_id = default_playlist.id.clone();
        
        Self {
            playlists: vec![default_playlist],
            active_playlist_id,
            selected_indices: HashSet::new(),
            current_playing_index: None,
            current_playing_playlist_id: None,
            shuffle_order: Vec::new(),
            shuffle_position: None,
            last_selected_index: None,
        }
    }

    // Step 4-1: 設定と連携したコンストラクタ
    pub fn new_with_settings(last_playlist_id: Option<&str>, playlist_order: &[String]) -> Self {
        let default_playlist = Playlist::new("default".to_string(), "デフォルト".to_string());
        
        // 最後に使用したプレイリストがあればそれを使用、なければデフォルト
        let active_playlist_id = last_playlist_id
            .unwrap_or("default")
            .to_string();
        
        let mut manager = Self {
            playlists: vec![default_playlist],
            active_playlist_id,
            selected_indices: HashSet::new(),
            current_playing_index: None,
            current_playing_playlist_id: None,
            shuffle_order: Vec::new(),
            shuffle_position: None,
            last_selected_index: None,
        };

        // プレイリストの表示順序を適用（永続化されたプレイリストが読み込まれた後に呼び出される）
        manager.apply_display_order(playlist_order);
        
        manager
    }

    // プレイリスト管理
    pub fn create_playlist(&mut self, name: String) -> String {
        // Step 4-2: プレイリスト名の検証
        let validated_name = Self::validate_playlist_name(&name, &self.playlists, None);
        
        let id = format!("playlist_{}", self.playlists.len());
        let playlist = Playlist::new(id.clone(), validated_name);
        self.playlists.push(playlist);
        id
    }

    // Step 4-2: プレイリスト名の検証機能
    pub fn validate_playlist_name(name: &str, existing_playlists: &[Playlist], excluding_id: Option<&str>) -> String {
        // 空文字・空白のみの名前をチェック
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return "新しいプレイリスト".to_string();
        }

        // 不正文字をチェック（ファイルシステムで問題となる文字）
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        let cleaned_name = trimmed_name.chars()
            .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
            .collect::<String>();

        // 長すぎる名前を制限（最大100文字）
        let truncated_name = if cleaned_name.len() > 100 {
            format!("{}...", &cleaned_name[..97])
        } else {
            cleaned_name
        };

        // 重複チェック
        let mut unique_name = truncated_name.clone();
        let mut counter = 1;
        
        while existing_playlists.iter().any(|p| {
            if let Some(exclude_id) = excluding_id {
                p.id != exclude_id && p.name == unique_name
            } else {
                p.name == unique_name
            }
        }) {
            counter += 1;
            unique_name = if counter == 2 {
                format!("{} ({})", truncated_name, counter)
            } else {
                // 既存の番号を置き換え
                if let Some(pos) = truncated_name.rfind(" (") {
                    format!("{} ({})", &truncated_name[..pos], counter)
                } else {
                    format!("{} ({})", truncated_name, counter)
                }
            };
        }

        unique_name
    }

    pub fn delete_playlist(&mut self, id: &str) -> bool {
        if id == "default" {
            return false; // デフォルトプレイリストは削除不可
        }
        
        if let Some(index) = self.playlists.iter().position(|p| p.id == id) {
            self.playlists.remove(index);
            
            // アクティブプレイリストが削除された場合はデフォルトに切り替え
            if self.active_playlist_id == id {
                self.active_playlist_id = "default".to_string();
                self.selected_indices.clear();
        self.last_selected_index = None;
            }
            
            // 現在再生中のプレイリストが削除された場合は再生状態をリセット
            if self.current_playing_playlist_id.as_deref() == Some(id) {
                self.current_playing_index = None;
                self.current_playing_playlist_id = None;
            }
            true
        } else {
            false
        }
    }

    pub fn rename_playlist(&mut self, id: &str, new_name: String) -> bool {
        if id == "default" {
            return false; // デフォルトプレイリストは名前変更不可
        }
        
        // Step 4-2: 名前変更時にも検証を実行（借用の競合を回避）
        let validated_name = Self::validate_playlist_name(&new_name, &self.playlists, Some(id));
        
        if let Some(playlist) = self.playlists.iter_mut().find(|p| p.id == id) {
            playlist.name = validated_name;
            playlist.modified_at = SystemTime::now();
            true
        } else {
            false
        }
    }

    pub fn get_playlist(&self, id: &str) -> Option<&Playlist> {
        self.playlists.iter().find(|p| p.id == id)
    }

    pub fn get_playlist_mut(&mut self, id: &str) -> Option<&mut Playlist> {
        self.playlists.iter_mut().find(|p| p.id == id)
    }

    pub fn get_playlists(&self) -> &Vec<Playlist> {
        &self.playlists
    }

    // アクティブプレイリスト管理
    pub fn set_active_playlist(&mut self, id: &str) -> bool {
        if self.playlists.iter().any(|p| p.id == id) {
            self.active_playlist_id = id.to_string();
            self.selected_indices.clear();
        self.last_selected_index = None;
            
            // 重要：プレイリスト切り替え時は再生状態を保持する
            // current_playing_index と current_playing_playlist_id は現在再生中の楽曲の管理情報であり、
            // プレイリスト表示切り替えとは独立して管理する
            
            true
        } else {
            false
        }
    }

    pub fn get_active_playlist_id(&self) -> &str {
        &self.active_playlist_id
    }

    pub fn get_active_playlist(&self) -> Option<&Playlist> {
        self.get_playlist(&self.active_playlist_id)
    }

    pub fn get_active_playlist_mut(&mut self) -> Option<&mut Playlist> {
        let id = self.active_playlist_id.clone();
        self.get_playlist_mut(&id)
    }

    pub fn get_current_playing_playlist(&self) -> Option<&Playlist> {
        if let Some(ref playlist_id) = self.current_playing_playlist_id {
            self.get_playlist(playlist_id)
        } else {
            self.get_active_playlist()
        }
    }

    // 楽曲操作（アクティブプレイリストに対して）
    pub fn add_track(&mut self, track: TrackInfo) {
        let active_id = self.active_playlist_id.clone();
        if let Some(playlist) = self.get_playlist_mut(&active_id) {
            playlist.add_track(track);
        }
    }

    // 指定されたプレイリストに楽曲を追加（重複チェック付き）
    pub fn add_track_to_playlist(&mut self, playlist_id: &str, track: TrackInfo) -> Result<(), String> {
        if let Some(playlist) = self.get_playlist_mut(playlist_id) {
            if playlist.contains_track(&track) {
                return Err("既に同一楽曲が存在するため追加できません".to_string());
            }
            playlist.add_track(track);
            Ok(())
        } else {
            Err("対象のプレイリストが見つかりません".to_string())
        }
    }

    pub fn remove_track(&mut self, index: usize) -> Option<TrackInfo> {
        // 先に現在再生中のトラックが削除される場合の処理
        if let Some(current_index) = self.current_playing_index {
            if index == current_index {
                self.current_playing_index = None;
            } else if index < current_index {
                self.current_playing_index = Some(current_index - 1);
            }
        }
        
        // 選択状態の更新
        self.selected_indices.remove(&index);
        let mut new_selected = HashSet::new();
        for &selected_index in &self.selected_indices {
            if selected_index > index {
                new_selected.insert(selected_index - 1);
            } else {
                new_selected.insert(selected_index);
            }
        }
        self.selected_indices = new_selected;
        
        // 最後にプレイリストから削除
        let active_id = self.active_playlist_id.clone();
        self.get_playlist_mut(&active_id)?.remove_track(index)
    }

    pub fn move_track(&mut self, from: usize, to: usize) -> bool {
        let active_id = self.active_playlist_id.clone();
        if let Some(playlist) = self.get_playlist_mut(&active_id) {
            if playlist.move_track(from, to) {
                // 現在再生中のインデックスの更新
                if let Some(current_index) = self.current_playing_index {
                    if current_index == from {
                        self.current_playing_index = Some(to);
                    } else if from < current_index && to >= current_index {
                        self.current_playing_index = Some(current_index - 1);
                    } else if from > current_index && to <= current_index {
                        self.current_playing_index = Some(current_index + 1);
                    }
                }
                
                // 選択状態の更新（簡略化のため一旦クリア）
                self.selected_indices.clear();
        self.last_selected_index = None;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn clear_active_playlist(&mut self) {
        let active_id = self.active_playlist_id.clone();
        if let Some(playlist) = self.get_playlist_mut(&active_id) {
            playlist.clear();
        }
        self.selected_indices.clear();
        self.last_selected_index = None;
        
        // アクティブプレイリストが現在再生中のプレイリストの場合、再生状態もリセット
        if self.current_playing_playlist_id.as_deref() == Some(&active_id) {
            self.current_playing_index = None;
            self.current_playing_playlist_id = None;
        }
    }

    // 選択管理
    pub fn get_selected_indices(&self) -> &HashSet<usize> {
        &self.selected_indices
    }

    pub fn set_selected(&mut self, index: usize, selected: bool) {
        if selected {
            self.selected_indices.insert(index);
        } else {
            self.selected_indices.remove(&index);
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_indices.clear();
        self.last_selected_index = None;
    }

    pub fn select_all(&mut self) {
        self.selected_indices.clear();
        if let Some(playlist) = self.get_active_playlist() {
            for i in 0..playlist.tracks.len() {
                self.selected_indices.insert(i);
            }
        }
        // 全選択時は最後の選択インデックスをクリア
        self.last_selected_index = None;
    }

    // 重複しないプレイリスト名を生成
    fn generate_unique_playlist_name(&self) -> String {
        let base_name = "新しいプレイリスト";
        let mut counter = 1;
        
        // 重複チェック
        loop {
            let candidate = if counter == 1 {
                base_name.to_string()
            } else {
                format!("{} {}", base_name, counter)
            };
            
            if !self.playlists.iter().any(|p| p.name == candidate) {
                return candidate;
            }
            counter += 1;
        }
    }

    // 新しいプレイリストを作成して選択中の楽曲をコピー
    pub fn copy_selected_to_new_playlist(&mut self) -> Result<String, String> {
        let selected_tracks: Vec<TrackInfo> = if let Some(playlist) = self.get_active_playlist() {
            self.selected_indices.iter()
                .filter_map(|&index| playlist.tracks.get(index).cloned())
                .collect()
        } else {
            return Err("アクティブなプレイリストが見つかりません".to_string());
        };

        if selected_tracks.is_empty() {
            return Err("選択された楽曲がありません".to_string());
        }

        let playlist_name = self.generate_unique_playlist_name();
        let playlist_id = format!("playlist_{}", Uuid::new_v4().to_string().replace("-", ""));
        
        let mut new_playlist = Playlist::new(playlist_id.clone(), playlist_name.clone());
        for track in selected_tracks {
            new_playlist.add_track(track);
        }
        
        self.playlists.push(new_playlist);
        Ok(playlist_id)
    }

    // 新しいプレイリストを作成して選択中の楽曲を移動
    pub fn move_selected_to_new_playlist(&mut self) -> Result<String, String> {
        let selected_tracks: Vec<TrackInfo> = if let Some(playlist) = self.get_active_playlist() {
            self.selected_indices.iter()
                .filter_map(|&index| playlist.tracks.get(index).cloned())
                .collect()
        } else {
            return Err("アクティブなプレイリストが見つかりません".to_string());
        };

        if selected_tracks.is_empty() {
            return Err("選択された楽曲がありません".to_string());
        }

        let playlist_name = self.generate_unique_playlist_name();
        let playlist_id = format!("playlist_{}", Uuid::new_v4().to_string().replace("-", ""));
        
        let mut new_playlist = Playlist::new(playlist_id.clone(), playlist_name.clone());
        for track in selected_tracks {
            new_playlist.add_track(track);
        }
        
        self.playlists.push(new_playlist);
        
        // 元のプレイリストから選択楽曲を削除
        self.remove_selected();
        
        Ok(playlist_id)
    }

    // 単一楽曲から新しいプレイリストを作成
    pub fn create_playlist_with_track(&mut self, track: TrackInfo) -> Result<String, String> {
        let playlist_name = self.generate_unique_playlist_name();
        let playlist_id = format!("playlist_{}", Uuid::new_v4().to_string().replace("-", ""));
        
        let mut new_playlist = Playlist::new(playlist_id.clone(), playlist_name.clone());
        new_playlist.add_track(track);
        
        self.playlists.push(new_playlist);
        Ok(playlist_id)
    }

    // 複数楽曲から新しいプレイリストを作成（アルバム・アーティスト用）
    pub fn create_playlist_with_tracks(&mut self, tracks: Vec<TrackInfo>) -> Result<String, String> {
        if tracks.is_empty() {
            return Err("楽曲が選択されていません".to_string());
        }

        let playlist_name = self.generate_unique_playlist_name();
        let playlist_id = format!("playlist_{}", Uuid::new_v4().to_string().replace("-", ""));
        
        let mut new_playlist = Playlist::new(playlist_id.clone(), playlist_name.clone());
        for track in tracks {
            new_playlist.add_track(track);
        }
        
        self.playlists.push(new_playlist);
        Ok(playlist_id)
    }

    pub fn is_selected(&self, index: usize) -> bool {
        self.selected_indices.contains(&index)
    }

    // 再生管理
    pub fn get_current_playing_index(&self) -> Option<usize> {
        self.current_playing_index
    }

    pub fn set_current_playing_index(&mut self, index: Option<usize>) {
        
        self.current_playing_index = index;
        if index.is_some() {
            self.current_playing_playlist_id = Some(self.active_playlist_id.clone());
        } else {
            self.current_playing_playlist_id = None;
        }
        
    }
    
    // 特定のプレイリストでの再生状態を設定するメソッド
    pub fn set_current_playing_with_playlist(&mut self, index: Option<usize>, playlist_id: String) {
        
        self.current_playing_index = index;
        if index.is_some() {
            self.current_playing_playlist_id = Some(playlist_id);
        } else {
            self.current_playing_playlist_id = None;
        }
        
    }

    pub fn get_current_track(&self) -> Option<&TrackInfo> {
        // 現在再生中の楽曲は、現在再生中のプレイリストから取得する
        if let (Some(playing_playlist_id), Some(index)) = (&self.current_playing_playlist_id, self.current_playing_index) {
            self.get_playlist(playing_playlist_id)
                .and_then(|playlist| playlist.get_track(index))
        } else {
            None
        }
    }
    
    pub fn get_current_playing_playlist_id(&self) -> Option<&str> {
        self.current_playing_playlist_id.as_deref()
    }

    // 便利メソッド
    pub fn get_active_tracks(&self) -> Option<&Vec<TrackInfo>> {
        self.get_active_playlist().map(|p| p.get_tracks())
    }

    pub fn get_active_track_count(&self) -> usize {
        self.get_active_playlist().map_or(0, |p| p.len())
    }

    pub fn is_active_playlist_empty(&self) -> bool {
        self.get_active_playlist().map_or(true, |p| p.is_empty())
    }

    // 再生制御メソッド（PlaybackQueueからの移行）
    pub fn move_to_next(&mut self) -> Option<TrackInfo> {
        // 現在再生中のプレイリストから次の楽曲を取得
        let playing_playlist_id = self.current_playing_playlist_id.clone()
            .unwrap_or_else(|| self.active_playlist_id.clone());
            
        let track_count = self.playlists.iter()
            .find(|p| p.id == playing_playlist_id)
            .map(|p| p.tracks.len())
            .unwrap_or(0);

        if track_count == 0 {
            return None;
        }

        let next_index = if let Some(current_index) = self.current_playing_index {
            current_index + 1
        } else {
            0
        };

        if next_index < track_count {
            self.set_current_playing_with_playlist(Some(next_index), playing_playlist_id.clone());
            
            self.playlists.iter()
                .find(|p| p.id == playing_playlist_id)
                .and_then(|p| p.tracks.get(next_index))
                .cloned()
        } else {
            None
        }
    }

    pub fn move_to_previous(&mut self) -> Option<TrackInfo> {
        if let Some(current_index) = self.current_playing_index {
            if current_index > 0 {
                let prev_index = current_index - 1;
                
                // 現在再生中のプレイリストから前の楽曲を取得
                let playing_playlist_id = self.current_playing_playlist_id.clone()
                    .unwrap_or_else(|| self.active_playlist_id.clone());
                
                self.set_current_playing_with_playlist(Some(prev_index), playing_playlist_id.clone());
                
                return self.playlists.iter()
                    .find(|p| p.id == playing_playlist_id)
                    .and_then(|p| p.tracks.get(prev_index))
                    .cloned();
            }
        }
        None
    }

    // PlaybackQueueの選択操作との互換性
    pub fn handle_item_selection(&mut self, index: usize, ctrl_held: bool, shift_held: bool) {
        if shift_held {
            // 範囲選択
            if let Some(last_selected) = self.last_selected_index {
                self.selected_indices.clear();
        self.last_selected_index = None;
                let start = last_selected.min(index);
                let end = last_selected.max(index);
                for i in start..=end {
                    self.selected_indices.insert(i);
                }
            } else {
                // 最後の選択がない場合は通常の選択
                self.selected_indices.clear();
        self.last_selected_index = None;
                self.selected_indices.insert(index);
            }
        } else if ctrl_held {
            // 複数選択のトグル
            if self.selected_indices.contains(&index) {
                self.selected_indices.remove(&index);
            } else {
                self.selected_indices.insert(index);
            }
        } else {
            // 単一選択
            self.selected_indices.clear();
        self.last_selected_index = None;
            self.selected_indices.insert(index);
        }
        
        // 最後に選択されたインデックスを更新（範囲選択時以外）
        if !shift_held {
            self.last_selected_index = Some(index);
        }
    }

    pub fn remove_selected(&mut self) {
        let mut indices_to_remove: Vec<usize> = self.selected_indices.iter().cloned().collect();
        indices_to_remove.sort_by(|a, b| b.cmp(a)); // 後ろから削除

        for index in indices_to_remove {
            self.remove_track(index);
        }
        self.selected_indices.clear();
        self.last_selected_index = None;
    }

    // プレイリスト内での移動操作
    pub fn move_selected_up(&mut self) {
        let mut indices: Vec<usize> = self.selected_indices.iter().cloned().collect();
        indices.sort();

        for index in indices {
            if index > 0 {
                self.move_track(index, index - 1);
                // 選択状態を更新
                self.selected_indices.remove(&index);
                self.selected_indices.insert(index - 1);
            }
        }
    }

    pub fn move_selected_down(&mut self) {
        let mut indices: Vec<usize> = self.selected_indices.iter().cloned().collect();
        indices.sort_by(|a, b| b.cmp(a)); // 後ろから処理

        let max_index = self.get_active_track_count().saturating_sub(1);
        for index in indices {
            if index < max_index {
                self.move_track(index, index + 1);
                // 選択状態を更新
                self.selected_indices.remove(&index);
                self.selected_indices.insert(index + 1);
            }
        }
    }

    pub fn move_selected_to_top(&mut self) {
        let mut indices: Vec<usize> = self.selected_indices.iter().cloned().collect();
        indices.sort();

        self.selected_indices.clear();
        self.last_selected_index = None;
        for (new_pos, index) in indices.into_iter().enumerate() {
            self.move_track(index - new_pos, new_pos);
            self.selected_indices.insert(new_pos);
        }
    }

    pub fn move_selected_to_bottom(&mut self) {
        let mut indices: Vec<usize> = self.selected_indices.iter().cloned().collect();
        indices.sort_by(|a, b| b.cmp(a));

        let track_count = self.get_active_track_count();
        self.selected_indices.clear();
        self.last_selected_index = None;
        
        for (offset, index) in indices.into_iter().enumerate() {
            let new_pos = track_count - 1 - offset;
            self.move_track(index, new_pos);
            self.selected_indices.insert(new_pos);
        }
    }

    pub fn clear(&mut self) {
        self.clear_active_playlist();
    }

    pub fn set_current_index(&mut self, index: usize) {
        if let Some(tracks) = self.get_active_tracks() {
            if index < tracks.len() {
                self.set_current_playing_index(Some(index));
            }
        }
    }

    pub fn get_current_index(&self) -> Option<usize> {
        self.current_playing_index
    }

    pub fn get_tracks(&self) -> Option<&Vec<TrackInfo>> {
        self.get_active_tracks()
    }

    // Step 4-1: 設定管理メソッド
    pub fn get_current_active_playlist_id(&self) -> &str {
        &self.active_playlist_id
    }

    pub fn get_ordered_playlist_ids(&self) -> Vec<String> {
        self.playlists.iter().map(|p| p.id.clone()).collect()
    }

    pub fn apply_display_order(&mut self, order: &[String]) {
        // 指定された順序でプレイリストを並び替え
        let mut ordered_playlists = Vec::new();
        let mut remaining_playlists = self.playlists.clone();

        // 順序指定されたプレイリストから追加
        for id in order {
            if let Some(pos) = remaining_playlists.iter().position(|p| p.id == *id) {
                ordered_playlists.push(remaining_playlists.remove(pos));
            }
        }

        // 順序指定されていないプレイリストを末尾に追加
        ordered_playlists.extend(remaining_playlists);

        self.playlists = ordered_playlists;
    }

    pub fn reorder_playlist(&mut self, from_index: usize, to_index: usize) -> bool {
        if from_index < self.playlists.len() && to_index < self.playlists.len() && from_index != to_index {
            let playlist = self.playlists.remove(from_index);
            self.playlists.insert(to_index, playlist);
            true
        } else {
            false
        }
    }

    // デフォルトプレイリスト設定の適用
    pub fn apply_default_playlist_settings(&mut self, _settings: &crate::settings::DefaultPlaylistSettings) {
        // 設定項目が削除されたため、現在は何も処理しない
    }

    // Step 4-3: パフォーマンス最適化メソッド
    
    /// 遅延ファイル存在チェック（大量プレイリスト用）
    pub fn validate_tracks_lazy(&mut self, playlist_id: &str) -> Result<usize, String> {
        let removed_count = if let Some(playlist) = self.get_playlist_mut(playlist_id) {
            let original_count = playlist.tracks.len();
            
            playlist.tracks.retain(|track| track.path.exists());
            
            let removed_count = original_count - playlist.tracks.len();
            if removed_count > 0 {
                playlist.modified_at = SystemTime::now();
                eprintln!("Info: Validated playlist '{}': removed {} missing track(s)", 
                         playlist.name, removed_count);
            }
            
            removed_count
        } else {
            return Err("Playlist not found".to_string());
        };
        
        Ok(removed_count)
    }

    /// プレイリストのメモリ使用量最適化（大量楽曲用）
    pub fn optimize_memory(&mut self) {
        for playlist in &mut self.playlists {
            // 楽曲ベクターの容量を実際のサイズに最適化
            playlist.tracks.shrink_to_fit();
        }
        
        // プレイリストベクターの容量も最適化
        self.playlists.shrink_to_fit();
    }

    /// 高速プレイリスト統計（メタデータ無し）
    pub fn get_quick_stats(&self) -> (usize, usize) {
        let total_playlists = self.playlists.len();
        let total_tracks: usize = self.playlists.iter().map(|p| p.tracks.len()).sum();
        (total_playlists, total_tracks)
    }

    /// 大量プレイリストでの効率的な楽曲検索
    pub fn find_track_in_playlists(&self, track_path: &std::path::Path) -> Vec<String> {
        let mut found_in = Vec::new();
        
        for playlist in &self.playlists {
            if playlist.tracks.iter().any(|t| t.path == track_path) {
                found_in.push(playlist.id.clone());
            }
        }
        
        found_in
    }

    // リピート・シャッフル機能
    pub fn generate_shuffle_order(&mut self) {
        let playlist = match self.get_current_playing_playlist() {
            Some(p) => p.clone(),
            None => return,
        };
        
        let track_count = playlist.tracks.len();
        if track_count == 0 {
            self.shuffle_order.clear();
            self.shuffle_position = None;
            return;
        }

        // 0からtrack_count-1までのインデックスを作成
        self.shuffle_order = (0..track_count).collect();
        
        // Fisher-Yates シャッフルアルゴリズム
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::SystemTime;
        
        let mut hasher = DefaultHasher::new();
        SystemTime::now().hash(&mut hasher);
        let seed = hasher.finish();
        let mut rng_state = seed;
        
        for i in (1..track_count).rev() {
            // 簡単な線形合同法でランダム数生成
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let j = (rng_state as usize) % (i + 1);
            self.shuffle_order.swap(i, j);
        }
        
        // 現在の再生インデックスがある場合、シャッフル順序内での位置を見つける
        if let Some(current_index) = self.current_playing_index {
            self.shuffle_position = self.shuffle_order.iter().position(|&x| x == current_index);
        } else {
            self.shuffle_position = None;
        }
    }

    pub fn move_to_next_with_modes(&mut self, repeat_mode: &RepeatMode, shuffle_enabled: bool) -> Option<TrackInfo> {
        // プレイリストのクローンを取得して借用の問題を回避
        let playlist = self.get_current_playing_playlist()?.clone();
        let track_count = playlist.tracks.len();
        if track_count == 0 {
            return None;
        }

        if shuffle_enabled {
            // シャッフル再生
            if self.shuffle_order.is_empty() {
                self.generate_shuffle_order();
            }

            if let Some(current_pos) = self.shuffle_position {
                match repeat_mode {
                    RepeatMode::RepeatOne => {
                        // 1曲リピート：シャッフル順序に関係なく現在の曲をリピート
                        let track_index = self.shuffle_order[current_pos];
                        return Some(playlist.tracks[track_index].clone());
                    }
                    _ => {
                        // 通常再生またはプレイリストリピート
                        let next_pos = current_pos + 1;
                        if next_pos < self.shuffle_order.len() {
                            // 次の曲があるので移動
                            self.shuffle_position = Some(next_pos);
                            let track_index = self.shuffle_order[next_pos];
                            self.current_playing_index = Some(track_index);
                            return Some(playlist.tracks[track_index].clone());
                        } else {
                            // シャッフル順序の最後に到達
                            match repeat_mode {
                                RepeatMode::RepeatAll => {
                                    // 新しいシャッフル順序を生成して最初から
                                    self.generate_shuffle_order();
                                    if !self.shuffle_order.is_empty() {
                                        self.shuffle_position = Some(0);
                                        let track_index = self.shuffle_order[0];
                                        self.current_playing_index = Some(track_index);
                                        return Some(playlist.tracks[track_index].clone());
                                    }
                                }
                                RepeatMode::Normal => {
                                    // リピートなし時は停止
                                    self.current_playing_index = None;
                                    self.shuffle_position = None;
                                    return None;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            } else {
                // シャッフル位置が不明な場合、最初から開始
                if !self.shuffle_order.is_empty() {
                    self.shuffle_position = Some(0);
                    let track_index = self.shuffle_order[0];
                    self.current_playing_index = Some(track_index);
                    return Some(playlist.tracks[track_index].clone());
                }
            }
        } else {
            // 通常再生
            if let Some(current_index) = self.current_playing_index {
                match repeat_mode {
                    RepeatMode::RepeatOne => {
                        // 1曲リピート：同じ曲を返す
                        return Some(playlist.tracks[current_index].clone());
                    }
                    RepeatMode::RepeatAll => {
                        // プレイリストリピート
                        let next_index = if current_index + 1 < track_count {
                            current_index + 1
                        } else {
                            0
                        };
                        self.current_playing_index = Some(next_index);
                        return Some(playlist.tracks[next_index].clone());
                    }
                    RepeatMode::Normal => {
                        // 通常再生
                        if current_index + 1 < track_count {
                            let next_index = current_index + 1;
                            self.current_playing_index = Some(next_index);
                            return Some(playlist.tracks[next_index].clone());
                        } else {
                            // プレイリストの最後に到達
                            self.current_playing_index = None;
                            return None;
                        }
                    }
                }
            } else {
                // 現在のインデックスがない場合、最初の曲を開始
                if track_count > 0 {
                    self.current_playing_index = Some(0);
                    return Some(playlist.tracks[0].clone());
                }
            }
        }
        None
    }

    pub fn move_to_previous_with_modes(&mut self, shuffle_enabled: bool) -> Option<TrackInfo> {
        let playlist = self.get_current_playing_playlist()?.clone();
        let track_count = playlist.tracks.len();
        if track_count == 0 {
            return None;
        }

        if shuffle_enabled {
            // シャッフル再生での前の曲
            if let Some(current_pos) = self.shuffle_position {
                if current_pos > 0 {
                    let prev_pos = current_pos - 1;
                    self.shuffle_position = Some(prev_pos);
                    let track_index = self.shuffle_order[prev_pos];
                    self.current_playing_index = Some(track_index);
                    return Some(playlist.tracks[track_index].clone());
                }
            }
        } else {
            // 通常再生での前の曲
            if let Some(current_index) = self.current_playing_index {
                if current_index > 0 {
                    let prev_index = current_index - 1;
                    self.current_playing_index = Some(prev_index);
                    return Some(playlist.tracks[prev_index].clone());
                }
            }
        }
        None
    }

    pub fn update_shuffle_when_settings_changed(&mut self, shuffle_enabled: bool) {
        if shuffle_enabled && self.shuffle_order.is_empty() {
            self.generate_shuffle_order();
        } else if !shuffle_enabled {
            // シャッフル無効時はシャッフル情報をクリア
            self.shuffle_order.clear();
            self.shuffle_position = None;
        }
    }
}