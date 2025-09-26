use crate::player::PlaybackState;
use crate::settings::RepeatMode;
use eframe::egui;

pub struct PlaybackButtonsUI;

impl PlaybackButtonsUI {
    pub fn show_controls_only(
        ui: &mut egui::Ui,
        playback_state: &PlaybackState,
        on_previous: &mut dyn FnMut(),
        on_seek_backward: &mut dyn FnMut(),
        on_play_pause: &mut dyn FnMut(),
        on_stop: &mut dyn FnMut(),
        on_seek_forward: &mut dyn FnMut(),
        on_next: &mut dyn FnMut(),
    ) {
        // Playback control buttons
        ui.horizontal(|ui| {
            ui.add_space(5.0);

            let button_size = [48.0, 48.0];

            // Previous button
            if ui.add_sized(button_size,
                egui::Button::new(
                    egui::RichText::new("⏮").size(24.0)
                )
            ).clicked() {
                on_previous();
            }

            ui.add_space(5.0);

            // Seek backward button (n秒前へ)
            if ui.add_sized(button_size,
                egui::Button::new(
                    egui::RichText::new("↩").size(24.0)
                )
            ).clicked() {
                on_seek_backward();
            }

            ui.add_space(10.0);

            // Play/pause button
            let play_pause_text = match playback_state {
                PlaybackState::Playing => "⏸",
                _ => "▶",
            };
            if ui.add_sized(button_size,
                egui::Button::new(
                    egui::RichText::new(play_pause_text).size(24.0)
                )
            ).clicked() {
                on_play_pause();
            }

            ui.add_space(10.0);

            // Stop button
            if ui.add_sized(button_size,
                egui::Button::new(
                    egui::RichText::new("⏹").size(24.0)
                )
            ).clicked() {
                on_stop();
            }

            ui.add_space(5.0);

            // Seek forward button (n秒後ろへ)
            if ui.add_sized(button_size,
                egui::Button::new(
                    egui::RichText::new("↪").size(24.0)
                )
            ).clicked() {
                on_seek_forward();
            }

            ui.add_space(10.0);

            // Next button
            if ui.add_sized(button_size,
                egui::Button::new(
                    egui::RichText::new("⏭").size(24.0)
                )
            ).clicked() {
                on_next();
            }
        });
    }

    pub fn show_repeat_shuffle_controls(
        ui: &mut egui::Ui,
        repeat_mode: &RepeatMode,
        shuffle_enabled: bool,
        on_repeat_mode_change: &mut dyn FnMut(RepeatMode),
        on_shuffle_change: &mut dyn FnMut(bool),
    ) {
        ui.horizontal(|ui| {
            ui.add_space(5.0);

            // リピートモード選択
            ui.label("リピート:");
            ui.add_space(5.0);

            let repeat_text = match repeat_mode {
                RepeatMode::Normal => "オフ",
                RepeatMode::RepeatOne => "1曲",
                RepeatMode::RepeatAll => "全曲",
            };

            let mut new_repeat_mode = repeat_mode.clone();
            egui::ComboBox::from_id_source("repeat_mode_selector")
                .selected_text(repeat_text)
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut new_repeat_mode, RepeatMode::Normal, "オフ").changed() {
                        on_repeat_mode_change(RepeatMode::Normal);
                    }
                    if ui.selectable_value(&mut new_repeat_mode, RepeatMode::RepeatOne, "1曲").changed() {
                        on_repeat_mode_change(RepeatMode::RepeatOne);
                    }
                    if ui.selectable_value(&mut new_repeat_mode, RepeatMode::RepeatAll, "全曲").changed() {
                        on_repeat_mode_change(RepeatMode::RepeatAll);
                    }
                });

            ui.add_space(20.0);

            // シャッフル選択
            ui.label("シャッフル:");
            ui.add_space(5.0);

            let shuffle_text = if shuffle_enabled { "オン" } else { "オフ" };
            let mut new_shuffle_enabled = shuffle_enabled;

            egui::ComboBox::from_id_source("shuffle_selector")
                .selected_text(shuffle_text)
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut new_shuffle_enabled, false, "オフ").changed() {
                        on_shuffle_change(false);
                    }
                    if ui.selectable_value(&mut new_shuffle_enabled, true, "オン").changed() {
                        on_shuffle_change(true);
                    }
                });
        });
    }
}