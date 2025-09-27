use eframe::egui;
use crate::app::MyApp;

pub struct TrackInfoUI;

impl TrackInfoUI {
    pub fn render(app: &mut MyApp, ui: &mut egui::Ui) {
        let selected_indices = app.playlist_manager.get_selected_indices();
        let selected_count = selected_indices.len();

        if selected_count == 0 {
            ui.label("楽曲を選択してください");
        } else if selected_count == 1 {
            // 単一選択の場合は従来通り
            if let Some(track) = app.selection_state.selected_track.clone() {
                ui.heading("📋 選択中の楽曲");
                Self::show_track_details(app, ui, &track);
            }
        } else {
            // 複数選択の場合
            ui.heading(&format!("📋 選択中の楽曲 ({}曲)", selected_count));

            // 選択された楽曲の情報を取得
            let selected_tracks: Vec<crate::music::TrackInfo> = if let Some(tracks) = app.playlist_manager.get_tracks() {
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

    fn show_track_details(app: &mut MyApp, ui: &mut egui::Ui, track: &crate::music::TrackInfo) {
        // カバーアートがある場合は先に表示
        if let Some(cover_art_data) = &track.cover_art {
            Self::show_cover_art(app, ui, track, cover_art_data);
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

    fn show_cover_art(app: &mut MyApp, ui: &mut egui::Ui, track: &crate::music::TrackInfo, cover_art_data: &[u8]) {
        // キャッシュから既存のテクスチャを確認
        if !app.cover_art_cache.contains_key(&track.path) {
            // 画像をデコードしてテクスチャを作成
            if let Ok(image) = image::load_from_memory(cover_art_data) {
                let rgba_image = image.to_rgba8();
                let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                let pixels = rgba_image.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                let texture = ui.ctx().load_texture("cover_art", color_image, egui::TextureOptions::default());
                app.cover_art_cache.insert(track.path.clone(), texture);
            }
        }

        // テクスチャがあれば表示
        if let Some(texture) = app.cover_art_cache.get(&track.path) {
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
}