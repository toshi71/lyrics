use super::MyApp;
use crate::ui::PlaybackControlsUI;
use eframe::egui;

impl MyApp {
    pub fn show_right_pane(&mut self, ui: &mut egui::Ui) {
        let available_rect = ui.available_rect_before_wrap();
        let available_height = available_rect.height();
        
        // 1. 再生コントロール（上部）の高さを計算
        let controls_height = available_height * self.right_top_bottom_position;
        
        // リサイズ可能な上下分割線
        let top_bottom_separator_id = ui.id().with("right_top_bottom_separator");
        let top_bottom_separator_y = available_rect.min.y + controls_height;
        let top_bottom_separator_rect = egui::Rect::from_min_size(
            egui::Pos2::new(available_rect.min.x, top_bottom_separator_y - 2.0),
            egui::Vec2::new(available_rect.width(), 4.0)
        );
        
        let top_bottom_separator_response = ui.interact(top_bottom_separator_rect, top_bottom_separator_id, egui::Sense::drag());
        if top_bottom_separator_response.dragged() {
            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                let new_controls_height = (pointer_pos.y - available_rect.min.y).max(50.0).min(available_height - 100.0);
                self.right_top_bottom_position = new_controls_height / available_height;
            }
        }
        
        // カーソル変更
        if top_bottom_separator_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
        
        let controls_height = available_height * self.right_top_bottom_position;
        
        // 上部：再生コントロール
        let top_rect = egui::Rect::from_min_size(
            available_rect.min,
            egui::Vec2::new(available_rect.width(), controls_height - 2.0)
        );
        let mut top_ui = ui.child_ui(top_rect, egui::Layout::top_down(egui::Align::LEFT), None);
        top_ui.set_clip_rect(top_rect);
        
        top_ui.vertical(|ui| {
            self.show_playback_controls_only(ui);
        });
        
        // 上下分割線の描画
        let separator_rect = egui::Rect::from_min_size(
            egui::Pos2::new(available_rect.min.x, available_rect.min.y + controls_height - 1.0),
            egui::Vec2::new(available_rect.width(), 2.0)
        );
        ui.allocate_ui_at_rect(separator_rect, |ui| {
            ui.separator();
        });
        
        // 下部のサイズ計算
        let bottom_height = available_height - controls_height - 2.0;
        let bottom_rect = egui::Rect::from_min_size(
            egui::Pos2::new(available_rect.min.x, available_rect.min.y + controls_height + 1.0),
            egui::Vec2::new(available_rect.width(), bottom_height)
        );
        let mut bottom_ui = ui.child_ui(bottom_rect, egui::Layout::top_down(egui::Align::LEFT), None);
        bottom_ui.set_clip_rect(bottom_rect);
        
        // 下部の左右分割
        let bottom_left_width = bottom_rect.width() * self.right_bottom_left_right_position;
        
        // リサイズ可能な左右分割線
        let left_right_separator_id = bottom_ui.id().with("right_bottom_left_right_separator");
        let left_right_separator_x = bottom_rect.min.x + bottom_left_width;
        let left_right_separator_rect = egui::Rect::from_min_size(
            egui::Pos2::new(left_right_separator_x - 2.0, bottom_rect.min.y),
            egui::Vec2::new(4.0, bottom_rect.height())
        );
        
        let left_right_separator_response = bottom_ui.interact(left_right_separator_rect, left_right_separator_id, egui::Sense::drag());
        if left_right_separator_response.dragged() {
            if let Some(pointer_pos) = bottom_ui.ctx().pointer_interact_pos() {
                let new_left_width = (pointer_pos.x - bottom_rect.min.x).max(50.0).min(bottom_rect.width() - 100.0);
                self.right_bottom_left_right_position = new_left_width / bottom_rect.width();
            }
        }
        
        // カーソル変更
        if left_right_separator_response.hovered() {
            bottom_ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        
        let bottom_left_width = bottom_rect.width() * self.right_bottom_left_right_position;
        
        // 下部左側：プレイリスト関連
        let bottom_left_rect = egui::Rect::from_min_size(
            bottom_rect.min,
            egui::Vec2::new(bottom_left_width - 2.0, bottom_rect.height())
        );
        let mut bottom_left_ui = bottom_ui.child_ui(bottom_left_rect, egui::Layout::top_down(egui::Align::LEFT), None);
        bottom_left_ui.set_clip_rect(bottom_left_rect);
        
        bottom_left_ui.vertical(|ui| {
            // プレイリストタブ
            self.show_playlist_tabs(ui);
            ui.separator();
            
            // プレイリスト楽曲表示（残りのスペースを使用）
            egui::ScrollArea::both()
                .id_source("playlist_scroll")
                .auto_shrink([false, false])
                .hscroll(true)
                .vscroll(true)
                .show(ui, |ui| {
                    self.show_playlist_list(ui);
                });
        });
        
        // 左右分割線の描画
        let lr_separator_rect = egui::Rect::from_min_size(
            egui::Pos2::new(bottom_rect.min.x + bottom_left_width - 1.0, bottom_rect.min.y),
            egui::Vec2::new(2.0, bottom_rect.height())
        );
        bottom_ui.allocate_ui_at_rect(lr_separator_rect, |ui| {
            ui.separator();
        });
        
        // 下部右側：情報・LRCタブ
        let bottom_right_rect = egui::Rect::from_min_size(
            egui::Pos2::new(bottom_rect.min.x + bottom_left_width + 1.0, bottom_rect.min.y),
            egui::Vec2::new(bottom_rect.width() - bottom_left_width - 1.0, bottom_rect.height())
        );
        let mut bottom_right_ui = bottom_ui.child_ui(bottom_right_rect, egui::Layout::top_down(egui::Align::LEFT), None);
        bottom_right_ui.set_clip_rect(bottom_right_rect);
        
        bottom_right_ui.vertical(|ui| {
            // 情報・LRCタブ切り替え
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.right_pane_tab, super::RightTab::Info, "情報");
                ui.selectable_value(&mut self.right_pane_tab, super::RightTab::Lrc, "LRC");
            });
            
            ui.separator();
            
            // タブのコンテンツを表示
            egui::ScrollArea::both()
                .id_source("info_lrc_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    match self.right_pane_tab {
                        super::RightTab::Playback => {
                            ui.label("プレイリスト表示（左側で表示）");
                        },
                        super::RightTab::Info => {
                            self.show_track_info(ui);
                        },
                        super::RightTab::Lrc => {
                            ui.label("LRC歌詞表示機能は未実装です");
                        },
                    }
                });
        });
    }

    pub fn show_track_info(&mut self, ui: &mut egui::Ui) {
        // 現在再生中の楽曲情報を優先表示
        if let Some(current_track) = self.playlist_manager.get_current_track() {
            ui.heading("🎵 現在再生中");
            if let Some(playing_playlist_id) = self.playlist_manager.get_current_playing_playlist_id() {
                if let Some(playlist) = self.playlist_manager.get_playlist(playing_playlist_id) {
                    ui.label(format!("プレイリスト: {}", playlist.name));
                }
            }
            ui.separator();
            
            self.show_track_details(ui, current_track);
            
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
        }
        
        if let Some(track) = &self.selected_track {
            ui.heading("選択中の楽曲");
            self.show_track_details(ui, track);
        } else if self.playlist_manager.get_current_track().is_none() {
            ui.label("楽曲を選択するか、再生を開始してください");
        }
    }

    fn show_track_details(&self, ui: &mut egui::Ui, track: &crate::music::TrackInfo) {
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
    }

    pub fn show_playlist_tabs(&mut self, ui: &mut egui::Ui) {
        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width(), ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Button)),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.add_space(2.0);
                let scroll_area_response = egui::ScrollArea::horizontal()
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
                            if response.lost_focus() || ui.input(|i| i.key_pressed(eframe::egui::Key::Enter)) {
                                playlist_rename_result = Some((playlist.id.clone(), self.editing_playlist_name.clone()));
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
                        
                        self.settings.add_to_display_order(new_id);
                        
                        let _ = self.playlist_manager.auto_save();
                        self.save_settings();
                    }
                    
                    // アクション実行（借用チェッカー対応）
                    if let Some(id) = playlist_to_activate {
                        self.playlist_manager.set_active_playlist(&id);
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
                        self.editing_playlist_id = Some(id);
                        self.editing_playlist_name = name;
                    }
                    if let Some((id, new_name)) = playlist_rename_result {
                        if self.playlist_manager.rename_playlist(&id, new_name) {
                            let _ = self.playlist_manager.auto_save();
                            self.save_settings();
                        }
                        self.editing_playlist_id = None;
                        self.editing_playlist_name.clear();
                    }
                    if cancel_editing {
                        self.editing_playlist_id = None;
                        self.editing_playlist_name.clear();
                    }
                    if let Some(playlist_id) = playlist_to_clear {
                        if let Some(playlist) = self.playlist_manager.get_playlist_mut(&playlist_id) {
                            // プレイリストをクリア
                            playlist.clear();
                            
                            // もしクリアしたプレイリストが現在再生中だった場合、再生を停止
                            if self.playlist_manager.get_current_playing_playlist_id() == Some(&playlist_id) {
                                self.audio_player.stop();
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

    pub fn show_playback_controls_only(&mut self, ui: &mut egui::Ui) {
        let playback_state = self.audio_player.get_state().clone();
        
        // Collect actions (removed clear_queue)
        let mut previous_clicked = false;
        let mut play_pause_clicked = false;
        let mut stop_clicked = false;
        let mut next_clicked = false;
        
        // 再生位置と総再生時間を取得
        let current_position = self.audio_player.get_playback_position();
        let total_duration = self.audio_player.get_total_duration();
        
        // 再生中の場合はUIを継続的に更新
        if playback_state == crate::player::PlaybackState::Playing {
            ui.ctx().request_repaint();
        }
        
        PlaybackControlsUI::show_controls_with_seek_bar(
            ui,
            &playback_state,
            current_position,
            total_duration,
            &mut || previous_clicked = true,
            &mut || play_pause_clicked = true,
            &mut || stop_clicked = true,
            &mut || next_clicked = true,
        );
        
        // Handle actions after UI (removed clear_queue handling)
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
}