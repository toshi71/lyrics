use crate::music::TrackInfo;

pub struct PlaybackQueue {
    tracks: Vec<TrackInfo>,
    current_index: Option<usize>,
    selected_indices: std::collections::HashSet<usize>,
}

impl PlaybackQueue {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            current_index: None,
            selected_indices: std::collections::HashSet::new(),
        }
    }

    pub fn add_track(&mut self, track: TrackInfo) {
        self.tracks.push(track);
        if self.current_index.is_none() {
            self.current_index = Some(0);
        }
    }

    pub fn add_track_at_front(&mut self, track: TrackInfo) {
        self.tracks.insert(0, track);
        if let Some(index) = self.current_index {
            self.current_index = Some(index + 1);
        }
        self.current_index = Some(0);
    }

    pub fn get_current_track(&self) -> Option<&TrackInfo> {
        if let Some(index) = self.current_index {
            self.tracks.get(index)
        } else {
            None
        }
    }

    pub fn get_next_track(&self) -> Option<&TrackInfo> {
        if let Some(index) = self.current_index {
            let next_index = index + 1;
            if next_index < self.tracks.len() {
                self.tracks.get(next_index)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_previous_track(&self) -> Option<&TrackInfo> {
        if let Some(index) = self.current_index {
            if index > 0 {
                self.tracks.get(index - 1)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn move_to_next(&mut self) -> Option<&TrackInfo> {
        if let Some(index) = self.current_index {
            let next_index = index + 1;
            if next_index < self.tracks.len() {
                self.current_index = Some(next_index);
                return self.tracks.get(next_index);
            }
        }
        None
    }

    pub fn move_to_previous(&mut self) -> Option<&TrackInfo> {
        if let Some(index) = self.current_index {
            if index > 0 {
                self.current_index = Some(index - 1);
                return self.tracks.get(index - 1);
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_index = None;
        self.selected_indices.clear();
    }

    pub fn get_tracks(&self) -> &Vec<TrackInfo> {
        &self.tracks
    }

    pub fn get_current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn set_current_index(&mut self, index: usize) {
        if index < self.tracks.len() {
            self.current_index = Some(index);
        }
    }

    pub fn toggle_selection(&mut self, index: usize) {
        if index < self.tracks.len() {
            if self.selected_indices.contains(&index) {
                self.selected_indices.remove(&index);
            } else {
                self.selected_indices.insert(index);
            }
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_indices.clear();
    }

    pub fn set_selection(&mut self, index: usize, selected: bool) {
        if index < self.tracks.len() {
            if selected {
                self.selected_indices.insert(index);
            } else {
                self.selected_indices.remove(&index);
            }
        }
    }

    pub fn is_selected(&self, index: usize) -> bool {
        self.selected_indices.contains(&index)
    }

    pub fn get_selected_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self.selected_indices.iter().cloned().collect();
        indices.sort();
        indices
    }

    pub fn remove_selected(&mut self) {
        let mut indices = self.get_selected_indices();
        indices.reverse(); // Remove from back to front to maintain valid indices
        
        for &index in &indices {
            self.tracks.remove(index);
            
            // Update current index if needed
            if let Some(current) = self.current_index {
                if current == index {
                    // Current track was removed
                    if index < self.tracks.len() {
                        // Keep current index if there's a next track
                        self.current_index = Some(index);
                    } else if !self.tracks.is_empty() {
                        // Move to last track if we removed the last one
                        self.current_index = Some(self.tracks.len() - 1);
                    } else {
                        // Queue is empty
                        self.current_index = None;
                    }
                } else if current > index {
                    // Shift current index down
                    self.current_index = Some(current - 1);
                }
            }
        }
        
        self.selected_indices.clear();
    }

    pub fn has_selection(&self) -> bool {
        !self.selected_indices.is_empty()
    }
}