use crate::player::PlaybackState;
use crate::music::TrackInfo;
use eframe::egui;

pub struct PlaybackControlsUI;

impl PlaybackControlsUI {
    pub fn show_track_list(
        ui: &mut egui::Ui,
        queue: &[TrackInfo],
        _current_index: Option<usize>,
        current_playing_playlist_id: Option<&str>,
        current_playing_track: Option<&TrackInfo>, // 現在再生中の楽曲情報
        selected_indices: &[usize],
        playlists: &[crate::playlist::Playlist],
        current_playlist_id: &str,
        on_queue_item_selected: &mut dyn FnMut(usize, bool, bool), // index, ctrl_held, shift_held
        on_queue_item_double_clicked: &mut dyn FnMut(usize), // index
        on_move_selected_up: &mut dyn FnMut(),
        on_move_selected_down: &mut dyn FnMut(),
        on_move_selected_to_top: &mut dyn FnMut(),
        on_move_selected_to_bottom: &mut dyn FnMut(),
        on_remove_selected: &mut dyn FnMut(),
        on_copy_to_playlist: &mut dyn FnMut(String), // playlist_id
        on_move_to_playlist: &mut dyn FnMut(String), // playlist_id
    ) {
        
        if queue.is_empty() {
            ui.label("プレイリストは空です");
        } else {
            for (index, track) in queue.iter().enumerate() {
                let is_selected = selected_indices.contains(&index);
                
                // 現在再生中の楽曲との比較
                let (is_current_playing_track, is_same_track_from_different_playlist) = 
                    if let Some(playing_track) = current_playing_track {
                        if track.path == playing_track.path {
                            // 同じ楽曲が再生中
                            if current_playing_playlist_id == Some(current_playlist_id) {
                                // 同じプレイリストから再生中
                                (true, false)
                            } else {
                                // 異なるプレイリストから再生中
                                (false, true)
                            }
                        } else {
                            // 異なる楽曲
                            (false, false)
                        }
                    } else {
                        // 何も再生していない
                        (false, false)
                    };
                
                ui.horizontal(|ui| {
                    // Current track indicator
                    if is_current_playing_track {
                        ui.label("🎵");
                    } else if is_same_track_from_different_playlist {
                        ui.label("🎵(他)"); // 他のプレイリストから再生中
                    } else {
                        ui.label("   ");
                    }
                    
                    let display_text = format!("{} - {}", track.artist, track.title);
                    
                    // Make the row selectable
                    let response = ui.selectable_label(is_selected, display_text);
                    
                    // Handle left click for selection with modifier keys
                    if response.clicked() {
                        let ctrl_held = ui.input(|i| i.modifiers.ctrl);
                        let shift_held = ui.input(|i| i.modifiers.shift);
                        on_queue_item_selected(index, ctrl_held, shift_held);
                    }
                    
                    // Handle double-click to start playback
                    if response.double_clicked() {
                        on_queue_item_double_clicked(index);
                    }
                    
                    // Handle right-click context menu
                    response.context_menu(|ui| {
                        // Determine if this item is currently selected
                        let item_is_selected = selected_indices.contains(&index);
                        
                        let selected_count = if !selected_indices.is_empty() {
                            selected_indices.len()
                        } else {
                            1 // Will be 1 after auto-selection
                        };
                        
                        // Delete option
                        let delete_text = if selected_count == 1 {
                            "プレイリストから削除".to_string()
                        } else {
                            format!("選択中の{}曲をプレイリストから削除", selected_count)
                        };
                        
                        if ui.button(delete_text).clicked() {
                            // If this item wasn't selected, select it first
                            if !item_is_selected {
                                on_queue_item_selected(index, false, false);
                            }
                            on_remove_selected();
                            ui.close_menu();
                        }
                        
                        ui.separator();
                        
                        // Movement options
                        let move_text = if selected_count == 1 {
                            "楽曲を移動".to_string()
                        } else {
                            format!("選択中の{}曲を移動", selected_count)
                        };
                        
                        ui.menu_button(move_text, |ui| {
                            if ui.button("⬆ 1つ上に移動").clicked() {
                                // If this item wasn't selected, select it first
                                if !item_is_selected {
                                    on_queue_item_selected(index, false, false);
                                }
                                on_move_selected_up();
                                ui.close_menu();
                            }
                            
                            if ui.button("⬇ 1つ下に移動").clicked() {
                                // If this item wasn't selected, select it first
                                if !item_is_selected {
                                    on_queue_item_selected(index, false, false);
                                }
                                on_move_selected_down();
                                ui.close_menu();
                            }
                            
                            ui.separator();
                            
                            if ui.button("⏫ 最初に移動").clicked() {
                                // If this item wasn't selected, select it first
                                if !item_is_selected {
                                    on_queue_item_selected(index, false, false);
                                }
                                on_move_selected_to_top();
                                ui.close_menu();
                            }
                            
                            if ui.button("⏬ 最後に移動").clicked() {
                                // If this item wasn't selected, select it first
                                if !item_is_selected {
                                    on_queue_item_selected(index, false, false);
                                }
                                on_move_selected_to_bottom();
                                ui.close_menu();
                            }
                        });
                        
                        ui.separator();
                        
                        // Copy to other playlists
                        let copy_text = if selected_count == 1 {
                            "他のプレイリストにコピー".to_string()
                        } else {
                            format!("選択中の{}曲を他のプレイリストにコピー", selected_count)
                        };
                        
                        ui.menu_button(copy_text, |ui| {
                            // プレイリスト名の最大幅を計算
                            let mut max_width: f32 = 100.0; // 最小幅
                            for playlist in playlists {
                                if playlist.id != current_playlist_id {
                                    let text_width = ui.fonts(|f| f.layout_no_wrap(
                                        playlist.name.clone(),
                                        egui::FontId::default(),
                                        egui::Color32::WHITE
                                    ).rect.width());
                                    max_width = max_width.max(text_width); // パディングなし
                                }
                            }
                            ui.set_min_width(max_width);
                            
                            for playlist in playlists {
                                if playlist.id != current_playlist_id {
                                    if ui.button(&playlist.name).clicked() {
                                        // If this item wasn't selected, select it first
                                        if !item_is_selected {
                                            on_queue_item_selected(index, false, false);
                                        }
                                        on_copy_to_playlist(playlist.id.clone());
                                        ui.close_menu();
                                    }
                                }
                            }
                        });
                        
                        // Move to other playlists  
                        let move_text = if selected_count == 1 {
                            "他のプレイリストに移動".to_string()
                        } else {
                            format!("選択中の{}曲を他のプレイリストに移動", selected_count)
                        };
                        
                        ui.menu_button(move_text, |ui| {
                            // プレイリスト名の最大幅を計算
                            let mut max_width: f32 = 100.0; // 最小幅
                            for playlist in playlists {
                                if playlist.id != current_playlist_id {
                                    let text_width = ui.fonts(|f| f.layout_no_wrap(
                                        playlist.name.clone(),
                                        egui::FontId::default(),
                                        egui::Color32::WHITE
                                    ).rect.width());
                                    max_width = max_width.max(text_width); // パディングなし
                                }
                            }
                            ui.set_min_width(max_width);
                            
                            for playlist in playlists {
                                if playlist.id != current_playlist_id {
                                    if ui.button(&playlist.name).clicked() {
                                        // If this item wasn't selected, select it first
                                        if !item_is_selected {
                                            on_queue_item_selected(index, false, false);
                                        }
                                        on_move_to_playlist(playlist.id.clone());
                                        ui.close_menu();
                                    }
                                }
                            }
                        });
                    });
                });
            }
        }
    }

    pub fn show_controls_only(
        ui: &mut egui::Ui,
        playback_state: &PlaybackState,
        on_clear_queue: &mut dyn FnMut(),
        on_previous: &mut dyn FnMut(),
        on_play_pause: &mut dyn FnMut(),
        on_stop: &mut dyn FnMut(),
        on_next: &mut dyn FnMut(),
    ) {
        // Clear button
        if ui.button("🗑 プレイリストをクリア").clicked() {
            on_clear_queue();
        }
        
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

    pub fn show(
        ui: &mut egui::Ui,
        queue: &[TrackInfo],
        current_index: Option<usize>,
        playback_state: &PlaybackState,
        selected_indices: &[usize],
        on_clear_queue: &mut dyn FnMut(),
        on_previous: &mut dyn FnMut(),
        on_play_pause: &mut dyn FnMut(),
        on_stop: &mut dyn FnMut(),
        on_next: &mut dyn FnMut(),
        on_queue_item_selected: &mut dyn FnMut(usize, bool, bool), // index, ctrl_held, shift_held
        on_queue_item_double_clicked: &mut dyn FnMut(usize), // index
        on_move_selected_up: &mut dyn FnMut(),
        on_move_selected_down: &mut dyn FnMut(),
        on_move_selected_to_top: &mut dyn FnMut(),
        on_move_selected_to_bottom: &mut dyn FnMut(),
        on_remove_selected: &mut dyn FnMut(),
    ) {
        // Queue header
        ui.horizontal(|ui| {
            ui.label("プレイリスト:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑 クリア").clicked() {
                    on_clear_queue();
                }
            });
        });
        ui.separator();

        // Queue display area with drag and drop support
        let queue_height = ui.text_style_height(&egui::TextStyle::Body) * 12.0;
        egui::ScrollArea::vertical()
            .id_source("playback_queue_scroll")
            .max_height(queue_height)
            .auto_shrink([false, true])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .show(ui, |ui| {
                if queue.is_empty() {
                    ui.label("プレイリストは空です");
                } else {
                    for (index, track) in queue.iter().enumerate() {
                        let is_current = current_index == Some(index);
                        let is_selected = selected_indices.contains(&index);
                        
                        ui.horizontal(|ui| {
                            // Current track indicator
                            if is_current {
                                ui.label("🎵");
                            } else {
                                ui.label("   ");
                            }
                            
                            let display_text = format!("{} - {}", track.artist, track.title);
                            
                            // Make the row selectable
                            let response = ui.selectable_label(is_selected, display_text);
                            
                            // Handle left click for selection with modifier keys
                            if response.clicked() {
                                let ctrl_held = ui.input(|i| i.modifiers.ctrl);
                                let shift_held = ui.input(|i| i.modifiers.shift);
                                on_queue_item_selected(index, ctrl_held, shift_held);
                            }
                            
                            // Handle double-click to start playback
                            if response.double_clicked() {
                                on_queue_item_double_clicked(index);
                            }
                            
                            // Handle right-click context menu
                            response.context_menu(|ui| {
                                // Determine if this item is currently selected
                                let item_is_selected = selected_indices.contains(&index);
                                
                                let selected_count = if !selected_indices.is_empty() {
                                    selected_indices.len()
                                } else {
                                    1 // Will be 1 after auto-selection
                                };
                                
                                // Delete option
                                let delete_text = if selected_count == 1 {
                                    "プレイリストから削除".to_string()
                                } else {
                                    format!("選択中の{}曲をプレイリストから削除", selected_count)
                                };
                                
                                if ui.button(delete_text).clicked() {
                                    // If this item wasn't selected, select it first
                                    if !item_is_selected {
                                        on_queue_item_selected(index, false, false);
                                    }
                                    on_remove_selected();
                                    ui.close_menu();
                                }
                                
                                ui.separator();
                                
                                // Movement options
                                let move_text = if selected_count == 1 {
                                    "楽曲を移動".to_string()
                                } else {
                                    format!("選択中の{}曲を移動", selected_count)
                                };
                                
                                ui.menu_button(move_text, |ui| {
                                    if ui.button("⬆ 1つ上に移動").clicked() {
                                        // If this item wasn't selected, select it first
                                        if !item_is_selected {
                                            on_queue_item_selected(index, false, false);
                                        }
                                        on_move_selected_up();
                                        ui.close_menu();
                                    }
                                    
                                    if ui.button("⬇ 1つ下に移動").clicked() {
                                        // If this item wasn't selected, select it first
                                        if !item_is_selected {
                                            on_queue_item_selected(index, false, false);
                                        }
                                        on_move_selected_down();
                                        ui.close_menu();
                                    }
                                    
                                    ui.separator();
                                    
                                    if ui.button("⏫ 最初に移動").clicked() {
                                        // If this item wasn't selected, select it first
                                        if !item_is_selected {
                                            on_queue_item_selected(index, false, false);
                                        }
                                        on_move_selected_to_top();
                                        ui.close_menu();
                                    }
                                    
                                    if ui.button("⏬ 最後に移動").clicked() {
                                        // If this item wasn't selected, select it first
                                        if !item_is_selected {
                                            on_queue_item_selected(index, false, false);
                                        }
                                        on_move_selected_to_bottom();
                                        ui.close_menu();
                                    }
                                });
                            });
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