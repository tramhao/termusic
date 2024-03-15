#![cfg_attr(test, deny(missing_docs))]

mod conversions;
#[allow(unused)]
mod sink;
mod stream;

pub mod buffer;
pub mod decoder;
pub mod dynamic_mixer;
pub mod queue;
pub mod source;

use async_trait::async_trait;
pub use conversions::Sample;
pub use cpal::{traits::StreamTrait, ChannelCount, SampleRate};
pub use decoder::Symphonia;
pub use sink::Sink;
pub use source::Source;
pub use stream::OutputStream;

use self::decoder::buffered_source::BufferedSource;

use super::{PlayerCmd, PlayerProgress, PlayerTrait};
use anyhow::Result;
use parking_lot::Mutex;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;
use std::{fs::File, io::Cursor};
use symphonia::core::io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions};
use termusic_stream::StreamDownload;
use termusiclib::config::Settings;
use termusiclib::track::{MediaType, Track};

static VOLUME_STEP: u16 = 5;

pub type TotalDuration = Option<Duration>;
pub type ArcTotalDuration = Arc<Mutex<TotalDuration>>;

#[allow(unused)]
#[derive(Clone, Debug)]
pub enum PlayerInternalCmd {
    MessageOnEnd,
    /// Enqueue a new track to be played, and skip to it
    /// (Track, gapless)
    Play(Box<Track>, bool),
    Progress(Duration),
    /// Enqueue a new track to be played, but do not skip current track
    /// (Track URI, gapless)
    QueueNext(String, bool),
    Resume,
    SeekAbsolute(Duration),
    SeekRelative(i64),
    Skip,
    Speed(i32),
    Stop,
    TogglePause,
    Volume(u16),
    Eos,
}
pub struct RustyBackend {
    volume: Arc<AtomicU16>,
    speed: i32,
    gapless: bool,
    command_tx: Sender<PlayerInternalCmd>,
    position: Arc<Mutex<Duration>>,
    total_duration: ArcTotalDuration,
    pub radio_title: Arc<Mutex<String>>,
    pub radio_downloaded: Arc<Mutex<u64>>,
    // cmd_tx_outside: crate::PlayerCmdSender,
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
impl RustyBackend {
    #[allow(clippy::similar_names)]
    #[allow(clippy::too_many_lines)]
    pub fn new(config: &Settings, cmd_tx: crate::PlayerCmdSender) -> Self {
        let (picmd_tx, picmd_rx): (Sender<PlayerInternalCmd>, Receiver<PlayerInternalCmd>) =
            mpsc::channel();
        let picmd_tx_local = picmd_tx.clone();
        let volume = Arc::new(AtomicU16::from(config.player_volume));
        let volume_local = volume.clone();
        let speed = config.player_speed;
        let gapless = config.player_gapless;
        let position = Arc::new(Mutex::new(Duration::default()));
        let total_duration = Arc::new(Mutex::new(None));
        let total_duration_local = total_duration.clone();
        let position_local = position.clone();
        let pcmd_tx_local = cmd_tx;
        let radio_title = Arc::new(Mutex::new(String::new()));
        let radio_title_local = radio_title.clone();
        let radio_downloaded = Arc::new(Mutex::new(100_u64));
        let radio_downloaded_local = radio_downloaded.clone();

        std::thread::Builder::new()
            .name("playback player loop".into())
            .spawn(move || {
                player_thread(
                    total_duration_local,
                    pcmd_tx_local,
                    picmd_tx_local,
                    picmd_rx,
                    radio_title_local,
                    radio_downloaded_local,
                    position_local,
                    volume_local,
                    speed,
                );
            })
            .expect("failed to spawn thread");

        Self {
            total_duration,
            volume,
            speed,
            gapless,
            command_tx: picmd_tx,
            position,
            radio_title,
            radio_downloaded,
            // cmd_tx_outside: cmd_tx,
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn command(&self, cmd: PlayerInternalCmd) {
        if let Err(e) = self.command_tx.send(cmd.clone()) {
            error!("error in {cmd:?}: {e}");
        }
    }

    fn cache_complete(url: &str) -> Result<Cursor<Vec<u8>>> {
        let agent = reqwest::blocking::ClientBuilder::new()
            .build()
            .expect("build client error.");
        let mut res = agent.get(url).send()?;
        let mut len = 99;

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

    fn enqueue(&mut self, item: &Track) {
        self.command(PlayerInternalCmd::Play(
            Box::new(item.clone()),
            self.gapless,
        ));
    }

    fn enqueue_next(&mut self, item: &str) {
        self.command(PlayerInternalCmd::QueueNext(item.to_string(), self.gapless));
    }

    fn play(&mut self, current_item: &Track) {
        self.enqueue(current_item);
        self.resume();
    }

    fn stop(&mut self) {
        self.command(PlayerInternalCmd::Stop);
    }

    fn skip_one(&mut self) {
        self.command(PlayerInternalCmd::Skip);
    }

    pub fn message_on_end(&self) {
        self.command(PlayerInternalCmd::MessageOnEnd);
    }
}

#[async_trait]
impl PlayerTrait for RustyBackend {
    async fn add_and_play(&mut self, current_track: &Track) {
        self.play(current_track);
    }

    fn volume(&self) -> u16 {
        self.volume.load(Ordering::SeqCst)
    }

    fn volume_up(&mut self) {
        let volume = self
            .volume
            .load(Ordering::SeqCst)
            .saturating_add(VOLUME_STEP);
        self.set_volume(volume);
    }

    fn volume_down(&mut self) {
        let volume = self
            .volume
            .load(Ordering::SeqCst)
            .saturating_sub(VOLUME_STEP);
        self.set_volume(volume);
    }

    fn set_volume(&mut self, volume: u16) {
        let volume = volume.min(100);
        self.volume.store(volume, Ordering::SeqCst);
        self.command(PlayerInternalCmd::Volume(volume));
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
        self.command(PlayerInternalCmd::SeekRelative(offset));
        Ok(())
    }

    #[allow(clippy::cast_possible_wrap)]
    fn seek_to(&mut self, position: Duration) {
        self.command(PlayerInternalCmd::SeekAbsolute(position));
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
    fn get_progress(&self) -> PlayerProgress {
        PlayerProgress {
            position: *self.position.lock(),
            total_duration: *self.total_duration.lock(),
        }
    }

    fn gapless(&self) -> bool {
        self.gapless
    }

    fn set_gapless(&mut self, to: bool) {
        self.gapless = to;
    }

    fn skip_one(&mut self) {
        self.skip_one();
    }

    fn enqueue_next(&mut self, file: &str) {
        self.enqueue_next(file);
    }
}

/// Append the `media_source` to the `sink`, while allowing different functions to run with `func`
fn append_to_sink_inner<F: FnOnce(&Symphonia)>(
    media_source: Box<dyn MediaSource>,
    trace: &str,
    sink: &Sink,
    gapless: bool,
    func: F,
) {
    let mss = MediaSourceStream::new(media_source, MediaSourceStreamOptions::default());
    match Symphonia::new(mss, gapless) {
        Ok(decoder) => {
            func(&decoder);
            sink.append(decoder);
        }
        Err(e) => error!("error decoding '{trace}' is: {e:?}"),
    }
}

/// Append the `media_source` to the `sink`, while also setting `total_duration*`
fn append_to_sink(
    media_source: Box<dyn MediaSource>,
    trace: &str,
    sink: &Sink,
    gapless: bool,
    total_duration_local: &ArcTotalDuration,
) {
    append_to_sink_inner(media_source, trace, sink, gapless, |decoder| {
        std::mem::swap(
            &mut *total_duration_local.lock(),
            &mut decoder.total_duration(),
        );
    });
}

/// Append the `media_source` to the `sink`, while setting duration to be unknown (to [`None`])
fn append_to_sink_no_duration(
    media_source: Box<dyn MediaSource>,
    trace: &str,
    sink: &Sink,
    gapless: bool,
    total_duration_local: &ArcTotalDuration,
) {
    append_to_sink_inner(media_source, trace, sink, gapless, |_| {
        // remove old stale duration
        total_duration_local.lock().take();
    });
}

/// Append the `media_source` to the `sink`, while also setting `total_duration_opt`
///
/// This is used for enqueued entries which do not start immediately
fn append_to_sink_queue(
    media_source: Box<dyn MediaSource>,
    trace: &str,
    sink: &Sink,
    gapless: bool,
    // total_duration_local: &ArcTotalDuration,
    next_duration_opt: &mut Option<Duration>,
) {
    append_to_sink_inner(media_source, trace, sink, gapless, |decoder| {
        std::mem::swap(next_duration_opt, &mut decoder.total_duration());
        // rely on EOS message to set next duration
        sink.message_on_end();
    });
}

/// Player thread loop
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_pass_by_value,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]
fn player_thread(
    total_duration: ArcTotalDuration,
    pcmd_tx: crate::PlayerCmdSender,
    picmd_tx: Sender<PlayerInternalCmd>,
    picmd_rx: Receiver<PlayerInternalCmd>,
    radio_title: Arc<Mutex<String>>,
    radio_downloaded: Arc<Mutex<u64>>,
    position: Arc<Mutex<Duration>>,
    volume_inside: Arc<AtomicU16>,
    mut speed_inside: i32,
) {
    let mut is_radio = false;

    // option to store enqueued's duration
    // note that the current implementation is only meant to have 1 enqueued next after the current playing song
    let mut next_duration_opt = None;
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let mut sink = Sink::try_new(&handle, picmd_tx.clone(), pcmd_tx.clone()).unwrap();
    sink.set_speed(speed_inside as f32 / 10.0);
    sink.set_volume(f32::from(volume_inside.load(Ordering::SeqCst)) / 100.0);
    loop {
        let cmd = match picmd_rx.recv_timeout(Duration::from_micros(100)) {
            Ok(v) => v,
            Err(RecvTimeoutError::Disconnected) => break,
            Err(_) => continue,
        };

        match cmd {
            PlayerInternalCmd::Play(track, gapless) => match track.media_type {
                Some(MediaType::Music) => {
                    is_radio = false;
                    if let Some(file_path) = track.file() {
                        match File::open(Path::new(file_path)) {
                            Ok(file) => append_to_sink(
                                Box::new(BufferedSource::new_default_size(file)),
                                file_path,
                                &sink,
                                gapless,
                                &total_duration,
                            ),
                            Err(e) => error!("error open file: {e}"),
                        }
                    }
                }
                Some(MediaType::Podcast) => {
                    is_radio = false;
                    if let Some(url_str) = track.file() {
                        let url = match url_str.parse::<reqwest::Url>() {
                            Ok(v) => v,
                            Err(err) => {
                                error!("error parse url: {:#?}", err);
                                continue;
                            }
                        };

                        match StreamDownload::new_http(
                            url,
                            false,
                            radio_title.clone(),
                            radio_downloaded.clone(),
                        ) {
                            Ok(reader) => {
                                append_to_sink(
                                    Box::new(reader),
                                    url_str,
                                    &sink,
                                    gapless,
                                    &total_duration,
                                );
                            }
                            Err(e) => {
                                error!("download error: {e}");
                                continue;
                            }
                        }
                    }
                }

                Some(MediaType::LiveRadio) => {
                    is_radio = true;
                    if let Some(url_str) = track.file() {
                        let url = match url_str.parse::<reqwest::Url>() {
                            Ok(v) => v,
                            Err(err) => {
                                error!("error parse url: {:#?}", err);
                                continue;
                            }
                        };

                        match StreamDownload::new_http(
                            url,
                            true,
                            radio_title.clone(),
                            radio_downloaded.clone(),
                        ) {
                            Ok(reader) => {
                                append_to_sink_no_duration(
                                    Box::new(reader),
                                    url_str,
                                    &sink,
                                    gapless,
                                    &total_duration,
                                );
                            }
                            Err(e) => {
                                error!("download error: {e}");
                                continue;
                            }
                        }
                    }
                }
                None => {}
            },
            PlayerInternalCmd::TogglePause => {
                sink.toggle_playback();
            }
            PlayerInternalCmd::QueueNext(url, gapless) => {
                match File::open(Path::new(&url)) {
                    Ok(file) => {
                        append_to_sink_queue(
                            Box::new(BufferedSource::new_default_size(file)),
                            &url,
                            &sink,
                            gapless,
                            &mut next_duration_opt,
                        );
                    }

                    Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                        if let Ok(cursor) = RustyBackend::cache_complete(&url) {
                            // TODO: replace "trace" param once knowing what to set for trace
                            append_to_sink_queue(
                                Box::new(cursor),
                                // maybe there is a better trace point?
                                "QueueNext Error cache_complete",
                                &sink,
                                gapless,
                                &mut next_duration_opt,
                            );
                        }
                    }
                    Err(e) => {
                        error!("error is now: {e:?}");
                    }
                }
            }
            PlayerInternalCmd::Resume => {
                sink.play();
            }
            PlayerInternalCmd::Speed(speed) => {
                speed_inside = speed;
                sink.set_speed(speed_inside as f32 / 10.0);
            }
            PlayerInternalCmd::Stop => {
                sink = Sink::try_new(&handle, picmd_tx.clone(), pcmd_tx.clone()).unwrap();
                sink.set_speed(speed_inside as f32 / 10.0);
                sink.set_volume(f32::from(volume_inside.load(Ordering::SeqCst)) / 100.0);
            }
            PlayerInternalCmd::Volume(volume) => {
                sink.set_volume(f32::from(volume) / 100.0);
                volume_inside.store(volume, Ordering::SeqCst);
            }
            PlayerInternalCmd::Skip => {
                sink.skip_one();
                if sink.is_paused() {
                    sink.play();
                }
            }
            PlayerInternalCmd::Progress(new_position) => {
                // let position = sink.elapsed().as_secs() as i64;
                // error!("position in rusty backend is: {}", position);
                *position.lock() = new_position;

                // About to finish signal is a simulation of gstreamer, and used for gapless
                if !is_radio {
                    if let Some(d) = *total_duration.lock() {
                        let progress = new_position.as_secs_f64() / d.as_secs_f64();
                        if progress >= 0.5
                            && d.saturating_sub(new_position) < Duration::from_secs(2)
                        {
                            if let Err(e) = pcmd_tx.send(PlayerCmd::AboutToFinish) {
                                error!("command AboutToFinish sent failed: {e}");
                            }
                        }
                    }
                }
            }
            PlayerInternalCmd::SeekAbsolute(position) => {
                sink.seek(position);
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
                    if let Some(d) = *total_duration.lock() {
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
                    sink.set_volume(f32::from(volume_inside.load(Ordering::SeqCst)) / 100.0);
                }
            }

            PlayerInternalCmd::Eos => {
                // replace the current total_duration with the next one
                // this is only present when QueueNext was used; which is only used if gapless is enabled
                if next_duration_opt.is_some() {
                    *total_duration.lock() = next_duration_opt;
                }
            }
        }
    }
}
