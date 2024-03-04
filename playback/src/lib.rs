/*
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

// #![forbid(unsafe_code)]
#![recursion_limit = "2048"]
#![warn(clippy::all, clippy::correctness)]
#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]
#[allow(clippy::pedantic)]
pub mod player {
    tonic::include_proto!("player");

    // implement transform function for easy use
    impl From<Duration> for std::time::Duration {
        fn from(value: Duration) -> Self {
            std::time::Duration::new(value.secs, value.nanos)
        }
    }

    impl From<std::time::Duration> for Duration {
        fn from(value: std::time::Duration) -> Self {
            Self {
                secs: value.as_secs(),
                nanos: value.subsec_nanos(),
            }
        }
    }
}

#[cfg(feature = "gst")]
mod gstreamer_backend;
#[cfg(feature = "mpv")]
mod mpv_backend;
#[cfg(feature = "rusty")]
mod rusty_backend;

mod discord;
mod mpris;
pub mod playlist;

use anyhow::{Context, Result};
use async_trait::async_trait;
pub use playlist::{Playlist, Status};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use termusiclib::config::{LastPosition, SeekStep, Settings};
use termusiclib::podcast::db::Database as DBPod;
use termusiclib::sqlite::DataBase;
use termusiclib::track::{MediaType, Track};
use termusiclib::utils::get_app_config_path;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[macro_use]
extern crate log;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendSelect {
    #[cfg(feature = "mpv")]
    Mpv,
    #[cfg(feature = "rusty")]
    Rusty,
    #[cfg(feature = "gst")]
    GStreamer,
    /// Create a new Backend with default backend ordering
    ///
    /// Order:
    /// - [`RustyBackend`](rusty_backend::RustyBackend) (feature `rusty`)
    /// - [`GstreamerBackend`](gstreamer_backend::GStreamerBackend) (feature `gst`)
    /// - [`MpvBackend`](mpv_backend::MpvBackend) (feature `mpv`)
    /// - Compile Error
    #[default]
    Default,
}

/// Enum to choose backend at runtime
pub enum Backend {
    #[cfg(feature = "mpv")]
    Mpv(mpv_backend::MpvBackend),
    #[cfg(feature = "rusty")]
    Rusty(rusty_backend::RustyBackend),
    #[cfg(feature = "gst")]
    GStreamer(gstreamer_backend::GStreamerBackend),
}

pub type PlayerCmdReciever = UnboundedReceiver<PlayerCmd>;
pub type PlayerCmdSender = UnboundedSender<PlayerCmd>;

impl Backend {
    /// Create a new Backend based on `backend`([`BackendSelect`])
    fn new_select(backend: BackendSelect, config: &Settings, cmd_tx: PlayerCmdSender) -> Self {
        match backend {
            #[cfg(feature = "mpv")]
            BackendSelect::Mpv => Self::new_mpv(config, cmd_tx),
            #[cfg(feature = "rusty")]
            BackendSelect::Rusty => Self::new_rusty(config, cmd_tx),
            #[cfg(feature = "gst")]
            BackendSelect::GStreamer => Self::new_gstreamer(config, cmd_tx),
            BackendSelect::Default => Self::new_default(config, cmd_tx),
        }
    }

    /// Create a new Backend with default backend ordering
    ///
    /// For the order see [`BackendSelect::Default`]
    #[allow(unreachable_code)]
    fn new_default(config: &Settings, cmd_tx: PlayerCmdSender) -> Self {
        #[cfg(feature = "rusty")]
        return Self::new_rusty(config, cmd_tx);
        #[cfg(feature = "gst")]
        return Self::new_gstreamer(config, cmd_tx);
        #[cfg(feature = "mpv")]
        return Self::new_mpv(config, cmd_tx);

        #[cfg(not(any(feature = "rusty", feature = "gst", feature = "mpv")))]
        compile_error!("No useable backend feature!");
    }

    /// Explicitly choose Backend [`RustyBackend`](rusty_backend::RustyBackend)
    #[cfg(feature = "rusty")]
    fn new_rusty(config: &Settings, cmd_tx: PlayerCmdSender) -> Self {
        info!("Using Backend \"rusty\"");
        Self::Rusty(rusty_backend::RustyBackend::new(config, cmd_tx))
    }

    /// Explicitly choose Backend [`GstreamerBackend`](gstreamer_backend::GStreamerBackend)
    #[cfg(feature = "gst")]
    fn new_gstreamer(config: &Settings, cmd_tx: PlayerCmdSender) -> Self {
        info!("Using Backend \"GStreamer\"");
        Self::GStreamer(gstreamer_backend::GStreamerBackend::new(config, cmd_tx))
    }

    /// Explicitly choose Backend [`MpvBackend`](mpv_backend::MpvBackend)
    #[cfg(feature = "mpv")]
    fn new_mpv(config: &Settings, cmd_tx: PlayerCmdSender) -> Self {
        info!("Using Backend \"mpv\"");
        Self::Mpv(mpv_backend::MpvBackend::new(config, cmd_tx))
    }

    #[must_use]
    pub fn as_player(&self) -> &dyn PlayerTrait {
        match self {
            #[cfg(feature = "mpv")]
            Backend::Mpv(v) => v,
            #[cfg(feature = "rusty")]
            Backend::Rusty(v) => v,
            #[cfg(feature = "gst")]
            Backend::GStreamer(v) => v,
        }
    }

    #[must_use]
    pub fn as_player_mut(&mut self) -> &mut (dyn PlayerTrait + Send) {
        match self {
            #[cfg(feature = "mpv")]
            Backend::Mpv(v) => v,
            #[cfg(feature = "rusty")]
            Backend::Rusty(v) => v,
            #[cfg(feature = "gst")]
            Backend::GStreamer(v) => v,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum PlayerCmd {
    AboutToFinish,
    CycleLoop,
    Eos,
    GetProgress,
    PlaySelected,
    SkipPrevious,
    Pause,
    Play,
    ProcessID,
    Quit,
    ReloadConfig,
    ReloadPlaylist,
    SeekBackward,
    SeekForward,
    SkipNext,
    SpeedDown,
    SpeedUp,
    Tick,
    ToggleGapless,
    TogglePause,
    VolumeDown,
    VolumeUp,
}

#[allow(clippy::module_name_repetitions)]
pub struct GeneralPlayer {
    pub backend: Backend,
    pub playlist: Playlist,
    pub config: Settings,
    pub current_track_updated: bool,
    pub mpris: mpris::Mpris,
    pub discord: discord::Rpc,
    pub db: DataBase,
    pub db_podcast: DBPod,
    pub cmd_tx: PlayerCmdSender,
}

impl GeneralPlayer {
    /// Create a new [`GeneralPlayer`], with the selected `backend`
    ///
    /// # Errors
    ///
    /// - if connecting to the database fails
    /// - if config path creation fails
    pub fn new_backend(
        backend: BackendSelect,
        config: &Settings,
        cmd_tx: PlayerCmdSender,
    ) -> Result<Self> {
        let backend = Backend::new_select(backend, config, cmd_tx.clone());
        let playlist = Playlist::new(config).unwrap_or_default();

        let db_path = get_app_config_path().with_context(|| "failed to get podcast db path.")?;

        let db_podcast =
            DBPod::connect(&db_path).with_context(|| "error connecting to podcast db.")?;
        Ok(Self {
            backend,
            playlist,
            config: config.clone(),
            mpris: mpris::Mpris::new(cmd_tx.clone()),
            discord: discord::Rpc::default(),
            db: DataBase::new(config),
            db_podcast,
            cmd_tx,
            current_track_updated: false,
        })
    }

    /// Create a new [`GeneralPlayer`], with the [`BackendSelect::Default`] backend
    ///
    /// # Errors
    ///
    /// - if connecting to the database fails
    /// - if config path creation fails
    #[allow(clippy::missing_panics_doc)]
    pub fn new(config: &Settings, cmd_tx: PlayerCmdSender) -> Result<Self> {
        Self::new_backend(BackendSelect::Default, config, cmd_tx)
    }

    fn get_player(&self) -> &dyn PlayerTrait {
        self.backend.as_player()
    }

    fn get_player_mut(&mut self) -> &mut (dyn PlayerTrait + Send) {
        self.backend.as_player_mut()
    }

    pub fn toggle_gapless(&mut self) -> bool {
        let new_gapless = !self.backend.as_player().gapless();
        self.backend.as_player_mut().set_gapless(new_gapless);
        self.config.player_gapless = new_gapless;
        new_gapless
    }

    /// # Panics
    ///
    /// panics if the [`tokio::runtime::Runtime`] fails to build
    pub fn start_play(&mut self) {
        if self.playlist.is_stopped() | self.playlist.is_paused() {
            self.playlist.set_status(Status::Running);
        }

        self.playlist.proceed();

        if let Some(track) = self.playlist.current_track() {
            let track = track.clone();
            if self.playlist.has_next_track() {
                self.playlist.set_next_track(None);
                self.current_track_updated = true;
                info!("gapless next track played");
                #[cfg(feature = "rusty")]
                #[allow(irrefutable_let_patterns)]
                if let Backend::Rusty(ref mut backend) = self.backend {
                    backend.message_on_end();
                }
                self.add_and_play_mpris_discord();
                return;
            }

            self.current_track_updated = true;
            let wait = async {
                self.add_and_play(&track).await;
            };
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to create runtime")
                .block_on(wait);

            self.add_and_play_mpris_discord();
            self.player_restore_last_position();
            #[cfg(feature = "rusty")]
            #[allow(irrefutable_let_patterns)]
            if let Backend::Rusty(ref mut backend) = self.backend {
                backend.message_on_end();
            }
        }
    }

    fn add_and_play_mpris_discord(&mut self) {
        if let Some(track) = self.playlist.current_track() {
            if self.config.player_use_mpris {
                self.mpris.add_and_play(track);
            }

            if self.config.player_use_discord {
                self.discord.update(track);
            }
        }
    }
    pub fn enqueue_next_from_playlist(&mut self) {
        if self.playlist.next_track().is_some() {
            return;
        }

        let track = match self.playlist.fetch_next_track() {
            Some(t) => t.clone(),
            None => return,
        };

        self.playlist.set_next_track(Some(&track));
        if let Some(file) = track.file() {
            self.get_player_mut().enqueue_next(file);

            info!("Next track enqueued: {:#?}", file);
        }
    }

    pub fn next(&mut self) {
        if self.playlist.current_track().is_some() {
            info!("skip route 1 which is in most cases.");
            self.playlist.set_next_track(None);
            self.get_player_mut().skip_one();
        } else {
            info!("skip route 2 cause no current track.");
            self.stop();
            // if let Err(e) = crate::audio_cmd::<()>(PlayerCmd::StartPlay, false) {
            //     debug!("Error in skip route 2: {e}");
            // }
        }
    }
    pub fn previous(&mut self) {
        self.playlist.previous();
        self.playlist.proceed_false();
        self.next();
    }
    pub fn toggle_pause(&mut self) {
        match self.playlist.status() {
            Status::Running => {
                self.get_player_mut().pause();
                if self.config.player_use_mpris {
                    self.mpris.pause();
                }
                if self.config.player_use_discord {
                    self.discord.pause();
                }
                self.playlist.set_status(Status::Paused);
            }
            Status::Stopped => {}
            Status::Paused => {
                self.get_player_mut().resume();
                if self.config.player_use_mpris {
                    self.mpris.resume();
                }
                if self.config.player_use_discord {
                    let time_pos = self.get_player().position();
                    self.discord.resume(time_pos);
                }
                self.playlist.set_status(Status::Running);
            }
        }
    }

    pub fn pause(&mut self) {
        match self.playlist.status() {
            Status::Running => {
                self.get_player_mut().pause();
                if self.config.player_use_mpris {
                    self.mpris.pause();
                }
                if self.config.player_use_discord {
                    self.discord.pause();
                }
                self.playlist.set_status(Status::Paused);
            }
            Status::Stopped | Status::Paused => {}
        }
    }

    pub fn play(&mut self) {
        match self.playlist.status() {
            Status::Running | Status::Stopped => {}
            Status::Paused => {
                self.get_player_mut().resume();
                if self.config.player_use_mpris {
                    self.mpris.resume();
                }
                if self.config.player_use_discord {
                    let time_pos = self.get_player().position();
                    self.discord.resume(time_pos);
                }
                self.playlist.set_status(Status::Running);
            }
        }
    }
    /// # Panics
    ///
    /// if the underlying "seek" returns a error (which current never happens)
    pub fn seek_relative(&mut self, forward: bool) {
        let mut offset = match self.config.player_seek_step {
            SeekStep::Short => -5_i64,
            SeekStep::Long => -30,
            SeekStep::Auto => {
                if let Some(track) = self.playlist.current_track() {
                    if track.duration().as_secs() >= 600 {
                        -30
                    } else {
                        -5
                    }
                } else {
                    -5
                }
            }
        };
        if forward {
            offset = -offset;
        }
        self.get_player_mut()
            .seek(offset)
            .expect("Error in player seek.");
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn player_save_last_position(&mut self) {
        match self.config.player_remember_last_played_position {
            LastPosition::Yes => {
                if let Some(track) = self.playlist.current_track() {
                    // let time_pos = self.player.position.lock().unwrap();
                    let time_pos = self.get_player().position();
                    match track.media_type {
                        Some(MediaType::Music) => self.db.set_last_position(track, time_pos),
                        Some(MediaType::Podcast) => {
                            self.db_podcast.set_last_position(track, time_pos);
                        }
                        Some(MediaType::LiveRadio) | None => {}
                    }
                }
            }
            LastPosition::No => {}
            LastPosition::Auto => {
                if let Some(track) = self.playlist.current_track() {
                    // 10 minutes
                    if track.duration().as_secs() >= 600 {
                        // let time_pos = self.player.position.lock().unwrap();
                        let time_pos = self.get_player().position();
                        match track.media_type {
                            Some(MediaType::Music) => self.db.set_last_position(track, time_pos),
                            Some(MediaType::Podcast) => {
                                self.db_podcast.set_last_position(track, time_pos);
                            }
                            Some(MediaType::LiveRadio) | None => {}
                        }
                    }
                }
            }
        }
    }

    // #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    pub fn player_restore_last_position(&mut self) {
        let mut restored = false;
        match self.config.player_remember_last_played_position {
            LastPosition::Yes => {
                if let Some(track) = self.playlist.current_track() {
                    match track.media_type {
                        Some(MediaType::Music) => {
                            if let Ok(last_pos) = self.db.get_last_position(track) {
                                self.get_player_mut().seek_to(last_pos);
                                restored = true;
                            }
                        }

                        Some(MediaType::Podcast) => {
                            if let Ok(last_pos) = self.db_podcast.get_last_position(track) {
                                self.get_player_mut().seek_to(last_pos);
                                restored = true;
                            }
                        }
                        Some(MediaType::LiveRadio) | None => {}
                    }
                }
            }
            LastPosition::No => {}
            LastPosition::Auto => {
                if let Some(track) = self.playlist.current_track() {
                    // 10 minutes
                    if track.duration().as_secs() >= 600 {
                        match track.media_type {
                            Some(MediaType::Music) => {
                                if let Ok(last_pos) = self.db.get_last_position(track) {
                                    self.get_player_mut().seek_to(last_pos);
                                    restored = true;
                                }
                            }

                            Some(MediaType::Podcast) => {
                                if let Ok(last_pos) = self.db_podcast.get_last_position(track) {
                                    self.get_player_mut().seek_to(last_pos);
                                    restored = true;
                                }
                            }
                            Some(MediaType::LiveRadio) | None => {}
                        }
                    }
                }
            }
        }

        if restored {
            if let Some(track) = self.playlist.current_track() {
                self.db.set_last_position(track, Duration::from_secs(0));
            }
        }
    }
}

#[async_trait]
impl PlayerTrait for GeneralPlayer {
    async fn add_and_play(&mut self, current_track: &Track) {
        self.get_player_mut().add_and_play(current_track).await;
    }
    fn volume(&self) -> u16 {
        self.get_player().volume()
    }
    fn volume_up(&mut self) {
        self.get_player_mut().volume_up();
    }
    fn volume_down(&mut self) {
        self.get_player_mut().volume_down();
    }
    fn set_volume(&mut self, volume: u16) {
        self.get_player_mut().set_volume(volume);
    }
    fn pause(&mut self) {
        self.playlist.set_status(Status::Paused);
        self.get_player_mut().pause();
    }
    fn resume(&mut self) {
        self.playlist.set_status(Status::Running);
        self.get_player_mut().resume();
    }
    fn is_paused(&self) -> bool {
        self.get_player().is_paused()
    }
    fn seek(&mut self, secs: i64) -> Result<()> {
        self.get_player_mut().seek(secs)
    }
    fn seek_to(&mut self, position: Duration) {
        self.get_player_mut().seek_to(position);
    }

    fn set_speed(&mut self, speed: i32) {
        self.get_player_mut().set_speed(speed);
    }

    fn speed_up(&mut self) {
        self.get_player_mut().speed_up();
    }

    fn speed_down(&mut self) {
        self.get_player_mut().speed_down();
    }

    fn speed(&self) -> i32 {
        self.get_player().speed()
    }

    fn stop(&mut self) {
        self.playlist.set_status(Status::Stopped);
        self.playlist.set_next_track(None);
        self.playlist.clear_current_track();
        self.get_player_mut().stop();
    }

    fn get_progress(&self) -> PlayerProgress {
        self.get_player().get_progress()
    }

    fn gapless(&self) -> bool {
        self.get_player().gapless()
    }

    fn set_gapless(&mut self, to: bool) {
        self.get_player_mut().set_gapless(to);
    }

    fn skip_one(&mut self) {
        self.get_player_mut().skip_one();
    }

    fn position(&self) -> PlayerTimeUnit {
        self.get_player().position()
    }

    fn enqueue_next(&mut self, file: &str) {
        self.get_player_mut().enqueue_next(file);
    }
}

/// The primitive in which time (current position / total duration) will be stored as
pub type PlayerTimeUnit = Duration;

/// Struct to keep both values with a name, as tuples cannot have named fields
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerProgress {
    pub position: PlayerTimeUnit,
    /// Total duration of the currently playing track, if there is a known total duration
    pub total_duration: Option<PlayerTimeUnit>,
}

impl From<crate::player::PlayerTime> for PlayerProgress {
    fn from(value: crate::player::PlayerTime) -> Self {
        Self {
            position: value.position.unwrap_or_default().into(),
            total_duration: value.total_duration.map(std::convert::Into::into),
        }
    }
}

impl From<PlayerProgress> for crate::player::PlayerTime {
    fn from(value: PlayerProgress) -> Self {
        Self {
            position: Some(value.position.into()),
            total_duration: value.total_duration.map(std::convert::Into::into),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[async_trait]
pub trait PlayerTrait {
    async fn add_and_play(&mut self, current_track: &Track);
    fn volume(&self) -> u16;
    fn volume_up(&mut self);
    fn volume_down(&mut self);
    fn set_volume(&mut self, volume: u16);
    fn pause(&mut self);
    fn resume(&mut self);
    fn is_paused(&self) -> bool;
    /// Seek relatively to the current time
    ///
    /// # Errors
    ///
    /// Depending on different backend, there could be different errors during seek.
    fn seek(&mut self, secs: i64) -> Result<()>;
    // TODO: sync return types between "seek" and "seek_to"?
    /// Seek to a absolute position
    fn seek_to(&mut self, position: Duration);
    /// Get current track time position
    fn get_progress(&self) -> PlayerProgress;
    fn set_speed(&mut self, speed: i32);
    fn speed_up(&mut self);
    fn speed_down(&mut self);
    fn speed(&self) -> i32;
    fn stop(&mut self);
    fn gapless(&self) -> bool;
    fn set_gapless(&mut self, to: bool);
    fn skip_one(&mut self);
    /// Quickly access the position.
    ///
    /// This should ALWAYS match up with [`PlayerTrait::get_progress`]'s `.position`!
    fn position(&self) -> PlayerTimeUnit {
        self.get_progress().position
    }
    fn enqueue_next(&mut self, file: &str);
}
