use crate::music::{MusicTreeNode, MusicNodeType, TrackInfo};
use crate::ui::components::show_clickable_highlighted_text;
use crate::playlist::Playlist;
use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct MusicTreeUI;

impl MusicTreeUI {
    pub fn show(
        ui: &mut egui::Ui,
        nodes: &mut Vec<MusicTreeNode>,
        search_query: &str,
        selected_track: Option<&TrackInfo>,
        selected_tracks: &HashSet<PathBuf>,
        playlists: &Vec<Playlist>,
        on_track_selected: &mut dyn FnMut(TrackInfo, bool, bool), // ctrl_held, shift_held
        on_track_double_clicked: &mut dyn FnMut(TrackInfo),
        on_add_to_playlist: &mut dyn FnMut(TrackInfo, String), // Add track to specific playlist
        on_add_album_to_playlist: &mut dyn FnMut(&MusicTreeNode, String), // Add album to specific playlist
        on_add_artist_to_playlist: &mut dyn FnMut(&MusicTreeNode, String), // Add artist to specific playlist
        on_create_playlist_with_track: &mut dyn FnMut(TrackInfo), // Create new playlist with track
        on_create_playlist_with_album: &mut dyn FnMut(&MusicTreeNode), // Create new playlist with album
        on_create_playlist_with_artist: &mut dyn FnMut(&MusicTreeNode), // Create new playlist with artist
    ) {
        let mut actions = Vec::new();
        
        for (i, node) in nodes.iter().enumerate() {
            if let Some(action) = Self::show_node_recursive(
                ui,
                node,
                search_query,
                selected_track,
                selected_tracks,
                playlists,
                i,
                &[],
                on_track_selected,
                on_track_double_clicked,
                on_add_to_playlist,
                on_add_album_to_playlist,
                on_add_artist_to_playlist,
                on_create_playlist_with_track,
                on_create_playlist_with_album,
                on_create_playlist_with_artist,
            ) {
                actions.push(action);
            }
        }
        
        // Apply actions after iteration
        for action in actions {
            match action {
                TreeAction::ToggleExpanded { path } => {
                    Self::toggle_expanded_at_path(nodes, &path);
                }
            }
        }
    }

    fn show_node_recursive(
        ui: &mut egui::Ui,
        node: &MusicTreeNode,
        search_query: &str,
        selected_track: Option<&TrackInfo>,
        selected_tracks: &HashSet<PathBuf>,
        playlists: &Vec<Playlist>,
        index: usize,
        parent_path: &[usize],
        on_track_selected: &mut dyn FnMut(TrackInfo, bool, bool),
        on_track_double_clicked: &mut dyn FnMut(TrackInfo),
        on_add_to_playlist: &mut dyn FnMut(TrackInfo, String),
        on_add_album_to_playlist: &mut dyn FnMut(&MusicTreeNode, String),
        on_add_artist_to_playlist: &mut dyn FnMut(&MusicTreeNode, String),
        on_create_playlist_with_track: &mut dyn FnMut(TrackInfo),
        on_create_playlist_with_album: &mut dyn FnMut(&MusicTreeNode),
        on_create_playlist_with_artist: &mut dyn FnMut(&MusicTreeNode),
    ) -> Option<TreeAction> {
        let mut current_path = parent_path.to_vec();
        current_path.push(index);
        
        let icon = Self::get_node_icon(&node.node_type, node.expanded);
        let mut action = None;

        ui.horizontal(|ui| {
            if node.node_type != MusicNodeType::Track && !node.children.is_empty() {
                let (clicked, response) = show_clickable_highlighted_text(ui, icon, &node.name, search_query);
                if clicked {
                    action = Some(TreeAction::ToggleExpanded { path: current_path.clone() });
                }
                
                // Add context menu for different node types
                match node.node_type {
                    MusicNodeType::Album => {
                        response.context_menu(|ui| {
                            // プレイリストに追加メニュー
                            ui.menu_button("アルバムをプレイリストに追加", |ui| {
                                // 新プレイリスト作成オプション
                                ui.separator();
                                if ui.button("➕ 新たなプレイリストを作成して追加").clicked() {
                                    on_create_playlist_with_album(node);
                                    ui.close_menu();
                                }
                                ui.separator();
                                
                                // 既存のプレイリスト一覧
                                for playlist in playlists {
                                    if ui.button(&playlist.name).clicked() {
                                        on_add_album_to_playlist(node, playlist.id.clone());
                                        ui.close_menu();
                                    }
                                }
                            });
                        });
                    },
                    MusicNodeType::Artist => {
                        response.context_menu(|ui| {
                            // プレイリストに追加メニュー
                            ui.menu_button("アーティストをプレイリストに追加", |ui| {
                                // 新プレイリスト作成オプション
                                ui.separator();
                                if ui.button("➕ 新たなプレイリストを作成して追加").clicked() {
                                    on_create_playlist_with_artist(node);
                                    ui.close_menu();
                                }
                                ui.separator();
                                
                                // 既存のプレイリスト一覧
                                for playlist in playlists {
                                    if ui.button(&playlist.name).clicked() {
                                        on_add_artist_to_playlist(node, playlist.id.clone());
                                        ui.close_menu();
                                    }
                                }
                            });
                        });
                    },
                    MusicNodeType::Composer => {
                        response.context_menu(|ui| {
                            // プレイリストに追加メニュー
                            ui.menu_button("作曲家をプレイリストに追加", |ui| {
                                // 新プレイリスト作成オプション
                                ui.separator();
                                if ui.button("➕ 新たなプレイリストを作成して追加").clicked() {
                                    on_create_playlist_with_artist(node);
                                    ui.close_menu();
                                }
                                ui.separator();
                                
                                // 既存のプレイリスト一覧
                                for playlist in playlists {
                                    if ui.button(&playlist.name).clicked() {
                                        on_add_artist_to_playlist(node, playlist.id.clone());
                                        ui.close_menu();
                                    }
                                }
                            });
                        });
                    },
                    _ => {} // No context menu for section headers
                }
            } else if node.node_type == MusicNodeType::Track {
                // Check if this track is selected (either single or multiple selection)
                let is_selected = if let Some(track) = &node.track_info {
                    selected_tracks.contains(&track.path) || 
                    (selected_track.map(|st| st.path == track.path).unwrap_or(false))
                } else {
                    false
                };
                
                let display_text = format!("{} {}", icon, node.name);
                let response = ui.selectable_label(is_selected, display_text);
                
                if let Some(track_info) = &node.track_info {
                    if response.clicked() {
                        let ctrl_held = ui.input(|i| i.modifiers.ctrl);
                        let shift_held = ui.input(|i| i.modifiers.shift);
                        on_track_selected(track_info.clone(), ctrl_held, shift_held);
                    }
                    if response.double_clicked() {
                        on_track_double_clicked(track_info.clone());
                    }
                    
                    // Right-click context menu with auto-selection
                    response.context_menu(|ui| {
                        // プレイリストに追加メニュー
                        ui.menu_button("プレイリストに追加", |ui| {
                            // 新プレイリスト作成オプション
                            ui.separator();
                            if ui.button("➕ 新たなプレイリストを作成して追加").clicked() {
                                on_create_playlist_with_track(track_info.clone());
                                ui.close_menu();
                            }
                            ui.separator();
                            
                            // 既存のプレイリスト一覧
                            for playlist in playlists {
                                if ui.button(&playlist.name).clicked() {
                                    on_add_to_playlist(track_info.clone(), playlist.id.clone());
                                    ui.close_menu();
                                }
                            }
                        });
                    });
                }
            } else {
                ui.label(format!("{} {}", icon, node.name));
            }
        });

        if node.expanded && !node.children.is_empty() {
            ui.indent(format!("indent_{}", current_path.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("_")), |ui| {
                for (child_index, child) in node.children.iter().enumerate() {
                    if let Some(child_action) = Self::show_node_recursive(
                        ui,
                        child,
                        search_query,
                        selected_track,
                        selected_tracks,
                        playlists,
                        child_index,
                        &current_path,
                        on_track_selected,
                        on_track_double_clicked,
                        on_add_to_playlist,
                        on_add_album_to_playlist,
                        on_add_artist_to_playlist,
                        on_create_playlist_with_track,
                        on_create_playlist_with_album,
                        on_create_playlist_with_artist,
                    ) {
                        if action.is_none() {
                            action = Some(child_action);
                        }
                    }
                }
            });
        }

        action
    }

    fn toggle_expanded_at_path(nodes: &mut Vec<MusicTreeNode>, path: &[usize]) {
        if path.is_empty() {
            return;
        }

        if path.len() == 1 {
            if let Some(node) = nodes.get_mut(path[0]) {
                node.expanded = !node.expanded;
            }
        } else {
            if let Some(node) = nodes.get_mut(path[0]) {
                Self::toggle_expanded_at_path_recursive(&mut node.children, &path[1..]);
            }
        }
    }

    fn toggle_expanded_at_path_recursive(nodes: &mut Vec<MusicTreeNode>, path: &[usize]) {
        if path.is_empty() {
            return;
        }

        if path.len() == 1 {
            if let Some(node) = nodes.get_mut(path[0]) {
                node.expanded = !node.expanded;
            }
        } else {
            if let Some(node) = nodes.get_mut(path[0]) {
                Self::toggle_expanded_at_path_recursive(&mut node.children, &path[1..]);
            }
        }
    }

    fn get_node_icon(node_type: &MusicNodeType, expanded: bool) -> &'static str {
        match node_type {
            MusicNodeType::SectionHeader => {
                if expanded { "▼" } else { "▶" }
            },
            MusicNodeType::Composer => "🎼",
            MusicNodeType::Artist => "👤",
            MusicNodeType::Album => "💿",
            MusicNodeType::Track => "🎵",
        }
    }
}

#[derive(Debug)]
enum TreeAction {
    ToggleExpanded { path: Vec<usize> },
}