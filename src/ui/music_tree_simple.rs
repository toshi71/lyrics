use crate::music::{MusicTreeNode, MusicNodeType, TrackInfo};
use crate::ui::components::show_clickable_highlighted_text;
use eframe::egui;

pub struct MusicTreeUI;

impl MusicTreeUI {
    pub fn show(
        ui: &mut egui::Ui,
        nodes: &mut Vec<MusicTreeNode>,
        search_query: &str,
        on_track_selected: &mut dyn FnMut(TrackInfo),
        on_track_double_clicked: &mut dyn FnMut(TrackInfo),
    ) {
        let mut actions = Vec::new();
        
        for (i, node) in nodes.iter().enumerate() {
            if let Some(action) = Self::show_node_recursive(
                ui,
                node,
                search_query,
                i,
                &[],
                on_track_selected,
                on_track_double_clicked,
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
        index: usize,
        parent_path: &[usize],
        on_track_selected: &mut dyn FnMut(TrackInfo),
        on_track_double_clicked: &mut dyn FnMut(TrackInfo),
    ) -> Option<TreeAction> {
        let mut current_path = parent_path.to_vec();
        current_path.push(index);
        
        let icon = Self::get_node_icon(&node.node_type, node.expanded);
        let mut action = None;

        ui.horizontal(|ui| {
            if node.node_type != MusicNodeType::Track && !node.children.is_empty() {
                let (clicked, _) = show_clickable_highlighted_text(ui, icon, &node.name, search_query);
                if clicked {
                    action = Some(TreeAction::ToggleExpanded { path: current_path.clone() });
                }
            } else if node.node_type == MusicNodeType::Track {
                let (clicked, double_clicked) = show_clickable_highlighted_text(ui, icon, &node.name, search_query);
                if let Some(track_info) = &node.track_info {
                    if clicked {
                        on_track_selected(track_info.clone());
                    }
                    if double_clicked {
                        on_track_double_clicked(track_info.clone());
                    }
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
                        child_index,
                        &current_path,
                        on_track_selected,
                        on_track_double_clicked,
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