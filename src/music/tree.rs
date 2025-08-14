use super::metadata::TrackInfo;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MusicTreeNode {
    pub name: String,
    pub node_type: MusicNodeType,
    pub children: Vec<MusicTreeNode>,
    pub expanded: bool,
    pub file_path: Option<PathBuf>,
    pub track_info: Option<TrackInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MusicNodeType {
    Composer,
    Artist,
    Album,
    Track,
    SectionHeader,
}

impl MusicTreeNode {
    pub fn new(name: String, node_type: MusicNodeType) -> Self {
        Self {
            name,
            node_type,
            children: Vec::new(),
            expanded: false,
            file_path: None,
            track_info: None,
        }
    }

    pub fn with_track_info(mut self, track_info: TrackInfo) -> Self {
        self.file_path = Some(track_info.path.clone());
        self.track_info = Some(track_info);
        self
    }

    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn add_child(&mut self, child: MusicTreeNode) {
        self.children.push(child);
    }

    pub fn sort_children_by_name(&mut self) {
        self.children.sort_by(|a, b| a.name.cmp(&b.name));
    }
}