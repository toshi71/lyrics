use super::MyApp;
use crate::music::{MusicTreeNode, TrackInfo};
use crate::player::PlaybackState;

impl MyApp {
    pub fn handle_keyboard_shortcuts(&mut self, ctx: &eframe::egui::Context) {
        if ctx.input(|i| i.key_pressed(eframe::egui::Key::F) && i.modifiers.ctrl) {
            self.current_tab = super::Tab::Main;
            self.focus_search = true;
        }
        if ctx.input(|i| i.key_pressed(eframe::egui::Key::Period) && i.modifiers.ctrl) {
            self.current_tab = super::Tab::Settings;
        }
        if ctx.input(|i| i.key_pressed(eframe::egui::Key::Q) && i.modifiers.ctrl) {
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
        }
    }

    pub fn handle_track_selection(&mut self, track: TrackInfo, ctrl_held: bool, shift_held: bool) {
        if shift_held && self.last_selected_path.is_some() {
            self.handle_range_selection(track.clone());
        } else if ctrl_held {
            if let Some(ref current_track) = self.selected_track {
                if !self.selected_tracks.contains(&current_track.path) {
                    self.selected_tracks.insert(current_track.path.clone());
                }
            }
            
            if self.selected_tracks.contains(&track.path) {
                self.selected_tracks.remove(&track.path);
            } else {
                self.selected_tracks.insert(track.path.clone());
            }
            
            self.selected_track = Some(track.clone());
            self.last_selected_path = Some(track.path);
        } else {
            self.selected_tracks.clear();
            self.selected_track = Some(track.clone());
            self.last_selected_path = Some(track.path);
        }
    }

    pub fn handle_range_selection(&mut self, end_track: TrackInfo) {
        let start_path = match &self.last_selected_path {
            Some(path) => path.clone(),
            None => return,
        };

        if start_path == end_track.path {
            self.selected_tracks.clear();
            self.selected_tracks.insert(end_track.path.clone());
            self.selected_track = Some(end_track);
            return;
        }

        let all_tracks = self.get_all_tracks_in_display_order();
        
        let start_index = all_tracks.iter().position(|t| t.path == start_path);
        let end_index = all_tracks.iter().position(|t| t.path == end_track.path);
        
        if let (Some(start_idx), Some(end_idx)) = (start_index, end_index) {
            self.selected_tracks.clear();
            
            let (min_idx, max_idx) = if start_idx <= end_idx {
                (start_idx, end_idx)
            } else {
                (end_idx, start_idx)
            };
            
            for track in &all_tracks[min_idx..=max_idx] {
                self.selected_tracks.insert(track.path.clone());
            }
            
            self.selected_track = Some(end_track);
        } else {
            self.selected_tracks.clear();
            self.selected_tracks.insert(start_path);
            self.selected_tracks.insert(end_track.path.clone());
            self.selected_track = Some(end_track);
        }
    }

    pub fn handle_previous_button(&mut self) {
        let position = self.audio_player.get_playback_position();
        
        if position.as_secs() <= 3 {
            if let Some(prev_track) = self.playlist_manager.move_to_previous() {
                if let Err(_) = self.audio_player.play(prev_track) {
                    // Handle error silently
                }
            }
        } else {
            if let Err(_) = self.audio_player.restart_current() {
                // Handle error silently
            }
        }
    }

    pub fn handle_play_pause(&mut self) {
        match self.audio_player.get_state() {
            PlaybackState::Playing => {
                self.audio_player.pause();
            },
            PlaybackState::Paused => {
                self.audio_player.resume();
            },
            PlaybackState::Stopped => {
                if let Some(track) = self.playlist_manager.get_current_track() {
                    if let Err(_) = self.audio_player.play(track.clone()) {
                        // Handle error silently for now
                    }
                }
            },
        }
    }

    pub fn handle_stop(&mut self) {
        self.audio_player.stop();
        self.playlist_manager.set_current_playing_index(None);
    }

    pub fn handle_next(&mut self) {
        if let Some(next_track) = self.playlist_manager.move_to_next() {
            if let Err(_) = self.audio_player.play(next_track) {
                // Handle error silently
            }
        }
    }

    pub fn clear_playback_queue(&mut self) {
        self.audio_player.stop();
        self.playlist_manager.clear();
        self.playlist_manager.set_current_playing_index(None);
    }

    pub fn handle_queue_item_double_clicked(&mut self, index: usize) {
        self.playlist_manager.set_current_index(index);
        if let Some(track) = self.playlist_manager.get_current_track() {
            if let Err(_) = self.audio_player.play(track.clone()) {
                // Handle error silently
            }
        }
    }

    pub fn handle_remove_selected_from_queue(&mut self) {
        if let Some(current_index) = self.playlist_manager.get_current_index() {
            if self.playlist_manager.is_selected(current_index) {
                self.audio_player.stop();
                self.playlist_manager.set_current_playing_index(None);
            }
        }
        
        self.playlist_manager.remove_selected();
    }

    pub fn handle_add_to_playlist(&mut self, track: TrackInfo, playlist_id: String) {
        if let Some(playlist) = self.playlist_manager.get_playlist_mut(&playlist_id) {
            playlist.add_track(track);
            let _ = self.playlist_manager.auto_save();
        }
    }

    pub fn handle_add_album_to_playlist(&mut self, node: MusicTreeNode, playlist_id: String) {
        let tracks = self.collect_all_tracks_from_node(&node);
        if let Some(playlist) = self.playlist_manager.get_playlist_mut(&playlist_id) {
            for track in tracks {
                playlist.add_track(track);
            }
            let _ = self.playlist_manager.auto_save();
        }
    }

    pub fn handle_add_artist_to_playlist(&mut self, node: MusicTreeNode, playlist_id: String) {
        let tracks = self.collect_all_tracks_from_node(&node);
        if let Some(playlist) = self.playlist_manager.get_playlist_mut(&playlist_id) {
            for track in tracks {
                playlist.add_track(track);
            }
            let _ = self.playlist_manager.auto_save();
        }
    }

    pub fn handle_copy_selected_to_playlist(&mut self, target_playlist_id: String) {
        let selected_tracks = self.get_selected_tracks_from_active_playlist();
        
        if let Some(target_playlist) = self.playlist_manager.get_playlist_mut(&target_playlist_id) {
            for track in selected_tracks {
                target_playlist.add_track(track);
            }
            let _ = self.playlist_manager.auto_save();
        }
    }

    pub fn handle_move_selected_to_playlist(&mut self, target_playlist_id: String) {
        let selected_tracks = self.get_selected_tracks_from_active_playlist();
        
        if let Some(target_playlist) = self.playlist_manager.get_playlist_mut(&target_playlist_id) {
            for track in selected_tracks {
                target_playlist.add_track(track);
            }
        }
        
        self.playlist_manager.remove_selected();
        let _ = self.playlist_manager.auto_save();
    }

    // Helper methods
    pub fn get_all_tracks_in_display_order(&self) -> Vec<TrackInfo> {
        let mut tracks = Vec::new();
        self.music_library.collect_displayed_tracks(&mut tracks);
        tracks
    }

    pub fn collect_all_tracks_from_node(&self, node: &MusicTreeNode) -> Vec<TrackInfo> {
        let mut tracks = Vec::new();
        self.collect_all_tracks_recursive(node, &mut tracks);
        tracks
    }

    pub fn collect_all_tracks_recursive(&self, node: &MusicTreeNode, tracks: &mut Vec<TrackInfo>) {
        if let Some(track_info) = &node.track_info {
            tracks.push(track_info.clone());
        }
        
        for child in &node.children {
            self.collect_all_tracks_recursive(child, tracks);
        }
    }

    pub fn get_selected_tracks_from_active_playlist(&self) -> Vec<TrackInfo> {
        let mut selected_tracks = Vec::new();
        
        if let Some(tracks) = self.playlist_manager.get_active_tracks() {
            let selected_indices = self.playlist_manager.get_selected_indices();
            
            for &index in selected_indices {
                if let Some(track) = tracks.get(index) {
                    selected_tracks.push(track.clone());
                }
            }
        }
        
        selected_tracks
    }

    pub fn handle_seek_backward(&mut self) {
        let seek_seconds = self.settings.get_seek_seconds();
        if let Err(_) = self.audio_player.seek_backward(seek_seconds) {
            // Handle error silently
        }
    }

    pub fn handle_seek_forward(&mut self) {
        let seek_seconds = self.settings.get_seek_seconds();
        if let Err(_) = self.audio_player.seek_forward(seek_seconds) {
            // Handle error silently
        }
    }

    pub fn handle_seek_to_position(&mut self, position: std::time::Duration) {
        if let Err(_) = self.audio_player.seek_to_position(position) {
            // Handle error silently
        }
    }

    pub fn handle_seek_start(&mut self) {
        // ドラッグ開始時に現在の再生状態を保存し、再生を一時停止
        let current_state = self.audio_player.get_state().clone();
        self.seek_drag_state = Some(current_state.clone());
        
        if current_state == crate::player::PlaybackState::Playing {
            self.audio_player.pause();
        }
    }

    pub fn handle_seek_end(&mut self) {
        // ドラッグ終了時に保存した再生状態を復元
        if let Some(previous_state) = self.seek_drag_state.take() {
            if previous_state == crate::player::PlaybackState::Playing {
                self.audio_player.resume();
            }
        }
    }
}