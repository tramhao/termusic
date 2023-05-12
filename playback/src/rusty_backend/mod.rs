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
use anyhow::Result;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::time::Duration;
use std::{fs::File, io::Cursor};
use symphonia::core::io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions};
use termusiclib::config::Settings;

static VOLUME_STEP: u16 = 5;

pub struct Player {
    pub total_duration: Option<Duration>,
    volume: u16,
    speed: i32,
    pub gapless: bool,
    pub message_tx: Sender<PlayerMsg>,
    sink: Sink,
    handle: OutputStreamHandle,
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
        let volume = config.volume.try_into().unwrap();
        let speed = config.speed;
        let gapless = config.gapless;
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle, gapless, tx.clone()).unwrap();
        sink.set_volume(<f32 as From<u16>>::from(volume) / 100.0);
        sink.set_speed(speed as f32 / 10.0);
        Self {
            total_duration: None,
            volume,
            speed,
            gapless,
            message_tx: tx.clone(),
            sink,
            handle,
        }
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
        // self.command_tx
        //     .send(PlayerCmd::Play(item.to_string(), self.gapless))
        //     .ok();

        match File::open(Path::new(item)) {
            Ok(file) => {
                let mss = MediaSourceStream::new(
                    Box::new(file) as Box<dyn MediaSource>,
                    MediaSourceStreamOptions::default(),
                );
                match Symphonia::new(mss, self.gapless) {
                    Ok(decoder) => {
                        self.total_duration = decoder.total_duration();
                        // if let Some(t) = total_duration {
                        //     message_tx.send(PlayerMsg::Duration(t.as_secs())).ok();
                        // }
                        self.sink.append(decoder);
                    }
                    Err(e) => eprintln!("error is: {e:?}"),
                }
            }

            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                self.message_tx
                    .send(PlayerMsg::CacheStart(item.to_string()))
                    .ok();

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
                if let Ok(cursor) = Self::cache_complete(item) {
                    self.message_tx
                        .send(PlayerMsg::CacheEnd(item.to_string()))
                        .ok();
                    let mss = MediaSourceStream::new(
                        Box::new(cursor) as Box<dyn MediaSource>,
                        MediaSourceStreamOptions::default(),
                    );

                    match Symphonia::new(mss, self.gapless) {
                        Ok(decoder) => {
                            self.total_duration = decoder.total_duration();
                            // if let Some(t) = total_duration {
                            //     message_tx.send(PlayerMsg::DurationNext(t.as_secs())).ok();
                            // }
                            self.sink.append(decoder);
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
        };
    }

    pub fn enqueue_next(&mut self, item: &str) {
        match File::open(Path::new(item)) {
            Ok(file) => {
                let mss = MediaSourceStream::new(
                    Box::new(file) as Box<dyn MediaSource>,
                    MediaSourceStreamOptions::default(),
                );
                match Symphonia::new(mss, self.gapless) {
                    Ok(decoder) => {
                        self.total_duration = decoder.total_duration();
                        self.sink.append(decoder);
                    }
                    Err(e) => eprintln!("error is: {e:?}"),
                }
            }

            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                if let Ok(cursor) = Self::cache_complete(item) {
                    let mss = MediaSourceStream::new(
                        Box::new(cursor) as Box<dyn MediaSource>,
                        MediaSourceStreamOptions::default(),
                    );

                    match Symphonia::new(mss, self.gapless) {
                        Ok(decoder) => {
                            self.total_duration = decoder.total_duration();
                            self.sink.append(decoder);
                        }
                        Err(e) => eprintln!("error is: {e:?}"),
                    }
                }
            }
            Err(e) => {
                eprintln!("error is now: {e:?}");
            }
        }
    }

    fn play(&mut self, current_item: &str) {
        self.enqueue(current_item);
        self.resume();
    }

    fn stop(&mut self) {
        self.sink = Sink::try_new(&self.handle, self.gapless, self.message_tx.clone()).unwrap();
    }

    pub fn skip_one(&mut self) {
        self.sink.skip_one();
        if self.sink.is_paused() {
            self.sink.play();
        }
    }

    pub fn get_progress(&self) {
        let position = self.sink.elapsed().as_secs() as i64;
        let mut duration_i64 = 102;
        if let Some(d) = self.total_duration {
            duration_i64 = d.as_secs() as i64;
        }
        self.message_tx
            .send(PlayerMsg::Progress(position, duration_i64))
            .ok();
    }

    pub fn message_on_end(&self) {
        self.sink.message_on_end();
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
        self.sink.set_volume(self.volume as f32 / 100.0);
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

    fn seek(&mut self, offset: i64) -> Result<()> {
        if offset.is_positive() {
            let new_pos = self.sink.elapsed().as_secs() + offset as u64;
            if let Some(d) = self.total_duration {
                if new_pos < d.as_secs() - offset as u64 {
                    self.sink.seek(Duration::from_secs(new_pos));
                }
            }
        } else {
            let new_pos = self
                .sink
                .elapsed()
                .as_secs()
                .saturating_sub(offset.unsigned_abs());
            self.sink.seek(Duration::from_secs(new_pos));
        }
        Ok(())
    }

    #[allow(clippy::cast_possible_wrap)]
    fn seek_to(&mut self, time: Duration) {
        let time_i64 = time.as_secs() as i64;

        self.sink.seek(Duration::from_secs(time_i64 as u64));
        // self.get_progress();
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
        self.sink.set_speed(speed as f32 / 10.0);
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
