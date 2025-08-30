use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub album_artist: Option<String>,
    pub album: String,
    pub composer: Option<String>,
    pub genre: Option<String>,
    pub track_number: Option<u32>,
    pub track_total: Option<u32>,
    pub disc_number: Option<u32>,
    pub disc_total: Option<u32>,
    pub date: Option<String>,
    pub cover_art: Option<Vec<u8>>,
    pub path: PathBuf,
}

pub fn get_flac_metadata(path: &Path) -> Option<TrackInfo> {
    // Step 4-2: ファイル存在確認とエラーハンドリング強化
    if !path.exists() {
        eprintln!("Warning: Audio file not found: {}", path.display());
        return None;
    }

    if !is_flac_file(path) {
        eprintln!("Warning: File is not a FLAC file: {}", path.display());
        return None;
    }

    match metaflac::Tag::read_from_path(path) {
        Ok(tag) => {
            let title = tag.get_vorbis("TITLE")
                .and_then(|mut iter| iter.next())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    path.file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                });
            
            let album_artist = tag.get_vorbis("ALBUMARTIST")
                .and_then(|mut iter| iter.next())
                .map(|s| s.to_string());
            
            let artist = tag.get_vorbis("ARTIST")
                .and_then(|mut iter| iter.next())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown Artist".to_string());
            
            let album = tag.get_vorbis("ALBUM")
                .and_then(|mut iter| iter.next())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown Album".to_string());
            
            let track_number = tag.get_vorbis("TRACKNUMBER")
                .and_then(|mut iter| iter.next())
                .and_then(|s| s.split('/').next())
                .and_then(|s| s.parse::<u32>().ok());
            
            let track_total = tag.get_vorbis("TRACKTOTAL")
                .and_then(|mut iter| iter.next())
                .and_then(|s| s.parse::<u32>().ok())
                .or_else(|| {
                    // TRACKNUMBER フィールドに "X/Y" 形式が含まれている場合の Y を取得
                    tag.get_vorbis("TRACKNUMBER")
                        .and_then(|mut iter| iter.next())
                        .and_then(|s| s.split('/').nth(1))
                        .and_then(|s| s.parse::<u32>().ok())
                });
            
            let disc_number = tag.get_vorbis("DISCNUMBER")
                .and_then(|mut iter| iter.next())
                .and_then(|s| s.split('/').next())
                .and_then(|s| s.parse::<u32>().ok());
            
            let disc_total = tag.get_vorbis("DISCTOTAL")
                .and_then(|mut iter| iter.next())
                .and_then(|s| s.parse::<u32>().ok())
                .or_else(|| {
                    // DISCNUMBER フィールドに "X/Y" 形式が含まれている場合の Y を取得
                    tag.get_vorbis("DISCNUMBER")
                        .and_then(|mut iter| iter.next())
                        .and_then(|s| s.split('/').nth(1))
                        .and_then(|s| s.parse::<u32>().ok())
                });
            
            let date = tag.get_vorbis("DATE")
                .and_then(|mut iter| iter.next())
                .map(|s| s.to_string());
            
            let composer = tag.get_vorbis("COMPOSER")
                .and_then(|mut iter| iter.next())
                .map(|s| s.to_string());
            
            let genre = tag.get_vorbis("GENRE")
                .and_then(|mut iter| iter.next())
                .map(|s| s.to_string());
            
            // カバーアート取得
            let pictures: Vec<_> = tag.pictures().collect();
            let cover_art = pictures.iter()
                // 1. CoverFrontを最優先
                .find(|pic| pic.picture_type == metaflac::block::PictureType::CoverFront)
                // 2. Otherタイプも考慮（一部のエンコーダーが使用）
                .or_else(|| pictures.iter().find(|pic| pic.picture_type == metaflac::block::PictureType::Other))
                // 3. 最初の画像を使用
                .or_else(|| pictures.first())
                .map(|pic| pic.data.clone());
            
            Some(TrackInfo {
                title,
                artist,
                album_artist,
                album,
                composer,
                genre,
                track_number,
                track_total,
                disc_number,
                disc_total,
                date,
                cover_art,
                path: path.to_path_buf(),
            })
        },
        Err(e) => {
            eprintln!("Warning: Failed to read FLAC metadata from '{}': {}", path.display(), e);
            None
        }
    }
}

pub fn is_flac_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        extension.to_string_lossy().to_lowercase() == "flac"
    } else {
        false
    }
}