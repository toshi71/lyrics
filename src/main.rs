mod music;
mod player;
mod settings;
mod ui;

use eframe::egui;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;

use music::{MusicLibrary, TrackInfo};
use player::{AudioPlayer, PlaybackQueue, PlaybackState};
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
    playback_queue: PlaybackQueue,
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
            playback_queue: PlaybackQueue::new(),
            settings,
        };
        app.refresh_music_library();
        app
    }

    fn save_settings(&self) {
        let _ = self.settings.save();
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
        let mut add_to_queue_track = None;
        let mut add_album_to_queue_node = None;
        let mut add_artist_to_queue_node = None;
        
        MusicTreeUI::show(
            ui,
            self.music_library.get_tree_mut(),
            &self.search_query,
            self.selected_track.as_ref(),
            &self.selected_tracks,
            &mut |track, ctrl_held, shift_held| track_selection = Some((track, ctrl_held, shift_held)),
            &mut |track| double_clicked_track = Some(track),
            &mut |track| add_to_queue_track = Some(track),
            &mut |node| add_album_to_queue_node = Some(node.clone()),
            &mut |node| add_artist_to_queue_node = Some(node.clone()),
        );
        
        if let Some((track, ctrl_held, shift_held)) = track_selection {
            self.handle_track_selection(track, ctrl_held, shift_held);
        }
        
        if let Some(track) = add_to_queue_track {
            self.handle_add_to_queue_from_context_menu(track);
        }
        
        if let Some(node) = add_album_to_queue_node {
            self.handle_add_album_to_queue(node);
        }
        
        if let Some(node) = add_artist_to_queue_node {
            self.handle_add_artist_to_queue(node);
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

    fn add_selected_tracks_to_queue(&mut self) {
        let mut tracks_to_add = Vec::new();
        
        // If there are multiple selections, add all of them
        if !self.selected_tracks.is_empty() {
            let all_tracks = self.get_all_tracks_in_display_order();
            for track in all_tracks {
                if self.selected_tracks.contains(&track.path) {
                    tracks_to_add.push(track);
                }
            }
        } else if let Some(ref selected_track) = self.selected_track {
            // If only single selection, add that track
            tracks_to_add.push(selected_track.clone());
        }
        
        // Add tracks to queue in order
        for track in tracks_to_add {
            self.playback_queue.add_track(track);
        }
    }

    fn handle_add_to_queue_from_context_menu(&mut self, clicked_track: TrackInfo) {
        // Check if the clicked track is already selected
        let track_is_selected = self.selected_tracks.contains(&clicked_track.path) || 
            (self.selected_track.as_ref().map(|st| st.path == clicked_track.path).unwrap_or(false));
        
        if track_is_selected {
            // If the track is already selected, add all currently selected tracks
            self.add_selected_tracks_to_queue();
        } else {
            // If the track is not selected, select it first and then add only that track
            self.selected_tracks.clear();
            self.selected_track = Some(clicked_track.clone());
            self.last_selected_path = Some(clicked_track.path.clone());
            
            // Add only this track to queue
            self.playback_queue.add_track(clicked_track);
        }
    }

    fn handle_add_album_to_queue(&mut self, album_node: crate::music::MusicTreeNode) {
        // Collect all tracks from the album node
        let mut album_tracks = Vec::new();
        self.collect_tracks_from_node(&album_node, &mut album_tracks);
        
        // Add tracks to queue in order (they should already be sorted by track number)
        for track in album_tracks {
            self.playback_queue.add_track(track);
        }
    }

    fn handle_add_artist_to_queue(&mut self, artist_node: crate::music::MusicTreeNode) {
        // Collect all tracks from the artist/composer node
        let mut artist_tracks = Vec::new();
        self.collect_tracks_from_node(&artist_node, &mut artist_tracks);
        
        // Add tracks to queue in order (they should already be sorted)
        for track in artist_tracks {
            self.playback_queue.add_track(track);
        }
    }

    fn collect_tracks_from_node(&self, node: &crate::music::MusicTreeNode, tracks: &mut Vec<TrackInfo>) {
        // If this is a track node, add it
        if let Some(track_info) = &node.track_info {
            tracks.push(track_info.clone());
        }
        
        // Recursively collect from children
        for child in &node.children {
            self.collect_tracks_from_node(child, tracks);
        }
    }

    // Removed play_track method - now using queue-only playback

    // Removed get_playable_track method - now using direct queue access

    fn handle_previous_button(&mut self) {
        let position = self.audio_player.get_playback_position();
        
        if position.as_secs() <= 3 {
            if let Some(prev_track) = self.playback_queue.move_to_previous() {
                if let Err(_) = self.audio_player.play(prev_track.clone()) {
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
                if let Some(track) = self.playback_queue.get_current_track() {
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
    }

    fn handle_next(&mut self) {
        if let Some(next_track) = self.playback_queue.move_to_next() {
            if let Err(_) = self.audio_player.play(next_track.clone()) {
                // Handle error silently
            }
        }
    }

    fn clear_playback_queue(&mut self) {
        self.audio_player.stop();
        self.playback_queue.clear();
    }

    fn handle_queue_item_double_clicked(&mut self, index: usize) {
        // Set the current index to the double-clicked track and start playing
        self.playback_queue.set_current_index(index);
        if let Some(track) = self.playback_queue.get_current_track() {
            if let Err(_) = self.audio_player.play(track.clone()) {
                // Handle error silently
            }
        }
    }

    fn handle_remove_selected_from_queue(&mut self) {
        // If current playing track is being removed, stop playback
        if let Some(current_index) = self.playback_queue.get_current_index() {
            if self.playback_queue.is_selected(current_index) {
                self.audio_player.stop();
            }
        }
        
        self.playback_queue.remove_selected();
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

    fn show_right_pane(&mut self, ui: &mut egui::Ui) {
        // Tab header
        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width(), ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Button)),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.selectable_value(&mut self.right_pane_tab, RightTab::Playback, "再生");
                    ui.selectable_value(&mut self.right_pane_tab, RightTab::Info, "情報");
                    ui.selectable_value(&mut self.right_pane_tab, RightTab::Lrc, "LRC");
                });
            }
        );
        ui.separator();
        
        // Tab content
        egui::ScrollArea::both()
            .id_source("right_pane_scroll")
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .show(ui, |ui| {
                match self.right_pane_tab {
                    RightTab::Playback => {
                        // Store data needed for UI
                        let queue_tracks = self.playback_queue.get_tracks().clone();
                        let current_index = self.playback_queue.get_current_index();
                        let playback_state = self.audio_player.get_state().clone();
                        let selected_indices = self.playback_queue.get_selected_indices();
                        
                        // Collect actions
                        let mut clear_queue = false;
                        let mut previous_clicked = false;
                        let mut play_pause_clicked = false;
                        let mut stop_clicked = false;
                        let mut next_clicked = false;
                        let mut queue_item_selection: Option<(usize, bool, bool)> = None;
                        let mut queue_item_double_clicked: Option<usize> = None;
                        let mut move_selected_up = false;
                        let mut move_selected_down = false;
                        let mut move_selected_to_top = false;
                        let mut move_selected_to_bottom = false;
                        let mut remove_selected = false;
                        
                        PlaybackControlsUI::show(
                            ui,
                            &queue_tracks,
                            current_index,
                            &playback_state,
                            &selected_indices,
                            &mut || clear_queue = true,
                            &mut || previous_clicked = true,
                            &mut || play_pause_clicked = true,
                            &mut || stop_clicked = true,
                            &mut || next_clicked = true,
                            &mut |index, ctrl_held, shift_held| queue_item_selection = Some((index, ctrl_held, shift_held)),
                            &mut |index| queue_item_double_clicked = Some(index),
                            &mut || move_selected_up = true,
                            &mut || move_selected_down = true,
                            &mut || move_selected_to_top = true,
                            &mut || move_selected_to_bottom = true,
                            &mut || remove_selected = true,
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
                        if let Some((index, ctrl_held, shift_held)) = queue_item_selection {
                            self.playback_queue.handle_item_selection(index, ctrl_held, shift_held);
                        }
                        if let Some(index) = queue_item_double_clicked {
                            self.handle_queue_item_double_clicked(index);
                        }
                        if move_selected_up {
                            self.playback_queue.move_selected_up();
                        }
                        if move_selected_down {
                            self.playback_queue.move_selected_down();
                        }
                        if move_selected_to_top {
                            self.playback_queue.move_selected_to_top();
                        }
                        if move_selected_to_bottom {
                            self.playback_queue.move_selected_to_bottom();
                        }
                        if remove_selected {
                            self.handle_remove_selected_from_queue();
                        }
                    },
                    RightTab::Info => {
                        ui.vertical_centered(|ui| {
                            ui.add_space(50.0);
                            ui.label("情報タブ");
                            ui.label("ここに楽曲情報を表示予定");
                        });
                    },
                    RightTab::Lrc => {
                        ui.vertical_centered(|ui| {
                            ui.add_space(50.0);
                            ui.label("LRCタブ");
                            ui.label("ここに歌詞を表示予定");
                        });
                    },
                }
            });
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
}