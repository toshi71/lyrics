use super::MyApp;
use eframe::egui;

impl MyApp {
    pub fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(20.0);
        
        ui.horizontal(|ui| {
            ui.label("対象ディレクトリ:");
            ui.add_space(10.0);
            
            let response = ui.text_edit_singleline(&mut self.settings.target_directory);
            if response.changed() {
                self.save_settings();
            }
            
            if ui.button("選択").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.settings.target_directory = path.display().to_string();
                    self.save_settings();
                    self.refresh_music_library();
                }
            }
        });
        
        ui.add_space(20.0);
        
        ui.horizontal(|ui| {
            let response = ui.checkbox(&mut self.settings.classical_composer_hierarchy, 
                "クラシック音楽（ジャンルが\"Classical\"）では作曲家階層を追加");
            if response.changed() {
                self.music_library.set_classical_hierarchy(self.settings.classical_composer_hierarchy);
                self.save_settings();
            }
        });
        
        ui.add_space(20.0);
        
        ui.horizontal(|ui| {
            ui.label("フォント:");
            ui.add_space(10.0);
            
            let available_fonts = crate::settings::Settings::get_available_fonts();
            let mut font_changed = false;
            
            egui::ComboBox::from_id_source("font_selector")
                .selected_text(&self.settings.selected_font)
                .show_ui(ui, |ui| {
                    for font in &available_fonts {
                        if ui.selectable_value(&mut self.settings.selected_font, font.clone(), font).changed() {
                            font_changed = true;
                        }
                    }
                });
            
            ui.add_space(15.0);
            ui.label("日本語 한국어 汉语");
            
            if font_changed {
                self.save_settings();
                // フォント変更を適用
                super::setup_custom_fonts(ui.ctx(), &self.settings);
                ui.ctx().request_repaint();
            }
        });
        
        ui.add_space(20.0);
        
        // テーマ設定
        ui.horizontal(|ui| {
            ui.label("テーマ:");
            ui.add_space(10.0);
            
            let mut theme_changed = false;
            let current_dark_mode = self.settings.is_dark_mode();
            
            let mut selected_theme = if current_dark_mode { 1 } else { 0 };
            
            egui::ComboBox::from_id_source("theme_selector")
                .selected_text(if current_dark_mode { "ダーク" } else { "ライト" })
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut selected_theme, 0, "ライト").changed() {
                        theme_changed = true;
                    }
                    if ui.selectable_value(&mut selected_theme, 1, "ダーク").changed() {
                        theme_changed = true;
                    }
                });
            
            if theme_changed {
                let new_dark_mode = selected_theme == 1;
                self.settings.set_dark_mode(new_dark_mode);
                self.save_settings();
                ui.ctx().request_repaint();
            }
        });
        
        ui.add_space(20.0);
        ui.separator();
        ui.heading("プレイバック設定");
        ui.add_space(10.0);
        
        // シーク秒数設定
        ui.horizontal(|ui| {
            ui.label("シーク秒数:");
            ui.add_space(10.0);
            
            let mut seek_seconds = self.settings.seek_seconds as i32;
            let response = ui.add(egui::DragValue::new(&mut seek_seconds)
                .range(1..=60)
                .suffix("秒"));
            
            if response.changed() {
                self.settings.set_seek_seconds(seek_seconds as u32);
                self.save_settings();
            }
            
            ui.add_space(10.0);
            ui.label("(↩/↪ ボタンで前後にジャンプする秒数)");
        });
        
        ui.add_space(20.0);
        ui.separator();
        ui.heading("プレイリスト設定");
        ui.add_space(10.0);
        
        // デフォルトプレイリスト設定
        ui.heading("デフォルトプレイリスト");
        
        let _settings_changed = false;
        
        
        if _settings_changed {
            self.save_settings();
        }

        // システム統計の表示
        ui.add_space(20.0);
        ui.separator();
        ui.heading("システム統計");
        ui.add_space(10.0);

        let (total_playlists, total_tracks) = self.playlist_manager.get_quick_stats();
        let library_tracks = self.music_library.get_track_count();

        ui.horizontal(|ui| {
            ui.label("ライブラリ楽曲数:");
            ui.label(format!("{} 曲", library_tracks));
        });

        ui.horizontal(|ui| {
            ui.label("プレイリスト数:");
            ui.label(format!("{} 個", total_playlists));
        });

        ui.horizontal(|ui| {
            ui.label("プレイリスト総楽曲数:");
            ui.label(format!("{} 曲", total_tracks));
        });

        if ui.button("📊 メモリ最適化を実行").clicked() {
            self.music_library.optimize_memory();
            self.playlist_manager.optimize_memory();
        }
    }
}