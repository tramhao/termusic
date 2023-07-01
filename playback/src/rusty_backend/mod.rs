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

use async_trait::async_trait;
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
use parking_lot::Mutex;
use std::io::Read;
use std::str::FromStr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;
use std::{fs::File, io::Cursor};
use symphonia::core::io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions};
use termusic_stream::StreamDownload;
use termusiclib::config::Settings;
use termusiclib::track::{MediaType, Track};
use tokio::sync::mpsc::UnboundedSender;

static VOLUME_STEP: u16 = 5;

// #[allow(clippy::module_name_repetitions)]
// #[derive(Clone)]
#[allow(unused)]
#[derive(Clone, Debug)]
pub enum PlayerInternalCmd {
    MessageOnEnd,
    Play(Box<Track>, bool),
    // PlayLocal(Box<File>, bool),
    // PlayPod(Box<dyn MediaSource>, bool, Duration),
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
        let mut volume_inside = volume;
        let mut speed_inside = speed;
        std::thread::spawn(move || {
            let mut total_duration: Option<Duration> = None;
            let (_stream, handle) = OutputStream::try_default().unwrap();
            let mut sink =
                Sink::try_new(&handle, command_tx_inside.clone(), cmd_tx_inside.clone()).unwrap();
            sink.set_speed(speed_inside as f32 / 10.0);
            sink.set_volume(<f32 as From<u16>>::from(volume_inside) / 100.0);
            loop {
                if let Ok(cmd) = command_rx.try_recv() {
                    match cmd {
                        // PlayerInternalCmd::PlayPod(stream, gapless, duration) => {
                        //     let mss = MediaSourceStream::new(
                        //         stream as Box<dyn MediaSource>,
                        //         MediaSourceStreamOptions::default(),
                        //     );

                        //     match Symphonia::new(mss, gapless) {
                        //         Ok(decoder) => {
                        //             total_duration = Some(duration);
                        //             // total_duration = decoder.total_duration();

                        //             if let Some(t) = total_duration {
                        //                 let mut d = total_duration_local.lock();
                        //                 *d = t;
                        //             }
                        //             sink.append(decoder);
                        //         }
                        //         Err(e) => {
                        //             error!("error playing podcast is: {e:?}");
                        //         }
                        //     }
                        // }
                        // PlayerInternalCmd::PlayLocal(file, gapless) => {
                        //     let mss = MediaSourceStream::new(
                        //         file as Box<dyn MediaSource>,
                        //         MediaSourceStreamOptions::default(),
                        //     );
                        //     match Symphonia::new(mss, gapless) {
                        //         Ok(decoder) => {
                        //             total_duration = decoder.total_duration();
                        //             if let Some(t) = total_duration {
                        //                 let mut d = total_duration_local.lock();
                        //                 *d = t;
                        //             }
                        //             sink.append(decoder);
                        //         }
                        //         Err(e) => eprintln!("error is: {e:?}"),
                        //     }
                        // }
                        PlayerInternalCmd::Play(track, gapless) => {
                            match track.media_type {
                                Some(MediaType::Music) => {
                                    if let Some(file) = track.file() {
                                        match File::open(Path::new(file)) {
                                            Ok(file) => {
                                                let mss = MediaSourceStream::new(
                                                    Box::new(file) as Box<dyn MediaSource>,
                                                    MediaSourceStreamOptions::default(),
                                                );
                                                match Symphonia::new(mss, gapless) {
                                                    Ok(decoder) => {
                                                        total_duration = decoder.total_duration();
                                                        if let Some(t) = total_duration {
                                                            let mut d = total_duration_local.lock();
                                                            *d = t;
                                                        }
                                                        sink.append(decoder);
                                                    }
                                                    Err(e) => eprintln!("error is: {e:?}"),
                                                }
                                            }
                                            Err(e) => error!("error open file: {e}"),
                                        }
                                    }
                                }
                                Some(MediaType::Podcast) => {
                                    if let Some(url) = track.file() {
                                        let reader =
                                            StreamDownload::new_http(url.parse().unwrap()).unwrap();

                                        let mss = MediaSourceStream::new(
                                            Box::new(reader) as Box<dyn MediaSource>,
                                            MediaSourceStreamOptions::default(),
                                        );

                                        match Symphonia::new(mss, gapless) {
                                            Ok(decoder) => {
                                                // total_duration = Some(track.duration());
                                                total_duration = decoder.total_duration();

                                                if let Some(t) = total_duration {
                                                    let mut d = total_duration_local.lock();
                                                    *d = t;
                                                }
                                                sink.append(decoder);
                                            }
                                            Err(e) => {
                                                error!("error playing podcast is: {e:?}");
                                            }
                                        }
                                        // }
                                    }
                                }

                                Some(MediaType::LiveRadio) => {
                                    if let Some(url) = track.file() {
                                        let reader =
                                            StreamDownload::new_http(url.parse().unwrap()).unwrap();

                                        let mss = MediaSourceStream::new(
                                            Box::new(reader) as Box<dyn MediaSource>,
                                            MediaSourceStreamOptions::default(),
                                        );

                                        match Symphonia::new(mss, gapless) {
                                            Ok(decoder) => {
                                                // total_duration = Some(track.duration());
                                                total_duration = decoder.total_duration();

                                                if let Some(t) = total_duration {
                                                    let mut d = total_duration_local.lock();
                                                    *d = t;
                                                }
                                                sink.append(decoder);
                                            }
                                            Err(e) => {
                                                error!("error playing live radio: {e:?}");
                                            }
                                        }
                                        // }
                                    }
                                }
                                None => {}
                            }

                            // match File::open(Path::new(&track.file())) {
                            //     Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                            //         // message_tx.send(PlayerMsg::CacheStart(url.clone())).ok();

                            //         // // Create an HTTP client and request the URL
                            //         // let rt = tokio::runtime::Runtime::new().unwrap();
                            //         // rt.block_on(async {
                            //         //     let client = reqwest::Client::new();
                            //         //     let mut response = client.get(&url).send().await.unwrap();

                            //         //     // Create a buffer to store the streamed data
                            //         //     let mut buffer = Vec::new();

                            //         //     // Stream the data into the buffer
                            //         //     while let Some(chunk) = response.chunk().await.unwrap() {
                            //         //         buffer.extend_from_slice(&chunk);
                            //         //     }
                            //         //     let cursor = Cursor::new(buffer);

                            //         //     let mss = MediaSourceStream::new(
                            //         //         Box::new(cursor) as Box<dyn MediaSource>,
                            //         //         MediaSourceStreamOptions::default(),
                            //         //     );

                            //         //     match Symphonia::new(mss, gapless) {
                            //         //         Ok(decoder) => {
                            //         //             total_duration = decoder.total_duration();
                            //         //             if let Some(t) = total_duration {
                            //         //                 message_tx
                            //         //                     .send(PlayerMsg::DurationNext(t.as_secs()))
                            //         //                     .ok();
                            //         //             }
                            //         //             sink.append(decoder);
                            //         //         }
                            //         //         Err(e) => eprintln!("error playing podcast is: {e:?}"),
                            //         //     }
                            //         // });
                            //         // let len = ureq::head(&url)
                            //         //     .call()
                            //         //     .unwrap()
                            //         //     .header("Content-Length")
                            //         //     .and_then(|s| s.parse::<u64>().ok())
                            //         //     .unwrap();
                            //         // let request = SeekableRequest::get(&url);
                            //         // let buffer = SeekableBufReader::new(request);
                            //         // let mss = MediaSourceStream::new(
                            //         //     Box::new(buffer) as Box<dyn MediaSource>,
                            //         //     MediaSourceStreamOptions::default(),
                            //         // );

                            //         // match Symphonia::new(mss, gapless) {
                            //         //     Ok(decoder) => {
                            //         //         total_duration = decoder.total_duration();
                            //         //         if let Some(t) = total_duration {
                            //         //             message_tx.send(PlayerMsg::Duration(t.as_secs())).ok();
                            //         //         }
                            //         //         sink.append(decoder);
                            //         //     }
                            //         //     Err(e) => eprintln!("error is: {e:?}"),
                            //         // }
                            //     }
                            //     Err(e) => {
                            //         eprintln!("error is now: {e:?}");
                            //     }
                            // }
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
                                                let tx = cmd_tx_inside.lock();
                                                if let Err(e) =
                                                    tx.send(PlayerCmd::DurationNext(t.as_secs()))
                                                {
                                                    error!("command durationnext sent failed: {e}");
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
                                                    let tx = cmd_tx_inside.lock();
                                                    if let Err(e) = tx
                                                        .send(PlayerCmd::DurationNext(t.as_secs()))
                                                    {
                                                        error!(
                                                            "command durationnext sent failed: {e}"
                                                        );
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
                            speed_inside = speed;
                            sink.set_speed(speed_inside as f32 / 10.0);
                        }
                        PlayerInternalCmd::Stop => {
                            sink = Sink::try_new(
                                &handle,
                                command_tx_inside.clone(),
                                cmd_tx_inside.clone(),
                            )
                            .unwrap();
                            sink.set_speed(speed_inside as f32 / 10.0);
                            sink.set_volume(<f32 as From<u16>>::from(volume_inside) / 100.0);
                        }
                        PlayerInternalCmd::Volume(volume) => {
                            sink.set_volume(volume as f32 / 100.0);
                            volume_inside = volume as u16;
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
                            let mut p = position_local.lock();
                            *p = position;

                            // About to finish signal is a simulation of gstreamer, and used for gapless
                            if let Some(d) = total_duration {
                                let progress = position as f64 / d.as_secs_f64();
                                if progress >= 0.5 && (d.as_secs() - position as u64) < 2 {
                                    let tx = cmd_tx_inside.lock();
                                    if let Err(e) = tx.send(PlayerCmd::AboutToFinish) {
                                        error!("command AboutToFinish sent failed: {e}");
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
                                sink.set_volume(<f32 as From<u16>>::from(volume_inside) / 100.0);
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        });

        this
    }

    #[allow(clippy::needless_pass_by_value)]
    fn command(&self, cmd: PlayerInternalCmd) {
        if let Err(e) = self.command_tx.send(cmd.clone()) {
            error!("error in {cmd:?}: {e}");
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
        let agent = reqwest::blocking::ClientBuilder::new()
            .build()
            .expect("build client error.");
        let mut res = agent.get(url).send()?;
        let mut len = 99;
        // let len = res
        //     .headers()
        //     .get("Content-Length")
        //     .and_then(|s| s.parse::<usize>().ok())
        //     .unwrap();

        if let Some(length) = res.headers().get(reqwest::header::CONTENT_LENGTH) {
            let length = u64::from_str(length.to_str().map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            })?)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
            info!("Got content length {length}");
            len = length;
        } else {
            warn!("Content length header missing");
        }
        let mut bytes: Vec<u8> = Vec::with_capacity(len as usize);
        res.read_to_end(&mut bytes)?;
        Ok(Cursor::new(bytes))
    }

    #[allow(clippy::unused_async)]
    pub async fn enqueue(&mut self, item: &Track) {
        self.command(PlayerInternalCmd::Play(
            Box::new(item.clone()),
            self.gapless,
        ));
        // match item.media_type {
        //     Some(MediaType::Music) => {
        //         if let Some(file) = item.file() {
        //             match File::open(Path::new(file)) {
        //                 Ok(file) => {
        //                     self.command_tx
        //                         .send(PlayerInternalCmd::PlayLocal(Box::new(file), self.gapless))
        //                         .ok();
        //                 }
        //                 Err(e) => error!("track file not found: {}", e),
        //             }
        //         }
        //     }
        //     Some(MediaType::Podcast) => {
        //         if let Some(url) = item.file() {
        //             let reader = StreamDownload::new_http(url.parse().unwrap()).unwrap();
        //             let duration = item.duration();
        //             self.command_tx
        //                 .send(PlayerInternalCmd::PlayPod(
        //                     Box::new(reader),
        //                     self.gapless,
        //                     duration,
        //                 ))
        //                 .ok();
        //         }
        //     }
        //     None => {}
        // }
    }

    pub fn enqueue_next(&mut self, item: &str) {
        self.command(PlayerInternalCmd::QueueNext(item.to_string(), self.gapless));
    }

    async fn play(&mut self, current_item: &Track) {
        self.enqueue(current_item).await;
        self.resume();
    }

    fn stop(&mut self) {
        self.command(PlayerInternalCmd::Stop);
    }

    pub fn skip_one(&mut self) {
        self.command(PlayerInternalCmd::Skip);
    }

    pub fn message_on_end(&self) {
        self.command(PlayerInternalCmd::MessageOnEnd);
    }

    // fn command(&self, cmd: &PlayerCmd) {
    //     if let Ok(tx) = self.cmd_tx_outside.lock() {
    //         if let Err(e) = tx.send(cmd.clone()) {
    //             error!("command {cmd:?} sent failed: {e}");
    //         }
    //     }
    // }
}

#[async_trait]
impl PlayerTrait for Player {
    async fn add_and_play(&mut self, current_track: &Track) {
        self.play(current_track).await;
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
        self.command(PlayerInternalCmd::Volume(self.volume.into()));
    }

    fn pause(&mut self) {
        self.command(PlayerInternalCmd::TogglePause);
    }

    fn resume(&mut self) {
        self.command(PlayerInternalCmd::Resume);
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
        self.command(PlayerInternalCmd::Seek(time_i64));
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
        self.command(PlayerInternalCmd::Speed(speed));
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
        let time_pos = self.position.lock();
        let duration = self.total_duration.lock();
        let d_i64 = duration.as_secs() as i64;
        Ok((*time_pos, d_i64))
    }
}
