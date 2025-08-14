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
        let node_count = nodes.len();
        for i in 0..node_count {
            Self::show_node(
                ui,
                nodes,
                i,
                search_query,
                on_track_selected,
                on_track_double_clicked,
            );
        }
    }

    fn show_node(
        ui: &mut egui::Ui,
        nodes: &mut Vec<MusicTreeNode>,
        index: usize,
        search_query: &str,
        on_track_selected: &mut dyn FnMut(TrackInfo),
        on_track_double_clicked: &mut dyn FnMut(TrackInfo),
    ) {
        let node = &nodes[index];
        let icon = Self::get_node_icon(&node.node_type, node.expanded);

        ui.horizontal(|ui| {
            if node.node_type != MusicNodeType::Track && !node.children.is_empty() {
                let (clicked, _) = show_clickable_highlighted_text(ui, icon, &node.name, search_query);
                if clicked {
                    nodes[index].expanded = !nodes[index].expanded;
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
            ui.indent(format!("music_indent_{}", index), |ui| {
                let mut children = node.children.clone();
                Self::show_children(
                    ui,
                    &mut children,
                    search_query,
                    on_track_selected,
                    on_track_double_clicked,
                );
                // Update the original children
                nodes[index].children = children;
            });
        }
    }

    fn show_children(
        ui: &mut egui::Ui,
        children: &mut Vec<MusicTreeNode>,
        search_query: &str,
        on_track_selected: &mut dyn FnMut(TrackInfo),
        on_track_double_clicked: &mut dyn FnMut(TrackInfo),
    ) {
        for (i, child) in children.iter().enumerate() {
            Self::show_child_node(
                ui,
                children,
                i,
                search_query,
                on_track_selected,
                on_track_double_clicked,
            );
        }
    }

    fn show_child_node(
        ui: &mut egui::Ui,
        children: &mut Vec<MusicTreeNode>,
        index: usize,
        search_query: &str,
        on_track_selected: &mut dyn FnMut(TrackInfo),
        on_track_double_clicked: &mut dyn FnMut(TrackInfo),
    ) {
        let child = &children[index];
        let icon = Self::get_node_icon(&child.node_type, child.expanded);

        ui.horizontal(|ui| {
            if child.node_type != MusicNodeType::Track && !child.children.is_empty() {
                let (clicked, _) = show_clickable_highlighted_text(ui, icon, &child.name, search_query);
                if clicked {
                    children[index].expanded = !children[index].expanded;
                }
            } else if child.node_type == MusicNodeType::Track {
                let (clicked, double_clicked) = show_clickable_highlighted_text(ui, icon, &child.name, search_query);
                if let Some(track_info) = &child.track_info {
                    if clicked {
                        on_track_selected(track_info.clone());
                    }
                    if double_clicked {
                        on_track_double_clicked(track_info.clone());
                    }
                }
            } else {
                ui.label(format!("{} {}", icon, child.name));
            }
        });

        if child.expanded && !child.children.is_empty() {
            ui.indent(format!("child_indent_{}", index), |ui| {
                let mut grandchildren = child.children.clone();
                Self::show_children(
                    ui,
                    &mut grandchildren,
                    search_query,
                    on_track_selected,
                    on_track_double_clicked,
                );
                children[index].children = grandchildren;
            });
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