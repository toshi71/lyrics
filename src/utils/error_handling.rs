/// エラーハンドリング関連のユーティリティ
pub struct ErrorHandler;

impl ErrorHandler {
    /// 再生関連エラーの統一処理
    pub fn handle_playback_error(error: &str) {
        eprintln!("再生エラー: {}", error);
        // 統一されたエラー処理
        // TODO: 必要に応じてエラーログファイルへの出力や通知機能を追加
    }

    /// ファイル操作エラーの統一処理
    pub fn handle_file_error(error: &str, file_path: Option<&str>) {
        if let Some(path) = file_path {
            eprintln!("ファイルエラー ({}): {}", path, error);
        } else {
            eprintln!("ファイルエラー: {}", error);
        }
    }

    /// プレイリスト操作エラーの統一処理
    pub fn handle_playlist_error(error: &str, playlist_name: Option<&str>) {
        if let Some(name) = playlist_name {
            eprintln!("プレイリストエラー ({}): {}", name, error);
        } else {
            eprintln!("プレイリストエラー: {}", error);
        }
    }
}