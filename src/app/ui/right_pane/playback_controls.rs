use eframe::egui;
use crate::app::MyApp;

pub struct PlaybackControlsOnlyUI;

impl PlaybackControlsOnlyUI {
    pub fn render(app: &mut MyApp, ui: &mut egui::Ui) {
        let playback_state = app.player_state.audio_player.get_state().clone();

        // Collect actions (removed clear_queue)
        let mut previous_clicked = false;
        let mut seek_backward_clicked = false;
        let mut play_pause_clicked = false;
        let mut stop_clicked = false;
        let mut seek_forward_clicked = false;
        let mut next_clicked = false;
        let mut seek_position: Option<std::time::Duration> = None;
        let mut seek_started = false;
        let mut seek_ended = false;

        // Auto focus disabled
        let _auto_focus = false;

        // 再生位置と総再生時間を取得
        let current_position = app.player_state.audio_player.get_playback_position();
        let total_duration = app.player_state.audio_player.get_total_duration();

        // 再生中の場合はUIを継続的に更新
        if playback_state == crate::player::PlaybackState::Playing {
            ui.ctx().request_repaint();
        }

        // リピート・シャッフルモードの変更処理用変数
        let mut repeat_mode_changed = false;
        let mut new_repeat_mode = app.player_state.repeat_mode.clone();
        let mut shuffle_changed = false;
        let mut new_shuffle_enabled = app.player_state.shuffle_enabled;
        let mut add_seek_point_clicked = false;
        let mut seek_point_jump_position: Option<u64> = None;

        // 必要なデータをコピー
        let repeat_mode = app.player_state.repeat_mode.clone();
        let shuffle_enabled = app.player_state.shuffle_enabled;

        // PlaybackControlsの内部領域をデバッグ描画対応バージョンで表示
        crate::app::ui::debug::DebugPlaybackControls::show_controls_with_seek_bar_debug(
            app,
            ui,
            &playback_state,
            current_position,
            total_duration,
            &mut || previous_clicked = true,
            &mut || seek_backward_clicked = true,
            &mut || play_pause_clicked = true,
            &mut || stop_clicked = true,
            &mut || seek_forward_clicked = true,
            &mut || next_clicked = true,
            &mut |position| seek_position = Some(position),
            &mut || seek_started = true,
            &mut || seek_ended = true,
            _auto_focus,
            &repeat_mode,
            shuffle_enabled,
            &mut |mode| {
                new_repeat_mode = mode;
                repeat_mode_changed = true;
            },
            &mut |enabled| {
                new_shuffle_enabled = enabled;
                shuffle_changed = true;
            },
            &mut || add_seek_point_clicked = true,
            &mut |position_ms| seek_point_jump_position = Some(position_ms),
        );

        // Handle actions after UI (removed clear_queue handling)
        if previous_clicked {
            app.handle_previous_button();
        }
        if seek_backward_clicked {
            app.handle_seek_backward();
        }
        if play_pause_clicked {
            app.handle_play_pause();
        }
        if stop_clicked {
            app.handle_stop();
        }
        if seek_forward_clicked {
            app.handle_seek_forward();
        }
        if next_clicked {
            app.handle_next();
        }
        if let Some(position) = seek_position {
            app.handle_seek_to_position(position);
        }
        if seek_started {
            app.handle_seek_start();
        }
        if seek_ended {
            app.handle_seek_end();
        }

        // シークポイント追加処理
        if add_seek_point_clicked {
            Self::handle_add_seek_point(app, current_position);
        }

        // リピート・シャッフルモードの変更処理（永続化なし）
        if repeat_mode_changed {
            app.player_state.repeat_mode = new_repeat_mode;
        }
        if shuffle_changed {
            app.player_state.shuffle_enabled = new_shuffle_enabled;
            app.playlist_manager.update_shuffle_when_settings_changed(new_shuffle_enabled);
        }

        // シークポイントジャンプ処理
        if let Some(position_ms) = seek_point_jump_position {
            let jump_duration = std::time::Duration::from_millis(position_ms);
            if let Err(error) = app.player_state.audio_player.seek_to_position(jump_duration) {
                eprintln!("Error seeking to position: {}", error);
            }
        }

        // Focus flag reset removed - auto focus disabled
    }

    fn handle_add_seek_point(app: &mut MyApp, current_position: std::time::Duration) {
        if let Some(current_track) = app.playlist_manager.get_current_track() {
            let track_path = current_track.path.clone();

            // 現在の楽曲の既存シークポイント数を取得して連番を作成
            let existing_count = app.player_state.seek_point_manager
                .get_seek_points(&track_path)
                .map(|points| points.len())
                .unwrap_or(0);

            let point_name = format!("ポイント{}", existing_count + 1);
            let position_ms = current_position.as_millis() as u64;

            // シークポイントを追加
            if let Err(error) = app.add_seek_point(&track_path, point_name, position_ms) {
                eprintln!("Error adding seek point: {}", error);
            }
        }
    }
}