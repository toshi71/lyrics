use crate::player::PlaybackState;
use crate::music::TrackInfo;
use eframe::egui;

pub struct PlaybackControlsUI;

impl PlaybackControlsUI {
    pub fn show(
        ui: &mut egui::Ui,
        queue: &[TrackInfo],
        current_index: Option<usize>,
        playback_state: &PlaybackState,
        on_clear_queue: &mut dyn FnMut(),
        on_previous: &mut dyn FnMut(),
        on_play_pause: &mut dyn FnMut(),
        on_stop: &mut dyn FnMut(),
        on_next: &mut dyn FnMut(),
    ) {
        // Queue header
        ui.horizontal(|ui| {
            ui.label("再生キュー:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑 クリア").clicked() {
                    on_clear_queue();
                }
            });
        });
        ui.separator();

        // Queue display area
        let queue_height = ui.text_style_height(&egui::TextStyle::Body) * 12.0;
        egui::ScrollArea::vertical()
            .id_source("playback_queue_scroll")
            .max_height(queue_height)
            .auto_shrink([false, true])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .show(ui, |ui| {
                if queue.is_empty() {
                    ui.label("キューは空です");
                } else {
                    for (index, track) in queue.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let is_current = current_index == Some(index);
                            
                            if is_current {
                                ui.label("🎵");
                            } else {
                                ui.label("   ");
                            }
                            
                            let display_text = format!("{} - {}", track.artist, track.title);
                            ui.label(display_text);
                        });
                    }
                }
            });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Playback control buttons
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            
            let button_size = [48.0, 48.0];
            
            // Previous button
            if ui.add_sized(button_size, egui::Button::new("⏮")).clicked() {
                on_previous();
            }
            
            ui.add_space(10.0);
            
            // Play/pause button
            let play_pause_text = match playback_state {
                PlaybackState::Playing => "⏸",
                _ => "▶",
            };
            if ui.add_sized(button_size, egui::Button::new(play_pause_text)).clicked() {
                on_play_pause();
            }
            
            ui.add_space(10.0);
            
            // Stop button
            if ui.add_sized(button_size, egui::Button::new("⏹")).clicked() {
                on_stop();
            }
            
            ui.add_space(10.0);
            
            // Next button
            if ui.add_sized(button_size, egui::Button::new("⏭")).clicked() {
                on_next();
            }
        });
    }
}