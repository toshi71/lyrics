use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::music::TrackInfo;
use crate::player::{AudioPlayer, PlaybackState};
use crate::settings::{Settings, RepeatMode};

#[derive(PartialEq, Debug)]
pub enum Tab {
    Main,
    Settings,
}

#[derive(PartialEq, Debug)]
pub enum RightTab {
    Playback,
    Info,
    Lrc,
}

/// UI状態管理
pub struct UIState {
    pub show_dialog: bool,
    pub current_tab: Tab,
    pub right_pane_tab: RightTab,
    pub splitter_position: f32,
    pub right_top_bottom_position: f32,
    pub right_bottom_left_right_position: f32,
    pub should_focus_controls: bool,
}

impl UIState {
    pub fn new(settings: &Settings) -> Self {
        Self {
            show_dialog: false,
            current_tab: Tab::Main,
            right_pane_tab: RightTab::Info,
            splitter_position: settings.main_splitter_position,
            right_top_bottom_position: settings.right_top_bottom_position,
            right_bottom_left_right_position: settings.right_bottom_left_right_position,
            should_focus_controls: false,
        }
    }

    pub fn save_to_settings(&self, settings: &mut Settings) {
        settings.main_splitter_position = self.splitter_position;
        settings.right_top_bottom_position = self.right_top_bottom_position;
        settings.right_bottom_left_right_position = self.right_bottom_left_right_position;
    }
}

/// 選択・検索状態管理
pub struct SelectionState {
    pub selected_track: Option<TrackInfo>,
    pub selected_tracks: HashSet<PathBuf>,
    pub last_selected_path: Option<PathBuf>,
    pub search_query: String,
    pub focus_search: bool,
    pub search_has_focus: bool,
}

impl SelectionState {
    pub fn new() -> Self {
        Self {
            selected_track: None,
            selected_tracks: HashSet::new(),
            last_selected_path: None,
            search_query: String::new(),
            focus_search: false,
            search_has_focus: false,
        }
    }
}

/// プレイヤー状態管理
pub struct PlayerState {
    pub audio_player: AudioPlayer,
    pub seek_drag_state: Option<PlaybackState>,
    pub repeat_mode: RepeatMode,
    pub shuffle_enabled: bool,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            audio_player: AudioPlayer::new(),
            seek_drag_state: None,
            repeat_mode: RepeatMode::Normal,
            shuffle_enabled: false,
        }
    }
}

/// プレイリスト編集状態
pub struct PlaylistEditState {
    pub editing_playlist_id: Option<String>,
    pub editing_playlist_name: String,
}

impl PlaylistEditState {
    pub fn new() -> Self {
        Self {
            editing_playlist_id: None,
            editing_playlist_name: String::new(),
        }
    }
}

/// カバーアートキャッシュ管理
pub struct CoverArtCache {
    cache: HashMap<PathBuf, egui::TextureHandle>,
}

impl CoverArtCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get(&self, path: &PathBuf) -> Option<&egui::TextureHandle> {
        self.cache.get(path)
    }

    pub fn insert(&mut self, path: PathBuf, handle: egui::TextureHandle) {
        self.cache.insert(path, handle);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn contains_key(&self, path: &PathBuf) -> bool {
        self.cache.contains_key(path)
    }
}