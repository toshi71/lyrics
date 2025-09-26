use crate::player::PlaybackState;
use crate::music::TrackInfo;
use crate::settings::RepeatMode;
use crate::seek_points::SeekPoint;
use eframe::egui;

use super::playback::{TrackListUI, SeekBarUI, PlaybackButtonsUI, PlaybackUtils};

pub struct PlaybackControlsUI;

impl PlaybackControlsUI {
    pub fn show_track_list(
        ui: &mut egui::Ui,
        queue: &[TrackInfo],
        current_index: Option<usize>,
        current_playing_playlist_id: Option<&str>,
        current_playing_track: Option<&TrackInfo>,
        selected_indices: &[usize],
        playlists: &[crate::playlist::Playlist],
        current_playlist_id: &str,
        on_queue_item_selected: &mut dyn FnMut(usize, bool, bool),
        on_queue_item_double_clicked: &mut dyn FnMut(usize),
        on_move_selected_up: &mut dyn FnMut(),
        on_move_selected_down: &mut dyn FnMut(),
        on_move_selected_to_top: &mut dyn FnMut(),
        on_move_selected_to_bottom: &mut dyn FnMut(),
        on_remove_selected: &mut dyn FnMut(),
        on_copy_to_playlist: &mut dyn FnMut(String),
        on_move_to_playlist: &mut dyn FnMut(String),
        on_select_all: &mut dyn FnMut(),
        on_clear_selection: &mut dyn FnMut(),
        on_copy_to_new_playlist: &mut dyn FnMut(),
        on_move_to_new_playlist: &mut dyn FnMut(),
    ) {
        TrackListUI::show(
            ui,
            queue,
            current_index,
            current_playing_playlist_id,
            current_playing_track,
            selected_indices,
            playlists,
            current_playlist_id,
            on_queue_item_selected,
            on_queue_item_double_clicked,
            on_move_selected_up,
            on_move_selected_down,
            on_move_selected_to_top,
            on_move_selected_to_bottom,
            on_remove_selected,
            on_copy_to_playlist,
            on_move_to_playlist,
            on_select_all,
            on_clear_selection,
            on_copy_to_new_playlist,
            on_move_to_new_playlist,
        );
    }

    #[allow(dead_code)]
    pub fn show_controls_with_seek_bar(
        ui: &mut egui::Ui,
        playback_state: &PlaybackState,
        current_position: std::time::Duration,
        total_duration: Option<std::time::Duration>,
        current_track: Option<&TrackInfo>,
        seek_points: Option<&Vec<SeekPoint>>,
        on_previous: &mut dyn FnMut(),
        on_seek_backward: &mut dyn FnMut(),
        on_play_pause: &mut dyn FnMut(),
        on_stop: &mut dyn FnMut(),
        on_seek_forward: &mut dyn FnMut(),
        on_next: &mut dyn FnMut(),
        on_seek: &mut dyn FnMut(std::time::Duration),
        on_seek_start: &mut dyn FnMut(),
        on_seek_end: &mut dyn FnMut(),
        _auto_focus: bool,
        repeat_mode: &RepeatMode,
        shuffle_enabled: bool,
        on_repeat_mode_change: &mut dyn FnMut(RepeatMode),
        on_shuffle_change: &mut dyn FnMut(bool),
        on_add_seek_point: &mut dyn FnMut(),
        on_seek_to_point: &mut dyn FnMut(u64), // シークポイントジャンプ用コールバック
    ) {
        // シークバーを最初に表示（横幅全体を使用）
        SeekBarUI::show(ui, current_position, total_duration, seek_points, on_seek, on_seek_start, on_seek_end);
        
        ui.add_space(10.0);

        // 左右分割レイアウト
        ui.horizontal(|ui| {
            // 左側: 再生コントロール、シークポイント追加、リピート・シャッフル
            ui.vertical(|ui| {
                ui.set_min_width(380.0); // 左側の最小幅を確保
                
                // 再生コントロールボタン
                PlaybackButtonsUI::show_controls_only(
                    ui,
                    playback_state,
                    on_previous,
                    on_seek_backward,
                    on_play_pause,
                    on_stop,
                    on_seek_forward,
                    on_next,
                );
                
                ui.add_space(10.0);
                
                // シークポイント追加ボタン
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    
                    if ui.button("シークポイント追加").clicked() {
                        on_add_seek_point();
                    }
                });
                
                ui.add_space(10.0);
                
                // リピート・シャッフル選択UI
                PlaybackButtonsUI::show_repeat_shuffle_controls(
                    ui,
                    repeat_mode,
                    shuffle_enabled,
                    on_repeat_mode_change,
                    on_shuffle_change,
                );
            });
            
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            
            // 右側: 楽曲情報とシークポイント一覧
            ui.allocate_ui_with_layout(
                [ui.available_width(), ui.available_height()].into(),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    if let Some(track) = current_track {
                        // 楽曲情報表示（固定サイズ）
                        PlaybackUtils::show_track_info(ui, track);

                        ui.add_space(15.0);

                        // シークポイント一覧（残りの高さをすべて使用）
                        PlaybackUtils::show_current_track_seek_points(ui, seek_points, on_seek_to_point);
                    } else {
                        ui.label("楽曲が選択されていません");
                    }
                }
            );
        });
    }
    
    pub fn show_current_track_seek_points(
        ui: &mut egui::Ui,
        seek_points: Option<&Vec<SeekPoint>>,
        on_seek_to_point: &mut dyn FnMut(u64),
    ) {
        PlaybackUtils::show_current_track_seek_points(ui, seek_points, on_seek_to_point);
    }

    #[allow(dead_code)]
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
        PlaybackButtonsUI::show_controls_only(
            ui,
            playback_state,
            on_previous,
            on_seek_backward,
            on_play_pause,
            on_stop,
            on_seek_forward,
            on_next,
        );
    }

    #[allow(dead_code)]
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
        PlaybackButtonsUI::show_controls_only(
            ui,
            playback_state,
            on_previous,
            on_seek_backward,
            on_play_pause,
            on_stop,
            on_seek_forward,
            on_next,
        );
    }

    pub fn show_seek_bar(
        ui: &mut egui::Ui,
        current_position: std::time::Duration,
        total_duration: Option<std::time::Duration>,
        seek_points: Option<&Vec<SeekPoint>>,
        on_seek: &mut dyn FnMut(std::time::Duration),
        on_seek_start: &mut dyn FnMut(),
        on_seek_end: &mut dyn FnMut(),
    ) {
        SeekBarUI::show(
            ui,
            current_position,
            total_duration,
            seek_points,
            on_seek,
            on_seek_start,
            on_seek_end,
        );
    }
}