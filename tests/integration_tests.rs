use flac_music_player::app::{MyApp, Tab, RightTab};

#[cfg(test)]
mod app_tests {
    use super::*;
    
    #[test]
    fn test_app_creation() {
        let app = MyApp::new();
        
        // 基本的な初期状態をテスト
        assert_eq!(app.ui_state.current_tab, Tab::Main);
        assert_eq!(app.ui_state.right_pane_tab, RightTab::Info);
        assert_eq!(app.ui_state.show_dialog, false);
        assert_eq!(app.selection_state.search_query, "");
        assert_eq!(app.selection_state.focus_search, false);
        assert_eq!(app.selection_state.search_has_focus, false);
        assert_eq!(app.playlist_edit_state.editing_playlist_id, None);
        assert_eq!(app.playlist_edit_state.editing_playlist_name, "");
        assert_eq!(app.player_state.shuffle_enabled, false);
    }
    
    #[test]
    fn test_ui_state_isolation() {
        let mut app = MyApp::new();
        
        // UI状態変更前の初期値を保存
        let initial_splitter = app.ui_state.splitter_position;
        let initial_right_top_bottom = app.ui_state.right_top_bottom_position;
        let initial_right_bottom_lr = app.ui_state.right_bottom_left_right_position;
        
        // UI状態を変更
        app.ui_state.show_dialog = true;
        app.ui_state.current_tab = Tab::Settings;
        app.ui_state.right_pane_tab = RightTab::Lrc;
        
        // UI状態変更がアプリケーション状態（位置）に影響しないことを確認
        assert_eq!(app.ui_state.splitter_position, initial_splitter);
        assert_eq!(app.ui_state.right_top_bottom_position, initial_right_top_bottom);
        assert_eq!(app.ui_state.right_bottom_left_right_position, initial_right_bottom_lr);
        
        // 変更されたUI状態を確認
        assert_eq!(app.ui_state.show_dialog, true);
        assert_eq!(app.ui_state.current_tab, Tab::Settings);
        assert_eq!(app.ui_state.right_pane_tab, RightTab::Lrc);
    }
    
    #[test]
    fn test_search_functionality() {
        let mut app = MyApp::new();
        
        // 検索状態の初期値
        assert_eq!(app.selection_state.search_query, "");
        assert_eq!(app.selection_state.focus_search, false);
        assert_eq!(app.selection_state.search_has_focus, false);
        
        // 検索状態を変更
        app.selection_state.search_query = "test".to_string();
        app.selection_state.focus_search = true;
        app.selection_state.search_has_focus = true;
        
        // 変更が正しく反映されることを確認
        assert_eq!(app.selection_state.search_query, "test");
        assert_eq!(app.selection_state.focus_search, true);
        assert_eq!(app.selection_state.search_has_focus, true);
    }
    
    #[test]
    fn test_playlist_editing_state() {
        let mut app = MyApp::new();
        
        // プレイリスト編集状態の初期値
        assert_eq!(app.playlist_edit_state.editing_playlist_id, None);
        assert_eq!(app.playlist_edit_state.editing_playlist_name, "");
        
        // プレイリスト編集状態を変更
        app.playlist_edit_state.editing_playlist_id = Some("test_playlist".to_string());
        app.playlist_edit_state.editing_playlist_name = "Test Playlist".to_string();
        
        // 変更が正しく反映されることを確認
        assert_eq!(app.playlist_edit_state.editing_playlist_id, Some("test_playlist".to_string()));
        assert_eq!(app.playlist_edit_state.editing_playlist_name, "Test Playlist");
    }
    
    #[test]
    fn test_playback_state() {
        let mut app = MyApp::new();
        
        // 再生状態の初期値
        assert_eq!(app.player_state.shuffle_enabled, false);
        assert_eq!(app.player_state.seek_drag_state, None);
        
        // 再生状態を変更
        app.player_state.shuffle_enabled = true;
        
        // 変更が正しく反映されることを確認
        assert_eq!(app.player_state.shuffle_enabled, true);
    }
}

#[cfg(test)]
mod state_migration_tests {
    use super::*;
    
    #[test]
    fn test_state_migration_compatibility() {
        // 将来の構造体移行時の互換性テスト用
        // 現在は基本的な作成テストのみ実装
        let app = MyApp::new();
        
        // MyAppが正常に作成できることを確認
        assert_eq!(app.ui_state.current_tab, Tab::Main);
        
        // 将来的にここで新旧構造体の互換性をテスト予定
        // 例: 新しいUIState構造体への移行後も同じ値が取得できることを確認
    }
}

#[cfg(test)]
mod seek_points_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_seek_point_manager_creation() {
        // 新しいSeekPointManagerを直接作成してテスト（ファイル読み込みなし）
        let manager = flac_music_player::seek_points::SeekPointManager::new();
        
        // SeekPointManagerが正常に初期化されていることを確認
        let track_count = manager.get_track_count();
        let total_points = manager.get_total_seek_points_count();
        
        assert_eq!(track_count, 0);
        assert_eq!(total_points, 0);
    }

    #[test]
    fn test_add_and_get_seek_points() {
        let mut app = MyApp::new();
        let test_track = PathBuf::from("test_track.flac");
        
        // シークポイントを追加
        let result1 = app.add_seek_point(&test_track, "イントロ終了".to_string(), 30000);
        let result2 = app.add_seek_point(&test_track, "サビ開始".to_string(), 60000);
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        // 追加されたシークポイントを取得
        let seek_points = app.player_state.seek_point_manager.get_seek_points(&test_track);
        assert!(seek_points.is_some());
        
        let seek_points = seek_points.unwrap();
        assert_eq!(seek_points.len(), 2);
        
        // 位置順にソートされていることを確認
        assert_eq!(seek_points[0].position_ms, 30000);
        assert_eq!(seek_points[0].name, "イントロ終了");
        assert_eq!(seek_points[1].position_ms, 60000);
        assert_eq!(seek_points[1].name, "サビ開始");
    }

    #[test]
    fn test_remove_seek_point() {
        let mut app = MyApp::new();
        let test_track = PathBuf::from("test_track.flac");
        
        // シークポイントを追加
        let id = app.add_seek_point(&test_track, "テストポイント".to_string(), 45000).unwrap();
        
        // 追加されたことを確認
        let seek_points = app.player_state.seek_point_manager.get_seek_points(&test_track);
        assert!(seek_points.is_some());
        assert_eq!(seek_points.unwrap().len(), 1);
        
        // シークポイントを削除
        let result = app.remove_seek_point(&test_track, &id);
        assert!(result.is_ok());
        
        // 削除されたことを確認
        let seek_points = app.player_state.seek_point_manager.get_seek_points(&test_track);
        assert!(seek_points.is_none() || seek_points.unwrap().is_empty());
    }

    #[test]
    fn test_find_next_and_previous_seek_points() {
        let mut app = MyApp::new();
        let test_track = PathBuf::from("test_track.flac");
        
        // 複数のシークポイントを追加
        app.add_seek_point(&test_track, "ポイント1".to_string(), 20000).unwrap();
        app.add_seek_point(&test_track, "ポイント2".to_string(), 40000).unwrap();
        app.add_seek_point(&test_track, "ポイント3".to_string(), 60000).unwrap();
        
        // 次のシークポイントを検索
        let next_point = app.player_state.seek_point_manager.find_next_seek_point(&test_track, 30000);
        assert!(next_point.is_some());
        assert_eq!(next_point.unwrap().position_ms, 40000);
        assert_eq!(next_point.unwrap().name, "ポイント2");
        
        // 前のシークポイントを検索
        let prev_point = app.player_state.seek_point_manager.find_previous_seek_point(&test_track, 50000);
        assert!(prev_point.is_some());
        assert_eq!(prev_point.unwrap().position_ms, 40000);
        assert_eq!(prev_point.unwrap().name, "ポイント2");
        
        // 境界条件のテスト
        let next_from_end = app.player_state.seek_point_manager.find_next_seek_point(&test_track, 70000);
        assert!(next_from_end.is_none());
        
        let prev_from_start = app.player_state.seek_point_manager.find_previous_seek_point(&test_track, 10000);
        assert!(prev_from_start.is_none());
    }

    #[test]
    fn test_seek_points_persistence() {
        use std::fs;
        let test_file = PathBuf::from("test_seek_points.json");
        
        // テスト後にファイルを削除するクリーンアップ
        let _cleanup = || {
            let _ = fs::remove_file(&test_file);
        };
        
        {
            let mut manager = flac_music_player::seek_points::SeekPointManager::new();
            let test_track = PathBuf::from("test_track.flac");
            
            // シークポイントを追加
            manager.add_seek_point(&test_track, "テスト保存".to_string(), 123000).unwrap();
            
            // ファイルに保存
            let result = manager.save_to_file();
            assert!(result.is_ok());
        }
        
        // 新しいマネージャーでファイルから読み込み
        {
            let mut manager = flac_music_player::seek_points::SeekPointManager::new();
            let result = manager.load_from_file();
            assert!(result.is_ok());
            
            let test_track = PathBuf::from("test_track.flac");
            let seek_points = manager.get_seek_points(&test_track);
            assert!(seek_points.is_some());
            
            let seek_points = seek_points.unwrap();
            assert_eq!(seek_points.len(), 1);
            assert_eq!(seek_points[0].name, "テスト保存");
            assert_eq!(seek_points[0].position_ms, 123000);
        }
        
        _cleanup();
    }
}