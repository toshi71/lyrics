use crate::player::PlaybackState;
use crate::music::TrackInfo;
use crate::settings::RepeatMode;
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
                    ui.set_max_width(2000.0); // 水平スクロールを有効にするため（十分に大きな値）
                    
                    // Current track indicator
                    if is_current_playing_track {
                        ui.label("🎵");
                    } else if is_same_track_from_different_playlist {
                        ui.label("🎵(他)"); // 他のプレイリストから再生中
                    } else {
                        ui.label("   ");
                    }
                    
                    let artist_display = track.album_artist.as_deref().unwrap_or(&track.artist);
                    let display_text = format!("{} - {}", artist_display, track.title);
                    
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

    pub fn show_controls_with_seek_bar(
        ui: &mut egui::Ui,
        playback_state: &PlaybackState,
        current_position: std::time::Duration,
        total_duration: Option<std::time::Duration>,
        current_track: Option<&TrackInfo>,
        on_previous: &mut dyn FnMut(),
        on_seek_backward: &mut dyn FnMut(),
        on_play_pause: &mut dyn FnMut(),
        on_stop: &mut dyn FnMut(),
        on_seek_forward: &mut dyn FnMut(),
        on_next: &mut dyn FnMut(),
        on_seek: &mut dyn FnMut(std::time::Duration),
        on_seek_start: &mut dyn FnMut(),
        on_seek_end: &mut dyn FnMut(),
        auto_focus: bool,
        repeat_mode: &RepeatMode,
        shuffle_enabled: bool,
        on_repeat_mode_change: &mut dyn FnMut(RepeatMode),
        on_shuffle_change: &mut dyn FnMut(bool),
    ) {
        // シークバーを最初に表示（横幅全体を使用）
        Self::show_seek_bar(ui, current_position, total_duration, on_seek, on_seek_start, on_seek_end);
        
        ui.add_space(10.0);

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
            
            // 6つのボタンの右側のスペースに再生中楽曲情報を表示
            if let Some(track) = current_track {
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(&track.title).strong());
                    ui.label(format!("{} - {}", track.artist, track.album));
                });
            }
        });
        
        // リピート・シャッフル選択UI
        ui.add_space(10.0);
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

    pub fn show(
        ui: &mut egui::Ui,
        queue: &[TrackInfo],
        current_index: Option<usize>,
        playback_state: &PlaybackState,
        selected_indices: &[usize],
        on_clear_queue: &mut dyn FnMut(),
        on_previous: &mut dyn FnMut(),
        on_seek_backward: &mut dyn FnMut(),
        on_play_pause: &mut dyn FnMut(),
        on_stop: &mut dyn FnMut(),
        on_seek_forward: &mut dyn FnMut(),
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
                            ui.set_max_width(2000.0); // 水平スクロールを有効にするため（十分に大きな値）
                            
                            // Current track indicator
                            if is_current {
                                ui.label("🎵");
                            } else {
                                ui.label("   ");
                            }
                            
                            let artist_display = track.album_artist.as_deref().unwrap_or(&track.artist);
                            let display_text = format!("{} - {}", artist_display, track.title);
                            
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

    fn show_seek_bar(
        ui: &mut egui::Ui,
        current_position: std::time::Duration,
        total_duration: Option<std::time::Duration>,
        on_seek: &mut dyn FnMut(std::time::Duration),
        on_seek_start: &mut dyn FnMut(),
        on_seek_end: &mut dyn FnMut(),
    ) {
        ui.horizontal(|ui| {
            // 現在の再生時間を表示
            let current_text = Self::format_duration(current_position);
            ui.label(current_text);
            
            ui.add_space(10.0);
            
            // シークバー
            if let Some(total) = total_duration {
                let progress = if total.as_secs() > 0 {
                    current_position.as_secs_f64() / total.as_secs_f64()
                } else {
                    0.0
                };
                
                let available_width = ui.available_width() - 80.0; // 時間表示分を差し引く
                
                // カスタムのクリック・ドラッグ可能なプログレスバーを作成
                let desired_size = egui::vec2(available_width, 20.0);
                let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());
                
                // プログレスバーの背景を描画
                let bg_color = ui.style().visuals.extreme_bg_color;
                let fill_color = ui.style().visuals.selection.bg_fill;
                
                ui.painter().rect_filled(rect, 4.0, bg_color);
                
                // プログレス部分を描画
                if progress > 0.0 {
                    let progress_width = rect.width() * progress as f32;
                    let progress_rect = egui::Rect::from_min_size(
                        rect.min,
                        egui::vec2(progress_width, rect.height())
                    );
                    ui.painter().rect_filled(progress_rect, 4.0, fill_color);
                }
                
                // 現在の再生位置を赤い線で表示
                if progress > 0.0 {
                    let position_x = rect.left() + rect.width() * progress as f32;
                    let line_start = egui::pos2(position_x, rect.top());
                    let line_end = egui::pos2(position_x, rect.bottom());
                    ui.painter().line_segment([line_start, line_end], egui::Stroke::new(2.0, egui::Color32::RED));
                }
                
                // 枠線を描画
                ui.painter().rect_stroke(rect, 4.0, ui.style().visuals.widgets.inactive.bg_stroke);
                
                // ドラッグ開始時の処理
                if response.drag_started() {
                    on_seek_start();
                }
                
                // ドラッグ中またはクリック時の処理
                if response.dragged() || response.clicked() {
                    if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                        let bar_left = rect.left();
                        let bar_width = rect.width();
                        let click_x = pointer_pos.x - bar_left;
                        
                        // クリック位置を0.0-1.0の範囲に正規化
                        let click_progress = (click_x / bar_width).clamp(0.0, 1.0);
                        
                        // シーク位置を計算
                        let seek_position = std::time::Duration::from_secs_f64(
                            total.as_secs_f64() * click_progress as f64
                        );
                        
                        on_seek(seek_position);
                    }
                }
                
                // ドラッグ終了時の処理
                if response.drag_stopped() {
                    on_seek_end();
                }
            } else {
                // 総再生時間が不明な場合
                let available_width = ui.available_width() - 80.0;
                ui.add_sized(
                    [available_width, 20.0],
                    egui::ProgressBar::new(0.0)
                        .animate(false)
                );
            }
            
            ui.add_space(10.0);
            
            // 総再生時間を表示
            let total_text = total_duration
                .map(Self::format_duration)
                .unwrap_or_else(|| "--:--".to_string());
            ui.label(total_text);
        });
    }
    
    fn format_duration(duration: std::time::Duration) -> String {
        let total_seconds = duration.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
}