use eframe::egui;
use crate::app::MyApp;

pub struct PlaylistTabs;

impl PlaylistTabs {
    pub fn show(app: &mut MyApp, ui: &mut egui::Ui) {
        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width(), ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Button)),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.add_space(2.0);
                egui::ScrollArea::horizontal()
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
                    let is_default_active = app.playlist_manager.get_active_playlist_id() == "default";
                    let is_default_playing = app.playlist_manager.get_current_playing_playlist_id() == Some("default")
                        && app.playlist_manager.get_current_track().is_some();
                    let default_label = if is_default_playing {
                        "🎵 デフォルト"  // 再生中マーク付き
                    } else {
                        "デフォルト"
                    };

                    let default_response = ui.selectable_label(is_default_active, default_label);
                    if default_response.clicked() {
                        app.playlist_manager.set_active_playlist("default");
                        app.selection_state.selected_track = None; // Reset selected track when changing playlist
                        app.save_settings();
                    }

                    // デフォルトプレイリストの右クリックメニュー
                    default_response.context_menu(|ui| {
                        let track_count = app.playlist_manager.get_playlist("default")
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
                    let playlists = app.playlist_manager.get_playlists().clone();

                    for playlist in &playlists {
                        if playlist.id == "default" {
                            continue; // デフォルトは既に表示済み
                        }

                        let is_active = app.playlist_manager.get_active_playlist_id() == playlist.id;
                        let is_editing = app.playlist_edit_state.editing_playlist_id.as_ref() == Some(&playlist.id);
                        let is_playing = app.playlist_manager.get_current_playing_playlist_id() == Some(&playlist.id)
                            && app.playlist_manager.get_current_track().is_some();

                        if is_editing {
                            // 編集モード：テキスト入力フィールドを表示
                            let response = ui.text_edit_singleline(&mut app.playlist_edit_state.editing_playlist_name);

                            // フォーカスを設定（初回のみ）
                            if response.gained_focus() {
                                response.request_focus();
                            }

                            // Enter/Escapeキーの処理
                            if response.lost_focus() || ui.input(|i| i.key_pressed(eframe::egui::Key::Enter)) {
                                playlist_rename_result = Some((playlist.id.clone(), app.playlist_edit_state.editing_playlist_name.clone()));
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
                                let track_count = app.playlist_manager.get_playlist(&playlist.id)
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
                        let user_playlist_count = app.playlist_manager.get_playlists()
                            .iter()
                            .filter(|p| p.id != "default")
                            .count();
                        let new_name = format!("新しいプレイリスト{}", user_playlist_count + 1);
                        let new_id = app.playlist_manager.create_playlist(new_name);
                        app.playlist_manager.set_active_playlist(&new_id);
                        app.selection_state.selected_track = None; // Reset selected track when changing playlist

                        app.settings.add_to_display_order(new_id);

                        let _ = app.playlist_manager.auto_save();
                        app.save_settings();
                    }

                    // アクション実行（借用チェッカー対応）
                    if let Some(id) = playlist_to_activate {
                        app.playlist_manager.set_active_playlist(&id);
                        app.selection_state.selected_track = None; // Reset selected track when changing playlist
                        app.save_settings();
                    }
                    if let Some(id) = playlist_to_delete {
                        if app.playlist_manager.delete_playlist(&id) {
                            app.settings.remove_from_display_order(&id);
                            let _ = app.playlist_manager.auto_save();
                            app.save_settings();
                        }
                    }
                    if let Some((id, name)) = playlist_to_start_editing {
                        app.playlist_edit_state.editing_playlist_id = Some(id);
                        app.playlist_edit_state.editing_playlist_name = name;
                    }
                    if let Some((id, new_name)) = playlist_rename_result {
                        if app.playlist_manager.rename_playlist(&id, new_name) {
                            let _ = app.playlist_manager.auto_save();
                            app.save_settings();
                        }
                        app.playlist_edit_state.editing_playlist_id = None;
                        app.playlist_edit_state.editing_playlist_name.clear();
                    }
                    if cancel_editing {
                        app.playlist_edit_state.editing_playlist_id = None;
                        app.playlist_edit_state.editing_playlist_name.clear();
                    }
                    if let Some(playlist_id) = playlist_to_clear {
                        if let Some(playlist) = app.playlist_manager.get_playlist_mut(&playlist_id) {
                            // プレイリストをクリア
                            playlist.clear();

                            // もしクリアしたプレイリストが現在再生中だった場合、再生を停止
                            if app.playlist_manager.get_current_playing_playlist_id() == Some(&playlist_id) {
                                app.player_state.audio_player.stop();
                                app.playlist_manager.set_current_playing_index(None);
                            }

                            let _ = app.playlist_manager.auto_save();
                            app.save_settings();
                        }
                    }
                        });
                    });

            }
        );
    }
}