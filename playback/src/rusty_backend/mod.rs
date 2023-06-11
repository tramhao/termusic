#![cfg_attr(test, deny(missing_docs))]

mod conversions;
#[allow(unused)]
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

use super::PlayerCmd;
use super::PlayerTrait;
use anyhow::Result;
use std::path::Path;
// use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs::File, io::Cursor};
use symphonia::core::io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions};
use termusiclib::config::Settings;
use tokio::sync::mpsc::UnboundedSender;

static VOLUME_STEP: u16 = 5;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub enum PlayerInternalCmd {
    MessageOnEnd,
    Play(String, bool),
    Progress(i64),
    QueueNext(String, bool),
    Resume,
    Seek(i64),
    SeekRelative(i64),
    Skip,
    Speed(i32),
    Stop,
    TogglePause,
    Volume(i64),
}
pub struct Player {
    pub total_duration: Arc<Mutex<Duration>>,
    volume: u16,
    speed: i32,
    pub gapless: bool,
    command_tx: Sender<PlayerInternalCmd>,
    pub position: Arc<Mutex<i64>>,
    // cmd_tx_outside: Arc<Mutex<UnboundedSender<PlayerCmd>>>,
}

#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
impl Player {
    #[allow(clippy::too_many_lines)]
    pub fn new(config: &Settings, cmd_tx: Arc<Mutex<UnboundedSender<PlayerCmd>>>) -> Self {
        let (command_tx, command_rx): (Sender<PlayerInternalCmd>, Receiver<PlayerInternalCmd>) =
            mpsc::channel();
        let command_tx_inside = command_tx.clone();
        let volume = config.player_volume.try_into().unwrap();
        let speed = config.player_speed;
        let gapless = config.player_gapless;
        let position = Arc::new(Mutex::new(0_i64));
        let total_duration = Arc::new(Mutex::new(Duration::from_secs(0)));
        let total_duration_local = total_duration.clone();
        let position_local = position.clone();
        let cmd_tx_inside = cmd_tx;
        let this = Self {
            total_duration,
            volume,
            speed,
            gapless,
            command_tx,
            position,
            // cmd_tx_outside: cmd_tx,
        };
        std::thread::spawn(move || {
            let mut total_duration: Option<Duration> = None;
            let (_stream, handle) = OutputStream::try_default().unwrap();
            let mut sink =
                Sink::try_new(&handle, command_tx_inside.clone(), cmd_tx_inside.clone()).unwrap();
            let speed = speed as f32 / 10.0;
            sink.set_speed(speed);
            sink.set_volume(<f32 as From<u16>>::from(volume) / 100.0);
            loop {
                if let Ok(cmd) = command_rx.try_recv() {
                    match cmd {
                        PlayerInternalCmd::Play(url, gapless) => {
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
                                                let mut d = total_duration_local
                                                    .lock()
                                                    .expect("error lock duration_local");
                                                *d = t;
                                            }
                                            sink.append(decoder);
                                        }
                                        Err(e) => eprintln!("error is: {e:?}"),
                                    }
                                }

                                Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                                    // message_tx.send(PlayerMsg::CacheStart(url.clone())).ok();

                                    // // Create an HTTP client and request the URL
                                    // let rt = tokio::runtime::Runtime::new().unwrap();
                                    // rt.block_on(async {
                                    //     let client = reqwest::Client::new();
                                    //     let mut response = client.get(&url).send().await.unwrap();

                                    //     // Create a buffer to store the streamed data
                                    //     let mut buffer = Vec::new();

                                    //     // Stream the data into the buffer
                                    //     while let Some(chunk) = response.chunk().await.unwrap() {
                                    //         buffer.extend_from_slice(&chunk);
                                    //     }
                                    //     let cursor = Cursor::new(buffer);

                                    //     let mss = MediaSourceStream::new(
                                    //         Box::new(cursor) as Box<dyn MediaSource>,
                                    //         MediaSourceStreamOptions::default(),
                                    //     );

                                    //     match Symphonia::new(mss, gapless) {
                                    //         Ok(decoder) => {
                                    //             total_duration = decoder.total_duration();
                                    //             if let Some(t) = total_duration {
                                    //                 message_tx
                                    //                     .send(PlayerMsg::DurationNext(t.as_secs()))
                                    //                     .ok();
                                    //             }
                                    //             sink.append(decoder);
                                    //         }
                                    //         Err(e) => eprintln!("error playing podcast is: {e:?}"),
                                    //     }
                                    // });
                                    if let Ok(cursor) = Self::cache_complete(&url) {
                                        let mss = MediaSourceStream::new(
                                            Box::new(cursor) as Box<dyn MediaSource>,
                                            MediaSourceStreamOptions::default(),
                                        );

                                        match Symphonia::new(mss, gapless) {
                                            Ok(decoder) => {
                                                total_duration = decoder.total_duration();

                                                if let Some(t) = total_duration {
                                                    let mut d = total_duration_local
                                                        .lock()
                                                        .expect("error lock duration_local");
                                                    *d = t;
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
                            }
                        }
                        PlayerInternalCmd::TogglePause => {
                            sink.toggle_playback();
                        }
                        PlayerInternalCmd::QueueNext(url, gapless) => {
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
                                                if let Ok(tx) = cmd_tx_inside.lock() {
                                                    if let Err(e) = tx
                                                        .send(PlayerCmd::DurationNext(t.as_secs()))
                                                    {
                                                        error!(
                                                            "command durationnext sent failed: {e}"
                                                        );
                                                    }
                                                }
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
                                                    if let Ok(tx) = cmd_tx_inside.lock() {
                                                        if let Err(e) = tx.send(
                                                            PlayerCmd::DurationNext(t.as_secs()),
                                                        ) {
                                                            error!(
                                                            "command durationnext sent failed: {e}"
                                                        );
                                                        }
                                                    }
                                                    sink.append(decoder);
                                                }
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
                        PlayerInternalCmd::Resume => {
                            sink.play();
                        }
                        PlayerInternalCmd::Speed(speed) => {
                            let speed = speed as f32 / 10.0;
                            sink.set_speed(speed);
                        }
                        PlayerInternalCmd::Stop => {
                            sink = Sink::try_new(
                                &handle,
                                command_tx_inside.clone(),
                                cmd_tx_inside.clone(),
                            )
                            .unwrap();
                            sink.set_volume(<f32 as From<u16>>::from(volume) / 100.0);
                            sink.set_speed(speed);
                        }
                        PlayerInternalCmd::Volume(volume) => {
                            sink.set_volume(volume as f32 / 100.0);
                        }
                        PlayerInternalCmd::Skip => {
                            sink.skip_one();
                            if sink.is_paused() {
                                sink.play();
                            }
                        }
                        PlayerInternalCmd::Progress(position) => {
                            // let position = sink.elapsed().as_secs() as i64;
                            // eprintln!("position in rusty backend is: {}", position);
                            let mut p = position_local.lock().expect("error lock position_local");
                            *p = position;

                            // About to finish signal is a simulation of gstreamer, and used for gapless
                            if let Some(d) = total_duration {
                                let progress = position as f64 / d.as_secs_f64();
                                if progress >= 0.5 && (d.as_secs() - position as u64) < 2 {
                                    if let Ok(tx) = cmd_tx_inside.lock() {
                                        if let Err(e) = tx.send(PlayerCmd::AboutToFinish) {
                                            error!("command AboutToFinish sent failed: {e}");
                                        }
                                    }
                                }
                            }
                        }
                        PlayerInternalCmd::Seek(d_i64) => {
                            sink.seek(Duration::from_secs(d_i64 as u64));
                        }
                        PlayerInternalCmd::MessageOnEnd => {
                            sink.message_on_end();
                        }

                        PlayerInternalCmd::SeekRelative(offset) => {
                            let paused = sink.is_paused();
                            if paused {
                                sink.set_volume(0.0);
                            }
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
                            if paused {
                                std::thread::sleep(std::time::Duration::from_millis(50));
                                sink.pause();
                                sink.set_volume(<f32 as From<u16>>::from(volume) / 100.0);
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
            .send(PlayerInternalCmd::Play(item.to_string(), self.gapless))
            .ok();
    }

    pub fn enqueue_next(&mut self, item: &str) {
        self.command_tx
            .send(PlayerInternalCmd::QueueNext(item.to_string(), self.gapless))
            .ok();
    }

    fn play(&mut self, current_item: &str) {
        self.enqueue(current_item);
        self.resume();
    }

    fn stop(&mut self) {
        self.command_tx.send(PlayerInternalCmd::Stop).ok();
    }

    pub fn skip_one(&mut self) {
        self.command_tx.send(PlayerInternalCmd::Skip).ok();
    }

    pub fn message_on_end(&self) {
        self.command_tx.send(PlayerInternalCmd::MessageOnEnd).ok();
    }

    // fn command(&self, cmd: &PlayerCmd) {
    //     if let Ok(tx) = self.cmd_tx_outside.lock() {
    //         if let Err(e) = tx.send(cmd.clone()) {
    //             error!("command {cmd:?} sent failed: {e}");
    //         }
    //     }
    // }
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
            .send(PlayerInternalCmd::Volume(self.volume.into()))
            .ok();
    }

    fn pause(&mut self) {
        self.command_tx
            .send(PlayerInternalCmd::TogglePause)
            .expect("error sending pause command.");
    }

    fn resume(&mut self) {
        self.command_tx.send(PlayerInternalCmd::Resume).ok();
    }

    fn is_paused(&self) -> bool {
        // self.sink.is_paused()
        false
    }

    fn seek(&mut self, offset: i64) -> Result<()> {
        self.command_tx
            .send(PlayerInternalCmd::SeekRelative(offset))?;
        Ok(())
    }

    #[allow(clippy::cast_possible_wrap)]
    fn seek_to(&mut self, time: Duration) {
        let time_i64 = time.as_secs() as i64;
        self.command_tx.send(PlayerInternalCmd::Seek(time_i64)).ok();
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
        self.command_tx.send(PlayerInternalCmd::Speed(speed)).ok();
    }

    fn speed(&self) -> i32 {
        self.speed
    }
    fn stop(&mut self) {
        self.stop();
    }

    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_wrap)]
    fn get_progress(&self) -> Result<(i64, i64)> {
        let time_pos = self.position.lock().unwrap();
        let duration = self.total_duration.lock().unwrap();
        let d_i64 = duration.as_secs() as i64;
        Ok((*time_pos, d_i64))
    }
}

// how to write a music player in rust with rodio and symphonia, but can play online stream music while downloading.To create a music player in Rust that can play online stream music while downloading, you can use the `rodio` crate for audio playback and the `symphonia` crate for decoding. Here's a step-by-step guide:

// 1. Add dependencies to `Cargo.toml`:

// ```toml
// [dependencies]
// rodio = "0.13"
// symphonia = { version = "0.3", features = ["default", "mp3", "flac"] }
// reqwest = { version = "0.11", features = ["stream"] }
// tokio = { version = "1", features = ["full"] }
// ```

// 2. Create a `main.rs` file with the following code:

// ```rust
// use rodio::{Decoder, OutputStream, Sink};
// use std::io::Cursor;
// use std::sync::Arc;
// use symphonia::core::codecs::DecoderOptions;
// use symphonia::core::io::BufReader;
// use symphonia::core::meta::MetadataOptions;
// use symphonia::default::get_probe;
// use tokio::io::AsyncReadExt;

// async fn stream_music(url: &str) -> Result<(), Box<dyn std::error::Error>> {
//     // Create an HTTP client and request the URL
//     let client = reqwest::Client::new();
//     let mut response = client.get(url).send().await?;

//     // Create a buffer to store the streamed data
//     let mut buffer = Vec::new();

//     // Stream the data into the buffer
//     while let Some(chunk) = response.chunk().await? {
//         buffer.extend_from_slice(&chunk);
//     }

//     // Create a Symphonia decoder with the downloaded data
//     let options = DecoderOptions::default();
//     let metadata_options = MetadataOptions::default();
//     let probe = get_probe();
//     let reader = BufReader::new(Cursor::new(buffer));
//     let probed = probe
//         .format(reader, &metadata_options)
//         .expect("Failed to probe format");
//     let mut symphonia_decoder = probed
//         .format
//         .make_decoder(probed.stream, options)
//         .expect("Failed to create decoder");

//     // Create a Rodio output stream and sink
//     let (_stream, stream_handle) = OutputStream::try_default()?;
//     let sink = Sink::try_new(&stream_handle)?;

//     // Decode and play the audio
//     let rodio_decoder = Decoder::new_raw(Arc::new(sink), symphonia_decoder)?;
//     sink.append(rodio_decoder);

//     // Wait for the audio to finish playing
//     sink.sleep_until_end();

//     Ok(())
// }

// #[tokio::main]
// async fn main() {
//     let url = "https://example.com/path/to/audio.mp3";
//     match stream_music(url).await {
//         Ok(_) => println!("Finished playing."),
//         Err(e) => eprintln!("Error: {}", e),
//     }
// }
// ```

// 3. Replace `https://example.com/path/to/audio.mp3` with the URL of the audio file you want to stream.

// 4. Run the program using `cargo run`. The audio will be streamed and played while it's being downloaded.

// This example uses `reqwest` for HTTP streaming and `tokio` for async I/O. You can adapt the code for other audio formats by adding more features to the `symphonia` dependency in `Cargo.toml`.
