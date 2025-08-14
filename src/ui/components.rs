use eframe::egui;

pub fn show_highlighted_text(ui: &mut egui::Ui, text: &str, search_query: &str) {
    if search_query.is_empty() {
        ui.label(text);
    } else {
        let query_lower = search_query.to_lowercase();
        let text_lower = text.to_lowercase();
        
        if let Some(start_index) = text_lower.find(&query_lower) {
            let end_index = start_index + search_query.len();
            
            let before = &text[..start_index];
            let highlight = &text[start_index..end_index];
            let after = &text[end_index..];
            
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                
                if !before.is_empty() {
                    ui.label(before);
                }
                
                ui.label(
                    egui::RichText::new(highlight)
                        .background_color(egui::Color32::YELLOW)
                        .color(egui::Color32::BLACK)
                );
                
                if !after.is_empty() {
                    ui.label(after);
                }
            });
        } else {
            ui.label(text);
        }
    }
}

pub fn show_clickable_highlighted_text(
    ui: &mut egui::Ui, 
    icon: &str, 
    text: &str, 
    search_query: &str
) -> (bool, bool) {
    let mut clicked = false;
    let mut double_clicked = false;
    
    if search_query.is_empty() {
        let response = ui.selectable_label(false, format!("{} {}", icon, text));
        clicked = response.clicked();
        double_clicked = response.double_clicked();
    } else {
        let query_lower = search_query.to_lowercase();
        let text_lower = text.to_lowercase();
        
        ui.horizontal(|ui| {
            let response = ui.selectable_label(false, format!("{} ", icon));
            clicked = response.clicked();
            double_clicked = response.double_clicked();
            
            if let Some(start_index) = text_lower.find(&query_lower) {
                let end_index = start_index + search_query.len();
                
                let before = &text[..start_index];
                let highlight = &text[start_index..end_index];
                let after = &text[end_index..];
                
                ui.spacing_mut().item_spacing.x = 0.0;
                
                if !before.is_empty() {
                    let response = ui.selectable_label(false, before);
                    if response.clicked() {
                        clicked = true;
                    }
                    if response.double_clicked() {
                        double_clicked = true;
                    }
                }
                
                let response = ui.selectable_label(false, 
                    egui::RichText::new(highlight)
                        .background_color(egui::Color32::YELLOW)
                        .color(egui::Color32::BLACK)
                );
                if response.clicked() {
                    clicked = true;
                }
                if response.double_clicked() {
                    double_clicked = true;
                }
                
                if !after.is_empty() {
                    let response = ui.selectable_label(false, after);
                    if response.clicked() {
                        clicked = true;
                    }
                    if response.double_clicked() {
                        double_clicked = true;
                    }
                }
            } else {
                let response = ui.selectable_label(false, text);
                if response.clicked() {
                    clicked = true;
                }
                if response.double_clicked() {
                    double_clicked = true;
                }
            }
        });
    }
    
    (clicked, double_clicked)
}