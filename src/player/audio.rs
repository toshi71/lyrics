use crate::music::TrackInfo;
use kira::manager::{AudioManager, AudioManagerSettings, backend::cpal::CpalBackend};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::sound::{PlaybackState as KiraPlaybackState, FromFileError};
use kira::clock::{ClockHandle, ClockSpeed};
use kira::tween::Tween;
use std::time::Duration;

#[derive(PartialEq, Clone)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

pub struct AudioPlayer {
    manager: Option<AudioManager<CpalBackend>>,
    current_sound: Option<StreamingSoundHandle<FromFileError>>,
    current_track: Option<TrackInfo>,
    state: PlaybackState,
    clock: Option<ClockHandle>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).ok();
        let clock = manager.as_mut().and_then(|m| m.add_clock(ClockSpeed::TicksPerSecond(44100.0)).ok());
        
        Self {
            manager,
            current_sound: None,
            current_track: None,
            state: PlaybackState::Stopped,
            clock,
        }
    }

    pub fn play(&mut self, track: TrackInfo) -> Result<(), Box<dyn std::error::Error>> {
        self.stop();

        if let Some(manager) = &mut self.manager {
            let sound_data = StreamingSoundData::from_file(&track.path)?;
            let sound_handle = manager.play(sound_data)?;
            
            self.current_sound = Some(sound_handle);
            self.current_track = Some(track);
            self.state = PlaybackState::Playing;
        }

        Ok(())
    }

    pub fn pause(&mut self) {
        if let Some(ref mut sound) = self.current_sound {
            let _ = sound.pause(Tween::default());
            self.state = PlaybackState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if let Some(ref mut sound) = self.current_sound {
            let _ = sound.resume(Tween::default());
            self.state = PlaybackState::Playing;
        }
    }

    pub fn stop(&mut self) {
        if let Some(ref mut sound) = self.current_sound {
            let _ = sound.stop(Tween::default());
        }
        self.current_sound = None;
        self.current_track = None;
        self.state = PlaybackState::Stopped;
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
        if let Some(sound) = &self.current_sound {
            Duration::from_secs_f64(sound.position())
        } else {
            Duration::from_secs(0)
        }
    }

    pub fn restart_current(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(track) = self.current_track.clone() {
            self.play(track)?;
        }
        Ok(())
    }

    pub fn is_finished(&self) -> bool {
        if let Some(sound) = &self.current_sound {
            matches!(sound.state(), KiraPlaybackState::Stopped)
        } else {
            true
        }
    }
}