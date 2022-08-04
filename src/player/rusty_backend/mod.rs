#![cfg_attr(test, deny(missing_docs))]

mod conversions;
#[allow(
    unused,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::doc_markdown,
    clippy::semicolon_if_nothing_returned,
    clippy::missing_safety_doc,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnecessary_wraps,
    clippy::needless_pass_by_value,
    clippy::manual_assert,
    clippy::ptr_as_ptr,
    clippy::redundant_closure_for_method_calls,
    clippy::explicit_iter_loop,
    clippy::range_plus_one,
    clippy::default_trait_access,
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::items_after_statements,
    clippy::unreadable_literal,
    clippy::unnested_or_patterns
)]
mod cpal;
mod sink;
mod stream;

pub mod buffer;
pub mod decoder;
pub mod dynamic_mixer;
pub mod queue;
pub mod source;

pub use conversions::Sample;
pub use cpal::{
    traits::DeviceTrait, Device, Devices, DevicesError, InputDevices, OutputDevices,
    SupportedStreamConfig,
};
pub use decoder::Symphonia;
pub use sink::Sink;
pub use source::Source;
pub use stream::{OutputStream, OutputStreamHandle, PlayError, StreamError};

use super::{PlayerMsg, PlayerTrait};
use crate::config::Settings;
use anyhow::Result;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::time::Duration;

static VOLUME_STEP: u16 = 5;
static SEEK_STEP: f64 = 5.0;

pub struct Player {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    pub sink: Sink,
    pub total_duration: Option<Duration>,
    volume: u16,
    speed: i32,
    pub gapless: bool,
    pub message_tx: Sender<PlayerMsg>,
}

impl Player {
    pub fn new(config: &Settings, tx: Sender<PlayerMsg>) -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();
        let gapless = config.gapless;
        let sink = Sink::try_new(&handle, gapless, tx.clone()).unwrap();
        let volume = config.volume.try_into().unwrap();
        sink.set_volume(f32::from(volume) / 100.0);
        let speed = config.speed;

        let mut this = Self {
            _stream: stream,
            handle,
            sink,
            total_duration: None,
            volume,
            speed,
            gapless,
            message_tx: tx,
        };
        this.set_speed(speed);
        this
    }

    pub fn enqueue(&mut self, item: &str) {
        let p1 = Path::new(item);
        if let Ok(file) = File::open(p1) {
            // if let Ok(decoder) = Symphonia::new(file, self.gapless) {
            //     self.total_duration = decoder.total_duration();
            //     self.sink.append(decoder);
            //     self.set_speed(self.speed);
            //     // self.sink.message_on_end();
            // }
            match Symphonia::new(file, self.gapless) {
                Ok(decoder) => {
                    self.total_duration = decoder.total_duration();
                    self.sink.append(decoder);
                }
                Err(e) => eprintln!("error is: {:?}", e),
            }
        }
    }

    pub fn enqueue_next(&mut self, item: &str) -> Option<Duration> {
        let mut duration = None;
        let p1 = Path::new(item);
        if let Ok(file) = File::open(p1) {
            if let Ok(decoder) = Symphonia::new(file, self.gapless) {
                duration = decoder.total_duration();
                self.sink.append(decoder);
                // self.sink.message_on_end();
            }
        }
        duration
    }

    fn play(&mut self, current_item: &str) {
        self.enqueue(current_item);
    }

    fn stop(&mut self) {
        self.sink = Sink::try_new(&self.handle, self.gapless, self.message_tx.clone()).unwrap();
        self.sink.set_volume(f32::from(self.volume) / 100.0);
    }
    fn elapsed(&self) -> Duration {
        self.sink.elapsed()
    }
    fn duration(&self) -> Option<f64> {
        self.total_duration
            .map(|duration| duration.as_secs_f64() - 0.29)
    }

    fn seek_fw(&mut self) {
        let new_pos = self.elapsed().as_secs_f64() + SEEK_STEP;
        if let Some(duration) = self.duration() {
            if new_pos < duration - SEEK_STEP {
                self.seek_to(Duration::from_secs_f64(new_pos));
            }
        }
    }
    fn seek_bw(&mut self) {
        let mut new_pos = self.elapsed().as_secs_f64() - SEEK_STEP;
        if new_pos < 0.0 {
            new_pos = 0.0;
        }

        self.seek_to(Duration::from_secs_f64(new_pos));
    }
    fn seek_to(&self, time: Duration) {
        self.sink.seek(time);
        self.get_progress().ok();
    }

    pub fn skip_one(&mut self) {
        self.sink.skip_one();
        if self.is_paused() {
            self.sink.play();
        }
    }

    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation
    )]
    fn get_progress(&self) -> Result<()> {
        let position = self.elapsed().as_secs() as i64;
        let duration = self.duration().unwrap_or(99.0) as i64;
        self.message_tx
            .send(PlayerMsg::Progress(position, duration))?;
        Ok(())
    }
}

impl PlayerTrait for Player {
    fn add_and_play(&mut self, current_track: &str) {
        self.play(current_track);
    }

    fn volume(&self) -> i32 {
        self.volume.into()
    }

    fn volume_up(&mut self) {
        let volume = i32::from(self.volume) + i32::from(VOLUME_STEP);
        self.set_volume(volume);
    }

    fn volume_down(&mut self) {
        let volume = i32::from(self.volume) - i32::from(VOLUME_STEP);
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
        self.sink.pause();
    }

    fn resume(&mut self) {
        self.sink.play();
    }

    fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    fn seek(&mut self, secs: i64) -> Result<()> {
        if secs.is_positive() {
            self.seek_fw();
            return Ok(());
        }

        self.seek_bw();
        Ok(())
    }

    fn speed_up(&mut self) {
        let mut speed = self.speed + 1;
        if speed > 30 {
            speed = 30;
        }
        self.set_speed(speed);
    }

    fn speed_down(&mut self) {
        let mut speed = self.speed - 1;
        if speed < 1 {
            speed = 1;
        }
        self.set_speed(speed);
    }

    #[allow(clippy::cast_precision_loss)]
    fn set_speed(&mut self, speed: i32) {
        self.speed = speed;
        let speed = speed as f32 / 10.0;
        self.sink.set_speed(speed);
    }

    fn speed(&self) -> i32 {
        self.speed
    }
    fn stop(&mut self) {
        self.stop();
    }
}
