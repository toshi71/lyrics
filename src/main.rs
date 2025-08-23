mod music;
mod player;
mod playlist;
mod settings;
mod ui;

use eframe::egui;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use music::MusicTreeNode;

use music::{MusicLibrary, TrackInfo};
use player::{AudioPlayer, PlaybackState};
use playlist::PlaylistManager;
use settings::Settings;
use ui::{MusicTreeUI, PlaybackControlsUI, SearchUI};


fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_title("Hello World GUI"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Hello World GUI",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(MyApp::new()))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    let system_source = SystemSource::new();
    
    if let Ok(font) = system_source.select_best_match(&[FamilyName::Title("Meiryo".to_owned())], &Properties::new()) {
        if let Ok(font_data) = font.load() {
            if let Some(font_data_vec) = font_data.copy_font_data() {
                fonts.font_data.insert(
                    "meiryo".to_owned(),
                    egui::FontData::from_owned(font_data_vec.as_ref().clone()),
                );
                
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "meiryo".to_owned());
                
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("meiryo".to_owned());
            }
        }
    }
    
    ctx.set_fonts(fonts);
}

#[derive(PartialEq)]
enum Tab {
    Main,
    Settings,
}

#[derive(PartialEq)]
enum RightTab {
    Playback,
    Info,
    Lrc,
}

struct MyApp {
    show_dialog: bool,
    current_tab: Tab,
    settings: Settings,
    music_library: MusicLibrary,
    search_query: String,
    focus_search: bool,
    splitter_position: f32,
    right_pane_tab: RightTab,
    selected_track: Option<TrackInfo>,
    selected_tracks: std::collections::HashSet<std::path::PathBuf>, // Multiple selection support
    last_selected_path: Option<std::path::PathBuf>, // For range selection
    audio_player: AudioPlayer,
    playlist_manager: PlaylistManager,
    // プレイリスト名編集用
    editing_playlist_id: Option<String>,
    editing_playlist_name: String,
}

impl MyApp {
    fn new() -> Self {
        let settings = Settings::load();
        let mut app = Self {
            show_dialog: false,
            current_tab: Tab::Main,
            music_library: MusicLibrary::new(settings.classical_composer_hierarchy),
            search_query: String::new(),
            focus_search: false,
            splitter_position: 0.33,
            right_pane_tab: RightTab::Playback,
            selected_track: None,
            selected_tracks: std::collections::HashSet::new(),
            last_selected_path: None,
            audio_player: AudioPlayer::new(),
            playlist_manager: PlaylistManager::auto_load().unwrap_or_else(|_| PlaylistManager::new()),
            editing_playlist_id: None,
            editing_playlist_name: String::new(),
            settings,
        };
        app.refresh_music_library();
        app
    }

    fn save_settings(&self) {
        let _ = self.settings.save();
        let _ = self.playlist_manager.auto_save();
    }

    fn refresh_music_library(&mut self) {
        if !self.settings.target_directory.is_empty() {
            let target_path = std::path::PathBuf::from(&self.settings.target_directory);
            self.music_library.scan_directory(&target_path);
            self.apply_search_filter();
        }
    }

    fn apply_search_filter(&mut self) {
        self.music_library.apply_search_filter(&self.search_query);
    }

    fn show_music_tree(&mut self, ui: &mut egui::Ui) {
        let mut track_selection = None;
        let mut double_clicked_track = None;
        let mut add_to_playlist_track: Option<(TrackInfo, String)> = None; // (track, playlist_id)
        let mut add_album_to_playlist: Option<(MusicTreeNode, String)> = None; // (album_node, playlist_id)
        let mut add_artist_to_playlist: Option<(MusicTreeNode, String)> = None; // (artist_node, playlist_id)
        
        MusicTreeUI::show(
            ui,
            self.music_library.get_tree_mut(),
            &self.search_query,
            self.selected_track.as_ref(),
            &self.selected_tracks,
            &self.playlist_manager.get_playlists(),
            &mut |track, ctrl_held, shift_held| track_selection = Some((track, ctrl_held, shift_held)),
            &mut |track| double_clicked_track = Some(track),
            &mut |track, playlist_id| add_to_playlist_track = Some((track, playlist_id)),
            &mut |node, playlist_id| add_album_to_playlist = Some((node.clone(), playlist_id)),
            &mut |node, playlist_id| add_artist_to_playlist = Some((node.clone(), playlist_id)),
        );
        
        if let Some((track, ctrl_held, shift_held)) = track_selection {
            self.handle_track_selection(track, ctrl_held, shift_held);
        }
        
        if let Some((track, playlist_id)) = add_to_playlist_track {
            self.handle_add_to_playlist(track, playlist_id);
        }
        
        if let Some((node, playlist_id)) = add_album_to_playlist {
            self.handle_add_album_to_playlist(node, playlist_id);
        }
        
        if let Some((node, playlist_id)) = add_artist_to_playlist {
            self.handle_add_artist_to_playlist(node, playlist_id);
        }
        
        // TODO: Implement queue addition on double-click
        // For now, double-click does nothing
        if let Some(_track) = double_clicked_track {
            // Double-click functionality will be implemented later as "add to queue"
        }
    }

    fn handle_track_selection(&mut self, track: TrackInfo, ctrl_held: bool, shift_held: bool) {
        if shift_held && self.last_selected_path.is_some() {
            // Range selection mode
            self.handle_range_selection(track.clone());
        } else if ctrl_held {
            // Multiple selection mode - preserve existing selections
            
            // If there's a currently selected single track, add it to multiple selections first
            if let Some(ref current_track) = self.selected_track {
                if !self.selected_tracks.contains(&current_track.path) {
                    self.selected_tracks.insert(current_track.path.clone());
                }
            }
            
            // Toggle the clicked track
            if self.selected_tracks.contains(&track.path) {
                // Deselect if already selected
                self.selected_tracks.remove(&track.path);
            } else {
                // Add to selection
                self.selected_tracks.insert(track.path.clone());
            }
            
            // Keep the last clicked track as the primary selection
            self.selected_track = Some(track.clone());
            self.last_selected_path = Some(track.path);
        } else {
            // Single selection mode - clear multiple selections
            self.selected_tracks.clear();
            self.selected_track = Some(track.clone());
            self.last_selected_path = Some(track.path);
        }
    }

    fn handle_range_selection(&mut self, end_track: TrackInfo) {
        let start_path = match &self.last_selected_path {
            Some(path) => path.clone(),
            None => return,
        };

        if start_path == end_track.path {
            // Same track - just select it
            self.selected_tracks.clear();
            self.selected_tracks.insert(end_track.path.clone());
            self.selected_track = Some(end_track);
            return;
        }

        // Get all tracks in display order and find the range
        let all_tracks = self.get_all_tracks_in_display_order();
        
        // Find indices of start and end tracks
        let start_index = all_tracks.iter().position(|t| t.path == start_path);
        let end_index = all_tracks.iter().position(|t| t.path == end_track.path);
        
        if let (Some(start_idx), Some(end_idx)) = (start_index, end_index) {
            // Clear current selection
            self.selected_tracks.clear();
            
            // Select range (inclusive)
            let (min_idx, max_idx) = if start_idx <= end_idx {
                (start_idx, end_idx)
            } else {
                (end_idx, start_idx)
            };
            
            for track in &all_tracks[min_idx..=max_idx] {
                self.selected_tracks.insert(track.path.clone());
            }
            
            self.selected_track = Some(end_track);
        } else {
            // Fallback to just selecting both tracks
            self.selected_tracks.clear();
            self.selected_tracks.insert(start_path);
            self.selected_tracks.insert(end_track.path.clone());
            self.selected_track = Some(end_track);
        }
    }

    fn get_all_tracks_in_display_order(&self) -> Vec<TrackInfo> {
        let mut tracks = Vec::new();
        // Access the tree through the music library's immutable reference
        self.music_library.collect_displayed_tracks(&mut tracks);
        tracks
    }


    // Removed play_track method - now using queue-only playback

    // Removed get_playable_track method - now using direct queue access

    fn handle_previous_button(&mut self) {
        let position = self.audio_player.get_playback_position();
        
        if position.as_secs() <= 3 {
            if let Some(prev_track) = self.playlist_manager.move_to_previous() {
                if let Err(_) = self.audio_player.play(prev_track) {
                    // Handle error silently
                }
            }
        } else {
            if let Err(_) = self.audio_player.restart_current() {
                // Handle error silently
            }
        }
    }

    fn handle_play_pause(&mut self) {
        match self.audio_player.get_state() {
            PlaybackState::Playing => {
                self.audio_player.pause();
            },
            PlaybackState::Paused => {
                self.audio_player.resume();
            },
            PlaybackState::Stopped => {
                // Only play from queue, no fallback to selected track
                if let Some(track) = self.playlist_manager.get_current_track() {
                    if let Err(_) = self.audio_player.play(track.clone()) {
                        // Handle error silently for now
                    }
                }
                // If queue is empty, do nothing
            },
        }
    }

    fn handle_stop(&mut self) {
        self.audio_player.stop();
        // 停止時は再生状態をクリア
        self.playlist_manager.set_current_playing_index(None);
    }

    fn handle_next(&mut self) {
        if let Some(next_track) = self.playlist_manager.move_to_next() {
            if let Err(_) = self.audio_player.play(next_track) {
                // Handle error silently
            }
        }
    }

    fn clear_playback_queue(&mut self) {
        self.audio_player.stop();
        self.playlist_manager.clear();
        // クリア時は再生状態もリセット
        self.playlist_manager.set_current_playing_index(None);
    }

    fn handle_queue_item_double_clicked(&mut self, index: usize) {
        // Set the current index to the double-clicked track and start playing
        self.playlist_manager.set_current_index(index);
        if let Some(track) = self.playlist_manager.get_current_track() {
            if let Err(_) = self.audio_player.play(track.clone()) {
                // Handle error silently
            }
        }
    }

    fn handle_remove_selected_from_queue(&mut self) {
        // If current playing track is being removed, stop playback
        if let Some(current_index) = self.playlist_manager.get_current_index() {
            if self.playlist_manager.is_selected(current_index) {
                self.audio_player.stop();
                // 再生中の楽曲が削除される場合は再生状態もクリア
                self.playlist_manager.set_current_playing_index(None);
            }
        }
        
        self.playlist_manager.remove_selected();
    }
}

impl eframe::App for MyApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_settings();
    }
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl) {
            self.current_tab = Tab::Main;
            self.focus_search = true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Period) && i.modifiers.ctrl) {
            self.current_tab = Tab::Settings;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Q) && i.modifiers.ctrl) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("ファイル", |ui| {
                    if ui.add(egui::Button::new("検索").shortcut_text("Ctrl+F")).clicked() {
                        self.current_tab = Tab::Main;
                        self.focus_search = true;
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new("設定").shortcut_text("Ctrl+.")).clicked() {
                        self.current_tab = Tab::Settings;
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new("終了").shortcut_text("Ctrl+Q")).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });
        
        // Tab bar
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Main, "メイン");
                ui.selectable_value(&mut self.current_tab, Tab::Settings, "設定");
            });
        });
        
        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Main => {
                    self.show_main_tab(ui);
                },
                Tab::Settings => {
                    self.show_settings_tab(ui);
                },
            }
        });
        
        if self.show_dialog {
            egui::Window::new("ダイアログ")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Hello, World!");
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            self.show_dialog = false;
                        }
                    });
                });
        }
    }
}

impl MyApp {
    fn show_main_tab(&mut self, ui: &mut egui::Ui) {
        let available_rect = ui.available_rect_before_wrap();
        let available_width = available_rect.width();
        let available_height = available_rect.height();
        let left_width = available_width * self.splitter_position;
        
        // Left pane
        let left_rect = egui::Rect::from_min_size(
            available_rect.min,
            egui::Vec2::new(left_width - 1.0, available_height)
        );
        let mut left_ui = ui.child_ui(left_rect, egui::Layout::top_down(egui::Align::LEFT), None);
        left_ui.set_clip_rect(left_rect);
        
        if self.settings.target_directory.is_empty() {
            left_ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("対象ディレクトリが設定されていません。");
                ui.label("設定タブでディレクトリを選択してください。");
            });
        } else {
            egui::ScrollArea::both()
                .id_source("left_pane_scroll")
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .show(&mut left_ui, |ui| {
                    ui.label(format!("対象ディレクトリ: {}", self.settings.target_directory));
                    ui.separator();
                    
                    // Search UI
                    let mut search_changed = false;
                    SearchUI::show(
                        ui,
                        &mut self.search_query,
                        &mut self.focus_search,
                        &mut || search_changed = true,
                    );
                    
                    if search_changed {
                        self.apply_search_filter();
                    }
                    
                    ui.add_space(10.0);
                    self.show_music_tree(ui);
                });
        }
        
        // Separator
        let separator_rect = egui::Rect::from_min_size(
            egui::Pos2::new(available_rect.min.x + left_width, available_rect.min.y),
            egui::Vec2::new(2.0, available_height)
        );
        ui.allocate_ui_at_rect(separator_rect, |ui| {
            ui.separator();
        });
        
        // Right pane
        let right_rect = egui::Rect::from_min_size(
            egui::Pos2::new(available_rect.min.x + left_width + 2.0, available_rect.min.y),
            egui::Vec2::new(available_width - left_width - 2.0, available_height)
        );
        let mut right_ui = ui.child_ui(right_rect, egui::Layout::top_down(egui::Align::LEFT), None);
        right_ui.set_clip_rect(right_rect);
        
        self.show_right_pane(&mut right_ui);
    }

    fn show_playlist_tabs(&mut self, ui: &mut egui::Ui) {
        
        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width(), ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Button)),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    
                    // デフォルトプレイリストタブ (左端に固定)
                    let is_default_active = self.playlist_manager.get_active_playlist_id() == "default";
                    let is_default_playing = self.playlist_manager.get_current_playing_playlist_id() == Some("default") 
                        && self.playlist_manager.get_current_track().is_some();
                    let default_label = if is_default_playing {
                        "🎵 デフォルト"  // 再生中マーク付き
                    } else {
                        "デフォルト"
                    };
                    
                    if ui.selectable_label(is_default_active, default_label).clicked() {
                        self.playlist_manager.set_active_playlist("default");
                    }
                    
                    // ユーザー作成プレイリストタブ
                    let playlists = self.playlist_manager.get_playlists().clone();
                    let mut playlist_to_activate = None;
                    let mut playlist_to_delete = None;
                    let mut playlist_to_start_editing = None;
                    let mut playlist_rename_result: Option<(String, String)> = None; // (id, new_name)
                    let mut cancel_editing = false;
                    
                    for playlist in &playlists {
                        if playlist.id == "default" {
                            continue; // デフォルトは既に表示済み
                        }
                        
                        let is_active = self.playlist_manager.get_active_playlist_id() == playlist.id;
                        let is_editing = self.editing_playlist_id.as_ref() == Some(&playlist.id);
                        let is_playing = self.playlist_manager.get_current_playing_playlist_id() == Some(&playlist.id)
                            && self.playlist_manager.get_current_track().is_some();
                        
                        
                        if is_editing {
                            // 編集モード：テキスト入力フィールドを表示
                            let response = ui.text_edit_singleline(&mut self.editing_playlist_name);
                            
                            // フォーカスを設定（初回のみ）
                            if response.gained_focus() {
                                response.request_focus();
                            }
                            
                            // Enter/Escapeキーの処理
                            if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                let new_name = self.editing_playlist_name.trim();
                                if !new_name.is_empty() && self.is_playlist_name_unique(new_name, &playlist.id) {
                                    playlist_rename_result = Some((playlist.id.clone(), new_name.to_string()));
                                }
                                cancel_editing = true;
                            }
                            
                            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                cancel_editing = true;
                            }
                        } else {
                            // 通常モード：selectable_labelを表示
                            let display_name = if is_playing {
                                format!("🎵 {}", playlist.name)  // 再生中マーク付き
                            } else {
                                playlist.name.clone()
                            };
                            let response = ui.selectable_label(is_active, display_name);
                            
                            if response.clicked() {
                                playlist_to_activate = Some(playlist.id.clone());
                            }
                            
                            // 右クリックメニュー（デフォルトプレイリスト以外）
                            response.context_menu(|ui| {
                                if ui.button("✏ 名前を変更").clicked() {
                                    playlist_to_start_editing = Some((playlist.id.clone(), playlist.name.clone()));
                                    ui.close_menu();
                                }
                                
                                ui.separator();
                                
                                // サブメニューで削除確認
                                ui.menu_button("🗑 削除", |ui| {
                                    let track_count = self.playlist_manager.get_playlist(&playlist.id)
                                        .map(|p| p.tracks.len())
                                        .unwrap_or(0);
                                    
                                    if track_count > 0 {
                                        ui.label(format!("「{}」を削除しますか？", playlist.name));
                                        ui.label(format!("（{}曲が含まれています）", track_count));
                                        ui.separator();
                                    }
                                    
                                    if ui.button("削除を確認").clicked() {
                                        playlist_to_delete = Some(playlist.id.clone());
                                        ui.close_menu();
                                    }
                                });
                            });
                        }
                    }
                    
                    // アクション実行（借用チェッカー対応）
                    if let Some(id) = playlist_to_activate {
                        self.playlist_manager.set_active_playlist(&id);
                    }
                    if let Some(id) = playlist_to_delete {
                        if self.playlist_manager.delete_playlist(&id) {
                            // 削除成功時に自動保存
                            let _ = self.playlist_manager.auto_save();
                        }
                    }
                    if let Some((id, name)) = playlist_to_start_editing {
                        self.editing_playlist_id = Some(id);
                        self.editing_playlist_name = name;
                    }
                    if let Some((id, new_name)) = playlist_rename_result {
                        if self.playlist_manager.rename_playlist(&id, new_name) {
                            // 名前変更成功時に自動保存
                            let _ = self.playlist_manager.auto_save();
                        }
                        self.editing_playlist_id = None;
                        self.editing_playlist_name.clear();
                    }
                    if cancel_editing {
                        self.editing_playlist_id = None;
                        self.editing_playlist_name.clear();
                    }
                    
                    // + ボタン (新しいプレイリスト作成)
                    if ui.button("+").clicked() {
                        // ユーザープレイリストの数をカウント（デフォルト以外）
                        let user_playlist_count = self.playlist_manager.get_playlists()
                            .iter()
                            .filter(|p| p.id != "default")
                            .count();
                        let new_name = format!("新しいプレイリスト{}", user_playlist_count + 1);
                        let new_id = self.playlist_manager.create_playlist(new_name);
                        self.playlist_manager.set_active_playlist(&new_id);
                        
                        // 作成成功時に自動保存
                        let _ = self.playlist_manager.auto_save();
                    }
                });
            }
        );
    }
    
    fn show_right_pane(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. プレイリストタブを最上部に表示
            self.show_playlist_tabs(ui);
            ui.separator();
            
            // 2. プレイリスト楽曲表示（固定高さ、10曲分、スクロール対応）
            ui.heading("プレイリスト");
            let playlist_height = (ui.text_style_height(&egui::TextStyle::Body) + ui.spacing().item_spacing.y) * 10.0 + 40.0; // 10曲分の高さ + マージン
            
            egui::ScrollArea::vertical()
                .id_source("playlist_scroll")
                .auto_shrink([false, false])
                .max_height(playlist_height)
                .show(ui, |ui| {
                    self.show_playlist_list(ui);
                });
            
            ui.separator();
            
            // 3. 下部を左右に分割
            ui.horizontal(|ui| {
                // 左側：再生コントロール
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() * 0.4);
                    ui.heading("再生コントロール");
                    ui.separator();
                    self.show_playback_controls_only(ui);
                });
                
                ui.separator();
                
                // 右側：情報・LRCタブ
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    
                    // 情報・LRCタブ切り替え
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.right_pane_tab, RightTab::Info, "情報");
                        ui.selectable_value(&mut self.right_pane_tab, RightTab::Lrc, "LRC");
                    });
                    
                    ui.separator();
                    
                    // タブのコンテンツを表示
                    egui::ScrollArea::both()
                        .id_source("info_lrc_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            match self.right_pane_tab {
                                RightTab::Playback => {
                                    // このケースは使用しない（プレイリストは上部で表示）
                                    ui.label("プレイリスト表示（上部で表示）");
                                },
                                RightTab::Info => {
                                    // 現在再生中の楽曲情報を優先表示
                                    if let Some(current_track) = self.playlist_manager.get_current_track() {
                                        ui.heading("🎵 現在再生中");
                                        if let Some(playing_playlist_id) = self.playlist_manager.get_current_playing_playlist_id() {
                                            if let Some(playlist) = self.playlist_manager.get_playlist(playing_playlist_id) {
                                                ui.label(format!("プレイリスト: {}", playlist.name));
                                            }
                                        }
                                        ui.separator();
                                        
                                        egui::Grid::new("current_track_info_grid")
                                            .num_columns(2)
                                            .spacing([10.0, 5.0])
                                            .show(ui, |ui| {
                                                ui.label("タイトル:");
                                                ui.label(&current_track.title);
                                                ui.end_row();
                                                
                                                ui.label("アーティスト:");
                                                ui.label(&current_track.artist);
                                                ui.end_row();
                                                
                                                ui.label("アルバム:");
                                                ui.label(&current_track.album);
                                                ui.end_row();
                                                
                                                if let Some(composer) = &current_track.composer {
                                                    ui.label("作曲者:");
                                                    ui.label(composer);
                                                    ui.end_row();
                                                }
                                                
                                                if let Some(genre) = &current_track.genre {
                                                    ui.label("ジャンル:");
                                                    ui.label(genre);
                                                    ui.end_row();
                                                }
                                                
                                                if let Some(track_num) = current_track.track_number {
                                                    ui.label("トラック番号:");
                                                    ui.label(track_num.to_string());
                                                    ui.end_row();
                                                }
                                            });
                                            
                                        ui.add_space(20.0);
                                        ui.separator();
                                        ui.add_space(10.0);
                                    }
                                    
                                    if let Some(track) = &self.selected_track {
                                        ui.heading("選択中の楽曲");
                                        egui::Grid::new("track_info_grid")
                                            .num_columns(2)
                                            .spacing([10.0, 5.0])
                                            .show(ui, |ui| {
                                                ui.label("タイトル:");
                                                ui.label(&track.title);
                                                ui.end_row();
                                                
                                                ui.label("アーティスト:");
                                                ui.label(&track.artist);
                                                ui.end_row();
                                                
                                                ui.label("アルバム:");
                                                ui.label(&track.album);
                                                ui.end_row();
                                                
                                                if let Some(composer) = &track.composer {
                                                    ui.label("作曲者:");
                                                    ui.label(composer);
                                                    ui.end_row();
                                                }
                                                
                                                if let Some(genre) = &track.genre {
                                                    ui.label("ジャンル:");
                                                    ui.label(genre);
                                                    ui.end_row();
                                                }
                                                
                                                if let Some(track_num) = track.track_number {
                                                    ui.label("トラック番号:");
                                                    ui.label(track_num.to_string());
                                                    ui.end_row();
                                                }
                                                
                                                ui.label("ファイルパス:");
                                                ui.label(track.path.display().to_string());
                                                ui.end_row();
                                            });
                                    } else if self.playlist_manager.get_current_track().is_none() {
                                        ui.label("楽曲を選択するか、再生を開始してください");
                                    }
                                },
                                RightTab::Lrc => {
                                    ui.label("LRC歌詞表示機能は未実装です");
                                },
                            }
                        });
                });
            });
        });
    }

    fn show_playlist_list(&mut self, ui: &mut egui::Ui) {
        // Store data needed for UI
        let queue_tracks = self.playlist_manager.get_tracks().cloned().unwrap_or_default();
        let current_index = self.playlist_manager.get_current_index();
        let selected_indices: Vec<usize> = self.playlist_manager.get_selected_indices().iter().cloned().collect();
        let playlists = self.playlist_manager.get_playlists().clone();
        let current_playlist_id = self.playlist_manager.get_active_playlist_id().to_string();
        
        // Collect actions
        let mut queue_item_selection: Option<(usize, bool, bool)> = None;
        let mut queue_item_double_clicked: Option<usize> = None;
        let mut move_selected_up = false;
        let mut move_selected_down = false;
        let mut move_selected_to_top = false;
        let mut move_selected_to_bottom = false;
        let mut remove_selected = false;
        let mut copy_to_playlist: Option<String> = None;
        let mut move_to_playlist: Option<String> = None;
        
        PlaybackControlsUI::show_track_list(
            ui,
            &queue_tracks,
            current_index,
            self.playlist_manager.get_current_playing_playlist_id(),
            self.playlist_manager.get_current_track(),
            &selected_indices,
            &playlists,
            &current_playlist_id,
            &mut |index, ctrl_held, shift_held| queue_item_selection = Some((index, ctrl_held, shift_held)),
            &mut |index| queue_item_double_clicked = Some(index),
            &mut || move_selected_up = true,
            &mut || move_selected_down = true,
            &mut || move_selected_to_top = true,
            &mut || move_selected_to_bottom = true,
            &mut || remove_selected = true,
            &mut |playlist_id| copy_to_playlist = Some(playlist_id),
            &mut |playlist_id| move_to_playlist = Some(playlist_id),
        );
        
        // Handle actions after UI
        if let Some((index, ctrl_held, shift_held)) = queue_item_selection {
            self.playlist_manager.handle_item_selection(index, ctrl_held, shift_held);
        }
        if let Some(index) = queue_item_double_clicked {
            self.handle_queue_item_double_clicked(index);
        }
        if move_selected_up {
            self.playlist_manager.move_selected_up();
        }
        if move_selected_down {
            self.playlist_manager.move_selected_down();
        }
        if move_selected_to_top {
            self.playlist_manager.move_selected_to_top();
        }
        if move_selected_to_bottom {
            self.playlist_manager.move_selected_to_bottom();
        }
        if remove_selected {
            self.handle_remove_selected_from_queue();
        }
        if let Some(playlist_id) = copy_to_playlist {
            self.handle_copy_selected_to_playlist(playlist_id);
        }
        if let Some(playlist_id) = move_to_playlist {
            self.handle_move_selected_to_playlist(playlist_id);
        }
    }

    fn show_playback_controls_only(&mut self, ui: &mut egui::Ui) {
        let playback_state = self.audio_player.get_state().clone();
        
        // Collect actions
        let mut clear_queue = false;
        let mut previous_clicked = false;
        let mut play_pause_clicked = false;
        let mut stop_clicked = false;
        let mut next_clicked = false;
        
        PlaybackControlsUI::show_controls_only(
            ui,
            &playback_state,
            &mut || clear_queue = true,
            &mut || previous_clicked = true,
            &mut || play_pause_clicked = true,
            &mut || stop_clicked = true,
            &mut || next_clicked = true,
        );
        
        // Handle actions after UI
        if clear_queue {
            self.clear_playback_queue();
        }
        if previous_clicked {
            self.handle_previous_button();
        }
        if play_pause_clicked {
            self.handle_play_pause();
        }
        if stop_clicked {
            self.handle_stop();
        }
        if next_clicked {
            self.handle_next();
        }
    }

    fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(20.0);
        
        ui.horizontal(|ui| {
            ui.label("対象ディレクトリ:");
            ui.add_space(10.0);
            
            let response = ui.text_edit_singleline(&mut self.settings.target_directory);
            if response.changed() {
                self.save_settings();
            }
            
            if ui.button("選択").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.settings.target_directory = path.display().to_string();
                    self.save_settings();
                    self.refresh_music_library();
                }
            }
        });
        
        ui.add_space(20.0);
        
        ui.horizontal(|ui| {
            let response = ui.checkbox(&mut self.settings.classical_composer_hierarchy, 
                "クラシック音楽（ジャンルが\"Classical\"）では作曲家階層を追加");
            if response.changed() {
                self.music_library.set_classical_hierarchy(self.settings.classical_composer_hierarchy);
                self.save_settings();
            }
        });
    }

    // プレイリスト名の重複チェック
    fn is_playlist_name_unique(&self, name: &str, excluding_id: &str) -> bool {
        !self.playlist_manager.get_playlists()
            .iter()
            .any(|p| p.id != excluding_id && p.name == name)
    }

    // プレイリストに楽曲を追加
    fn handle_add_to_playlist(&mut self, track: TrackInfo, playlist_id: String) {
        // 指定されたプレイリストに楽曲を追加
        if let Some(playlist) = self.playlist_manager.get_playlist_mut(&playlist_id) {
            playlist.add_track(track);
            // 自動保存
            let _ = self.playlist_manager.auto_save();
        }
    }

    // アルバムをプレイリストに追加
    fn handle_add_album_to_playlist(&mut self, node: MusicTreeNode, playlist_id: String) {
        let tracks = self.collect_all_tracks_from_node(&node);
        if let Some(playlist) = self.playlist_manager.get_playlist_mut(&playlist_id) {
            for track in tracks {
                playlist.add_track(track);
            }
            // 自動保存
            let _ = self.playlist_manager.auto_save();
        }
    }

    // アーティスト・作曲家をプレイリストに追加
    fn handle_add_artist_to_playlist(&mut self, node: MusicTreeNode, playlist_id: String) {
        let tracks = self.collect_all_tracks_from_node(&node);
        if let Some(playlist) = self.playlist_manager.get_playlist_mut(&playlist_id) {
            for track in tracks {
                playlist.add_track(track);
            }
            // 自動保存
            let _ = self.playlist_manager.auto_save();
        }
    }

    // ノードから全ての楽曲を収集するヘルパー関数（プレイリスト用）
    fn collect_all_tracks_from_node(&self, node: &MusicTreeNode) -> Vec<TrackInfo> {
        let mut tracks = Vec::new();
        self.collect_all_tracks_recursive(node, &mut tracks);
        tracks
    }

    // 再帰的に楽曲を収集（プレイリスト用）
    fn collect_all_tracks_recursive(&self, node: &MusicTreeNode, tracks: &mut Vec<TrackInfo>) {
        if let Some(track_info) = &node.track_info {
            tracks.push(track_info.clone());
        }
        
        for child in &node.children {
            self.collect_all_tracks_recursive(child, tracks);
        }
    }

    // 選択中の楽曲を他のプレイリストにコピー
    fn handle_copy_selected_to_playlist(&mut self, target_playlist_id: String) {
        // 現在選択されている楽曲を取得
        let selected_tracks = self.get_selected_tracks_from_active_playlist();
        
        // ターゲットプレイリストに楽曲を追加
        if let Some(target_playlist) = self.playlist_manager.get_playlist_mut(&target_playlist_id) {
            for track in selected_tracks {
                target_playlist.add_track(track);
            }
            // 自動保存
            let _ = self.playlist_manager.auto_save();
        }
    }

    // 選択中の楽曲を他のプレイリストに移動
    fn handle_move_selected_to_playlist(&mut self, target_playlist_id: String) {
        // 現在選択されている楽曲を取得
        let selected_tracks = self.get_selected_tracks_from_active_playlist();
        
        // ターゲットプレイリストに楽曲を追加
        if let Some(target_playlist) = self.playlist_manager.get_playlist_mut(&target_playlist_id) {
            for track in selected_tracks {
                target_playlist.add_track(track);
            }
        }
        
        // 現在のプレイリストから選択されている楽曲を削除
        self.playlist_manager.remove_selected();
        
        // 自動保存
        let _ = self.playlist_manager.auto_save();
    }

    // アクティブプレイリストから選択中の楽曲を取得
    fn get_selected_tracks_from_active_playlist(&self) -> Vec<TrackInfo> {
        let mut selected_tracks = Vec::new();
        
        if let Some(tracks) = self.playlist_manager.get_active_tracks() {
            let selected_indices = self.playlist_manager.get_selected_indices();
            
            for &index in selected_indices {
                if let Some(track) = tracks.get(index) {
                    selected_tracks.push(track.clone());
                }
            }
        }
        
        selected_tracks
    }
}