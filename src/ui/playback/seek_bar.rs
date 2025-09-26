use crate::seek_points::SeekPoint;
use crate::utils::formatting::TimeFormatter;
use eframe::egui;

pub struct SeekBarUI;

impl SeekBarUI {
    pub fn show(
        ui: &mut egui::Ui,
        current_position: std::time::Duration,
        total_duration: Option<std::time::Duration>,
        seek_points: Option<&Vec<SeekPoint>>,
        on_seek: &mut dyn FnMut(std::time::Duration),
        on_seek_start: &mut dyn FnMut(),
        on_seek_end: &mut dyn FnMut(),
    ) {
        ui.horizontal(|ui| {
            // 現在の再生時間を表示
            let current_text = Self::format_duration(current_position);
            ui.label(current_text);

            ui.add_space(10.0);

            // シークバー
            if let Some(total) = total_duration {
                let progress = if total.as_secs() > 0 {
                    current_position.as_secs_f64() / total.as_secs_f64()
                } else {
                    0.0
                };

                let available_width = ui.available_width() - 80.0; // 時間表示分を差し引く

                // カスタムのクリック・ドラッグ可能なプログレスバーを作成
                let desired_size = egui::vec2(available_width, 20.0);
                let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

                // プログレスバーの背景を描画
                let bg_color = ui.style().visuals.extreme_bg_color;
                let fill_color = ui.style().visuals.selection.bg_fill;

                ui.painter().rect_filled(rect, 4.0, bg_color);

                // プログレス部分を描画
                if progress > 0.0 {
                    let progress_width = rect.width() * progress as f32;
                    let progress_rect = egui::Rect::from_min_size(
                        rect.min,
                        egui::vec2(progress_width, rect.height())
                    );
                    ui.painter().rect_filled(progress_rect, 4.0, fill_color);
                }

                // 現在の再生位置を赤い線で表示
                if progress > 0.0 {
                    let position_x = rect.left() + rect.width() * progress as f32;
                    let line_start = egui::pos2(position_x, rect.top());
                    let line_end = egui::pos2(position_x, rect.bottom());
                    ui.painter().line_segment([line_start, line_end], egui::Stroke::new(2.0, egui::Color32::RED));
                }

                // シークポイントのマーカーを表示
                if let Some(points) = seek_points {
                    for seek_point in points {
                        let point_position_secs = seek_point.position_ms as f64 / 1000.0;
                        let point_progress = if total.as_secs_f64() > 0.0 {
                            point_position_secs / total.as_secs_f64()
                        } else {
                            0.0
                        };

                        if point_progress >= 0.0 && point_progress <= 1.0 {
                            let marker_x = rect.left() + rect.width() * point_progress as f32;

                            // マーカーの三角形を描画（上向き三角）
                            let marker_size = 8.0;
                            let triangle_top = egui::pos2(marker_x, rect.top() - 2.0);
                            let triangle_left = egui::pos2(marker_x - marker_size/2.0, rect.top() + marker_size);
                            let triangle_right = egui::pos2(marker_x + marker_size/2.0, rect.top() + marker_size);

                            // マーカーのヒットボックスを作成（ホバー検知用）
                            let marker_hit_rect = egui::Rect::from_center_size(
                                egui::pos2(marker_x, rect.center().y),
                                egui::vec2(marker_size * 2.0, rect.height() + marker_size)
                            );

                            // マーカーのホバー判定とクリック検知
                            let marker_id = ui.id().with(format!("seek_marker_{}", seek_point.id));
                            let marker_response = ui.interact(marker_hit_rect, marker_id, egui::Sense::click_and_drag());

                            // ホバー状態を先に取得
                            let is_hovered = marker_response.hovered();

                            // マーカークリック時のシーク処理
                            if marker_response.clicked() {
                                let seek_position = std::time::Duration::from_millis(seek_point.position_ms);
                                on_seek(seek_position);
                            }

                            // ホバー時のツールチップ表示
                            let tooltip_text = format!(
                                "{}\n位置: {} (クリックでシーク)",
                                seek_point.name,
                                Self::format_duration(std::time::Duration::from_millis(seek_point.position_ms))
                            );
                            marker_response.on_hover_text(tooltip_text);

                            // より洗練された三角形マーカーを描画
                            let (fill_color, stroke_color, stroke_width) = if is_hovered {
                                (egui::Color32::from_rgb(50, 200, 255), egui::Color32::from_rgb(0, 150, 255), 2.0) // より明るい青・太い線
                            } else {
                                (egui::Color32::from_rgb(0, 150, 255), egui::Color32::from_rgb(0, 100, 200), 1.5) // 通常の青・中程度の線
                            };

                            // メイン三角形を描画
                            let triangle_points = vec![triangle_top, triangle_left, triangle_right];
                            ui.painter().add(egui::Shape::convex_polygon(
                                triangle_points,
                                fill_color,
                                egui::Stroke::new(stroke_width, stroke_color)
                            ));

                            // ホバー時の追加エフェクト：小さな光る円を追加
                            if is_hovered {
                                let glow_center = egui::pos2(marker_x, rect.top() + 4.0);
                                ui.painter().circle_filled(
                                    glow_center,
                                    3.0,
                                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)
                                );
                            }

                            // シークバー上に縦線を描画（より視認性を向上）
                            let marker_line_start = egui::pos2(marker_x, rect.top());
                            let marker_line_end = egui::pos2(marker_x, rect.bottom());
                            ui.painter().line_segment(
                                [marker_line_start, marker_line_end],
                                egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 150, 255, 150))
                            );
                        }
                    }
                }

                // 枠線を描画
                ui.painter().rect_stroke(rect, 4.0, ui.style().visuals.widgets.inactive.bg_stroke);

                // ドラッグ開始時の処理
                if response.drag_started() {
                    on_seek_start();
                }

                // ドラッグ中またはクリック時の処理
                if response.dragged() || response.clicked() {
                    if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                        let bar_left = rect.left();
                        let bar_width = rect.width();
                        let click_x = pointer_pos.x - bar_left;

                        // クリック位置を0.0-1.0の範囲に正規化
                        let click_progress = (click_x / bar_width).clamp(0.0, 1.0);

                        // シーク位置を計算
                        let seek_position = std::time::Duration::from_secs_f64(
                            total.as_secs_f64() * click_progress as f64
                        );

                        on_seek(seek_position);
                    }
                }

                // ドラッグ終了時の処理
                if response.drag_stopped() {
                    on_seek_end();
                }
            } else {
                // 総再生時間が不明な場合
                let available_width = ui.available_width() - 80.0;
                ui.add_sized(
                    [available_width, 20.0],
                    egui::ProgressBar::new(0.0)
                        .animate(false)
                );
            }

            ui.add_space(10.0);

            // 総再生時間を表示
            let total_text = total_duration
                .map(Self::format_duration)
                .unwrap_or_else(|| "--:--".to_string());
            ui.label(total_text);
        });
    }

    fn format_duration(duration: std::time::Duration) -> String {
        TimeFormatter::format_duration(duration)
    }
}