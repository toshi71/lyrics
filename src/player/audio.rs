use crate::music::TrackInfo;
use kira::manager::{AudioManager, AudioManagerSettings, backend::cpal::CpalBackend};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::sound::{PlaybackState as KiraPlaybackState, FromFileError};
use kira::clock::{ClockHandle, ClockSpeed};
use kira::tween::Tween;
use metaflac::Tag;
use std::time::{Duration, Instant};

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
    total_duration: Option<Duration>,
    state: PlaybackState,
    clock: Option<ClockHandle>,
    play_start_time: Option<Instant>,
    paused_duration: Duration,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).ok();
        let clock = manager.as_mut().and_then(|m| m.add_clock(ClockSpeed::TicksPerSecond(44100.0)).ok());
        
        Self {
            manager,
            current_sound: None,
            current_track: None,
            total_duration: None,
            state: PlaybackState::Stopped,
            clock,
            play_start_time: None,
            paused_duration: Duration::from_secs(0),
        }
    }

    pub fn play(&mut self, track: TrackInfo) -> Result<(), Box<dyn std::error::Error>> {
        self.stop();

        // 総再生時間をメタデータから取得（borrowingを避けるために先に実行）
        let duration = Self::get_flac_duration_static(&track.path);

        if let Some(manager) = &mut self.manager {
            let sound_data = StreamingSoundData::from_file(&track.path)?;
            let sound_handle = manager.play(sound_data)?;
            
            self.current_sound = Some(sound_handle);
            self.current_track = Some(track);
            self.total_duration = duration;
            self.state = PlaybackState::Playing;
            self.play_start_time = Some(Instant::now());
            self.paused_duration = Duration::from_secs(0);
        }

        Ok(())
    }

    pub fn pause(&mut self) {
        if let Some(ref mut sound) = self.current_sound {
            // 現在までの再生時間を記録
            if let Some(start_time) = self.play_start_time {
                self.paused_duration = self.paused_duration + start_time.elapsed();
            }
            let _ = sound.pause(Tween::default());
            self.state = PlaybackState::Paused;
            self.play_start_time = None;
        }
    }

    pub fn resume(&mut self) {
        if let Some(ref mut sound) = self.current_sound {
            let _ = sound.resume(Tween::default());
            self.state = PlaybackState::Playing;
            self.play_start_time = Some(Instant::now());
        }
    }

    pub fn stop(&mut self) {
        if let Some(ref mut sound) = self.current_sound {
            let _ = sound.stop(Tween::default());
        }
        self.current_sound = None;
        self.current_track = None;
        self.total_duration = None;
        self.state = PlaybackState::Stopped;
        self.play_start_time = None;
        self.paused_duration = Duration::from_secs(0);
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
        match self.state {
            PlaybackState::Playing => {
                if let Some(start_time) = self.play_start_time {
                    self.paused_duration + start_time.elapsed()
                } else {
                    self.paused_duration
                }
            },
            PlaybackState::Paused => {
                self.paused_duration
            },
            PlaybackState::Stopped => {
                Duration::from_secs(0)
            }
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

    pub fn get_total_duration(&self) -> Option<Duration> {
        self.total_duration
    }

    fn get_flac_duration_static(path: &std::path::Path) -> Option<Duration> {
        if path.extension()?.to_str()? == "flac" {
            let tag = Tag::read_from_path(path).ok()?;
            let streaminfo = tag.get_streaminfo()?;
            let total_samples = streaminfo.total_samples;
            let sample_rate = streaminfo.sample_rate;
            if total_samples > 0 && sample_rate > 0 {
                Some(Duration::from_secs_f64(total_samples as f64 / sample_rate as f64))
            } else {
                None
            }
        } else {
            // FLAC以外のファイルの場合はNone（後で他フォーマット対応可能）
            None
        }
    }
}