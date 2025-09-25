use super::MyApp;
use crate::ui::PlaybackControlsUI;
use eframe::egui;

impl MyApp {
    pub fn show_right_pane(&mut self, ui: &mut egui::Ui) {
        crate::app::ui::right_pane::RightPaneLayout::show_right_pane(self, ui);
    }





    


    pub fn show_playlist_tabs(&mut self, ui: &mut egui::Ui) {
        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width(), ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Button)),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.add_space(2.0);
                let _scroll_area_response = egui::ScrollArea::horizontal()
                    .id_source("playlist_tabs_scroll")
                    .auto_shrink([false, true])
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                    .hscroll(true)
                    .vscroll(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    
                    // Action collection variables
                    let mut playlist_to_activate = None;
                    let mut playlist_to_delete = None;
                    let mut playlist_to_start_editing = None;
                    let mut playlist_rename_result: Option<(String, String)> = None;
                    let mut playlist_to_clear = None;
                    let mut cancel_editing = false;
                    
                    // デフォルトプレイリストタブ (左端に固定)
                    let is_default_active = self.playlist_manager.get_active_playlist_id() == "default";
                    let is_default_playing = self.playlist_manager.get_current_playing_playlist_id() == Some("default") 
                        && self.playlist_manager.get_current_track().is_some();
                    let default_label = if is_default_playing {
                        "🎵 デフォルト"  // 再生中マーク付き
                    } else {
                        "デフォルト"
                    };
                    
                    let default_response = ui.selectable_label(is_default_active, default_label);
                    if default_response.clicked() {
                        self.playlist_manager.set_active_playlist("default");
                        self.selection_state.selected_track = None; // Reset selected track when changing playlist
                        self.save_settings();
                    }
                    
                    // デフォルトプレイリストの右クリックメニュー
                    default_response.context_menu(|ui| {
                        let track_count = self.playlist_manager.get_playlist("default")
                            .map(|p| p.tracks.len())
                            .unwrap_or(0);
                        
                        if track_count > 0 {
                            if ui.button("× プレイリストをクリア").clicked() {
                                playlist_to_clear = Some("default".to_string());
                                ui.close_menu();
                            }
                        } else {
                            ui.add_enabled(false, egui::Button::new("× プレイリストをクリア"));
                        }
                    });
                    
                    // ユーザー作成プレイリストタブ
                    let playlists = self.playlist_manager.get_playlists().clone();
                    
                    for playlist in &playlists {
                        if playlist.id == "default" {
                            continue; // デフォルトは既に表示済み
                        }
                        
                        let is_active = self.playlist_manager.get_active_playlist_id() == playlist.id;
                        let is_editing = self.playlist_edit_state.editing_playlist_id.as_ref() == Some(&playlist.id);
                        let is_playing = self.playlist_manager.get_current_playing_playlist_id() == Some(&playlist.id)
                            && self.playlist_manager.get_current_track().is_some();
                        
                        if is_editing {
                            // 編集モード：テキスト入力フィールドを表示
                            let response = ui.text_edit_singleline(&mut self.playlist_edit_state.editing_playlist_name);
                            
                            // フォーカスを設定（初回のみ）
                            if response.gained_focus() {
                                response.request_focus();
                            }
                            
                            // Enter/Escapeキーの処理
                            if response.lost_focus() || ui.input(|i| i.key_pressed(eframe::egui::Key::Enter)) {
                                playlist_rename_result = Some((playlist.id.clone(), self.playlist_edit_state.editing_playlist_name.clone()));
                                cancel_editing = true;
                            }
                            
                            if ui.input(|i| i.key_pressed(eframe::egui::Key::Escape)) {
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
                                
                                // プレイリストをクリア
                                let track_count = self.playlist_manager.get_playlist(&playlist.id)
                                    .map(|p| p.tracks.len())
                                    .unwrap_or(0);
                                
                                if track_count > 0 {
                                    if ui.button("× プレイリストをクリア").clicked() {
                                        playlist_to_clear = Some(playlist.id.clone());
                                        ui.close_menu();
                                    }
                                } else {
                                    ui.add_enabled(false, egui::Button::new("× プレイリストをクリア"));
                                }
                                
                                ui.separator();
                                
                                // サブメニューで削除確認
                                ui.menu_button("🗑 削除", |ui| {
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
                    
                    // + ボタン (新しいプレイリスト作成)
                    if ui.button("+").clicked() {
                        let user_playlist_count = self.playlist_manager.get_playlists()
                            .iter()
                            .filter(|p| p.id != "default")
                            .count();
                        let new_name = format!("新しいプレイリスト{}", user_playlist_count + 1);
                        let new_id = self.playlist_manager.create_playlist(new_name);
                        self.playlist_manager.set_active_playlist(&new_id);
                        self.selection_state.selected_track = None; // Reset selected track when changing playlist
                        
                        self.settings.add_to_display_order(new_id);
                        
                        let _ = self.playlist_manager.auto_save();
                        self.save_settings();
                    }
                    
                    // アクション実行（借用チェッカー対応）
                    if let Some(id) = playlist_to_activate {
                        self.playlist_manager.set_active_playlist(&id);
                        self.selection_state.selected_track = None; // Reset selected track when changing playlist
                        self.save_settings();
                    }
                    if let Some(id) = playlist_to_delete {
                        if self.playlist_manager.delete_playlist(&id) {
                            self.settings.remove_from_display_order(&id);
                            let _ = self.playlist_manager.auto_save();
                            self.save_settings();
                        }
                    }
                    if let Some((id, name)) = playlist_to_start_editing {
                        self.playlist_edit_state.editing_playlist_id = Some(id);
                        self.playlist_edit_state.editing_playlist_name = name;
                    }
                    if let Some((id, new_name)) = playlist_rename_result {
                        if self.playlist_manager.rename_playlist(&id, new_name) {
                            let _ = self.playlist_manager.auto_save();
                            self.save_settings();
                        }
                        self.playlist_edit_state.editing_playlist_id = None;
                        self.playlist_edit_state.editing_playlist_name.clear();
                    }
                    if cancel_editing {
                        self.playlist_edit_state.editing_playlist_id = None;
                        self.playlist_edit_state.editing_playlist_name.clear();
                    }
                    if let Some(playlist_id) = playlist_to_clear {
                        if let Some(playlist) = self.playlist_manager.get_playlist_mut(&playlist_id) {
                            // プレイリストをクリア
                            playlist.clear();
                            
                            // もしクリアしたプレイリストが現在再生中だった場合、再生を停止
                            if self.playlist_manager.get_current_playing_playlist_id() == Some(&playlist_id) {
                                self.player_state.audio_player.stop();
                                self.playlist_manager.set_current_playing_index(None);
                            }
                            
                            let _ = self.playlist_manager.auto_save();
                            self.save_settings();
                        }
                    }
                        });
                    });
                
            }
        );
    }

    pub fn show_playlist_list(&mut self, ui: &mut egui::Ui) {
        // Store data needed for UI
        let queue_tracks = self.playlist_manager.get_tracks().cloned().unwrap_or_default();
        let current_index = self.playlist_manager.get_current_index();
        let selected_indices: Vec<usize> = self.playlist_manager.get_selected_indices().iter().cloned().collect();
        let playlists = self.playlist_manager.get_playlists().clone();
        let current_playlist_id = self.playlist_manager.get_active_playlist_id().to_string();
        
        // 大量楽曲時の警告表示
        if queue_tracks.len() > 5000 {
            ui.label(format!("⚠ 大量楽曲 ({} 曲) - 表示に時間がかかる場合があります", queue_tracks.len()));
        }
        
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
        let mut select_all = false;
        let mut clear_selection = false;
        let mut copy_to_new_playlist = false;
        let mut move_to_new_playlist = false;
        
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
            &mut || select_all = true,
            &mut || clear_selection = true,
            &mut || copy_to_new_playlist = true,
            &mut || move_to_new_playlist = true,
        );
        
        // Handle actions after UI
        if let Some((index, ctrl_held, shift_held)) = queue_item_selection {
            self.playlist_manager.handle_item_selection(index, ctrl_held, shift_held);
            
            // Update selected_track for info display
            if let Some(tracks) = self.playlist_manager.get_tracks() {
                if index < tracks.len() {
                    self.selection_state.selected_track = Some(tracks[index].clone());
                }
            }
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
            if let Err(error_message) = self.handle_copy_selected_to_playlist(playlist_id) {
                self.show_error_dialog("コピーエラー", &error_message);
            }
        }
        if let Some(playlist_id) = move_to_playlist {
            if let Err(error_message) = self.handle_move_selected_to_playlist(playlist_id) {
                self.show_error_dialog("移動エラー", &error_message);
            }
        }
        if select_all {
            self.playlist_manager.select_all();
        }
        if clear_selection {
            self.playlist_manager.clear_selection();
        }
        if copy_to_new_playlist {
            self.handle_copy_to_new_playlist();
        }
        if move_to_new_playlist {
            self.handle_move_to_new_playlist();
        }
    }

    
    
    fn handle_copy_to_new_playlist(&mut self) {
        match self.playlist_manager.copy_selected_to_new_playlist() {
            Ok(_playlist_id) => {
                // 成功時は特に何もしない（プレイリストが作成された）
            },
            Err(error_message) => {
                self.show_error_dialog("新プレイリスト作成（コピー）エラー", &error_message);
            }
        }
    }
    
    fn handle_move_to_new_playlist(&mut self) {
        match self.playlist_manager.move_selected_to_new_playlist() {
            Ok(_playlist_id) => {
                // 成功時は特に何もしない（プレイリストが作成され、楽曲が移動された）
            },
            Err(error_message) => {
                self.show_error_dialog("新プレイリスト作成（移動）エラー", &error_message);
            }
        }
    }
    
    /// OS標準のエラーダイアログを表示
    fn show_error_dialog(&self, title: &str, message: &str) {
        rfd::MessageDialog::new()
            .set_title(title)
            .set_description(message)
            .set_level(rfd::MessageLevel::Error)
            .show();
    }

}