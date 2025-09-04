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
        assert_eq!(app.search_query, "");
        assert_eq!(app.focus_search, false);
        assert_eq!(app.search_has_focus, false);
        assert_eq!(app.editing_playlist_id, None);
        assert_eq!(app.editing_playlist_name, "");
        assert_eq!(app.shuffle_enabled, false);
        assert_eq!(app.ui_state.should_focus_controls, false);
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
        assert_eq!(app.search_query, "");
        assert_eq!(app.focus_search, false);
        assert_eq!(app.search_has_focus, false);
        
        // 検索状態を変更
        app.search_query = "test".to_string();
        app.focus_search = true;
        app.search_has_focus = true;
        
        // 変更が正しく反映されることを確認
        assert_eq!(app.search_query, "test");
        assert_eq!(app.focus_search, true);
        assert_eq!(app.search_has_focus, true);
    }
    
    #[test]
    fn test_playlist_editing_state() {
        let mut app = MyApp::new();
        
        // プレイリスト編集状態の初期値
        assert_eq!(app.editing_playlist_id, None);
        assert_eq!(app.editing_playlist_name, "");
        
        // プレイリスト編集状態を変更
        app.editing_playlist_id = Some("test_playlist".to_string());
        app.editing_playlist_name = "Test Playlist".to_string();
        
        // 変更が正しく反映されることを確認
        assert_eq!(app.editing_playlist_id, Some("test_playlist".to_string()));
        assert_eq!(app.editing_playlist_name, "Test Playlist");
    }
    
    #[test]
    fn test_playback_state() {
        let mut app = MyApp::new();
        
        // 再生状態の初期値
        assert_eq!(app.shuffle_enabled, false);
        assert_eq!(app.seek_drag_state, None);
        
        // 再生状態を変更
        app.shuffle_enabled = true;
        
        // 変更が正しく反映されることを確認
        assert_eq!(app.shuffle_enabled, true);
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