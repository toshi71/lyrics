use super::metadata::{TrackInfo, get_flac_metadata, is_flac_file};
use super::tree::{MusicTreeNode, MusicNodeType};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct MusicLibrary {
    tracks: Vec<TrackInfo>,
    tree: Vec<MusicTreeNode>,
    original_tree: Vec<MusicTreeNode>,
    classical_composer_hierarchy: bool,
}

impl MusicLibrary {
    pub fn new(classical_composer_hierarchy: bool) -> Self {
        Self {
            tracks: Vec::new(),
            tree: Vec::new(),
            original_tree: Vec::new(),
            classical_composer_hierarchy,
        }
    }

    pub fn scan_directory(&mut self, path: &Path) {
        self.tracks.clear();
        if path.exists() && path.is_dir() {
            self.collect_tracks_recursive(path);
            self.build_tree();
            
            // Step 4-3: スキャン後にメモリ最適化
            self.optimize_memory();
        }
    }

    pub fn get_tree(&self) -> &Vec<MusicTreeNode> {
        &self.tree
    }

    pub fn get_tree_mut(&mut self) -> &mut Vec<MusicTreeNode> {
        &mut self.tree
    }

    pub fn apply_search_filter(&mut self, query: &str) {
        if query.is_empty() {
            self.tree = self.original_tree.clone();
        } else {
            let query_lower = query.to_lowercase();
            self.tree = self.filter_tree_nodes(&self.original_tree, &query_lower);
        }
    }

    pub fn set_classical_hierarchy(&mut self, enabled: bool) {
        self.classical_composer_hierarchy = enabled;
        self.build_tree();
    }

    // Step 4-3: メモリ使用量最適化
    pub fn optimize_memory(&mut self) {
        self.tracks.shrink_to_fit();
        Self::optimize_tree_memory(&mut self.tree);
        Self::optimize_tree_memory(&mut self.original_tree);
    }

    fn optimize_tree_memory(tree: &mut Vec<MusicTreeNode>) {
        for node in &mut *tree {
            node.children.shrink_to_fit();
            Self::optimize_tree_memory(&mut node.children);
        }
        tree.shrink_to_fit();
    }

    pub fn get_track_count(&self) -> usize {
        self.tracks.len()
    }

    fn collect_tracks_recursive(&mut self, path: &Path) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                
                if entry_path.is_dir() {
                    self.collect_tracks_recursive(&entry_path);
                } else if is_flac_file(&entry_path) {
                    if let Some(track_info) = get_flac_metadata(&entry_path) {
                        self.tracks.push(track_info);
                    }
                }
            }
        }
    }

    fn build_tree(&mut self) {
        if self.classical_composer_hierarchy {
            self.original_tree = self.build_composer_based_tree();
        } else {
            self.original_tree = self.build_artist_based_tree();
        }
        self.tree = self.original_tree.clone();
    }

    fn build_composer_based_tree(&self) -> Vec<MusicTreeNode> {
        let mut classical_tracks = Vec::new();
        let mut non_classical_tracks = Vec::new();
        
        for track in &self.tracks {
            if self.is_classical_genre(&track.genre) {
                classical_tracks.push(track.clone());
            } else {
                non_classical_tracks.push(track.clone());
            }
        }
        
        let mut root_nodes = Vec::new();
        
        let classical_exists = !classical_tracks.is_empty();
        let non_classical_exists = !non_classical_tracks.is_empty();
        
        if classical_exists {
            let classical_tree = self.build_classical_hierarchy(classical_tracks);
            root_nodes.push(
                MusicTreeNode::new("═══ クラシック音楽 ═══".to_string(), MusicNodeType::SectionHeader)
                    .with_expanded(true)
            );
            if let Some(last) = root_nodes.last_mut() {
                last.children = classical_tree;
            }
        }
        
        if non_classical_exists {
            let non_classical_tree = self.build_artist_hierarchy(non_classical_tracks);
            
            if classical_exists {
                root_nodes.push(
                    MusicTreeNode::new("═══ 一般音楽 ═══".to_string(), MusicNodeType::SectionHeader)
                        .with_expanded(true)
                );
                if let Some(last) = root_nodes.last_mut() {
                    last.children = non_classical_tree;
                }
            } else {
                root_nodes.extend(non_classical_tree);
            }
        }
        
        root_nodes
    }

    fn build_artist_based_tree(&self) -> Vec<MusicTreeNode> {
        self.build_artist_hierarchy(self.tracks.clone())
    }

    fn build_classical_hierarchy(&self, tracks: Vec<TrackInfo>) -> Vec<MusicTreeNode> {
        let mut composer_map: HashMap<String, HashMap<String, HashMap<String, Vec<TrackInfo>>>> = HashMap::new();
        
        for track in tracks {
            let composer = track.composer.clone()
                .unwrap_or_else(|| "Unknown Composer".to_string());
            
            composer_map
                .entry(composer)
                .or_default()
                .entry(track.artist.clone())
                .or_default()
                .entry(track.album.clone())
                .or_default()
                .push(track);
        }
        
        let mut composer_nodes = Vec::new();
        
        for (composer_name, artists) in composer_map {
            let mut composer_node = MusicTreeNode::new(composer_name, MusicNodeType::Composer);
            
            for (artist_name, albums) in artists {
                let mut artist_node = MusicTreeNode::new(artist_name, MusicNodeType::Artist);
                
                for (album_name, mut album_tracks) in albums {
                    let mut album_node = MusicTreeNode::new(album_name, MusicNodeType::Album);
                    
                    self.sort_tracks(&mut album_tracks);
                    
                    for track in album_tracks {
                        let display_name = self.format_track_display_name(&track);
                        let track_node = MusicTreeNode::new(display_name, MusicNodeType::Track)
                            .with_track_info(track);
                        album_node.add_child(track_node);
                    }
                    
                    artist_node.add_child(album_node);
                }
                
                artist_node.sort_children_by_name();
                composer_node.add_child(artist_node);
            }
            
            composer_node.sort_children_by_name();
            composer_nodes.push(composer_node);
        }
        
        composer_nodes.sort_by(|a, b| a.name.cmp(&b.name));
        composer_nodes
    }

    fn build_artist_hierarchy(&self, tracks: Vec<TrackInfo>) -> Vec<MusicTreeNode> {
        let mut artist_map: HashMap<String, HashMap<String, Vec<TrackInfo>>> = HashMap::new();
        
        for track in tracks {
            artist_map
                .entry(track.artist.clone())
                .or_default()
                .entry(track.album.clone())
                .or_default()
                .push(track);
        }
        
        let mut artist_nodes = Vec::new();
        
        for (artist_name, albums) in artist_map {
            let mut artist_node = MusicTreeNode::new(artist_name, MusicNodeType::Artist);
            
            for (album_name, mut album_tracks) in albums {
                let mut album_node = MusicTreeNode::new(album_name, MusicNodeType::Album);
                
                self.sort_tracks(&mut album_tracks);
                
                for track in album_tracks {
                    let display_name = self.format_track_display_name(&track);
                    let track_node = MusicTreeNode::new(display_name, MusicNodeType::Track)
                        .with_track_info(track);
                    album_node.add_child(track_node);
                }
                
                artist_node.add_child(album_node);
            }
            
            artist_node.sort_children_by_name();
            artist_nodes.push(artist_node);
        }
        
        artist_nodes.sort_by(|a, b| a.name.cmp(&b.name));
        artist_nodes
    }

    fn is_classical_genre(&self, genre: &Option<String>) -> bool {
        if let Some(g) = genre {
            let g_lower = g.to_lowercase();
            g_lower == "classical" || g_lower == "クラシック"
        } else {
            false
        }
    }

    fn sort_tracks(&self, tracks: &mut Vec<TrackInfo>) {
        tracks.sort_by(|a, b| {
            let disc_cmp = a.disc_number.unwrap_or(0).cmp(&b.disc_number.unwrap_or(0));
            if disc_cmp != std::cmp::Ordering::Equal {
                return disc_cmp;
            }
            
            let track_cmp = a.track_number.unwrap_or(0).cmp(&b.track_number.unwrap_or(0));
            if track_cmp != std::cmp::Ordering::Equal {
                return track_cmp;
            }
            
            a.title.cmp(&b.title)
        });
    }

    fn format_track_display_name(&self, track: &TrackInfo) -> String {
        let mut parts = Vec::new();
        
        if let Some(disc) = track.disc_number {
            if let Some(track_num) = track.track_number {
                parts.push(format!("{}-{:02}", disc, track_num));
            } else {
                parts.push(format!("{}-", disc));
            }
        } else if let Some(track_num) = track.track_number {
            parts.push(format!("{:02}", track_num));
        }
        
        parts.push(track.title.clone());
        parts.join(" ")
    }

    fn filter_tree_nodes(&self, nodes: &[MusicTreeNode], query: &str) -> Vec<MusicTreeNode> {
        let mut filtered_nodes = Vec::new();
        
        for node in nodes {
            if let Some(filtered_node) = self.filter_single_node(node, query) {
                filtered_nodes.push(filtered_node);
            }
        }
        
        filtered_nodes
    }

    fn filter_single_node(&self, node: &MusicTreeNode, query: &str) -> Option<MusicTreeNode> {
        let name_matches = node.name.to_lowercase().contains(query);
        
        let mut filtered_children = Vec::new();
        for child in &node.children {
            if let Some(filtered_child) = self.filter_single_node(child, query) {
                filtered_children.push(filtered_child);
            }
        }
        
        if name_matches || !filtered_children.is_empty() {
            let children_to_use = if name_matches {
                node.children.clone()
            } else {
                filtered_children
            };
            
            let has_children = !children_to_use.is_empty();
            let should_expand = has_children || (name_matches && !node.children.is_empty());
            
            Some(MusicTreeNode {
                name: node.name.clone(),
                node_type: node.node_type.clone(),
                children: children_to_use,
                expanded: should_expand,
                file_path: node.file_path.clone(),
                track_info: node.track_info.clone(),
            })
        } else {
            None
        }
    }

    pub fn get_first_track(&self) -> Option<TrackInfo> {
        for node in &self.tree {
            if let Some(track) = self.get_first_track_from_node(node) {
                return Some(track);
            }
        }
        None
    }

    fn get_first_track_from_node(&self, node: &MusicTreeNode) -> Option<TrackInfo> {
        if node.node_type == MusicNodeType::Track {
            return node.track_info.clone();
        }
        
        for child in &node.children {
            if let Some(track) = self.get_first_track_from_node(child) {
                return Some(track);
            }
        }
        
        None
    }

    pub fn collect_displayed_tracks(&self, tracks: &mut Vec<TrackInfo>) {
        for node in &self.tree {
            self.collect_tracks_from_node(node, tracks);
        }
    }

    fn collect_tracks_from_node(&self, node: &MusicTreeNode, tracks: &mut Vec<TrackInfo>) {
        if let Some(track_info) = &node.track_info {
            tracks.push(track_info.clone());
        }
        
        // Only collect from expanded nodes to match UI display order
        if node.expanded {
            for child in &node.children {
                self.collect_tracks_from_node(child, tracks);
            }
        }
    }
}