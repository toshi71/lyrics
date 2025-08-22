use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub composer: Option<String>,
    pub genre: Option<String>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub path: PathBuf,
}

pub fn get_flac_metadata(path: &Path) -> Option<TrackInfo> {
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
            
            let artist = tag.get_vorbis("ALBUMARTIST")
                .and_then(|mut iter| iter.next())
                .map(|s| s.to_string())
                .or_else(|| {
                    tag.get_vorbis("ARTIST")
                        .and_then(|mut iter| iter.next())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "Unknown Artist".to_string());
            
            let album = tag.get_vorbis("ALBUM")
                .and_then(|mut iter| iter.next())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown Album".to_string());
            
            let track_number = tag.get_vorbis("TRACKNUMBER")
                .and_then(|mut iter| iter.next())
                .and_then(|s| s.split('/').next())
                .and_then(|s| s.parse::<u32>().ok());
            
            let disc_number = tag.get_vorbis("DISCNUMBER")
                .and_then(|mut iter| iter.next())
                .and_then(|s| s.split('/').next())
                .and_then(|s| s.parse::<u32>().ok());
            
            let composer = tag.get_vorbis("COMPOSER")
                .and_then(|mut iter| iter.next())
                .map(|s| s.to_string());
            
            let genre = tag.get_vorbis("GENRE")
                .and_then(|mut iter| iter.next())
                .map(|s| s.to_string());
            
            Some(TrackInfo {
                title,
                artist,
                album,
                composer,
                genre,
                track_number,
                disc_number,
                path: path.to_path_buf(),
            })
        },
        Err(_) => None,
    }
}

pub fn is_flac_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        extension.to_string_lossy().to_lowercase() == "flac"
    } else {
        false
    }
}