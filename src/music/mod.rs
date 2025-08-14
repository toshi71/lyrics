pub mod library;
pub mod metadata;
pub mod tree;

pub use library::MusicLibrary;
pub use metadata::{TrackInfo, get_flac_metadata};
pub use tree::{MusicTreeNode, MusicNodeType};