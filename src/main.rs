use eframe::egui;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use serde::{Deserialize, Serialize};
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
struct FileTreeNode {
    #[allow(dead_code)]
    name: String,
    display_name: String,
    #[allow(dead_code)]
    path: PathBuf,
    is_directory: bool,
    children: Vec<FileTreeNode>,
    expanded: bool,
}

struct MyApp {
    show_dialog: bool,
    current_tab: Tab,
    settings: Settings,
    file_tree: Vec<FileTreeNode>,
}

impl MyApp {
    fn new() -> Self {
        let settings = Self::load_settings();
        let mut app = Self {
            show_dialog: false,
            current_tab: Tab::Main,
            settings,
            file_tree: Vec::new(),
        };
        app.refresh_file_tree();
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
    
    fn refresh_file_tree(&mut self) {
        if self.settings.target_directory.is_empty() {
            self.file_tree.clear();
            return;
        }
        
        let target_path = PathBuf::from(&self.settings.target_directory);
        if target_path.exists() && target_path.is_dir() {
            self.file_tree = self.build_file_tree(&target_path);
        } else {
            self.file_tree.clear();
        }
    }
    
    fn build_file_tree(&self, path: &Path) -> Vec<FileTreeNode> {
        let mut nodes = Vec::new();
        
        if let Ok(entries) = fs::read_dir(path) {
            let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            entries.sort_by(|a, b| {
                let a_is_dir = a.path().is_dir();
                let b_is_dir = b.path().is_dir();
                match (a_is_dir, b_is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.file_name().cmp(&b.file_name()),
                }
            });
            
            for entry in entries {
                let entry_path = entry.path();
                let is_directory = entry_path.is_dir();
                let name = entry.file_name().to_string_lossy().to_string();
                
                // lrcファイルをフィルタリング（ディレクトリは除外しない）
                if !is_directory {
                    if let Some(extension) = entry_path.extension() {
                        if extension.to_string_lossy().to_lowercase() == "lrc" {
                            continue; // lrcファイルはスキップ
                        }
                    }
                }
                
                let children = if is_directory {
                    self.build_file_tree(&entry_path)
                } else {
                    Vec::new()
                };
                
                // 表示名を決定（FLACファイルの場合はタイトルを取得）
                let display_name = if !is_directory && self.is_flac_file(&entry_path) {
                    self.get_flac_title(&entry_path).unwrap_or_else(|| name.clone())
                } else {
                    name.clone()
                };
                
                nodes.push(FileTreeNode {
                    name,
                    display_name,
                    path: entry_path,
                    is_directory,
                    children,
                    expanded: false,
                });
            }
        }
        
        nodes
    }
    
    fn is_flac_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            extension.to_string_lossy().to_lowercase() == "flac"
        } else {
            false
        }
    }
    
    fn get_flac_title(&self, path: &Path) -> Option<String> {
        match metaflac::Tag::read_from_path(path) {
            Ok(tag) => {
                if let Some(title) = tag.get_vorbis("TITLE").and_then(|mut iter| iter.next()) {
                    Some(title.to_string())
                } else {
                    None
                }
            },
            Err(_) => None,
        }
    }
    
    fn show_file_tree(&mut self, ui: &mut egui::Ui) {
        let nodes = self.file_tree.clone();
        for (i, node) in nodes.iter().enumerate() {
            self.show_file_tree_node(ui, i, node);
        }
    }
    
    fn show_file_tree_node(&mut self, ui: &mut egui::Ui, index: usize, node: &FileTreeNode) {
        ui.horizontal(|ui| {
            if node.is_directory {
                let icon = if node.expanded { "📂" } else { "📁" };
                if ui.selectable_label(false, format!("{} {}", icon, node.display_name)).clicked() {
                    self.file_tree[index].expanded = !self.file_tree[index].expanded;
                }
            } else {
                ui.label(format!("🎵 {}", node.display_name));
            }
        });
        
        if node.is_directory && node.expanded {
            ui.indent(format!("folder_indent_{}", index), |ui| {
                for (child_index, child) in node.children.iter().enumerate() {
                    self.show_file_tree_child(ui, index, child_index, child);
                }
            });
        }
    }
    
    fn show_file_tree_child(&mut self, ui: &mut egui::Ui, parent_index: usize, child_index: usize, node: &FileTreeNode) {
        ui.horizontal(|ui| {
            if node.is_directory {
                let icon = if node.expanded { "📂" } else { "📁" };
                if ui.selectable_label(false, format!("{} {}", icon, node.display_name)).clicked() {
                    self.file_tree[parent_index].children[child_index].expanded = !self.file_tree[parent_index].children[child_index].expanded;
                }
            } else {
                ui.label(format!("🎵 {}", node.display_name));
            }
        });
        
        if node.is_directory && node.expanded {
            ui.indent(format!("folder_indent_{}_{}", parent_index, child_index), |ui| {
                for (_grandchild_index, grandchild) in node.children.iter().enumerate() {
                    self.show_simple_file_tree_node(ui, grandchild);
                }
            });
        }
    }
    
    fn show_simple_file_tree_node(&self, ui: &mut egui::Ui, node: &FileTreeNode) {
        ui.horizontal(|ui| {
            if node.is_directory {
                ui.label(format!("📁 {}", node.display_name));
            } else {
                ui.label(format!("🎵 {}", node.display_name));
            }
        });
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
                            self.show_file_tree(ui);
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
                                self.refresh_file_tree();
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