use eframe::egui;

pub struct LayoutDebugger;

impl LayoutDebugger {
    pub fn calculate_controls_area_rect(
        total_rect: egui::Rect,
        seek_bar_height: f32,
        space_height: f32
    ) -> egui::Rect {
        let controls_area_height = total_rect.height() - seek_bar_height - space_height;
        egui::Rect::from_min_size(
            total_rect.min + egui::Vec2::new(0.0, seek_bar_height + space_height),
            egui::Vec2::new(total_rect.width(), controls_area_height)
        )
    }

    pub fn calculate_left_right_split(
        controls_rect: egui::Rect,
        left_width: f32,
        separator_width: f32
    ) -> (egui::Rect, egui::Rect, f32) {
        let right_width = controls_rect.width() - left_width - separator_width;

        let left_rect = egui::Rect::from_min_size(
            controls_rect.min,
            egui::Vec2::new(left_width, controls_rect.height())
        );

        let right_rect = egui::Rect::from_min_size(
            controls_rect.min + egui::Vec2::new(left_width + separator_width, 0.0),
            egui::Vec2::new(right_width, controls_rect.height())
        );

        (left_rect, right_rect, right_width)
    }
}