use crate::music::TrackInfo;

pub struct PlaybackQueue {
    tracks: Vec<TrackInfo>,
    current_index: Option<usize>,
    selected_indices: std::collections::HashSet<usize>,
    last_selected_index: Option<usize>, // For range selection
}

impl PlaybackQueue {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            current_index: None,
            selected_indices: std::collections::HashSet::new(),
            last_selected_index: None,
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

    pub fn move_selected_up(&mut self) {
        if self.selected_indices.is_empty() {
            return;
        }
        
        let mut selected_indices: Vec<usize> = self.selected_indices.iter().cloned().collect();
        selected_indices.sort();
        
        // Can't move up if the first selected item is already at index 0
        if selected_indices[0] == 0 {
            return;
        }
        
        // Move each selected track up by one position
        let mut new_selected_indices = std::collections::HashSet::new();
        for &index in &selected_indices {
            let new_index = index - 1;
            self.tracks.swap(index, new_index);
            new_selected_indices.insert(new_index);
            
            // Update current_index if needed
            if let Some(current) = self.current_index {
                if current == index {
                    self.current_index = Some(new_index);
                } else if current == new_index {
                    self.current_index = Some(index);
                }
            }
        }
        
        self.selected_indices = new_selected_indices;
        
        // Update last_selected_index
        if let Some(last_selected) = self.last_selected_index {
            if selected_indices.contains(&last_selected) {
                self.last_selected_index = Some(last_selected - 1);
            }
        }
    }
    
    pub fn move_selected_down(&mut self) {
        if self.selected_indices.is_empty() || self.tracks.is_empty() {
            return;
        }
        
        let mut selected_indices: Vec<usize> = self.selected_indices.iter().cloned().collect();
        selected_indices.sort();
        selected_indices.reverse(); // Process from bottom to top
        
        // Can't move down if the last selected item is already at the end
        if selected_indices[0] >= self.tracks.len() - 1 {
            return;
        }
        
        // Move each selected track down by one position
        let mut new_selected_indices = std::collections::HashSet::new();
        for &index in &selected_indices {
            let new_index = index + 1;
            self.tracks.swap(index, new_index);
            new_selected_indices.insert(new_index);
            
            // Update current_index if needed
            if let Some(current) = self.current_index {
                if current == index {
                    self.current_index = Some(new_index);
                } else if current == new_index {
                    self.current_index = Some(index);
                }
            }
        }
        
        self.selected_indices = new_selected_indices;
        
        // Update last_selected_index
        if let Some(last_selected) = self.last_selected_index {
            if selected_indices.contains(&last_selected) {
                self.last_selected_index = Some(last_selected + 1);
            }
        }
    }
    
    pub fn move_selected_to_top(&mut self) {
        if self.selected_indices.is_empty() {
            return;
        }
        
        let mut selected_indices: Vec<usize> = self.selected_indices.iter().cloned().collect();
        selected_indices.sort();
        
        // Extract selected tracks
        let mut selected_tracks = Vec::new();
        for &index in selected_indices.iter().rev() {
            selected_tracks.insert(0, self.tracks.remove(index));
        }
        
        // Insert at the beginning
        for (i, track) in selected_tracks.into_iter().enumerate() {
            self.tracks.insert(i, track);
        }
        
        // Update selections to new positions (0, 1, 2, ...)
        self.selected_indices.clear();
        for i in 0..selected_indices.len() {
            self.selected_indices.insert(i);
        }
        
        // Update current_index and last_selected_index
        if let Some(current) = self.current_index {
            if let Some(pos) = selected_indices.iter().position(|&x| x == current) {
                self.current_index = Some(pos);
            } else {
                // Current track was not selected, adjust its position
                let mut new_current = current;
                for &selected_index in &selected_indices {
                    if selected_index < current {
                        new_current = new_current.saturating_sub(1);
                    }
                }
                self.current_index = Some(new_current + selected_indices.len());
            }
        }
        
        if let Some(last_selected) = self.last_selected_index {
            if let Some(pos) = selected_indices.iter().position(|&x| x == last_selected) {
                self.last_selected_index = Some(pos);
            }
        }
    }
    
    pub fn move_selected_to_bottom(&mut self) {
        if self.selected_indices.is_empty() {
            return;
        }
        
        let mut selected_indices: Vec<usize> = self.selected_indices.iter().cloned().collect();
        selected_indices.sort();
        
        // Extract selected tracks
        let mut selected_tracks = Vec::new();
        for &index in selected_indices.iter().rev() {
            selected_tracks.insert(0, self.tracks.remove(index));
        }
        
        // Append to the end
        let start_position = self.tracks.len();
        self.tracks.extend(selected_tracks);
        
        // Update selections to new positions
        self.selected_indices.clear();
        for i in 0..selected_indices.len() {
            self.selected_indices.insert(start_position + i);
        }
        
        // Update current_index and last_selected_index
        if let Some(current) = self.current_index {
            if let Some(pos) = selected_indices.iter().position(|&x| x == current) {
                self.current_index = Some(start_position + pos);
            } else {
                // Current track was not selected, adjust its position
                let mut new_current = current;
                for &selected_index in &selected_indices {
                    if selected_index < current {
                        new_current = new_current.saturating_sub(1);
                    }
                }
                self.current_index = Some(new_current);
            }
        }
        
        if let Some(last_selected) = self.last_selected_index {
            if let Some(pos) = selected_indices.iter().position(|&x| x == last_selected) {
                self.last_selected_index = Some(start_position + pos);
            }
        }
    }

    pub fn move_track(&mut self, from_index: usize, to_index: usize) {
        if from_index >= self.tracks.len() || to_index >= self.tracks.len() || from_index == to_index {
            return;
        }

        // Move the track
        let track = self.tracks.remove(from_index);
        self.tracks.insert(to_index, track);

        // Update current_index if needed
        if let Some(current) = self.current_index {
            if current == from_index {
                // The currently playing track was moved
                self.current_index = Some(to_index);
            } else if from_index < current && to_index >= current {
                // Track moved from before current to after current
                self.current_index = Some(current - 1);
            } else if from_index > current && to_index <= current {
                // Track moved from after current to before current
                self.current_index = Some(current + 1);
            }
        }

        // Update selected indices
        let mut new_selected_indices = std::collections::HashSet::new();
        for &selected_index in &self.selected_indices {
            let new_index = if selected_index == from_index {
                to_index
            } else if from_index < selected_index && to_index >= selected_index {
                selected_index - 1
            } else if from_index > selected_index && to_index <= selected_index {
                selected_index + 1
            } else {
                selected_index
            };
            new_selected_indices.insert(new_index);
        }
        self.selected_indices = new_selected_indices;

        // Update last_selected_index if needed
        if let Some(last_selected) = self.last_selected_index {
            if last_selected == from_index {
                self.last_selected_index = Some(to_index);
            } else if from_index < last_selected && to_index >= last_selected {
                self.last_selected_index = Some(last_selected - 1);
            } else if from_index > last_selected && to_index <= last_selected {
                self.last_selected_index = Some(last_selected + 1);
            }
        }
    }

    pub fn handle_item_selection(&mut self, index: usize, ctrl_held: bool, shift_held: bool) {
        if index >= self.tracks.len() {
            return; // Invalid index
        }

        if shift_held && self.last_selected_index.is_some() {
            // Range selection mode
            self.handle_range_selection(index);
        } else if ctrl_held {
            // Multiple selection mode - preserve existing selections
            
            // Toggle the clicked item
            if self.selected_indices.contains(&index) {
                // Deselect if already selected
                self.selected_indices.remove(&index);
            } else {
                // Add to selection
                self.selected_indices.insert(index);
            }
            
            // Update last selected index
            self.last_selected_index = Some(index);
        } else {
            // Single selection mode - clear multiple selections
            self.selected_indices.clear();
            self.selected_indices.insert(index);
            self.last_selected_index = Some(index);
        }
    }

    fn handle_range_selection(&mut self, end_index: usize) {
        let start_index = match self.last_selected_index {
            Some(idx) => idx,
            None => return,
        };

        if start_index == end_index {
            // Same item - just select it
            self.selected_indices.clear();
            self.selected_indices.insert(end_index);
            return;
        }

        // Select range (inclusive)
        let (min_idx, max_idx) = if start_index <= end_index {
            (start_index, end_index)
        } else {
            (end_index, start_index)
        };

        self.selected_indices.clear();
        for i in min_idx..=max_idx {
            self.selected_indices.insert(i);
        }
    }
}