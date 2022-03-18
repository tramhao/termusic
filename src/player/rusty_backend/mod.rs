#![cfg_attr(test, deny(missing_docs))]

mod conversions;
mod sink;
mod stream;

pub mod buffer;
pub mod decoder;
pub mod dynamic_mixer;
pub mod queue;
pub mod source;

pub use conversions::Sample;
pub use cpal::{
    self, traits::DeviceTrait, Device, Devices, DevicesError, InputDevices, OutputDevices,
    SupportedStreamConfig,
};
pub use decoder::Decoder;
pub use sink::Sink;
pub use source::Source;
pub use stream::{OutputStream, OutputStreamHandle, PlayError, StreamError};

use std::path::Path;
use std::time::Duration;
use std::{fs::File, io::BufReader};

use super::GeneralP;
use anyhow::Result;

static VOLUME_STEP: u16 = 5;

pub struct Player {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    total_duration: Option<Duration>,
    volume: u16,
    safe_guard: bool,
}
impl Default for Player {
    fn default() -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle).unwrap();
        let volume = 50;
        sink.set_volume(f32::from(volume) / 100.0);

        Self {
            _stream: stream,
            handle,
            sink,
            total_duration: None,
            volume,
            safe_guard: true,
        }
    }
}

#[allow(unused)]
impl Player {
    pub const fn set_volume_inside(mut self, volume: u16) -> Self {
        self.volume = volume;
        self
    }
    pub fn change_volume(&mut self, positive: bool) {
        if positive {
            self.volume += VOLUME_STEP;
        } else if self.volume >= VOLUME_STEP {
            self.volume -= VOLUME_STEP;
        } else {
            self.volume = 0;
        }

        if self.volume > 100 {
            self.volume = 100;
        }

        self.sink.set_volume(f32::from(self.volume) / 100.0);
    }
    pub fn sleep_until_end(&self) {
        self.sink.sleep_until_end();
    }
    pub fn play(&mut self, path: &Path) {
        self.stop();
        let file = File::open(path).unwrap();
        let decoder = Decoder::new_decoder(BufReader::new(file)).unwrap();
        self.total_duration = decoder.total_duration();
        self.sink.append(decoder);
    }
    pub fn stop(&mut self) {
        self.sink.destroy();
        self.sink = Sink::try_new(&self.handle).unwrap();
        self.sink.set_volume(f32::from(self.volume) / 100.0);
    }
    pub fn elapsed(&self) -> Duration {
        self.sink.elapsed()
    }
    pub fn duration(&self) -> Option<f64> {
        self.total_duration
            .map(|duration| duration.as_secs_f64() - 0.29)
    }
    pub fn toggle_playback(&self) {
        self.sink.toggle_playback();
    }
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
    pub fn seek_fw(&mut self) {
        let new_pos = self.elapsed().as_secs_f64() + 5.0;
        if let Some(duration) = self.duration() {
            if new_pos > duration {
                self.safe_guard = true;
            } else {
                self.seek_to(Duration::from_secs_f64(new_pos));
            }
        }
    }
    pub fn seek_bw(&mut self) {
        let mut new_pos = self.elapsed().as_secs_f64() - 5.0;
        if new_pos < 0.0 {
            new_pos = 0.0;
        }

        self.seek_to(Duration::from_secs_f64(new_pos));
    }
    pub fn seek_to(&self, time: Duration) {
        self.sink.seek(time);
    }
    pub fn percentage(&self) -> f64 {
        self.duration().map_or(0.0, |duration| {
            let elapsed = self.elapsed();
            elapsed.as_secs_f64() / duration
        })
    }
    pub fn trigger_next(&mut self) -> bool {
        // //TODO: duration is broken for certain files
        // //This will cause songs to play forever
        // if duration == -0.29 {
        //     return false;
        // }
        if let Some(duration) = self.duration() {
            if self.elapsed().as_secs_f64() > duration {
                self.safe_guard = true;
            }
        }

        // dbg!(self.safe_guard);

        if self.safe_guard {
            self.safe_guard = false;
            true
        } else {
            false
        }
    }
    pub const fn volume_percent(&self) -> u16 {
        self.volume
    }
}

impl GeneralP for Player {
    fn add_and_play(&mut self, song: &str) {
        let p = Path::new(song);
        self.play(p);
    }

    fn volume(&self) -> i32 {
        self.volume.into()
    }

    fn volume_up(&mut self) {
        let volume = i32::from(self.volume) + 5;
        self.set_volume(volume);
    }

    fn volume_down(&mut self) {
        let volume = i32::from(self.volume) - 5;
        self.set_volume(volume);
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn set_volume(&mut self, mut volume: i32) {
        if volume > 100 {
            volume = 100;
        } else if volume < 0 {
            volume = 0;
        }
        self.volume = volume as u16;
        self.sink.set_volume(f32::from(self.volume) / 100.0);
    }

    fn pause(&mut self) {
        self.toggle_playback();
    }

    fn resume(&mut self) {
        self.toggle_playback();
    }

    fn is_paused(&mut self) -> bool {
        false
    }

    fn seek(&mut self, secs: i64) -> Result<()> {
        if secs.is_positive() {
            self.seek_fw();
            return Ok(());
        }

        self.seek_bw();
        Ok(())
    }

    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation
    )]
    fn get_progress(&mut self) -> Result<(f64, i64, i64)> {
        let position = self.elapsed().as_secs() as i64;
        let duration = self.duration().unwrap_or(99.0) as i64;
        let mut percent = self.percentage() * 100.0;
        if percent > 100.0 {
            percent = 100.0;
        }
        Ok((percent, position, duration))
    }
}
