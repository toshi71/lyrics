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
        let mut selected_track = None;
        let mut double_clicked_track = None;
        
        MusicTreeUI::show(
            ui,
            self.music_library.get_tree_mut(),
            &self.search_query,
            &mut |track| selected_track = Some(track),
            &mut |track| double_clicked_track = Some(track),
        );
        
        if let Some(track) = selected_track {
            self.selected_track = Some(track);
        }
        
        if let Some(track) = double_clicked_track {
            self.play_track(track);
        }
    }

    fn play_track(&mut self, track: TrackInfo) {
        self.playback_queue.add_track_at_front(track.clone());
        if let Err(_) = self.audio_player.play(track) {
            // Handle error silently for now
        }
    }

    fn get_playable_track(&self) -> Option<TrackInfo> {
        if let Some(ref track) = self.selected_track {
            return Some(track.clone());
        }
        
        self.music_library.get_first_track()
    }

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
                if let Some(track) = self.get_playable_track() {
                    self.play_track(track);
                }
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
                    SearchUI::show(
                        ui,
                        &mut self.search_query,
                        &mut self.focus_search,
                        &mut || self.apply_search_filter(),
                    );
                    
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
                        PlaybackControlsUI::show(
                            ui,
                            self.playback_queue.get_tracks(),
                            self.playback_queue.get_current_index(),
                            self.audio_player.get_state(),
                            &mut || self.clear_playback_queue(),
                            &mut || self.handle_previous_button(),
                            &mut || self.handle_play_pause(),
                            &mut || self.handle_stop(),
                            &mut || self.handle_next(),
                        );
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