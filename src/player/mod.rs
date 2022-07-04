#[cfg(all(feature = "gst", not(feature = "mpv")))]
mod gstreamer_backend;
#[cfg(feature = "mpv")]
mod mpv_backend;
mod playlist;
#[cfg(not(any(feature = "mpv", feature = "gst")))]
mod rusty_backend;
use crate::config::Settings;
use crate::track::Track;
use anyhow::Result;
#[cfg(feature = "mpv")]
use mpv_backend::MpvBackend;
pub use playlist::Playlist;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Receiver, Sender};
#[cfg(not(any(feature = "mpv", feature = "gst")))]
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    Running,
    Stopped,
    Paused,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "Running"),
            Self::Stopped => write!(f, "Stopped"),
            Self::Paused => write!(f, "Paused"),
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum Loop {
    Single,
    Playlist,
    Queue,
}

#[allow(clippy::non_ascii_literal)]
impl Loop {
    pub fn display(&self, display_symbol: bool) -> String {
        if display_symbol {
            match self {
                Self::Single => "🔂".to_string(),
                Self::Playlist => "🔁".to_string(),
                Self::Queue => "⬇".to_string(),
            }
        } else {
            match self {
                Self::Single => "single".to_string(),
                Self::Playlist => "playlist".to_string(),
                Self::Queue => "consume".to_string(),
            }
        }
    }
}

pub enum PlayerMsg {
    Eos,
    AboutToFinish,
    CurrentTrackUpdated,
    Progress(i64, i64),
    PlayNextStart,
}

pub struct GeneralPlayer {
    #[cfg(all(feature = "gst", not(feature = "mpv")))]
    player: gstreamer_backend::GStreamer,
    #[cfg(feature = "mpv")]
    player: MpvBackend,
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    player: rusty_backend::Player,
    pub message_tx: Sender<PlayerMsg>,
    pub message_rx: Receiver<PlayerMsg>,
    pub playlist: Playlist,
    status: Status,
    pub config: Settings,
    next_track: Option<Track>,
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    next_track_duration: Duration,
}

impl GeneralPlayer {
    pub fn new(config: &Settings) -> Self {
        let (message_tx, message_rx): (Sender<PlayerMsg>, Receiver<PlayerMsg>) = mpsc::channel();
        #[cfg(all(feature = "gst", not(feature = "mpv")))]
        let player = gstreamer_backend::GStreamer::new(config, message_tx.clone());
        #[cfg(feature = "mpv")]
        let player = MpvBackend::new(config, message_tx.clone());
        #[cfg(not(any(feature = "mpv", feature = "gst")))]
        let player = rusty_backend::Player::new(config, message_tx.clone());
        let mut playlist = Playlist::default();
        if let Ok(p) = Playlist::new() {
            playlist = p;
        }
        Self {
            player,
            message_tx,
            message_rx,
            playlist,
            status: Status::Stopped,
            config: config.clone(),
            next_track: None,
            #[cfg(not(any(feature = "mpv", feature = "gst")))]
            next_track_duration: Duration::from_secs(0),
        }
    }
    pub fn toggle_gapless(&mut self) {
        self.player.gapless = !self.player.gapless;
    }

    pub fn start_play(&mut self) {
        if !self.is_running() {
            self.set_status(Status::Running);
        }
        if let Some(song) = self.playlist.tracks.pop_front() {
            if let Some(file) = song.file() {
                #[cfg(not(any(feature = "mpv", feature = "gst")))]
                if self.next_track.is_none() {
                    self.add_and_play(file);

                    self.player.sink.message_on_end();
                    self.message_tx
                        .send(PlayerMsg::CurrentTrackUpdated)
                        .expect("fail to send track updated signal");
                    eprintln!("add and play {}", file);
                } else {
                    self.next_track = None;
                    self.player.total_duration = Some(self.next_track_duration);
                    self.player.sink.message_on_end();
                    self.message_tx
                        .send(PlayerMsg::CurrentTrackUpdated)
                        .expect("fail to send track updated signal");
                    eprintln!("next track encountered");
                    // eprintln!("Length of queue: {}", self.player.len());
                }
                #[cfg(all(feature = "gst", not(feature = "mpv")))]
                if self.next_track.is_none() {
                    self.add_and_play(file);
                    eprintln!("add and play {}", file);
                } else {
                    self.next_track = None;
                    eprintln!("next track encountered");
                }

                #[cfg(all(feature = "mpv", not(feature = "gst")))]
                if !self.has_next_track() {
                    self.add_and_play(file);
                    eprintln!("add and play {}", file);
                    self.message_tx
                        .send(PlayerMsg::CurrentTrackUpdated)
                        .expect("fail to send track updated signal");
                } else {
                    self.next_track = None;
                    eprintln!("next track encountered");
                    self.message_tx
                        .send(PlayerMsg::CurrentTrackUpdated)
                        .expect("fail to send track updated signal");
                }
            }
            match self.config.loop_mode {
                Loop::Playlist => self.playlist.tracks.push_back(song.clone()),
                Loop::Single => self.playlist.tracks.push_front(song.clone()),
                Loop::Queue => {}
            }
            self.playlist.current_track = Some(song);
        } else {
            self.playlist.current_track = None;
            self.set_status(Status::Stopped);
        }
    }

    // #[cfg(all(feature = "mpv", not(feature = "gst")))]
    pub fn play_next_start(&mut self) {
        self.next_track = None;
        eprintln!("next track encountered");
        self.handle_current_track();
    }

    fn handle_current_track(&mut self) {
        if let Some(song) = self.playlist.tracks.pop_front() {
            match self.config.loop_mode {
                Loop::Playlist => self.playlist.tracks.push_back(song.clone()),
                Loop::Single => self.playlist.tracks.push_front(song.clone()),
                Loop::Queue => {}
            }
            self.playlist.current_track = Some(song);
            self.message_tx
                .send(PlayerMsg::CurrentTrackUpdated)
                .expect("fail to send track updated signal");
        } else {
            self.playlist.current_track = None;
            self.set_status(Status::Stopped);
        }
    }

    pub fn enqueue_next(&mut self) {
        if self.next_track.is_none() {
            if let Some(track) = self.playlist.tracks.get(0) {
                self.next_track = Some(track.clone());
                if let Some(file) = track.file() {
                    #[cfg(not(any(feature = "mpv", feature = "gst")))]
                    if let Some(d) = self.player.enqueue_next(file) {
                        self.next_track_duration = d;
                        eprintln!("next track queued");
                    }
                    #[cfg(all(feature = "gst", not(feature = "mpv")))]
                    {
                        self.player.enqueue_next(file);
                        eprintln!("next track queued");
                        self.next_track = None;
                        if let Some(song) = self.playlist.tracks.pop_front() {
                            match self.config.loop_mode {
                                Loop::Playlist => self.playlist.tracks.push_back(song.clone()),
                                Loop::Single => self.playlist.tracks.push_front(song.clone()),
                                Loop::Queue => {}
                            }
                            self.playlist.current_track = Some(song);
                        }
                    }

                    #[cfg(all(feature = "mpv", not(feature = "gst")))]
                    {
                        self.player.enqueue_next(file);
                        eprintln!("next track queued");
                    }
                }
            }
        }
    }

    pub fn has_next_track(&mut self) -> bool {
        self.next_track.is_some()
    }

    pub fn skip(&mut self) {
        self.next_track = None;
        self.player.skip_one();
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn is_stopped(&self) -> bool {
        self.status == Status::Stopped
    }

    pub fn is_running(&self) -> bool {
        self.status == Status::Running
    }
    pub fn status(&self) -> Status {
        self.status
    }
}

impl PlayerTrait for GeneralPlayer {
    fn add_and_play(&mut self, current_track: &str) {
        self.player.add_and_play(current_track);
    }
    fn volume(&self) -> i32 {
        self.player.volume()
    }
    fn volume_up(&mut self) {
        self.player.volume_up();
    }
    fn volume_down(&mut self) {
        self.player.volume_down();
    }
    fn set_volume(&mut self, volume: i32) {
        self.player.set_volume(volume);
    }
    fn pause(&mut self) {
        self.player.pause();
    }
    fn resume(&mut self) {
        self.player.resume();
    }
    fn is_paused(&self) -> bool {
        self.status == Status::Paused
    }
    fn seek(&mut self, secs: i64) -> Result<()> {
        self.player.seek(secs)
    }

    fn get_progress(&self) -> Result<()> {
        self.player.get_progress()
    }

    fn set_speed(&mut self, speed: i32) {
        self.player.set_speed(speed);
    }

    fn speed_up(&mut self) {
        self.player.speed_up();
    }

    fn speed_down(&mut self) {
        self.player.speed_down();
    }

    fn speed(&self) -> i32 {
        self.player.speed()
    }

    fn stop(&mut self) {
        self.status = Status::Stopped;
        self.next_track = None;
        self.player.stop();
    }
}

pub trait PlayerTrait {
    fn add_and_play(&mut self, current_track: &str);
    fn volume(&self) -> i32;
    fn volume_up(&mut self);
    fn volume_down(&mut self);
    fn set_volume(&mut self, volume: i32);
    fn pause(&mut self);
    fn resume(&mut self);
    fn is_paused(&self) -> bool;
    fn seek(&mut self, secs: i64) -> Result<()>;
    fn get_progress(&self) -> Result<()>;
    fn set_speed(&mut self, speed: i32);
    fn speed_up(&mut self);
    fn speed_down(&mut self);
    fn speed(&self) -> i32;
    fn stop(&mut self);
}
