use eframe::egui;

pub struct SearchUI;

impl SearchUI {
    pub fn show(
        ui: &mut egui::Ui,
        search_query: &mut String,
        focus_request: &mut bool,
        on_search_changed: &mut dyn FnMut(),
    ) {
        ui.horizontal(|ui| {
            ui.label("検索:");
            
            let available_width = ui.available_width() - 10.0;
            let response = ui.add_sized(
                [available_width, 20.0],
                egui::TextEdit::singleline(search_query)
            );
            
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
    }
}