use eframe::egui;

#[allow(dead_code)]
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

#[allow(unused_assignments)]
pub fn show_clickable_highlighted_text(
    ui: &mut egui::Ui, 
    icon: &str, 
    text: &str, 
    search_query: &str
) -> (bool, egui::Response) {
    let mut clicked = false;
    let mut main_response: Option<egui::Response> = None;
    
    if search_query.is_empty() {
        let response = ui.selectable_label(false, format!("{} {}", icon, text));
        clicked = response.clicked();
        main_response = Some(response);
    } else {
        let query_lower = search_query.to_lowercase();
        let text_lower = text.to_lowercase();
        
        let resp = ui.horizontal(|ui| {
            let response = ui.selectable_label(false, format!("{} ", icon));
            clicked = response.clicked();
            let mut combined_response = response;
            
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
                    combined_response = combined_response.union(response);
                }
                
                let response = ui.selectable_label(false, 
                    egui::RichText::new(highlight)
                        .background_color(egui::Color32::YELLOW)
                        .color(egui::Color32::BLACK)
                );
                if response.clicked() {
                    clicked = true;
                }
                combined_response = combined_response.union(response);
                
                if !after.is_empty() {
                    let response = ui.selectable_label(false, after);
                    if response.clicked() {
                        clicked = true;
                    }
                    combined_response = combined_response.union(response);
                }
            } else {
                let response = ui.selectable_label(false, text);
                if response.clicked() {
                    clicked = true;
                }
                combined_response = combined_response.union(response);
            }
            
            combined_response
        });
        main_response = Some(resp.inner);
    }
    
    (clicked, main_response.unwrap())
}