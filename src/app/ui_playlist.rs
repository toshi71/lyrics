use super::MyApp;
use eframe::egui;

impl MyApp {
    pub fn show_right_pane(&mut self, ui: &mut egui::Ui) {
        crate::app::ui::right_pane::RightPaneLayout::show_right_pane(self, ui);
    }
}