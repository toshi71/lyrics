use eframe::egui;
use crate::app::MyApp;

pub struct DebugPlaybackControls;

impl DebugPlaybackControls {
    #[allow(clippy::too_many_arguments)]
    pub fn show_controls_with_seek_bar_debug(
        app: &mut MyApp,
        ui: &mut egui::Ui,
        playback_state: &crate::player::PlaybackState,
        current_position: std::time::Duration,
        total_duration: Option<std::time::Duration>,
        on_previous: &mut dyn FnMut(),
        on_seek_backward: &mut dyn FnMut(),
        on_play_pause: &mut dyn FnMut(),
        on_stop: &mut dyn FnMut(),
        on_seek_forward: &mut dyn FnMut(),
        on_next: &mut dyn FnMut(),
        on_seek: &mut dyn FnMut(std::time::Duration),
        on_seek_start: &mut dyn FnMut(),
        on_seek_end: &mut dyn FnMut(),
        _auto_focus: bool,
        repeat_mode: &crate::settings::RepeatMode,
        shuffle_enabled: bool,
        on_repeat_mode_change: &mut dyn FnMut(crate::settings::RepeatMode),
        on_shuffle_change: &mut dyn FnMut(bool),
        on_add_seek_point: &mut dyn FnMut(),
        on_seek_to_point: &mut dyn FnMut(u64),
    ) {
        // 必要なデータを取得
        let current_track = app.playlist_manager.get_current_track();
        let seek_points = app.get_current_track_seek_points();

        // PlaybackControls全体の利用可能領域を取得
        let total_available_rect = ui.available_rect_before_wrap();

        // シークバーを最初に表示（横幅全体を使用）
        let seek_bar_height = 40.0; // 固定の高さ
        let seek_bar_rect = egui::Rect::from_min_size(
            total_available_rect.min,
            egui::Vec2::new(total_available_rect.width(), seek_bar_height)
        );

        // シークバー領域のデバッグ描画
        app.ui_state.debug_ui.draw_debug_rect_fixed(ui, seek_bar_rect, crate::debug_ui::ID_SEEK_BAR, "SeekBar");

        // シークバーの実際の描画
        crate::ui::PlaybackControlsUI::show_seek_bar(ui, current_position, total_duration, seek_points, on_seek, on_seek_start, on_seek_end);

        let space_height = 10.0;
        ui.add_space(space_height);

        // 左右分割エリアの高さを計算
        let controls_area_height = total_available_rect.height() - seek_bar_height - space_height;
        let controls_area_rect = egui::Rect::from_min_size(
            total_available_rect.min + egui::Vec2::new(0.0, seek_bar_height + space_height),
            egui::Vec2::new(total_available_rect.width(), controls_area_height)
        );

        // 左右分割レイアウト
        ui.horizontal(|ui| {
            let left_width = 380.0; // 左側の固定幅
            let separator_width = 40.0; // セパレーター部分の幅（スペース + セパレーター + スペース）
            let right_width = controls_area_rect.width() - left_width - separator_width;

            // 左側領域の正確なサイズ計算
            let left_rect = egui::Rect::from_min_size(
                controls_area_rect.min,
                egui::Vec2::new(left_width, controls_area_height)
            );

            // 左側領域のデバッグ描画
            app.ui_state.debug_ui.draw_debug_rect_fixed(ui, left_rect, crate::debug_ui::ID_LEFT_CONTROLS, "LeftControls");

            // 左側: 再生コントロール、シークポイント追加、リピート・シャッフル
            ui.allocate_ui_with_layout(
                egui::Vec2::new(left_width, controls_area_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    Self::show_playback_buttons(ui, playback_state, on_previous, on_seek_backward, on_play_pause, on_stop, on_seek_forward, on_next);

                    ui.add_space(15.0);

                    // シークポイント追加ボタン
                    if ui.button("📍 現在位置にシークポイントを追加").clicked() {
                        on_add_seek_point();
                    }

                    ui.add_space(15.0);

                    Self::show_repeat_shuffle_controls(ui, repeat_mode, shuffle_enabled, on_repeat_mode_change, on_shuffle_change);
                }
            );

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // 右側領域の正確なサイズ計算
            let right_rect = egui::Rect::from_min_size(
                controls_area_rect.min + egui::Vec2::new(left_width + separator_width, 0.0),
                egui::Vec2::new(right_width, controls_area_height)
            );

            // 右側領域のデバッグ描画
            app.ui_state.debug_ui.draw_debug_rect_fixed(ui, right_rect, crate::debug_ui::ID_RIGHT_INFO, "RightInfo");

            // 右側: 楽曲情報とシークポイント一覧
            ui.allocate_ui_with_layout(
                egui::Vec2::new(right_width, controls_area_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    if let Some(track) = current_track {
                        // 楽曲情報表示領域の開始位置を記録
                        let track_info_start_pos = ui.next_widget_position();

                        // 楽曲情報を実際に描画
                        ui.label(egui::RichText::new(&track.title).strong());
                        ui.label(format!("{} - {}", track.artist, track.album));

                        // 楽曲情報表示領域の終了位置を取得
                        let track_info_end_pos = ui.next_widget_position();
                        let actual_track_info_height = track_info_end_pos.y - track_info_start_pos.y;

                        // 楽曲情報領域のデバッグ描画（実際のサイズで）
                        let track_info_rect = egui::Rect::from_min_size(
                            track_info_start_pos,
                            egui::Vec2::new(right_width, actual_track_info_height)
                        );
                        app.ui_state.debug_ui.draw_debug_rect_fixed(ui, track_info_rect, crate::debug_ui::ID_TRACK_INFO, "TrackInfo");

                        // スペースを追加
                        let space_height = 15.0;
                        ui.add_space(space_height);

                        // シークポイント一覧領域の開始位置
                        let seek_points_start_pos = ui.next_widget_position();

                        // 残りの高さを正確に計算
                        let used_height = actual_track_info_height + space_height;
                        let seek_points_list_height = controls_area_height - used_height;

                        // シークポイント一覧領域のデバッグ描画（正確なサイズで）
                        let seek_points_rect = egui::Rect::from_min_size(
                            seek_points_start_pos,
                            egui::Vec2::new(right_width, seek_points_list_height)
                        );
                        app.ui_state.debug_ui.draw_debug_rect_fixed(ui, seek_points_rect, crate::debug_ui::ID_SEEK_POINTS_LIST, "SeekPointsList");

                        // 残りのスペースでシークポイント一覧を表示
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(right_width, seek_points_list_height),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                crate::ui::PlaybackControlsUI::show_current_track_seek_points(ui, seek_points, on_seek_to_point);
                            }
                        );
                    } else {
                        ui.label("楽曲が選択されていません");
                    }
                }
            );
        });
    }

    fn show_playback_buttons(
        ui: &mut egui::Ui,
        playback_state: &crate::player::PlaybackState,
        on_previous: &mut dyn FnMut(),
        on_seek_backward: &mut dyn FnMut(),
        on_play_pause: &mut dyn FnMut(),
        on_stop: &mut dyn FnMut(),
        on_seek_forward: &mut dyn FnMut(),
        on_next: &mut dyn FnMut(),
    ) {
        // 再生コントロールボタン
        ui.horizontal(|ui| {
            ui.add_space(5.0);

            let button_size = [48.0, 48.0];

            // Previous button
            if ui.add_sized(button_size,
                egui::Button::new(
                    egui::RichText::new("⏮").size(24.0)
                )
            ).clicked() {
                on_previous();
            }

            ui.add_space(5.0);

            // Seek backward button
            if ui.add_sized(button_size,
                egui::Button::new(
                    egui::RichText::new("↩").size(24.0)
                )
            ).clicked() {
                on_seek_backward();
            }

            ui.add_space(10.0);

            // Play/pause button
            let play_pause_text = match playback_state {
                crate::player::PlaybackState::Playing => "⏸",
                _ => "▶",
            };
            if ui.add_sized(button_size,
                egui::Button::new(
                    egui::RichText::new(play_pause_text).size(24.0)
                )
            ).clicked() {
                on_play_pause();
            }

            ui.add_space(10.0);

            // Stop button
            if ui.add_sized(button_size,
                egui::Button::new(
                    egui::RichText::new("⏹").size(24.0)
                )
            ).clicked() {
                on_stop();
            }

            ui.add_space(5.0);

            // Seek forward button
            if ui.add_sized(button_size,
                egui::Button::new(
                    egui::RichText::new("↪").size(24.0)
                )
            ).clicked() {
                on_seek_forward();
            }

            ui.add_space(10.0);

            // Next button
            if ui.add_sized(button_size,
                egui::Button::new(
                    egui::RichText::new("⏭").size(24.0)
                )
            ).clicked() {
                on_next();
            }
        });
    }

    fn show_repeat_shuffle_controls(
        ui: &mut egui::Ui,
        repeat_mode: &crate::settings::RepeatMode,
        shuffle_enabled: bool,
        on_repeat_mode_change: &mut dyn FnMut(crate::settings::RepeatMode),
        on_shuffle_change: &mut dyn FnMut(bool),
    ) {
        // リピート・シャッフル設定
        ui.horizontal(|ui| {
            ui.label("リピート:");
            ui.add_space(10.0);

            let repeat_text = match repeat_mode {
                crate::settings::RepeatMode::Normal => "オフ",
                crate::settings::RepeatMode::RepeatOne => "1曲",
                crate::settings::RepeatMode::RepeatAll => "全曲",
            };

            let mut new_repeat_mode = repeat_mode.clone();
            egui::ComboBox::from_id_source("repeat_selector")
                .selected_text(repeat_text)
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut new_repeat_mode, crate::settings::RepeatMode::Normal, "オフ").changed() {
                        on_repeat_mode_change(crate::settings::RepeatMode::Normal);
                    }
                    if ui.selectable_value(&mut new_repeat_mode, crate::settings::RepeatMode::RepeatOne, "1曲").changed() {
                        on_repeat_mode_change(crate::settings::RepeatMode::RepeatOne);
                    }
                    if ui.selectable_value(&mut new_repeat_mode, crate::settings::RepeatMode::RepeatAll, "全曲").changed() {
                        on_repeat_mode_change(crate::settings::RepeatMode::RepeatAll);
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("シャッフル:");
            ui.add_space(10.0);

            let shuffle_text = if shuffle_enabled { "オン" } else { "オフ" };
            let mut new_shuffle_enabled = shuffle_enabled;
            egui::ComboBox::from_id_source("shuffle_selector")
                .selected_text(shuffle_text)
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut new_shuffle_enabled, false, "オフ").changed() {
                        on_shuffle_change(false);
                    }
                    if ui.selectable_value(&mut new_shuffle_enabled, true, "オン").changed() {
                        on_shuffle_change(true);
                    }
                });
        });
    }
}