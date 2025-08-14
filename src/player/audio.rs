use crate::music::TrackInfo;
use rodio::{Decoder, OutputStream, Sink};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(PartialEq, Clone)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

pub struct AudioPlayer {
    _stream: Option<OutputStream>,
    sink: Option<Arc<Sink>>,
    current_track: Option<TrackInfo>,
    state: PlaybackState,
    start_time: Option<Instant>,
    paused_duration: Duration,
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            _stream: None,
            sink: None,
            current_track: None,
            state: PlaybackState::Stopped,
            start_time: None,
            paused_duration: Duration::new(0, 0),
        }
    }

    pub fn play(&mut self, track: TrackInfo) -> Result<(), Box<dyn std::error::Error>> {
        self.stop();

        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Arc::new(Sink::try_new(&stream_handle)?);

        let file = std::fs::File::open(&track.path)?;
        let source = Decoder::new(std::io::BufReader::new(file))?;
        sink.append(source);

        self._stream = Some(_stream);
        self.sink = Some(sink);
        self.current_track = Some(track);
        self.state = PlaybackState::Playing;
        self.start_time = Some(Instant::now());
        self.paused_duration = Duration::new(0, 0);

        Ok(())
    }

    pub fn pause(&mut self) {
        if let Some(ref sink) = self.sink {
            if let Some(start_time) = self.start_time {
                let elapsed = start_time.elapsed();
                self.paused_duration = self.paused_duration + elapsed;
            }
            sink.pause();
            self.state = PlaybackState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.play();
            self.state = PlaybackState::Playing;
            self.start_time = Some(Instant::now());
        }
    }

    pub fn stop(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.stop();
        }
        self._stream = None;
        self.sink = None;
        self.current_track = None;
        self.state = PlaybackState::Stopped;
        self.start_time = None;
        self.paused_duration = Duration::new(0, 0);
    }

    pub fn toggle_play_pause(&mut self) {
        match self.state {
            PlaybackState::Playing => self.pause(),
            PlaybackState::Paused => self.resume(),
            PlaybackState::Stopped => {
                // Note: This requires external track selection logic
            }
        }
    }

    pub fn get_state(&self) -> &PlaybackState {
        &self.state
    }

    pub fn get_current_track(&self) -> Option<&TrackInfo> {
        self.current_track.as_ref()
    }

    pub fn get_playback_position(&self) -> Duration {
        if let Some(start_time) = self.start_time {
            if self.state == PlaybackState::Playing {
                self.paused_duration + start_time.elapsed()
            } else {
                self.paused_duration
            }
        } else {
            Duration::new(0, 0)
        }
    }

    pub fn restart_current(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(track) = self.current_track.clone() {
            self.play(track)?;
        }
        Ok(())
    }

    pub fn is_finished(&self) -> bool {
        if let Some(ref sink) = self.sink {
            sink.empty()
        } else {
            true
        }
    }
}