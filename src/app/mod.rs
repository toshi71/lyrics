pub mod handlers;
pub mod ui_main;
pub mod ui_playlist;
pub mod ui_settings;

use eframe::egui;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;

use crate::music::{MusicLibrary, TrackInfo};
use crate::player::{AudioPlayer, PlaybackState};
use crate::playlist::PlaylistManager;
use crate::settings::Settings;

pub use state::{UIState, SelectionState, PlayerState, PlaylistEditState, CoverArtCache, Tab, RightTab};
mod state;


pub struct MyApp {
    pub ui_state: UIState,
    pub selection_state: SelectionState,
    pub settings: Settings,
    pub music_library: MusicLibrary,
    pub last_selected_path: Option<std::path::PathBuf>,
    pub audio_player: AudioPlayer,
    pub playlist_manager: PlaylistManager,
    pub editing_playlist_id: Option<String>,
    pub editing_playlist_name: String,
    pub seek_drag_state: Option<PlaybackState>,
    #[allow(dead_code)]
    pub cover_art_cache: std::collections::HashMap<std::path::PathBuf, egui::TextureHandle>,
    pub repeat_mode: crate::settings::RepeatMode,
    pub shuffle_enabled: bool,
}

impl MyApp {
    pub fn new() -> Self {
        let settings = Settings::load();
        let mut app = Self {
            ui_state: UIState::new(&settings),
            selection_state: SelectionState::new(),
            music_library: MusicLibrary::new(settings.classical_composer_hierarchy),
            last_selected_path: None,
            audio_player: AudioPlayer::new(),
            playlist_manager: {
                let mut manager = PlaylistManager::auto_load().unwrap_or_else(|_| {
                    PlaylistManager::new_with_settings(
                        settings.get_last_used_playlist_id(),
                        settings.get_playlist_display_order()
                    )
                });
                
                manager.apply_default_playlist_settings(&settings.default_playlist_settings);
                
                // 起動時にデフォルトプレイリストをクリア
                if let Some(default_playlist) = manager.get_playlist_mut("default") {
                    default_playlist.clear();
                }
                
                manager
            },
            editing_playlist_id: None,
            editing_playlist_name: String::new(),
            seek_drag_state: None,
            cover_art_cache: std::collections::HashMap::new(),
            repeat_mode: crate::settings::RepeatMode::Normal,
            shuffle_enabled: false,
            settings,
        };
        app.refresh_music_library();
        app
    }

    pub fn save_settings(&mut self) {
        self.settings.set_last_used_playlist(self.playlist_manager.get_current_active_playlist_id().to_string());
        self.settings.update_playlist_display_order(self.playlist_manager.get_ordered_playlist_ids());
        
        // 分割比率を保存
        self.ui_state.save_to_settings(&mut self.settings);
        
        self.playlist_manager.optimize_memory();
        
        let _ = self.settings.save();
        let _ = self.playlist_manager.auto_save();
    }

    pub fn refresh_music_library(&mut self) {
        if !self.settings.target_directory.is_empty() {
            let target_path = std::path::PathBuf::from(&self.settings.target_directory);
            
            eprintln!("Info: Scanning music directory: {}", target_path.display());
            let start_time = std::time::Instant::now();
            
            self.music_library.scan_directory(&target_path);
            self.apply_search_filter();
            
            let duration = start_time.elapsed();
            let track_count = self.music_library.get_track_count();
            eprintln!("Info: Scanned {} tracks in {:.2}s", track_count, duration.as_secs_f64());
            
            if track_count > 1000 {
                eprintln!("Info: Large library detected. Consider using search filters for better performance.");
            }
        }
    }

    pub fn apply_search_filter(&mut self) {
        self.music_library.apply_search_filter(&self.selection_state.search_query);
    }

    pub fn check_playback_finished(&mut self) {
        // 楽曲が終了したかチェック
        if *self.audio_player.get_state() == PlaybackState::Playing && self.audio_player.is_finished() {
            // 現在の楽曲が終了した場合、リピート・シャッフルモードに応じて次の楽曲を自動再生
            let repeat_mode = &self.repeat_mode;
            let shuffle_enabled = self.shuffle_enabled;
            
            if let Some(next_track) = self.playlist_manager.move_to_next_with_modes(repeat_mode, shuffle_enabled) {
                if let Err(_) = self.audio_player.play(next_track) {
                    // エラーの場合は停止状態にする
                    self.audio_player.stop();
                }
            } else {
                // 次の楽曲がない場合は停止
                self.audio_player.stop();
                self.playlist_manager.set_current_playing_index(None);
            }
        }
    }
}

impl eframe::App for MyApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_settings();
    }
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // テーマの適用
        if self.settings.is_dark_mode() {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        
        self.handle_keyboard_shortcuts(ctx);
        self.check_playback_finished(); // 楽曲終了チェック
        self.show_menu_bar(ctx);
        self.show_tab_bar(ctx);
        self.show_central_panel(ctx);
        self.show_dialog_if_needed(ctx);
    }
}

pub fn setup_custom_fonts(ctx: &egui::Context, settings: &Settings) {
    let mut fonts = egui::FontDefinitions::default();
    let system_source = SystemSource::new();
    
    let font_name = &settings.selected_font;
    
    if let Ok(font) = system_source.select_best_match(&[FamilyName::Title(font_name.clone())], &Properties::new()) {
        if let Ok(font_data) = font.load() {
            if let Some(font_data_vec) = font_data.copy_font_data() {
                fonts.font_data.insert(
                    "selected_font".to_owned(),
                    egui::FontData::from_owned(font_data_vec.as_ref().clone()),
                );
                
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "selected_font".to_owned());
                
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "selected_font".to_owned());
                
                eprintln!("Info: Successfully loaded font: {}", font_name);
            } else {
                eprintln!("Warning: Could not extract font data for: {}", font_name);
            }
        } else {
            eprintln!("Warning: Could not load font file for: {}", font_name);
        }
    } else {
        eprintln!("Warning: Font not found: {}", font_name);
    }
    
    ctx.set_fonts(fonts);
}