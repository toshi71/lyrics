use eframe::egui;
use std::cell::RefCell;

// 固定ID定数
pub const ID_MAIN_TAB: u32 = 1;
pub const ID_LEFT_PANE: u32 = 2;
pub const ID_RIGHT_PANE: u32 = 3;
pub const ID_RIGHT_PANE_INNER: u32 = 4;
pub const ID_PLAYBACK_CONTROLS: u32 = 5;
pub const ID_SEEK_BAR: u32 = 6;
pub const ID_LEFT_CONTROLS: u32 = 7;
pub const ID_RIGHT_INFO: u32 = 8;
pub const ID_TRACK_INFO: u32 = 9;
pub const ID_SEEK_POINTS_LIST: u32 = 10;
pub const ID_BOTTOM_AREA: u32 = 11;
pub const ID_PLAYLIST_AREA: u32 = 12;
pub const ID_INFO_TAB_AREA: u32 = 13;

pub struct DebugUIRegions {
    counter: RefCell<u32>,
    enabled: bool,
}

impl DebugUIRegions {
    pub fn new(enabled: bool) -> Self {
        Self {
            counter: RefCell::new(0),
            enabled,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.reset_counter();
        }
    }

    pub fn reset_counter(&self) {
        *self.counter.borrow_mut() = 0;
    }

    pub fn next_id(&self) -> u32 {
        if self.enabled {
            let mut counter = self.counter.borrow_mut();
            *counter += 1;
            *counter
        } else {
            0
        }
    }

    pub fn draw_debug_rect(&self, ui: &mut egui::Ui, rect: egui::Rect, region_id: u32, label: &str) {
        if !self.enabled {
            return;
        }

        let painter = ui.painter();
        
        // 外枠を描画（赤い枠線）
        painter.rect_stroke(
            rect,
            egui::Rounding::ZERO,
            egui::Stroke::new(2.0, egui::Color32::RED),
        );

        // 通し番号とラベルを左上に表示
        let text = format!("{}: {}", region_id, label);
        let text_pos = rect.min + egui::Vec2::new(4.0, 4.0);
        
        // 背景を黒にして文字を見やすくする
        let text_size = painter.layout_no_wrap(
            text.clone(),
            egui::FontId::monospace(12.0),
            egui::Color32::YELLOW,
        ).size();
        
        let text_bg_rect = egui::Rect::from_min_size(
            text_pos - egui::Vec2::new(2.0, 2.0),
            text_size + egui::Vec2::new(4.0, 4.0),
        );
        
        painter.rect_filled(
            text_bg_rect,
            egui::Rounding::same(2.0),
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200),
        );

        painter.text(
            text_pos,
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::monospace(12.0),
            egui::Color32::YELLOW,
        );
    }

    pub fn draw_debug_rect_fixed(&self, ui: &mut egui::Ui, rect: egui::Rect, fixed_id: u32, label: &str) {
        self.draw_debug_rect(ui, rect, fixed_id, label);
    }

    pub fn debug_ui_region<R>(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        let region_id = self.next_id();
        let rect_before = ui.max_rect();
        
        let result = add_contents(ui);
        
        let rect_after = ui.max_rect();
        // より正確な領域を取得するために、実際に使用された領域を計算
        let used_rect = if rect_after.height() > rect_before.height() {
            rect_after
        } else {
            ui.min_rect()
        };
        
        self.draw_debug_rect(ui, used_rect, region_id, label);
        
        result
    }
}

impl Default for DebugUIRegions {
    fn default() -> Self {
        Self::new(false)
    }
}