use eframe::egui;
use crate::app::MyApp;
use crate::ui::PlaybackControlsUI;

pub struct PlaylistListUI;

impl PlaylistListUI {
    pub fn render(app: &mut MyApp, ui: &mut egui::Ui) {
        // Store data needed for UI
        let queue_tracks = app.playlist_manager.get_tracks().cloned().unwrap_or_default();
        let current_index = app.playlist_manager.get_current_index();
        let selected_indices: Vec<usize> = app.playlist_manager.get_selected_indices().iter().cloned().collect();
        let playlists = app.playlist_manager.get_playlists().clone();
        let current_playlist_id = app.playlist_manager.get_active_playlist_id().to_string();

        // 大量楽曲時の警告表示
        if queue_tracks.len() > 5000 {
            ui.label(format!("⚠ 大量楽曲 ({} 曲) - 表示に時間がかかる場合があります", queue_tracks.len()));
        }

        // Collect actions
        let mut queue_item_selection: Option<(usize, bool, bool)> = None;
        let mut queue_item_double_clicked: Option<usize> = None;
        let mut move_selected_up = false;
        let mut move_selected_down = false;
        let mut move_selected_to_top = false;
        let mut move_selected_to_bottom = false;
        let mut remove_selected = false;
        let mut copy_to_playlist: Option<String> = None;
        let mut move_to_playlist: Option<String> = None;
        let mut select_all = false;
        let mut clear_selection = false;
        let mut copy_to_new_playlist = false;
        let mut move_to_new_playlist = false;

        PlaybackControlsUI::show_track_list(
            ui,
            &queue_tracks,
            current_index,
            app.playlist_manager.get_current_playing_playlist_id(),
            app.playlist_manager.get_current_track(),
            &selected_indices,
            &playlists,
            &current_playlist_id,
            &mut |index, ctrl_held, shift_held| queue_item_selection = Some((index, ctrl_held, shift_held)),
            &mut |index| queue_item_double_clicked = Some(index),
            &mut || move_selected_up = true,
            &mut || move_selected_down = true,
            &mut || move_selected_to_top = true,
            &mut || move_selected_to_bottom = true,
            &mut || remove_selected = true,
            &mut |playlist_id| copy_to_playlist = Some(playlist_id),
            &mut |playlist_id| move_to_playlist = Some(playlist_id),
            &mut || select_all = true,
            &mut || clear_selection = true,
            &mut || copy_to_new_playlist = true,
            &mut || move_to_new_playlist = true,
        );

        // Handle actions after UI
        if let Some((index, ctrl_held, shift_held)) = queue_item_selection {
            app.playlist_manager.handle_item_selection(index, ctrl_held, shift_held);

            // Update selected_track for info display
            if let Some(tracks) = app.playlist_manager.get_tracks() {
                if index < tracks.len() {
                    app.selection_state.selected_track = Some(tracks[index].clone());
                }
            }
        }
        if let Some(index) = queue_item_double_clicked {
            app.handle_queue_item_double_clicked(index);
        }
        if move_selected_up {
            app.playlist_manager.move_selected_up();
        }
        if move_selected_down {
            app.playlist_manager.move_selected_down();
        }
        if move_selected_to_top {
            app.playlist_manager.move_selected_to_top();
        }
        if move_selected_to_bottom {
            app.playlist_manager.move_selected_to_bottom();
        }
        if remove_selected {
            app.handle_remove_selected_from_queue();
        }
        if let Some(playlist_id) = copy_to_playlist {
            if let Err(error_message) = app.handle_copy_selected_to_playlist(playlist_id) {
                Self::show_error_dialog("コピーエラー", &error_message);
            }
        }
        if let Some(playlist_id) = move_to_playlist {
            if let Err(error_message) = app.handle_move_selected_to_playlist(playlist_id) {
                Self::show_error_dialog("移動エラー", &error_message);
            }
        }
        if select_all {
            app.playlist_manager.select_all();
        }
        if clear_selection {
            app.playlist_manager.clear_selection();
        }
        if copy_to_new_playlist {
            match app.playlist_manager.copy_selected_to_new_playlist() {
                Ok(_) => {},
                Err(error_message) => {
                    Self::show_error_dialog("新プレイリスト作成（コピー）エラー", &error_message);
                }
            }
        }
        if move_to_new_playlist {
            match app.playlist_manager.move_selected_to_new_playlist() {
                Ok(_) => {},
                Err(error_message) => {
                    Self::show_error_dialog("新プレイリスト作成（移動）エラー", &error_message);
                }
            }
        }
    }


    /// OS標準のエラーダイアログを表示
    fn show_error_dialog(title: &str, message: &str) {
        rfd::MessageDialog::new()
            .set_title(title)
            .set_description(message)
            .set_level(rfd::MessageLevel::Error)
            .show();
    }
}