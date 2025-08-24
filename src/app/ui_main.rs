use super::MyApp;
use crate::music::{MusicTreeNode, TrackInfo};
use crate::ui::{MusicTreeUI, SearchUI};
use eframe::egui;

impl MyApp {
    pub fn show_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("ファイル", |ui| {
                    if ui.add(egui::Button::new("検索").shortcut_text("Ctrl+F")).clicked() {
                        self.current_tab = super::Tab::Main;
                        self.focus_search = true;
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new("設定").shortcut_text("Ctrl+.")).clicked() {
                        self.current_tab = super::Tab::Settings;
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new("終了").shortcut_text("Ctrl+Q")).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });
    }

    pub fn show_tab_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, super::Tab::Main, "メイン");
                ui.selectable_value(&mut self.current_tab, super::Tab::Settings, "設定");
            });
        });
    }

    pub fn show_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                super::Tab::Main => {
                    self.show_main_tab(ui);
                },
                super::Tab::Settings => {
                    self.show_settings_tab(ui);
                },
            }
        });
    }

    pub fn show_dialog_if_needed(&mut self, ctx: &egui::Context) {
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

    pub fn show_main_tab(&mut self, ui: &mut egui::Ui) {
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
        
        self.show_left_pane(&mut left_ui);
        
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

    pub fn show_left_pane(&mut self, ui: &mut egui::Ui) {
        if self.settings.target_directory.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("対象ディレクトリが設定されていません。");
                ui.label("設定タブでディレクトリを選択してください。");
            });
        } else {
            egui::ScrollArea::both()
                .id_source("left_pane_scroll")
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .show(ui, |ui| {
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
    }

    pub fn show_music_tree(&mut self, ui: &mut egui::Ui) {
        let mut track_selection = None;
        let mut double_clicked_track = None;
        let mut add_to_playlist_track: Option<(TrackInfo, String)> = None;
        let mut add_album_to_playlist: Option<(MusicTreeNode, String)> = None;
        let mut add_artist_to_playlist: Option<(MusicTreeNode, String)> = None;
        
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
        
        // TODO: Implement playlist addition on double-click
        // For now, double-click does nothing
        if let Some(_track) = double_clicked_track {
            // Double-click functionality will be implemented later as "add to playlist"
        }
    }
}