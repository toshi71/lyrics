use eframe::egui;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_title("Hello World GUI"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Hello World GUI",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(MyApp::new()))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    let system_source = SystemSource::new();
    
    if let Ok(font) = system_source.select_best_match(&[FamilyName::Title("Meiryo".to_owned())], &Properties::new()) {
        if let Ok(font_data) = font.load() {
            if let Some(font_data_vec) = font_data.copy_font_data() {
                fonts.font_data.insert(
                    "meiryo".to_owned(),
                    egui::FontData::from_owned(font_data_vec.as_ref().clone()),
                );
                
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "meiryo".to_owned());
                
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("meiryo".to_owned());
            }
        }
    }
    
    ctx.set_fonts(fonts);
}

#[derive(PartialEq)]
enum Tab {
    Main,
    Settings,
}

#[derive(Serialize, Deserialize, Clone)]
struct Settings {
    target_directory: String,
    classical_composer_hierarchy: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            target_directory: String::new(),
            classical_composer_hierarchy: false,
        }
    }
}

#[derive(Debug, Clone)]
struct MusicTreeNode {
    name: String,
    node_type: MusicNodeType,
    children: Vec<MusicTreeNode>,
    expanded: bool,
    #[allow(dead_code)]
    file_path: Option<PathBuf>,
    #[allow(dead_code)]
    track_info: Option<TrackInfo>,
}

#[derive(Debug, Clone, PartialEq)]
enum MusicNodeType {
    Composer,
    Artist,
    Album,
    Track,
    SectionHeader,
}

#[derive(Debug, Clone)]
struct TrackInfo {
    title: String,
    artist: String,
    album: String,
    composer: Option<String>,
    genre: Option<String>,
    track_number: Option<u32>,
    disc_number: Option<u32>,
    path: PathBuf,
}

struct MyApp {
    show_dialog: bool,
    current_tab: Tab,
    settings: Settings,
    music_tree: Vec<MusicTreeNode>,
    original_music_tree: Vec<MusicTreeNode>,
    search_query: String,
    focus_search: bool,
    splitter_position: f32,
}

impl MyApp {
    fn new() -> Self {
        let settings = Self::load_settings();
        let mut app = Self {
            show_dialog: false,
            current_tab: Tab::Main,
            settings,
            music_tree: Vec::new(),
            original_music_tree: Vec::new(),
            search_query: String::new(),
            focus_search: false,
            splitter_position: 0.33, // 左:右 = 1:2
        };
        app.refresh_music_tree();
        app
    }
    
    fn get_settings_file_path() -> PathBuf {
        let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        path.push("settings.json");
        path
    }
    
    fn load_settings() -> Settings {
        let settings_path = Self::get_settings_file_path();
        if let Ok(contents) = fs::read_to_string(&settings_path) {
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            Settings::default()
        }
    }
    
    fn save_settings(&self) {
        let settings_path = Self::get_settings_file_path();
        if let Ok(json) = serde_json::to_string_pretty(&self.settings) {
            let _ = fs::write(&settings_path, json);
        }
    }
    
    fn refresh_music_tree(&mut self) {
        if self.settings.target_directory.is_empty() {
            self.music_tree.clear();
            return;
        }
        
        let target_path = PathBuf::from(&self.settings.target_directory);
        if target_path.exists() && target_path.is_dir() {
            let tracks = self.collect_all_tracks(&target_path);
            self.original_music_tree = self.build_music_tree(tracks);
            self.apply_search_filter();
        } else {
            self.music_tree.clear();
            self.original_music_tree.clear();
        }
    }
    
    fn collect_all_tracks(&self, path: &Path) -> Vec<TrackInfo> {
        let mut tracks = Vec::new();
        self.collect_tracks_recursive(path, &mut tracks);
        tracks
    }
    
    fn collect_tracks_recursive(&self, path: &Path, tracks: &mut Vec<TrackInfo>) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                
                if entry_path.is_dir() {
                    self.collect_tracks_recursive(&entry_path, tracks);
                } else if self.is_flac_file(&entry_path) {
                    if let Some(track_info) = self.get_flac_metadata(&entry_path) {
                        tracks.push(track_info);
                    }
                }
            }
        }
    }
    
    fn build_music_tree(&self, tracks: Vec<TrackInfo>) -> Vec<MusicTreeNode> {
        if self.settings.classical_composer_hierarchy {
            self.build_composer_based_tree(tracks)
        } else {
            self.build_artist_based_tree(tracks)
        }
    }
    
    fn is_classical_genre(&self, genre: &Option<String>) -> bool {
        if let Some(g) = genre {
            let g_lower = g.to_lowercase();
            g_lower == "classical" || g_lower == "クラシック"
        } else {
            false
        }
    }
    
    fn build_composer_based_tree(&self, tracks: Vec<TrackInfo>) -> Vec<MusicTreeNode> {
        let mut classical_tracks = Vec::new();
        let mut non_classical_tracks = Vec::new();
        
        // クラシックと非クラシックを分ける
        for track in tracks {
            if self.is_classical_genre(&track.genre) {
                classical_tracks.push(track);
            } else {
                non_classical_tracks.push(track);
            }
        }
        
        let mut root_nodes = Vec::new();
        
        let classical_exists = !classical_tracks.is_empty();
        let non_classical_exists = !non_classical_tracks.is_empty();
        
        // クラシック音楽（作曲家 > アーティスト > アルバム > 曲）を先に表示
        if classical_exists {
            // まずクラシック音楽の構造を構築
            let mut classical_nodes = Vec::new();
            let mut composer_map: HashMap<String, HashMap<String, HashMap<String, Vec<TrackInfo>>>> = HashMap::new();
            
            for track in classical_tracks {
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
                let mut artist_nodes = Vec::new();
                
                for (artist_name, albums) in artists {
                    let mut album_nodes = Vec::new();
                    
                    for (album_name, mut album_tracks) in albums {
                        self.sort_tracks(&mut album_tracks);
                        
                        let mut track_nodes = Vec::new();
                        for track in album_tracks {
                            let display_name = self.format_track_display_name(&track);
                            
                            track_nodes.push(MusicTreeNode {
                                name: display_name,
                                node_type: MusicNodeType::Track,
                                children: Vec::new(),
                                expanded: false,
                                file_path: Some(track.path.clone()),
                                track_info: Some(track),
                            });
                        }
                        
                        album_nodes.push(MusicTreeNode {
                            name: album_name,
                            node_type: MusicNodeType::Album,
                            children: track_nodes,
                            expanded: false,
                            file_path: None,
                            track_info: None,
                        });
                    }
                    
                    album_nodes.sort_by(|a, b| a.name.cmp(&b.name));
                    
                    artist_nodes.push(MusicTreeNode {
                        name: artist_name,
                        node_type: MusicNodeType::Artist,
                        children: album_nodes,
                        expanded: false,
                        file_path: None,
                        track_info: None,
                    });
                }
                
                artist_nodes.sort_by(|a, b| a.name.cmp(&b.name));
                
                composer_nodes.push(MusicTreeNode {
                    name: composer_name,
                    node_type: MusicNodeType::Composer,
                    children: artist_nodes,
                    expanded: false,
                    file_path: None,
                    track_info: None,
                });
            }
            
            composer_nodes.sort_by(|a, b| a.name.cmp(&b.name));
            classical_nodes.extend(composer_nodes);
            
            // クラシック音楽セクションヘッダーを作成
            root_nodes.push(MusicTreeNode {
                name: "═══ クラシック音楽 ═══".to_string(),
                node_type: MusicNodeType::SectionHeader,
                children: classical_nodes,
                expanded: true, // デフォルトで展開
                file_path: None,
                track_info: None,
            });
        }
        
        // 非クラシック音楽（通常の階層）を後に表示
        if non_classical_exists {
            let non_classical_tree = self.build_artist_based_tree(non_classical_tracks);
            
            // 非クラシック音楽セクションの目印を追加（両方のセクションがある場合のみ）
            if classical_exists {
                root_nodes.push(MusicTreeNode {
                    name: "═══ 一般音楽 ═══".to_string(),
                    node_type: MusicNodeType::SectionHeader,
                    children: non_classical_tree,
                    expanded: true, // デフォルトで展開
                    file_path: None,
                    track_info: None,
                });
            } else {
                // クラシック音楽がない場合はセクションヘッダーなしで直接表示
                root_nodes.extend(non_classical_tree);
            }
        }
        
        root_nodes
    }
    
    fn build_artist_based_tree(&self, tracks: Vec<TrackInfo>) -> Vec<MusicTreeNode> {
        let mut artist_map: HashMap<String, HashMap<String, Vec<TrackInfo>>> = HashMap::new();
        
        // トラックをアーティスト > アルバム > 曲 でグループ化
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
            let mut album_nodes = Vec::new();
            
            for (album_name, mut album_tracks) in albums {
                self.sort_tracks(&mut album_tracks);
                
                let mut track_nodes = Vec::new();
                for track in album_tracks {
                    let display_name = self.format_track_display_name(&track);
                    
                    track_nodes.push(MusicTreeNode {
                        name: display_name,
                        node_type: MusicNodeType::Track,
                        children: Vec::new(),
                        expanded: false,
                        file_path: Some(track.path.clone()),
                        track_info: Some(track),
                    });
                }
                
                album_nodes.push(MusicTreeNode {
                    name: album_name,
                    node_type: MusicNodeType::Album,
                    children: track_nodes,
                    expanded: false,
                    file_path: None,
                    track_info: None,
                });
            }
            
            // アルバムをアルファベット順でソート
            album_nodes.sort_by(|a, b| a.name.cmp(&b.name));
            
            artist_nodes.push(MusicTreeNode {
                name: artist_name,
                node_type: MusicNodeType::Artist,
                children: album_nodes,
                expanded: false,
                file_path: None,
                track_info: None,
            });
        }
        
        // アーティストをアルファベット順でソート
        artist_nodes.sort_by(|a, b| a.name.cmp(&b.name));
        
        artist_nodes
    }
    
    fn sort_tracks(&self, tracks: &mut Vec<TrackInfo>) {
        tracks.sort_by(|a, b| {
            // ディスク番号でソート（Noneは0として扱う）
            let disc_cmp = a.disc_number.unwrap_or(0).cmp(&b.disc_number.unwrap_or(0));
            if disc_cmp != std::cmp::Ordering::Equal {
                return disc_cmp;
            }
            
            // ディスク番号が同じ場合はトラック番号でソート
            let track_cmp = a.track_number.unwrap_or(0).cmp(&b.track_number.unwrap_or(0));
            if track_cmp != std::cmp::Ordering::Equal {
                return track_cmp;
            }
            
            // トラック番号も同じ場合はタイトルでソート
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
    
    fn is_flac_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            extension.to_string_lossy().to_lowercase() == "flac"
        } else {
            false
        }
    }
    
    fn get_flac_metadata(&self, path: &Path) -> Option<TrackInfo> {
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
                
                // トラック番号とディスク番号を取得
                let track_number = tag.get_vorbis("TRACKNUMBER")
                    .and_then(|mut iter| iter.next())
                    .and_then(|s| s.split('/').next()) // "3/12" のような形式に対応
                    .and_then(|s| s.parse::<u32>().ok());
                
                let disc_number = tag.get_vorbis("DISCNUMBER")
                    .and_then(|mut iter| iter.next())
                    .and_then(|s| s.split('/').next()) // "1/2" のような形式に対応
                    .and_then(|s| s.parse::<u32>().ok());
                
                // 作曲家とジャンル情報を取得
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
    
    fn show_music_tree(&mut self, ui: &mut egui::Ui) {
        let nodes = self.music_tree.clone();
        for (i, node) in nodes.iter().enumerate() {
            self.show_music_tree_node(ui, i, node);
        }
    }
    
    fn show_music_tree_node(&mut self, ui: &mut egui::Ui, index: usize, node: &MusicTreeNode) {
        ui.horizontal(|ui| {
            let (icon, _label) = match node.node_type {
                MusicNodeType::SectionHeader => {
                    let icon = if node.expanded { "▼" } else { "▶" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Composer => {
                    let icon = if node.expanded { "🎼" } else { "🎼" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Artist => {
                    let icon = if node.expanded { "👤" } else { "👤" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Album => {
                    let icon = if node.expanded { "💿" } else { "💿" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Track => {
                    ("🎵", format!("🎵 {}", node.name))
                },
            };
            
            if (node.node_type != MusicNodeType::Track) && !node.children.is_empty() {
                if self.show_clickable_highlighted_text(ui, icon, &node.name, &self.search_query) {
                    self.music_tree[index].expanded = !self.music_tree[index].expanded;
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label(format!("{} ", icon));
                    self.show_highlighted_text(ui, &node.name, &self.search_query);
                });
            }
        });
        
        if node.expanded && !node.children.is_empty() {
            ui.indent(format!("music_indent_{}", index), |ui| {
                for (child_index, child) in node.children.iter().enumerate() {
                    self.show_music_tree_child(ui, index, child_index, child);
                }
            });
        }
    }
    
    fn show_music_tree_child(&mut self, ui: &mut egui::Ui, parent_index: usize, child_index: usize, node: &MusicTreeNode) {
        ui.horizontal(|ui| {
            let (icon, _label) = match node.node_type {
                MusicNodeType::SectionHeader => {
                    let icon = if node.expanded { "▼" } else { "▶" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Composer => {
                    let icon = if node.expanded { "🎼" } else { "🎼" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Artist => {
                    let icon = if node.expanded { "👤" } else { "👤" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Album => {
                    let icon = if node.expanded { "💿" } else { "💿" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Track => {
                    ("🎵", format!("🎵 {}", node.name))
                },
            };
            
            if (node.node_type != MusicNodeType::Track) && !node.children.is_empty() {
                if self.show_clickable_highlighted_text(ui, icon, &node.name, &self.search_query) {
                    self.music_tree[parent_index].children[child_index].expanded = !self.music_tree[parent_index].children[child_index].expanded;
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label(format!("{} ", icon));
                    self.show_highlighted_text(ui, &node.name, &self.search_query);
                });
            }
        });
        
        if node.expanded && !node.children.is_empty() {
            ui.indent(format!("music_indent_{}_{}", parent_index, child_index), |ui| {
                for (grandchild_index, grandchild) in node.children.iter().enumerate() {
                    self.show_music_tree_grandchild(ui, parent_index, child_index, grandchild_index, grandchild);
                }
            });
        }
    }
    
    fn show_music_tree_grandchild(&mut self, ui: &mut egui::Ui, parent_index: usize, child_index: usize, grandchild_index: usize, node: &MusicTreeNode) {
        ui.horizontal(|ui| {
            let (icon, _label) = match node.node_type {
                MusicNodeType::SectionHeader => {
                    let icon = if node.expanded { "▼" } else { "▶" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Composer => {
                    let icon = if node.expanded { "🎼" } else { "🎼" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Artist => {
                    let icon = if node.expanded { "👤" } else { "👤" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Album => {
                    let icon = if node.expanded { "💿" } else { "💿" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Track => {
                    ("🎵", format!("🎵 {}", node.name))
                },
            };
            
            if (node.node_type != MusicNodeType::Track) && !node.children.is_empty() {
                if self.show_clickable_highlighted_text(ui, icon, &node.name, &self.search_query) {
                    self.music_tree[parent_index].children[child_index].children[grandchild_index].expanded 
                        = !self.music_tree[parent_index].children[child_index].children[grandchild_index].expanded;
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label(format!("{} ", icon));
                    self.show_highlighted_text(ui, &node.name, &self.search_query);
                });
            }
        });
        
        if node.expanded && !node.children.is_empty() {
            ui.indent(format!("music_indent_{}_{}_{}", parent_index, child_index, grandchild_index), |ui| {
                for (greatgrandchild_index, greatgrandchild) in node.children.iter().enumerate() {
                    self.show_music_tree_greatgrandchild(ui, parent_index, child_index, grandchild_index, greatgrandchild_index, greatgrandchild);
                }
            });
        }
    }
    
    fn show_music_tree_greatgrandchild(&mut self, ui: &mut egui::Ui, parent_index: usize, child_index: usize, grandchild_index: usize, greatgrandchild_index: usize, node: &MusicTreeNode) {
        ui.horizontal(|ui| {
            let (icon, _label) = match node.node_type {
                MusicNodeType::SectionHeader => {
                    let icon = if node.expanded { "▼" } else { "▶" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Composer => {
                    let icon = if node.expanded { "🎼" } else { "🎼" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Artist => {
                    let icon = if node.expanded { "👤" } else { "👤" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Album => {
                    let icon = if node.expanded { "💿" } else { "💿" };
                    (icon, format!("{} {}", icon, node.name))
                },
                MusicNodeType::Track => {
                    ("🎵", format!("🎵 {}", node.name))
                },
            };
            
            if (node.node_type != MusicNodeType::Track) && !node.children.is_empty() {
                if self.show_clickable_highlighted_text(ui, icon, &node.name, &self.search_query) {
                    self.music_tree[parent_index].children[child_index].children[grandchild_index].children[greatgrandchild_index].expanded 
                        = !self.music_tree[parent_index].children[child_index].children[grandchild_index].children[greatgrandchild_index].expanded;
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label(format!("{} ", icon));
                    self.show_highlighted_text(ui, &node.name, &self.search_query);
                });
            }
        });
        
        if node.expanded && !node.children.is_empty() {
            ui.indent(format!("music_indent_{}_{}_{}_{}", parent_index, child_index, grandchild_index, greatgrandchild_index), |ui| {
                for child in &node.children {
                    ui.horizontal(|ui| {
                        ui.label("🎵 ");
                        self.show_highlighted_text(ui, &child.name, &self.search_query);
                    });
                }
            });
        }
    }
    
    fn apply_search_filter(&mut self) {
        if self.search_query.is_empty() {
            self.music_tree = self.original_music_tree.clone();
        } else {
            let query = self.search_query.to_lowercase();
            self.music_tree = self.filter_music_tree(&self.original_music_tree.clone(), &query);
        }
    }
    
    fn filter_music_tree(&self, nodes: &[MusicTreeNode], query: &str) -> Vec<MusicTreeNode> {
        let mut filtered_nodes = Vec::new();
        
        for node in nodes {
            if let Some(filtered_node) = self.filter_node(node, query) {
                filtered_nodes.push(filtered_node);
            }
        }
        
        filtered_nodes
    }
    
    fn filter_node(&self, node: &MusicTreeNode, query: &str) -> Option<MusicTreeNode> {
        let name_matches = node.name.to_lowercase().contains(query);
        
        let mut filtered_children = Vec::new();
        for child in &node.children {
            if let Some(filtered_child) = self.filter_node(child, query) {
                filtered_children.push(filtered_child);
            }
        }
        
        if name_matches || !filtered_children.is_empty() {
            // 名前がマッチした場合は、すべての子要素を含める
            let children_to_use = if name_matches {
                node.children.clone()
            } else {
                filtered_children
            };
            
            let has_children = !children_to_use.is_empty();
            // 検索結果では、子要素がある場合や名前がマッチした場合は展開状態にする
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
    
    
    fn show_highlighted_text(&self, ui: &mut egui::Ui, text: &str, search_query: &str) {
        if search_query.is_empty() {
            ui.label(text);
        } else {
            let query_lower = search_query.to_lowercase();
            let text_lower = text.to_lowercase();
            
            if let Some(start_index) = text_lower.find(&query_lower) {
                let end_index = start_index + search_query.len();
                
                let before = &text[..start_index];
                let highlight = &text[start_index..end_index];
                let after = &text[end_index..];
                
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    
                    if !before.is_empty() {
                        ui.label(before);
                    }
                    
                    ui.label(
                        egui::RichText::new(highlight)
                            .background_color(egui::Color32::YELLOW)
                            .color(egui::Color32::BLACK)
                    );
                    
                    if !after.is_empty() {
                        ui.label(after);
                    }
                });
            } else {
                ui.label(text);
            }
        }
    }
    
    fn show_clickable_highlighted_text(&self, ui: &mut egui::Ui, icon: &str, text: &str, search_query: &str) -> bool {
        let mut clicked = false;
        
        if search_query.is_empty() {
            clicked = ui.selectable_label(false, format!("{} {}", icon, text)).clicked();
        } else {
            let query_lower = search_query.to_lowercase();
            let text_lower = text.to_lowercase();
            
            ui.horizontal(|ui| {
                let response = ui.selectable_label(false, format!("{} ", icon));
                clicked = response.clicked();
                
                if let Some(start_index) = text_lower.find(&query_lower) {
                    let end_index = start_index + search_query.len();
                    
                    let before = &text[..start_index];
                    let highlight = &text[start_index..end_index];
                    let after = &text[end_index..];
                    
                    ui.spacing_mut().item_spacing.x = 0.0;
                    
                    if !before.is_empty() {
                        if ui.selectable_label(false, before).clicked() {
                            clicked = true;
                        }
                    }
                    
                    if ui.selectable_label(false, 
                        egui::RichText::new(highlight)
                            .background_color(egui::Color32::YELLOW)
                            .color(egui::Color32::BLACK)
                    ).clicked() {
                        clicked = true;
                    }
                    
                    if !after.is_empty() {
                        if ui.selectable_label(false, after).clicked() {
                            clicked = true;
                        }
                    }
                } else {
                    if ui.selectable_label(false, text).clicked() {
                        clicked = true;
                    }
                }
            });
        }
        
        clicked
    }
}

impl eframe::App for MyApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_settings();
    }
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // キーボードショートカットの処理
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl) {
            // Ctrl+F: 検索
            self.current_tab = Tab::Main;
            self.focus_search = true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Period) && i.modifiers.ctrl) {
            // Ctrl+.: 設定
            self.current_tab = Tab::Settings;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Q) && i.modifiers.ctrl) {
            // Ctrl+Q: 終了
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("ファイル", |ui| {
                    if ui.add(egui::Button::new("検索").shortcut_text("Ctrl+F")).clicked() {
                        self.current_tab = Tab::Main;
                        self.focus_search = true;
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new("設定").shortcut_text("Ctrl+.")).clicked() {
                        self.current_tab = Tab::Settings;
                        ui.close_menu();
                    }
                    if ui.add(egui::Button::new("終了").shortcut_text("Ctrl+Q")).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });
        
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Main, "メイン");
                ui.selectable_value(&mut self.current_tab, Tab::Settings, "設定");
            });
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Main => {
                    let available_rect = ui.available_rect_before_wrap();
                    let available_width = available_rect.width();
                    let available_height = available_rect.height();
                    let left_width = available_width * self.splitter_position;
                    
                    // 左ペイン（既存のメインコンテンツ）
                    let left_rect = egui::Rect::from_min_size(
                        available_rect.min,
                        egui::Vec2::new(left_width - 1.0, available_height)
                    );
                    let mut left_ui = ui.child_ui(left_rect, egui::Layout::top_down(egui::Align::LEFT), None);
                    left_ui.set_clip_rect(left_rect);
                    
                    if self.settings.target_directory.is_empty() {
                        left_ui.vertical_centered(|ui| {
                            ui.add_space(50.0);
                            ui.label("対象ディレクトリが設定されていません。");
                            ui.label("設定タブでディレクトリを選択してください。");
                        });
                    } else {
                        egui::ScrollArea::both()
                            .id_source("left_pane_scroll")
                            .auto_shrink([false, false])
                            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                            .show(&mut left_ui, |ui| {
                                ui.label(format!("対象ディレクトリ: {}", self.settings.target_directory));
                                ui.separator();
                                
                                ui.horizontal(|ui| {
                                    ui.label("検索:");
                                    
                                    let available_width = ui.available_width() - 10.0;
                                    let response = ui.add_sized(
                                        [available_width, 20.0],
                                        egui::TextEdit::singleline(&mut self.search_query)
                                    );
                                    
                                    // フォーカス要求がある場合
                                    if self.focus_search {
                                        response.request_focus();
                                        self.focus_search = false;
                                        // テキストを全選択
                                        if !self.search_query.is_empty() {
                                            ui.ctx().memory_mut(|mem| {
                                                mem.request_focus(response.id);
                                            });
                                        }
                                    }
                                    
                                    if response.changed() {
                                        self.apply_search_filter();
                                    }
                                });
                                ui.add_space(10.0);
                                
                                self.show_music_tree(ui);
                            });
                    }
                    
                    // セパレーター
                    let separator_rect = egui::Rect::from_min_size(
                        egui::Pos2::new(available_rect.min.x + left_width, available_rect.min.y),
                        egui::Vec2::new(2.0, available_height)
                    );
                    ui.allocate_ui_at_rect(separator_rect, |ui| {
                        ui.separator();
                    });
                    
                    // 右ペイン（新しい領域）
                    let right_rect = egui::Rect::from_min_size(
                        egui::Pos2::new(available_rect.min.x + left_width + 2.0, available_rect.min.y),
                        egui::Vec2::new(available_width - left_width - 2.0, available_height)
                    );
                    let mut right_ui = ui.child_ui(right_rect, egui::Layout::top_down(egui::Align::LEFT), None);
                    right_ui.set_clip_rect(right_rect);
                    
                    egui::ScrollArea::both()
                        .id_source("right_pane_scroll")
                        .auto_shrink([false, false])
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                        .show(&mut right_ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.add_space(50.0);
                                ui.label("右側ペイン");
                                ui.label("ここに新しいコンテンツを追加できます");
                                
                                // テスト用の長いテキスト
                                ui.label("これは非常に長いテキストの例です。ペインの幅を超える場合、水平スクロールによって全体を表示できるようになります。");
                            });
                        });
                },
                Tab::Settings => {
                    ui.add_space(20.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("対象ディレクトリ:");
                        ui.add_space(10.0);
                        
                        let response = ui.text_edit_singleline(&mut self.settings.target_directory);
                        if response.changed() {
                            self.save_settings();
                        }
                        
                        if ui.button("選択").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.settings.target_directory = path.display().to_string();
                                self.save_settings();
                                self.refresh_music_tree();
                            }
                        }
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.horizontal(|ui| {
                        let response = ui.checkbox(&mut self.settings.classical_composer_hierarchy, 
                            "クラシック音楽（ジャンルが\"Classical\"）では作曲家階層を追加");
                        if response.changed() {
                            self.save_settings();
                            self.refresh_music_tree();
                        }
                    });
                },
            }
        });
        
        if self.show_dialog {
            egui::Window::new("ダイアログ")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Hello, World!");
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            self.show_dialog = false;
                        }
                    });
                });
        }
        
    }
}