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
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            target_directory: String::new(),
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
}

#[derive(Debug, Clone, PartialEq)]
enum MusicNodeType {
    Artist,
    Album,
    Track,
}

#[derive(Debug, Clone)]
struct TrackInfo {
    title: String,
    artist: String,
    album: String,
    path: PathBuf,
}

struct MyApp {
    show_dialog: bool,
    current_tab: Tab,
    settings: Settings,
    music_tree: Vec<MusicTreeNode>,
}

impl MyApp {
    fn new() -> Self {
        let settings = Self::load_settings();
        let mut app = Self {
            show_dialog: false,
            current_tab: Tab::Main,
            settings,
            music_tree: Vec::new(),
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
            self.music_tree = self.build_music_tree(tracks);
        } else {
            self.music_tree.clear();
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
                // 曲をタイトル順でソート
                album_tracks.sort_by(|a, b| a.title.cmp(&b.title));
                
                let mut track_nodes = Vec::new();
                for track in album_tracks {
                    track_nodes.push(MusicTreeNode {
                        name: track.title.clone(),
                        node_type: MusicNodeType::Track,
                        children: Vec::new(),
                        expanded: false,
                        file_path: Some(track.path),
                    });
                }
                
                album_nodes.push(MusicTreeNode {
                    name: album_name,
                    node_type: MusicNodeType::Album,
                    children: track_nodes,
                    expanded: false,
                    file_path: None,
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
            });
        }
        
        // アーティストをアルファベット順でソート
        artist_nodes.sort_by(|a, b| a.name.cmp(&b.name));
        
        artist_nodes
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
                
                let artist = tag.get_vorbis("ARTIST")
                    .and_then(|mut iter| iter.next())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Unknown Artist".to_string());
                
                let album = tag.get_vorbis("ALBUM")
                    .and_then(|mut iter| iter.next())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Unknown Album".to_string());
                
                Some(TrackInfo {
                    title,
                    artist,
                    album,
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
            let (icon, label) = match node.node_type {
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
            
            if node.node_type != MusicNodeType::Track && !node.children.is_empty() {
                if ui.selectable_label(false, label).clicked() {
                    self.music_tree[index].expanded = !self.music_tree[index].expanded;
                }
            } else {
                ui.label(label);
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
            let (icon, label) = match node.node_type {
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
            
            if node.node_type != MusicNodeType::Track && !node.children.is_empty() {
                if ui.selectable_label(false, label).clicked() {
                    self.music_tree[parent_index].children[child_index].expanded = !self.music_tree[parent_index].children[child_index].expanded;
                }
            } else {
                ui.label(label);
            }
        });
        
        if node.expanded && !node.children.is_empty() {
            ui.indent(format!("music_indent_{}_{}", parent_index, child_index), |ui| {
                for (_grandchild_index, grandchild) in node.children.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("🎵 {}", grandchild.name));
                    });
                }
            });
        }
    }
}

impl eframe::App for MyApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_settings();
    }
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("ファイル", |ui| {
                    if ui.button("設定").clicked() {
                        self.current_tab = Tab::Settings;
                        ui.close_menu();
                    }
                    if ui.button("終了").clicked() {
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
                    if self.settings.target_directory.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(50.0);
                            ui.label("対象ディレクトリが設定されていません。");
                            ui.label("設定タブでディレクトリを選択してください。");
                        });
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.label(format!("対象ディレクトリ: {}", self.settings.target_directory));
                            ui.separator();
                            self.show_music_tree(ui);
                        });
                    }
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