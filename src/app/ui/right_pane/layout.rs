use eframe::egui;
use crate::app::MyApp;

pub struct RightPaneLayout;

impl RightPaneLayout {
    pub fn show_right_pane(app: &mut MyApp, ui: &mut egui::Ui) {
        let available_rect = ui.available_rect_before_wrap();
        let available_height = available_rect.height();

        // 右ペイン全体のデバッグ描画
        app.debug_ui.draw_debug_rect_fixed(ui, available_rect, crate::debug_ui::ID_RIGHT_PANE_INNER, "RightPaneInner");

        // 1. 再生コントロール（上部）の高さを計算
        let controls_height = available_height * app.ui_state.right_top_bottom_position;

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
                app.ui_state.right_top_bottom_position = new_controls_height / available_height;
            }
        }

        // カーソル変更
        if top_bottom_separator_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }

        let controls_height = available_height * app.ui_state.right_top_bottom_position;

        // 上部：再生コントロール
        let top_rect = egui::Rect::from_min_size(
            available_rect.min,
            egui::Vec2::new(available_rect.width(), controls_height - 2.0)
        );
        let mut top_ui = ui.child_ui(top_rect, egui::Layout::top_down(egui::Align::LEFT), None);
        top_ui.set_clip_rect(top_rect);

        // 上部コントロール領域のデバッグ描画
        app.debug_ui.draw_debug_rect_fixed(ui, top_rect, crate::debug_ui::ID_PLAYBACK_CONTROLS, "PlaybackControls");

        top_ui.vertical(|ui| {
            crate::app::ui::right_pane::PlaybackControlsOnly::show(app, ui);
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

        // 下部領域のデバッグ描画
        app.debug_ui.draw_debug_rect_fixed(ui, bottom_rect, crate::debug_ui::ID_BOTTOM_AREA, "BottomArea");

        Self::show_bottom_split_area(app, &mut bottom_ui, bottom_rect);
    }

    fn show_bottom_split_area(app: &mut MyApp, bottom_ui: &mut egui::Ui, bottom_rect: egui::Rect) {
        // 下部の左右分割
        let bottom_left_width = bottom_rect.width() * app.ui_state.right_bottom_left_right_position;

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
                app.ui_state.right_bottom_left_right_position = new_left_width / bottom_rect.width();
            }
        }

        // カーソル変更
        if left_right_separator_response.hovered() {
            bottom_ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }

        let bottom_left_width = bottom_rect.width() * app.ui_state.right_bottom_left_right_position;

        // 下部左側：プレイリスト関連
        let bottom_left_rect = egui::Rect::from_min_size(
            bottom_rect.min,
            egui::Vec2::new(bottom_left_width - 2.0, bottom_rect.height())
        );
        let mut bottom_left_ui = bottom_ui.child_ui(bottom_left_rect, egui::Layout::top_down(egui::Align::LEFT), None);
        bottom_left_ui.set_clip_rect(bottom_left_rect);

        // 下部左側のデバッグ描画
        app.debug_ui.draw_debug_rect_fixed(bottom_ui, bottom_left_rect, crate::debug_ui::ID_PLAYLIST_AREA, "PlaylistArea");

        bottom_left_ui.vertical(|ui| {
            // Add 3px top padding for playlist tab area
            ui.add_space(3.0);

            // プレイリストタブ
            app.show_playlist_tabs(ui);
            ui.separator();

            // プレイリスト楽曲表示（残りのスペースを使用）
            egui::ScrollArea::both()
                .id_source("playlist_scroll")
                .auto_shrink([false, false])
                .hscroll(true)
                .vscroll(true)
                .show(ui, |ui| {
                    app.show_playlist_list(ui);
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

        // 下部右側のデバッグ描画
        app.debug_ui.draw_debug_rect_fixed(bottom_ui, bottom_right_rect, crate::debug_ui::ID_INFO_TAB_AREA, "InfoTabArea");

        bottom_right_ui.vertical(|ui| {
            // Add 5px top padding for info/LRC tab area
            ui.add_space(5.0);

            // Add 5px left padding for tab header
            ui.horizontal(|ui| {
                ui.add_space(5.0); // Left padding
                // 情報・シークポイント・LRCタブ切り替え
                ui.selectable_value(&mut app.ui_state.right_pane_tab, crate::app::state::RightTab::Info, "情報");
                ui.selectable_value(&mut app.ui_state.right_pane_tab, crate::app::state::RightTab::SeekPoints, "シークポイント");
                ui.selectable_value(&mut app.ui_state.right_pane_tab, crate::app::state::RightTab::Lrc, "LRC");
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
                            match app.ui_state.right_pane_tab {
                                crate::app::state::RightTab::Playback => {
                                    ui.label("プレイリスト表示（左側で表示）");
                                },
                                crate::app::state::RightTab::Info => {
                                    crate::app::ui::right_pane::TrackInfo::show(app, ui);
                                },
                                crate::app::state::RightTab::SeekPoints => {
                                    crate::app::ui::right_pane::SeekPoints::show(app, ui);
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
}