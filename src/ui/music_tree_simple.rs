use crate::music::{MusicTreeNode, MusicNodeType, TrackInfo};
use crate::ui::components::show_clickable_highlighted_text;
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
        on_track_selected: &mut dyn FnMut(TrackInfo, bool, bool), // ctrl_held, shift_held
        on_track_double_clicked: &mut dyn FnMut(TrackInfo),
        on_add_to_queue: &mut dyn FnMut(TrackInfo), // Add specific track or selected tracks to queue
        on_add_album_to_queue: &mut dyn FnMut(&MusicTreeNode), // Add entire album to queue
        on_add_artist_to_queue: &mut dyn FnMut(&MusicTreeNode), // Add entire artist to queue
    ) {
        let mut actions = Vec::new();
        
        for (i, node) in nodes.iter().enumerate() {
            if let Some(action) = Self::show_node_recursive(
                ui,
                node,
                search_query,
                selected_track,
                selected_tracks,
                i,
                &[],
                on_track_selected,
                on_track_double_clicked,
                on_add_to_queue,
                on_add_album_to_queue,
                on_add_artist_to_queue,
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
        index: usize,
        parent_path: &[usize],
        on_track_selected: &mut dyn FnMut(TrackInfo, bool, bool),
        on_track_double_clicked: &mut dyn FnMut(TrackInfo),
        on_add_to_queue: &mut dyn FnMut(TrackInfo),
        on_add_album_to_queue: &mut dyn FnMut(&MusicTreeNode),
        on_add_artist_to_queue: &mut dyn FnMut(&MusicTreeNode),
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
                            if ui.button("アルバムをキューに追加").clicked() {
                                on_add_album_to_queue(node);
                                ui.close_menu();
                            }
                        });
                    },
                    MusicNodeType::Artist => {
                        response.context_menu(|ui| {
                            if ui.button("アーティストの楽曲をキューに追加").clicked() {
                                on_add_artist_to_queue(node);
                                ui.close_menu();
                            }
                        });
                    },
                    MusicNodeType::Composer => {
                        response.context_menu(|ui| {
                            if ui.button("作曲家の楽曲をキューに追加").clicked() {
                                on_add_artist_to_queue(node);
                                ui.close_menu();
                            }
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
                        // Determine if this track is selected and what text to show
                        let track_is_selected = selected_tracks.contains(&track_info.path) || 
                            (selected_track.map(|st| st.path == track_info.path).unwrap_or(false));
                        
                        let menu_text = if track_is_selected {
                            // If the track is selected, show count based on current selection
                            let selected_count = if !selected_tracks.is_empty() {
                                selected_tracks.len()
                            } else {
                                1
                            };
                            
                            if selected_count == 1 {
                                "楽曲をキューに追加".to_string()
                            } else {
                                format!("{}曲をキューに追加", selected_count)
                            }
                        } else {
                            // If the track is not selected, it will auto-select this single track
                            "楽曲をキューに追加".to_string()
                        };
                        
                        if ui.button(menu_text).clicked() {
                            on_add_to_queue(track_info.clone());
                            ui.close_menu();
                        }
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
                        child_index,
                        &current_path,
                        on_track_selected,
                        on_track_double_clicked,
                        on_add_to_queue,
                        on_add_album_to_queue,
                        on_add_artist_to_queue,
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