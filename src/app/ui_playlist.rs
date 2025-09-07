use super::MyApp;
use crate::ui::PlaybackControlsUI;
use eframe::egui;

impl MyApp {
    pub fn show_right_pane(&mut self, ui: &mut egui::Ui) {
        let available_rect = ui.available_rect_before_wrap();
        let available_height = available_rect.height();
        
        // 1. 再生コントロール（上部）の高さを計算
        let controls_height = available_height * self.ui_state.right_top_bottom_position;
        
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
                self.ui_state.right_top_bottom_position = new_controls_height / available_height;
            }
        }
        
        // カーソル変更
        if top_bottom_separator_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
        
        let controls_height = available_height * self.ui_state.right_top_bottom_position;
        
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
        let bottom_left_width = bottom_rect.width() * self.ui_state.right_bottom_left_right_position;
        
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
                self.ui_state.right_bottom_left_right_position = new_left_width / bottom_rect.width();
            }
        }
        
        // カーソル変更
        if left_right_separator_response.hovered() {
            bottom_ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        
        let bottom_left_width = bottom_rect.width() * self.ui_state.right_bottom_left_right_position;
        
        // 下部左側：プレイリスト関連
        let bottom_left_rect = egui::Rect::from_min_size(
            bottom_rect.min,
            egui::Vec2::new(bottom_left_width - 2.0, bottom_rect.height())
        );
        let mut bottom_left_ui = bottom_ui.child_ui(bottom_left_rect, egui::Layout::top_down(egui::Align::LEFT), None);
        bottom_left_ui.set_clip_rect(bottom_left_rect);
        
        bottom_left_ui.vertical(|ui| {
            // Add 3px top padding for playlist tab area
            ui.add_space(3.0);
            
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
            // Add 5px top padding for info/LRC tab area
            ui.add_space(5.0);
            
            // Add 5px left padding for tab header
            ui.horizontal(|ui| {
                ui.add_space(5.0); // Left padding
                // 情報・LRCタブ切り替え
                ui.selectable_value(&mut self.ui_state.right_pane_tab, crate::app::state::RightTab::Info, "情報");
                ui.selectable_value(&mut self.ui_state.right_pane_tab, crate::app::state::RightTab::Lrc, "LRC");
            });
            
            ui.separator();
            
            // タブのコンテンツを表示（残りのスペースを使用） - same structure as playlist area
            egui::ScrollArea::both()
                .id_source("info_lrc_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Add 5px left padding inside scroll area
                    ui.horizontal(|ui| {
                        ui.add_space(5.0);
                        ui.vertical(|ui| {
                            match self.ui_state.right_pane_tab {
                                crate::app::state::RightTab::Playback => {
                                    ui.label("プレイリスト表示（左側で表示）");
                                },
                                crate::app::state::RightTab::Info => {
                                    self.show_track_info(ui);
                                },
                                crate::app::state::RightTab::Lrc => {
                                    ui.label("LRC歌詞表示機能は未実装です");
                                },
                            }
                        });
                    });
                });
        });
    }

    pub fn show_track_info(&mut self, ui: &mut egui::Ui) {
        let selected_indices = self.playlist_manager.get_selected_indices();
        let selected_count = selected_indices.len();
        
        if selected_count == 0 {
            ui.label("楽曲を選択してください");
        } else if selected_count == 1 {
            // 単一選択の場合は従来通り
            if let Some(track) = self.selection_state.selected_track.clone() {
                ui.heading("📋 選択中の楽曲");
                self.show_track_details(ui, &track);
            }
        } else {
            // 複数選択の場合
            ui.heading(&format!("📋 選択中の楽曲 ({}曲)", selected_count));
            
            // 選択された楽曲の情報を取得
            let selected_tracks: Vec<crate::music::TrackInfo> = if let Some(tracks) = self.playlist_manager.get_tracks() {
                selected_indices.iter()
                    .filter_map(|&index| tracks.get(index).cloned())
                    .collect()
            } else {
                Vec::new()
            };
                
            if !selected_tracks.is_empty() {
                Self::show_multiple_tracks_details_static(ui, &selected_tracks);
            }
        }
    }

    fn show_track_details(&mut self, ui: &mut egui::Ui, track: &crate::music::TrackInfo) {
        // カバーアートがある場合は先に表示
        if let Some(cover_art_data) = &track.cover_art {
            self.show_cover_art(ui, track, cover_art_data);
            ui.add_space(10.0);
        }
        
        egui::Grid::new("track_info_grid")
            .num_columns(2)
            .spacing([15.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                        // 基本情報
                        ui.strong("タイトル:");
                        ui.label(&track.title);
                        ui.end_row();
                        
                        ui.strong("アーティスト:");
                        ui.label(&track.artist);
                        ui.end_row();
                        
                        ui.strong("アルバムアーティスト:");
                        ui.label(track.album_artist.as_deref().unwrap_or(""));
                        ui.end_row();
                        
                        ui.strong("アルバム:");
                        ui.label(&track.album);
                        ui.end_row();
                        
                        ui.strong("作曲者:");
                        ui.label(track.composer.as_deref().unwrap_or(""));
                        ui.end_row();
                        
                        ui.strong("ジャンル:");
                        ui.label(track.genre.as_deref().unwrap_or(""));
                        ui.end_row();
                        
                        ui.strong("トラック番号:");
                        match (track.track_number, track.track_total) {
                            (Some(track_num), Some(track_total)) => ui.label(format!("{}/{}", track_num, track_total)),
                            (Some(track_num), None) => ui.label(track_num.to_string()),
                            (None, Some(track_total)) => ui.label(format!("?/{}", track_total)),
                            (None, None) => ui.label(""),
                        };
                        ui.end_row();
                        
                        ui.strong("ディスク番号:");
                        match (track.disc_number, track.disc_total) {
                            (Some(disc_num), Some(disc_total)) => ui.label(format!("{}/{}", disc_num, disc_total)),
                            (Some(disc_num), None) => ui.label(disc_num.to_string()),
                            (None, Some(disc_total)) => ui.label(format!("?/{}", disc_total)),
                            (None, None) => ui.label(""),
                        };
                        ui.end_row();
                        
                        ui.strong("日付:");
                        ui.label(track.date.as_deref().unwrap_or(""));
                        ui.end_row();
                        
                        ui.strong("カバーアート:");
                        if track.cover_art.is_some() {
                            ui.label("あり");
                        } else {
                            ui.label("なし");
                        }
                        ui.end_row();
                        
                        // ファイル情報
                        ui.strong("ファイル名:");
                        if let Some(filename) = track.path.file_name() {
                            ui.label(filename.to_string_lossy().to_string());
                        } else {
                            ui.label("N/A");
                        }
                        ui.end_row();
                        
                        ui.strong("ファイル形式:");
                        if let Some(extension) = track.path.extension() {
                            ui.label(extension.to_string_lossy().to_uppercase());
                        } else {
                            ui.label("N/A");
                        }
                        ui.end_row();
                        
                        // ファイルパス（折り返し表示）
                        ui.strong("ファイルパス:");
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            ui.add(
                                egui::Label::new(track.path.display().to_string())
                                    .wrap()
                                    .selectable(true)
                            );
                        });
                        ui.end_row();
                    });
    }

    fn show_multiple_tracks_details_static(ui: &mut egui::Ui, tracks: &[crate::music::TrackInfo]) {
        if tracks.is_empty() {
            return;
        }

        // カバーアートの処理
        Self::show_multiple_cover_arts(ui, tracks);
        ui.add_space(10.0);

        // 共通の値を持つかどうかを判定するヘルパー関数
        let get_unified_string = |get_field: fn(&crate::music::TrackInfo) -> &str| -> String {
            let first_value = get_field(&tracks[0]);
            if tracks.iter().all(|track| get_field(track) == first_value) {
                first_value.to_string()
            } else {
                "複数の値があります".to_string()
            }
        };

        let get_unified_option_string = |get_field: fn(&crate::music::TrackInfo) -> &Option<String>| -> String {
            let first_value = get_field(&tracks[0]);
            if tracks.iter().all(|track| get_field(track) == first_value) {
                first_value.as_deref().unwrap_or("").to_string()
            } else {
                "複数の値があります".to_string()
            }
        };

        let _get_unified_option_u32 = |get_field: fn(&crate::music::TrackInfo) -> &Option<u32>| -> String {
            let first_value = get_field(&tracks[0]);
            if tracks.iter().all(|track| get_field(track) == first_value) {
                match first_value {
                    Some(value) => value.to_string(),
                    None => "".to_string(),
                }
            } else {
                "複数の値があります".to_string()
            }
        };

        egui::Grid::new("multiple_track_info_grid")
            .num_columns(2)
            .spacing([15.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                // 基本情報
                ui.strong("タイトル:");
                ui.label(get_unified_string(|track| &track.title));
                ui.end_row();
                
                ui.strong("アーティスト:");
                ui.label(get_unified_string(|track| &track.artist));
                ui.end_row();
                
                ui.strong("アルバムアーティスト:");
                ui.label(get_unified_option_string(|track| &track.album_artist));
                ui.end_row();
                
                ui.strong("アルバム:");
                ui.label(get_unified_string(|track| &track.album));
                ui.end_row();
                
                ui.strong("作曲者:");
                ui.label(get_unified_option_string(|track| &track.composer));
                ui.end_row();
                
                ui.strong("ジャンル:");
                ui.label(get_unified_option_string(|track| &track.genre));
                ui.end_row();
                
                // トラック番号（複雑な比較）
                ui.strong("トラック番号:");
                let first_track = &tracks[0];
                let track_display = if tracks.iter().all(|track| 
                    track.track_number == first_track.track_number && 
                    track.track_total == first_track.track_total
                ) {
                    match (first_track.track_number, first_track.track_total) {
                        (Some(track_num), Some(track_total)) => format!("{}/{}", track_num, track_total),
                        (Some(track_num), None) => track_num.to_string(),
                        (None, Some(track_total)) => format!("?/{}", track_total),
                        (None, None) => "".to_string(),
                    }
                } else {
                    "複数の値があります".to_string()
                };
                ui.label(track_display);
                ui.end_row();
                
                // ディスク番号（複雑な比較）
                ui.strong("ディスク番号:");
                let disc_display = if tracks.iter().all(|track| 
                    track.disc_number == first_track.disc_number && 
                    track.disc_total == first_track.disc_total
                ) {
                    match (first_track.disc_number, first_track.disc_total) {
                        (Some(disc_num), Some(disc_total)) => format!("{}/{}", disc_num, disc_total),
                        (Some(disc_num), None) => disc_num.to_string(),
                        (None, Some(disc_total)) => format!("?/{}", disc_total),
                        (None, None) => "".to_string(),
                    }
                } else {
                    "複数の値があります".to_string()
                };
                ui.label(disc_display);
                ui.end_row();
                
                ui.strong("日付:");
                ui.label(get_unified_option_string(|track| &track.date));
                ui.end_row();
                
                // カバーアート
                ui.strong("カバーアート:");
                let first_has_cover = tracks[0].cover_art.is_some();
                if tracks.iter().all(|track| track.cover_art.is_some() == first_has_cover) {
                    if first_has_cover {
                        ui.label("あり");
                    } else {
                        ui.label("なし");
                    }
                } else {
                    ui.label("複数の値があります");
                }
                ui.end_row();
                
                // ファイル形式
                ui.strong("ファイル形式:");
                let first_extension = tracks[0].path.extension()
                    .map(|ext| ext.to_string_lossy().to_uppercase())
                    .unwrap_or_else(|| "N/A".into());
                if tracks.iter().all(|track| {
                    track.path.extension()
                        .map(|ext| ext.to_string_lossy().to_uppercase())
                        .unwrap_or_else(|| "N/A".into()) == first_extension
                }) {
                    ui.label(first_extension);
                } else {
                    ui.label("複数の値があります");
                }
                ui.end_row();
            });
    }
    
    fn show_cover_art(&mut self, ui: &mut egui::Ui, track: &crate::music::TrackInfo, cover_art_data: &[u8]) {
        // キャッシュから既存のテクスチャを確認
        if !self.cover_art_cache.contains_key(&track.path) {
            // 画像をデコードしてテクスチャを作成
            if let Ok(image) = image::load_from_memory(cover_art_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let pixels = rgba_image.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                let texture = ui.ctx().load_texture("cover_art", color_image, egui::TextureOptions::default());
                self.cover_art_cache.insert(track.path.clone(), texture);
            }
        }
        
        // テクスチャがあれば表示
        if let Some(texture) = self.cover_art_cache.get(&track.path) {
            let _available_width = ui.available_width();
            let max_size = 200.0; // 最大サイズを200pxに制限
            let image_size = texture.size_vec2();
            let scale = (max_size / image_size.x.max(image_size.y)).min(1.0);
            let scaled_size = image_size * scale;
            
            // 画像を中央揃えで表示
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add(egui::Image::from_texture(texture).max_size(scaled_size));
            });
        }
    }

    fn show_multiple_cover_arts(ui: &mut egui::Ui, tracks: &[crate::music::TrackInfo]) {
        if tracks.is_empty() {
            return;
        }

        // カバーアートの比較
        let first_cover_art = &tracks[0].cover_art;
        let all_same_cover = tracks.iter().all(|track| {
            match (&track.cover_art, first_cover_art) {
                (Some(data1), Some(data2)) => data1 == data2,
                (None, None) => true,
                _ => false,
            }
        });

        let max_size = 200.0; // 単一選択時と同じサイズ

        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            if all_same_cover {
                if let Some(cover_art_data) = first_cover_art {
                    // 同じカバーアートの場合は実際の画像を表示
                    if let Ok(image) = image::load_from_memory(cover_art_data) {
                        let rgba_image = image.to_rgba8();
                        let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                        let pixels = rgba_image.as_flat_samples();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                        let texture = ui.ctx().load_texture("multiple_cover_art", color_image, egui::TextureOptions::default());
                        
                        let image_size = texture.size_vec2();
                        let scale = (max_size / image_size.x.max(image_size.y)).min(1.0);
                        let scaled_size = image_size * scale;
                        
                        ui.add(egui::Image::from_texture(&texture).max_size(scaled_size));
                    } else {
                        // 画像の読み込みに失敗した場合
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(max_size, max_size),
                            egui::Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| {
                                ui.label("画像の読み込みに失敗しました");
                            }
                        );
                    }
                } else {
                    // 全ての楽曲にカバーアートがない場合
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(max_size, max_size),
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            ui.label("カバーアートがありません");
                        }
                    );
                }
            } else {
                // 異なるカバーアートがある場合
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(max_size, max_size),
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.label("複数のカバーアートがあります");
                    }
                );
            }
        });
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

    pub fn show_playback_controls_only(&mut self, ui: &mut egui::Ui) {
        let playback_state = self.player_state.audio_player.get_state().clone();
        
        // Collect actions (removed clear_queue)
        let mut previous_clicked = false;
        let mut seek_backward_clicked = false;
        let mut play_pause_clicked = false;
        let mut stop_clicked = false;
        let mut seek_forward_clicked = false;
        let mut next_clicked = false;
        let mut seek_position: Option<std::time::Duration> = None;
        let mut seek_started = false;
        let mut seek_ended = false;
        
        // Auto focus disabled
        let _auto_focus = false;
        
        // 再生位置と総再生時間を取得
        let current_position = self.player_state.audio_player.get_playback_position();
        let total_duration = self.player_state.audio_player.get_total_duration();
        
        // 再生中の場合はUIを継続的に更新
        if playback_state == crate::player::PlaybackState::Playing {
            ui.ctx().request_repaint();
        }
        
        let current_track = self.playlist_manager.get_current_track();
        
        // リピート・シャッフルモードの変更処理用変数
        let mut repeat_mode_changed = false;
        let mut new_repeat_mode = self.player_state.repeat_mode.clone();
        let mut shuffle_changed = false;
        let mut new_shuffle_enabled = self.player_state.shuffle_enabled;
        
        let seek_points = self.get_current_track_seek_points();
        
        PlaybackControlsUI::show_controls_with_seek_bar(
            ui,
            &playback_state,
            current_position,
            total_duration,
            current_track,
            seek_points,
            &mut || previous_clicked = true,
            &mut || seek_backward_clicked = true,
            &mut || play_pause_clicked = true,
            &mut || stop_clicked = true,
            &mut || seek_forward_clicked = true,
            &mut || next_clicked = true,
            &mut |position| seek_position = Some(position),
            &mut || seek_started = true,
            &mut || seek_ended = true,
            _auto_focus,
            &self.player_state.repeat_mode,
            self.player_state.shuffle_enabled,
            &mut |mode| {
                new_repeat_mode = mode;
                repeat_mode_changed = true;
            },
            &mut |enabled| {
                new_shuffle_enabled = enabled;
                shuffle_changed = true;
            },
        );
        
        // Handle actions after UI (removed clear_queue handling)
        if previous_clicked {
            self.handle_previous_button();
        }
        if seek_backward_clicked {
            self.handle_seek_backward();
        }
        if play_pause_clicked {
            self.handle_play_pause();
        }
        if stop_clicked {
            self.handle_stop();
        }
        if seek_forward_clicked {
            self.handle_seek_forward();
        }
        if next_clicked {
            self.handle_next();
        }
        if let Some(position) = seek_position {
            self.handle_seek_to_position(position);
        }
        if seek_started {
            self.handle_seek_start();
        }
        if seek_ended {
            self.handle_seek_end();
        }
        
        // リピート・シャッフルモードの変更処理（永続化なし）
        if repeat_mode_changed {
            self.player_state.repeat_mode = new_repeat_mode;
        }
        if shuffle_changed {
            self.player_state.shuffle_enabled = new_shuffle_enabled;
            self.playlist_manager.update_shuffle_when_settings_changed(new_shuffle_enabled);
        }
        
        // Focus flag reset removed - auto focus disabled
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