use eframe::egui;

pub struct SearchUI;

impl SearchUI {
    pub fn show(
        ui: &mut egui::Ui,
        search_query: &mut String,
        focus_request: &mut bool,
        on_search_changed: &mut dyn FnMut(),
    ) -> bool {
        let mut has_focus = false;
        ui.horizontal(|ui| {
            ui.label("検索:");
            
            let available_width = ui.available_width() - 10.0;
            let response = ui.add_sized(
                [available_width, 20.0],
                egui::TextEdit::singleline(search_query)
            );
            
            has_focus = response.has_focus();
            
            if *focus_request {
                response.request_focus();
                *focus_request = false;
                if !search_query.is_empty() {
                    ui.ctx().memory_mut(|mem| {
                        mem.request_focus(response.id);
                    });
                }
            }
            
            if response.changed() {
                on_search_changed();
            }
        });
        has_focus
    }
}