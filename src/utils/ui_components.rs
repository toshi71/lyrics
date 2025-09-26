use crate::music::TrackInfo;
use eframe::egui;

/// 共通UIコンポーネント
pub struct CommonUI;

impl CommonUI {
    /// 楽曲情報をグリッド形式で表示
    pub fn show_track_info_grid(ui: &mut egui::Ui, tracks: &[TrackInfo]) {
        egui::Grid::new("track_info_grid")
            .num_columns(3)
            .spacing([10.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                // ヘッダー
                ui.label(egui::RichText::new("タイトル").strong());
                ui.label(egui::RichText::new("アーティスト").strong());
                ui.label(egui::RichText::new("アルバム").strong());
                ui.end_row();

                // 楽曲データ
                for track in tracks {
                    ui.label(&track.title);
                    ui.label(crate::utils::formatting::StringFormatter::format_artist_name(&track.artist, track.album_artist.as_deref()));
                    ui.label(&track.album);
                    ui.end_row();
                }
            });
    }

    /// 楽曲情報を単一行で表示
    pub fn show_track_info_single(ui: &mut egui::Ui, track: &TrackInfo) {
        ui.label(egui::RichText::new(&track.title).strong());
        ui.label(format!("{} - {}",
            crate::utils::formatting::StringFormatter::format_artist_name(&track.artist, track.album_artist.as_deref()),
            track.album
        ));
    }

    /// 統一されたボタンスタイル
    pub fn styled_button(text: &str, size: Option<[f32; 2]>) -> egui::Button {
        let mut button = egui::Button::new(text);
        if let Some(s) = size {
            button = button.min_size(s.into());
        }
        button
    }

    /// プログレスバーの統一スタイル
    pub fn show_progress_bar(ui: &mut egui::Ui, progress: f32, width: f32) {
        ui.add_sized(
            [width, 20.0],
            egui::ProgressBar::new(progress)
                .animate(false)
                .show_percentage()
        );
    }
}