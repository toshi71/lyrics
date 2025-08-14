use crate::music::TrackInfo;

pub struct PlaybackQueue {
    tracks: Vec<TrackInfo>,
    current_index: Option<usize>,
}

impl PlaybackQueue {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            current_index: None,
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
}