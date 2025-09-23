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
                        self.ui_state.current_tab = crate::app::state::Tab::Main;
                        self.selection_state.focus_search = true;
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new("設定").shortcut_text("Ctrl+.")).clicked() {
                        self.ui_state.current_tab = crate::app::state::Tab::Settings;
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new("終了").shortcut_text("Ctrl+Q")).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("再生", |ui| {
                    if ui.add(egui::Button::new("再生/一時停止").shortcut_text("Space")).clicked() {
                        self.handle_play_pause();
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new("前の曲").shortcut_text("Ctrl+B")).clicked() {
                        self.handle_previous_button();
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new("次の曲").shortcut_text("Ctrl+P")).clicked() {
                        self.handle_next();
                        ui.close_menu();
                    }
                    let seek_seconds = self.settings.get_seek_seconds();
                    if ui.add(egui::Button::new(format!("シーク（{}秒戻る）", seek_seconds)).shortcut_text("Shift+B")).clicked() {
                        self.handle_seek_backward();
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new(format!("シーク（{}秒進む）", seek_seconds)).shortcut_text("Shift+P")).clicked() {
                        self.handle_seek_forward();
                        ui.close_menu();
                    }
                });
            });
        });
    }

    pub fn show_tab_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.ui_state.current_tab, crate::app::state::Tab::Main, "メイン");
                ui.selectable_value(&mut self.ui_state.current_tab, crate::app::state::Tab::Settings, "設定");
            });
        });
    }

    pub fn show_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.ui_state.current_tab {
                crate::app::state::Tab::Main => {
                    self.show_main_tab(ui);
                },
                crate::app::state::Tab::Settings => {
                    self.show_settings_tab(ui);
                },
            }
        });
    }

    pub fn show_dialog_if_needed(&mut self, ctx: &egui::Context) {
        if self.ui_state.show_dialog {
            egui::Window::new("ダイアログ")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Hello, World!");
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            self.ui_state.show_dialog = false;
                        }
                    });
                });
        }
    }

    pub fn show_main_tab(&mut self, ui: &mut egui::Ui) {
        
        let available_rect = ui.available_rect_before_wrap();
        let available_width = available_rect.width();
        let available_height = available_rect.height();
        
        // メインタブ全体のデバッグ描画
        self.debug_ui.draw_debug_rect_fixed(ui, available_rect, crate::debug_ui::ID_MAIN_TAB, "MainTab");
        
        // リサイズ可能な水平分割線
        let separator_id = ui.id().with("main_horizontal_separator");
        let _separator_response = ui.allocate_response(
            egui::Vec2::new(available_width, available_height),
            egui::Sense::hover()
        );
        
        // マウスドラッグによる分割位置の更新
        let left_width = available_width * self.ui_state.splitter_position;
        let separator_x = available_rect.min.x + left_width;
        let separator_rect = egui::Rect::from_min_size(
            egui::Pos2::new(separator_x - 2.0, available_rect.min.y),
            egui::Vec2::new(4.0, available_height)
        );
        
        let separator_response = ui.interact(separator_rect, separator_id, egui::Sense::drag());
        if separator_response.dragged() {
            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                let new_left_width = (pointer_pos.x - available_rect.min.x).max(50.0).min(available_width - 50.0);
                self.ui_state.splitter_position = new_left_width / available_width;
            }
        }
        
        // カーソル変更
        if separator_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        
        let left_width = available_width * self.ui_state.splitter_position;
        
        // Left pane
        let left_rect = egui::Rect::from_min_size(
            available_rect.min,
            egui::Vec2::new(left_width - 2.0, available_height)
        );
        let mut left_ui = ui.child_ui(left_rect, egui::Layout::top_down(egui::Align::LEFT), None);
        left_ui.set_clip_rect(left_rect);
        
        // 左ペインのデバッグ描画
        self.debug_ui.draw_debug_rect_fixed(ui, left_rect, crate::debug_ui::ID_LEFT_PANE, "LeftPane");
        
        self.show_left_pane(&mut left_ui);
        
        // Separator visualization
        let separator_rect = egui::Rect::from_min_size(
            egui::Pos2::new(available_rect.min.x + left_width - 1.0, available_rect.min.y),
            egui::Vec2::new(2.0, available_height)
        );
        ui.allocate_ui_at_rect(separator_rect, |ui| {
            ui.separator();
        });
        
        // Right pane
        let right_rect = egui::Rect::from_min_size(
            egui::Pos2::new(available_rect.min.x + left_width + 1.0, available_rect.min.y),
            egui::Vec2::new(available_width - left_width - 1.0, available_height)
        );
        let mut right_ui = ui.child_ui(right_rect, egui::Layout::top_down(egui::Align::LEFT), None);
        right_ui.set_clip_rect(right_rect);
        
        // 右ペインのデバッグ描画
        self.debug_ui.draw_debug_rect_fixed(ui, right_rect, crate::debug_ui::ID_RIGHT_PANE, "RightPane");
        
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
                    let search_has_focus = SearchUI::show(
                        ui,
                        &mut self.selection_state.search_query,
                        &mut self.selection_state.focus_search,
                        &mut || search_changed = true,
                    );
                    
                    // Store search focus state for global shortcut handling
                    self.selection_state.search_has_focus = search_has_focus;
                    
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
        let mut create_playlist_with_track: Option<TrackInfo> = None;
        let mut create_playlist_with_album: Option<MusicTreeNode> = None;
        let mut create_playlist_with_artist: Option<MusicTreeNode> = None;
        
        MusicTreeUI::show(
            ui,
            self.music_library.get_tree_mut(),
            &self.selection_state.search_query,
            self.selection_state.selected_track.as_ref(),
            &self.selection_state.selected_tracks,
            &self.playlist_manager.get_playlists(),
            &mut |track, ctrl_held, shift_held| track_selection = Some((track, ctrl_held, shift_held)),
            &mut |track| double_clicked_track = Some(track),
            &mut |track, playlist_id| add_to_playlist_track = Some((track, playlist_id)),
            &mut |node, playlist_id| add_album_to_playlist = Some((node.clone(), playlist_id)),
            &mut |node, playlist_id| add_artist_to_playlist = Some((node.clone(), playlist_id)),
            &mut |track| create_playlist_with_track = Some(track),
            &mut |node| create_playlist_with_album = Some(node.clone()),
            &mut |node| create_playlist_with_artist = Some(node.clone()),
        );
        
        if let Some((track, ctrl_held, shift_held)) = track_selection {
            self.handle_track_selection(track, ctrl_held, shift_held);
        }
        
        if let Some((track, playlist_id)) = add_to_playlist_track {
            if let Err(error_message) = self.handle_add_to_playlist(track, playlist_id) {
                self.show_error_dialog_main("楽曲追加エラー", &error_message);
            }
        }
        
        if let Some((node, playlist_id)) = add_album_to_playlist {
            if let Err(error_message) = self.handle_add_album_to_playlist(node, playlist_id) {
                self.show_error_dialog_main("アルバム追加エラー", &error_message);
            }
        }
        
        if let Some((node, playlist_id)) = add_artist_to_playlist {
            if let Err(error_message) = self.handle_add_artist_to_playlist(node, playlist_id) {
                self.show_error_dialog_main("アーティスト追加エラー", &error_message);
            }
        }
        
        // ダブルクリック時にデフォルトプレイリストに楽曲を追加
        if let Some(track) = double_clicked_track {
            if let Err(error_message) = self.playlist_manager.add_track_to_playlist("default", track) {
                self.show_error_dialog_main("楽曲追加エラー", &error_message);
            } else {
                let _ = self.playlist_manager.auto_save();
            }
        }
        
        // 新プレイリスト作成処理
        if let Some(track) = create_playlist_with_track {
            if let Err(error_message) = self.handle_create_playlist_with_track(track) {
                self.show_error_dialog_main("新プレイリスト作成エラー", &error_message);
            }
        }
        
        if let Some(node) = create_playlist_with_album {
            if let Err(error_message) = self.handle_create_playlist_with_album(&node) {
                self.show_error_dialog_main("新プレイリスト作成エラー", &error_message);
            }
        }
        
        if let Some(node) = create_playlist_with_artist {
            if let Err(error_message) = self.handle_create_playlist_with_artist(&node) {
                self.show_error_dialog_main("新プレイリスト作成エラー", &error_message);
            }
        }
    }

    /// OS標準のエラーダイアログを表示
    fn show_error_dialog_main(&self, title: &str, message: &str) {
        rfd::MessageDialog::new()
            .set_title(title)
            .set_description(message)
            .set_level(rfd::MessageLevel::Error)
            .show();
    }
}