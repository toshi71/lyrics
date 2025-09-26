use crate::music::TrackInfo;
use crate::seek_points::SeekPoint;
use crate::utils::formatting::TimeFormatter;
use eframe::egui;

pub struct PlaybackUtils;

impl PlaybackUtils {
    pub fn format_duration(duration: std::time::Duration) -> String {
        TimeFormatter::format_duration(duration)
    }

    pub fn show_current_track_seek_points(
        ui: &mut egui::Ui,
        seek_points: Option<&Vec<SeekPoint>>,
        on_seek_to_point: &mut dyn FnMut(u64),
    ) {
        if let Some(points) = seek_points {
            if !points.is_empty() {
                ui.label(egui::RichText::new(format!("シークポイント ({}個)", points.len())).size(14.0).strong());
                ui.add_space(8.0);

                // 固定ヘッダー行（スクロールしない）
                egui::Grid::new("seek_points_header_grid")
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .min_col_width(ui.available_width() * 0.4)
                    .max_col_width(ui.available_width() * 0.6)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("名前").size(13.0).strong().color(egui::Color32::from_gray(180)));
                        ui.label(egui::RichText::new("再生位置").size(13.0).strong().color(egui::Color32::from_gray(180)));
                        ui.end_row();
                    });

                ui.add_space(3.0);

                // 動的高さのスクロールエリア（残りスペースをすべて使用）
                // available_height()を取得し、最小値も設定
                let remaining_height = ui.available_height().max(100.0);
                egui::ScrollArea::vertical()
                    .max_height(remaining_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // シークポイントデータのグリッド
                        egui::Grid::new("current_track_seek_points_data_grid")
                            .num_columns(2)
                            .spacing([10.0, 6.0])
                            .striped(true)
                            .min_col_width(ui.available_width() * 0.4)
                            .max_col_width(ui.available_width() * 0.6)
                            .show(ui, |ui| {
                                // 各シークポイントの行
                                for seek_point in points {
                                    // 名前（クリック可能ボタン）
                                    let button_response = ui.add_sized(
                                        [ui.available_width(), 25.0],
                                        egui::Button::new(egui::RichText::new(&seek_point.name).size(12.0))
                                            .fill(egui::Color32::from_rgba_premultiplied(70, 130, 180, 40))
                                    );
                                    if button_response.clicked() {
                                        on_seek_to_point(seek_point.position_ms);
                                    }

                                    // 位置表示（MM:SS.sss形式）
                                    let duration = std::time::Duration::from_millis(seek_point.position_ms);
                                    let total_seconds = duration.as_secs_f64();
                                    let minutes = (total_seconds / 60.0) as u32;
                                    let seconds = total_seconds % 60.0;
                                    let time_text = format!("{:02}:{:06.3}", minutes, seconds);
                                    ui.label(egui::RichText::new(&time_text).size(12.0).color(egui::Color32::from_gray(200)));

                                    ui.end_row();
                                }
                            });
                    });
            } else {
                ui.label(egui::RichText::new("シークポイントなし").size(12.0).color(egui::Color32::from_gray(150)));
            }
        } else {
            ui.label(egui::RichText::new("シークポイントなし").size(12.0).color(egui::Color32::from_gray(150)));
        }
    }

    pub fn show_track_info(ui: &mut egui::Ui, track: &TrackInfo) {
        // 楽曲情報表示（固定サイズ）
        ui.label(egui::RichText::new(&track.title).strong());
        ui.label(format!("{} - {}", track.artist, track.album));
    }
}