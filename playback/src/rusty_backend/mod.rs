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

use crate::audio_cmd;

use super::PlayerCmd;
use super::{PlayerMsg, PlayerTrait};
use anyhow::Result;
use std::path::Path;
// use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs::File, io::Cursor};
use symphonia::core::io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions};
use termusiclib::config::Settings;

static VOLUME_STEP: u16 = 5;

pub struct Player {
    pub total_duration: Arc<Mutex<Duration>>,
    volume: u16,
    speed: i32,
    pub gapless: bool,
    pub message_tx: Sender<PlayerMsg>,
    command_tx: Sender<PlayerCmd>,
    pub position: Arc<Mutex<i64>>,
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
        let command_tx_inside = command_tx.clone();
        let volume = config.volume.try_into().unwrap();
        let speed = config.speed;
        let gapless = config.gapless;
        let position = Arc::new(Mutex::new(0_i64));
        let total_duration = Arc::new(Mutex::new(Duration::from_secs(0)));
        let total_duration_local = total_duration.clone();
        let position_local = position.clone();
        let this = Self {
            total_duration,
            volume,
            speed,
            gapless,
            message_tx: tx.clone(),
            command_tx,
            position,
        };
        std::thread::spawn(move || {
            let message_tx = tx.clone();
            let mut total_duration: Option<Duration> = None;
            let (_stream, handle) = OutputStream::try_default().unwrap();
            let mut sink = Sink::try_new(&handle, command_tx_inside.clone()).unwrap();
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
                                            let mut d = total_duration_local
                                                .lock()
                                                .expect("error lock duration_local");
                                            *d = t;
                                            // message_tx.send(PlayerMsg::Duration(t.as_secs())).ok();
                                        }
                                        sink.append(decoder);
                                    }
                                    Err(e) => eprintln!("error is: {e:?}"),
                                }
                            }

                            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                                message_tx.send(PlayerMsg::CacheStart(url.clone())).ok();

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
                        PlayerCmd::TogglePause => {
                            sink.toggle_playback();
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
                            sink = Sink::try_new(&handle, command_tx_inside.clone()).unwrap();
                            sink.set_volume(<f32 as From<u16>>::from(volume) / 100.0);
                            sink.set_speed(speed);
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
                        PlayerCmd::Progress(position) => {
                            // let position = sink.elapsed().as_secs() as i64;
                            // eprintln!("position in rusty backend is: {}", position);
                            let mut p = position_local.lock().expect("error lock position_local");
                            *p = position;

                            if let Some(d) = total_duration {
                                let progress = position as f64 / d.as_secs_f64();
                                if progress >= 0.5 && (d.as_secs() - position as u64) < 2 && gapless
                                {
                                    audio_cmd::<()>(PlayerCmd::AboutToFinish, false).ok();
                                }
                            }

                            // About to finish signal is a simulation of gstreamer, and used for gapless
                            // #[cfg(any(not(feature = "gst"), feature = "mpv"))]
                            // if !self.player.playlist.is_empty()
                            //     && !self.player.playlist.has_next_track()
                            //     && new_prog >= 0.5
                            //     && duration - time_pos < 2
                            //     && self.config.gapless
                            // {
                            //     // eprintln!("about to finish sent");
                            //     self.player
                            //         .message_tx
                            //         .send(termusicplayback::PlayerMsg::AboutToFinish)
                            //         .ok();
                            // }

                            // let mut duration_i64 = 102;
                            // if let Some(d) = total_duration {
                            //     duration_i64 = d.as_secs() as i64;
                            // }
                            // *d = total_duration;
                            // message_tx
                            //     .send(PlayerMsg::Progress(position, duration_i64))
                            //     .ok();
                        }
                        PlayerCmd::Seek(d_i64) => sink.seek(Duration::from_secs(d_i64 as u64)),
                        PlayerCmd::MessageOnEnd => {
                            sink.message_on_end();
                        }

                        PlayerCmd::SeekRelative(offset) => {
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
                        PlayerCmd::ProcessID => {
                            let _id = std::process::id() as usize;
                            // send_val(&mut out_stream, &id);
                        }
                        _ => {}
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
            .send(PlayerCmd::TogglePause)
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
