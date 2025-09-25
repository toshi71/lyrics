use eframe::egui;

pub struct ContextMenu;

impl ContextMenu {
    pub fn show_playlist_context_menu(
        ui: &mut egui::Ui,
        playlist_name: &str,
        track_count: usize,
        on_rename: &mut dyn FnMut(),
        on_clear: &mut dyn FnMut(),
        on_delete: &mut dyn FnMut(),
    ) {
        if ui.button("✏ 名前を変更").clicked() {
            on_rename();
            ui.close_menu();
        }

        ui.separator();

        // プレイリストをクリア
        if track_count > 0 {
            if ui.button("× プレイリストをクリア").clicked() {
                on_clear();
                ui.close_menu();
            }
        } else {
            ui.add_enabled(false, egui::Button::new("× プレイリストをクリア"));
        }

        ui.separator();

        // サブメニューで削除確認
        ui.menu_button("🗑 削除", |ui| {
            if track_count > 0 {
                ui.label(format!("「{}」を削除しますか？", playlist_name));
                ui.label(format!("（{}曲が含まれています）", track_count));
                ui.separator();
            }

            if ui.button("削除を確認").clicked() {
                on_delete();
                ui.close_menu();
            }
        });
    }

    pub fn show_default_playlist_context_menu(
        ui: &mut egui::Ui,
        track_count: usize,
        on_clear: &mut dyn FnMut(),
    ) {
        if track_count > 0 {
            if ui.button("× プレイリストをクリア").clicked() {
                on_clear();
                ui.close_menu();
            }
        } else {
            ui.add_enabled(false, egui::Button::new("× プレイリストをクリア"));
        }
    }
}