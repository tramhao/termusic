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
#[cfg(target_os = "linux")]
mod cpal;
mod sink;
mod stream;

pub mod buffer;
pub mod decoder;
pub mod dynamic_mixer;
pub mod queue;
// pub mod seekable_buffer;
pub mod source;

pub use conversions::Sample;
pub use cpal::{
    default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BuildStreamError, ChannelCount, DefaultStreamConfigError, Device, Devices, DevicesError,
    InputDevices, OutputDevices, PlayStreamError, Sample as CpalSample, SampleFormat, SampleRate,
    Stream, SupportedStreamConfig, SupportedStreamConfigsError,
};
pub use decoder::Symphonia;
// pub use seekable_buffer::{Cache, SeekableBufReader};
pub use sink::Sink;
// use source::SeekableRequest;
pub use source::Source;
pub use stream::{OutputStream, OutputStreamHandle, PlayError, StreamError};

use super::{PlayerMsg, PlayerTrait};
use crate::config::Settings;
use anyhow::Result;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
use std::{fs::File, io::Cursor};
use symphonia::core::io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions};

static VOLUME_STEP: u16 = 5;

enum PlayerCmd {
    GetProgress,
    MessageOnEnd,
    Play(String, bool),
    Pause,
    QueueNext(String, bool),
    Resume,
    Seek(i64),
    SeekRelative(i64),
    Skip,
    Speed(i32),
    Stop,
    Volume(i64),
}

pub struct Player {
    pub total_duration: Option<Duration>,
    volume: u16,
    speed: i32,
    pub gapless: bool,
    pub message_tx: Sender<PlayerMsg>,
    command_tx: Sender<PlayerCmd>,
}

#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
impl Player {
    #[allow(clippy::too_many_lines)]
    pub fn new(config: &Settings, tx: Sender<PlayerMsg>) -> Self {
        let (command_tx, command_rx): (Sender<PlayerCmd>, Receiver<PlayerCmd>) = mpsc::channel();
        let volume = config.volume.try_into().unwrap();
        let speed = config.speed;
        let gapless = config.gapless;
        let this = Self {
            total_duration: None,
            volume,
            speed,
            gapless,
            message_tx: tx.clone(),
            command_tx,
        };
        std::thread::spawn(move || {
            let message_tx = tx.clone();
            let mut total_duration: Option<Duration> = None;
            let (_stream, handle) = OutputStream::try_default().unwrap();
            let mut sink = Sink::try_new(&handle, gapless, tx).unwrap();
            let speed = speed as f32 / 10.0;
            sink.set_speed(speed);
            sink.set_volume(<f32 as From<u16>>::from(volume) / 100.0);
            loop {
                if let Ok(cmd) = command_rx.try_recv() {
                    match cmd {
                        PlayerCmd::Play(url, gapless) => match File::open(Path::new(&url)) {
                            Ok(file) => {
                                let mss = MediaSourceStream::new(
                                    Box::new(file) as Box<dyn MediaSource>,
                                    MediaSourceStreamOptions::default(),
                                );
                                match Symphonia::new(mss, gapless) {
                                    Ok(decoder) => {
                                        total_duration = decoder.total_duration();
                                        if let Some(t) = total_duration {
                                            message_tx.send(PlayerMsg::Duration(t.as_secs())).ok();
                                        }
                                        sink.append(decoder);
                                    }
                                    Err(e) => eprintln!("error is: {e:?}"),
                                }
                            }

                            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                                message_tx.send(PlayerMsg::CacheStart(url.clone())).ok();
                                if let Ok(cursor) = Self::cache_complete(&url) {
                                    message_tx.send(PlayerMsg::CacheEnd(url.clone())).ok();
                                    let mss = MediaSourceStream::new(
                                        Box::new(cursor) as Box<dyn MediaSource>,
                                        MediaSourceStreamOptions::default(),
                                    );

                                    match Symphonia::new(mss, gapless) {
                                        Ok(decoder) => {
                                            total_duration = decoder.total_duration();
                                            if let Some(t) = total_duration {
                                                message_tx
                                                    .send(PlayerMsg::DurationNext(t.as_secs()))
                                                    .ok();
                                            }
                                            sink.append(decoder);
                                        }
                                        Err(e) => eprintln!("error playing podcast is: {e:?}"),
                                    }
                                }

                                // let len = ureq::head(&url)
                                //     .call()
                                //     .unwrap()
                                //     .header("Content-Length")
                                //     .and_then(|s| s.parse::<u64>().ok())
                                //     .unwrap();
                                // let request = SeekableRequest::get(&url);
                                // let buffer = SeekableBufReader::new(request);
                                // let mss = MediaSourceStream::new(
                                //     Box::new(buffer) as Box<dyn MediaSource>,
                                //     MediaSourceStreamOptions::default(),
                                // );

                                // match Symphonia::new(mss, gapless) {
                                //     Ok(decoder) => {
                                //         total_duration = decoder.total_duration();
                                //         if let Some(t) = total_duration {
                                //             message_tx.send(PlayerMsg::Duration(t.as_secs())).ok();
                                //         }
                                //         sink.append(decoder);
                                //     }
                                //     Err(e) => eprintln!("error is: {e:?}"),
                                // }
                            }
                            Err(e) => {
                                eprintln!("error is now: {e:?}");
                            }
                        },
                        PlayerCmd::Pause => {
                            sink.pause();
                        }
                        PlayerCmd::QueueNext(url, gapless) => {
                            match File::open(Path::new(&url)) {
                                Ok(file) => {
                                    let mss = MediaSourceStream::new(
                                        Box::new(file) as Box<dyn MediaSource>,
                                        MediaSourceStreamOptions::default(),
                                    );
                                    match Symphonia::new(mss, gapless) {
                                        Ok(decoder) => {
                                            total_duration = decoder.total_duration();
                                            if let Some(t) = total_duration {
                                                message_tx
                                                    .send(PlayerMsg::DurationNext(t.as_secs()))
                                                    .ok();
                                            }
                                            sink.append(decoder);
                                        }
                                        Err(e) => eprintln!("error is: {e:?}"),
                                    }
                                }

                                Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                                    if let Ok(cursor) = Self::cache_complete(&url) {
                                        let mss = MediaSourceStream::new(
                                            Box::new(cursor) as Box<dyn MediaSource>,
                                            MediaSourceStreamOptions::default(),
                                        );

                                        match Symphonia::new(mss, gapless) {
                                            Ok(decoder) => {
                                                total_duration = decoder.total_duration();
                                                if let Some(t) = total_duration {
                                                    message_tx
                                                        .send(PlayerMsg::DurationNext(t.as_secs()))
                                                        .ok();
                                                }
                                                sink.append(decoder);
                                            }
                                            Err(e) => eprintln!("error is: {e:?}"),
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("error is now: {e:?}");
                                }
                            }
                            // duration
                        }
                        PlayerCmd::Resume => {
                            sink.play();
                        }
                        PlayerCmd::Speed(speed) => {
                            let speed = speed as f32 / 10.0;
                            sink.set_speed(speed);
                        }
                        PlayerCmd::Stop => {
                            sink = Sink::try_new(&handle, gapless, message_tx.clone()).unwrap();
                        }
                        PlayerCmd::Volume(volume) => {
                            sink.set_volume(volume as f32 / 100.0);
                        }
                        PlayerCmd::Skip => {
                            sink.skip_one();
                            if sink.is_paused() {
                                sink.play();
                            }
                        }
                        PlayerCmd::GetProgress => {
                            let position = sink.elapsed().as_secs() as i64;
                            let mut duration_i64 = 102;
                            if let Some(d) = total_duration {
                                duration_i64 = d.as_secs() as i64;
                            }
                            message_tx
                                .send(PlayerMsg::Progress(position, duration_i64))
                                .ok();
                        }
                        PlayerCmd::Seek(d_i64) => sink.seek(Duration::from_secs(d_i64 as u64)),
                        PlayerCmd::MessageOnEnd => {
                            sink.message_on_end();
                        }

                        PlayerCmd::SeekRelative(offset) => {
                            if offset.is_positive() {
                                let new_pos = sink.elapsed().as_secs() + offset as u64;
                                if let Some(d) = total_duration {
                                    if new_pos < d.as_secs() - offset as u64 {
                                        sink.seek(Duration::from_secs(new_pos));
                                    }
                                }
                            } else {
                                let new_pos = sink
                                    .elapsed()
                                    .as_secs()
                                    .saturating_sub(offset.unsigned_abs());
                                sink.seek(Duration::from_secs(new_pos));
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        });

        this
    }

    // fn cache(url: &str) -> Result<Cursor<Vec<u8>>> {
    //     let agent = ureq::AgentBuilder::new().build();
    //     let len = ureq::head("Content-Length")
    //         .call()?
    //         .header("Content-Length")
    //         .and_then(|s| s.parse::<usize>().ok())
    //         .unwrap();
    //     let res = agent
    //         .get(url)
    //         .set("Range", &format!("bytes=0-{}", 1_000_000))
    //         .call()?;

    //     let mut bytes: Vec<u8> = Vec::with_capacity(1_000_001);
    //     res.into_reader().read_to_end(&mut bytes)?;
    //     Ok(Cursor::new(bytes))
    // }

    fn cache_complete(url: &str) -> Result<Cursor<Vec<u8>>> {
        let agent = ureq::AgentBuilder::new().build();
        let res = agent.get(url).call()?;
        let len = res
            .header("Content-Length")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap();
        let mut bytes: Vec<u8> = Vec::with_capacity(len);
        res.into_reader().read_to_end(&mut bytes)?;
        Ok(Cursor::new(bytes))
    }
    pub fn enqueue(&mut self, item: &str) {
        self.command_tx
            .send(PlayerCmd::Play(item.to_string(), self.gapless))
            .ok();
    }

    pub fn enqueue_next(&mut self, item: &str) {
        self.command_tx
            .send(PlayerCmd::QueueNext(item.to_string(), self.gapless))
            .ok();
    }

    fn play(&mut self, current_item: &str) {
        self.enqueue(current_item);
        self.resume();
    }

    fn stop(&mut self) {
        self.command_tx.send(PlayerCmd::Stop).ok();
    }

    pub fn skip_one(&mut self) {
        self.command_tx.send(PlayerCmd::Skip).ok();
    }

    fn get_progress(&self) {
        self.command_tx.send(PlayerCmd::GetProgress).ok();
    }

    pub fn message_on_end(&self) {
        self.command_tx.send(PlayerCmd::MessageOnEnd).ok();
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

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_lossless
    )]
    fn set_volume(&mut self, volume: i32) {
        self.volume = volume.clamp(0, 100) as u16;
        self.command_tx
            .send(PlayerCmd::Volume(self.volume.into()))
            .ok();
    }

    fn pause(&mut self) {
        self.command_tx
            .send(PlayerCmd::Pause)
            .expect("error sending pause command.");
    }

    fn resume(&mut self) {
        self.command_tx.send(PlayerCmd::Resume).ok();
    }

    fn is_paused(&self) -> bool {
        // self.sink.is_paused()
        false
    }

    fn seek(&mut self, offset: i64) -> Result<()> {
        self.command_tx.send(PlayerCmd::SeekRelative(offset))?;
        Ok(())
    }

    #[allow(clippy::cast_possible_wrap)]
    fn seek_to(&mut self, time: Duration) {
        let time_i64 = time.as_secs() as i64;
        self.command_tx.send(PlayerCmd::Seek(time_i64)).ok();
        self.get_progress();
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

    fn set_speed(&mut self, speed: i32) {
        self.speed = speed;
        self.command_tx.send(PlayerCmd::Speed(speed)).ok();
    }

    fn speed(&self) -> i32 {
        self.speed
    }
    fn stop(&mut self) {
        self.stop();
    }
}
