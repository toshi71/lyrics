/// 時間フォーマット関連のユーティリティ
pub struct TimeFormatter;

impl TimeFormatter {
    /// 時間をMM:SS形式でフォーマット
    pub fn format_duration(duration: std::time::Duration) -> String {
        let total_seconds = duration.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// 時間をMM:SS.sss形式でフォーマット（ミリ秒精度）
    pub fn format_duration_with_millis(duration: std::time::Duration) -> String {
        let total_seconds = duration.as_secs_f64();
        let minutes = (total_seconds / 60.0) as u32;
        let seconds = total_seconds % 60.0;
        format!("{:02}:{:06.3}", minutes, seconds)
    }
}

/// 文字列フォーマット関連のユーティリティ
pub struct StringFormatter;

impl StringFormatter {
    /// アーティスト名を適切に表示（album_artistを優先）
    pub fn format_artist_name(artist: &str, album_artist: Option<&str>) -> String {
        album_artist.unwrap_or(artist).to_string()
    }

    /// トラック表示形式の統一
    pub fn format_track_display(title: &str, artist: &str, album_artist: Option<&str>) -> String {
        let artist_display = Self::format_artist_name(artist, album_artist);
        format!("{} - {}", artist_display, title)
    }
}