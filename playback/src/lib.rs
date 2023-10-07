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
// #![warn(clippy::nursery)]
#![warn(clippy::pedantic)]

mod discord;
#[cfg(all(feature = "gst", not(feature = "mpv")))]
mod gstreamer_backend;
mod mpris;
#[cfg(feature = "mpv")]
mod mpv_backend;
pub mod playlist;
#[cfg(not(any(feature = "mpv", feature = "gst")))]
mod rusty_backend;
use anyhow::Result;
#[cfg(feature = "mpv")]
use mpv_backend::MpvBackend;
pub use playlist::{Playlist, Status};
// use std::sync::RwLock;
// use std::sync::{Arc, Mutex};
use termusiclib::config::{LastPosition, SeekStep, Settings};
// use tokio::sync::Mutex;
// use parking_lot::Mutex;
// use std::sync::Arc;
// use tokio::sync::mpsc::{self, Receiver, Sender};
// #[cfg(not(any(feature = "mpv", feature = "gst")))]
use async_trait::async_trait;
use std::time::Duration;
use termusiclib::podcast::db::Database as DBPod;
use termusiclib::sqlite::DataBase;
use termusiclib::track::{MediaType, Track};
use termusiclib::types::{DaemonUpdate, DaemonUpdateChangedTrack};
use termusiclib::utils::get_app_config_path;
use tokio::sync::mpsc::{self, UnboundedSender};

#[macro_use]
extern crate log;

#[derive(Clone, Debug)]
pub enum PlayerCmd {
    AboutToFinish,
    CycleLoop,
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    DurationNext(u64),
    /// Sent on end-of-stream by playback backend or by the player if it wants to skip to next song
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
    SubscribeToUpdates(UnboundedSender<DaemonUpdate>),
}

/// # Errors
///
///

#[allow(clippy::module_name_repetitions)]
pub struct GeneralPlayer {
    #[cfg(all(feature = "gst", not(feature = "mpv")))]
    pub backend: gstreamer_backend::GStreamer,
    #[cfg(feature = "mpv")]
    pub backend: MpvBackend,
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    pub backend: rusty_backend::Player,
    pub playlist: Playlist,
    pub config: Settings,
    pub current_track_updated: bool,
    pub mpris: mpris::Mpris,
    pub discord: discord::Rpc,
    pub db: DataBase,
    pub db_podcast: DBPod,
    pub cmd_tx: UnboundedSender<PlayerCmd>,
    pub subscribers: Vec<UnboundedSender<DaemonUpdate>>,
}

impl GeneralPlayer {
    #[must_use]
    pub fn new(config: &Settings, cmd_tx: mpsc::UnboundedSender<PlayerCmd>) -> Self {
        #[cfg(all(feature = "gst", not(feature = "mpv")))]
        let backend = gstreamer_backend::GStreamer::new(config, cmd_tx.clone());
        #[cfg(feature = "mpv")]
        let backend = MpvBackend::new(config, cmd_tx.clone());
        #[cfg(not(any(feature = "mpv", feature = "gst")))]
        let backend = rusty_backend::Player::new(config, cmd_tx.clone());
        let playlist = Playlist::new(config).unwrap_or_default();

        let db_path = get_app_config_path().expect("failed to get podcast db path.");

        let db_podcast = DBPod::connect(&db_path).expect("error connecting to podcast db.");
        Self {
            backend,
            playlist,
            config: config.clone(),
            mpris: mpris::Mpris::default(),
            discord: discord::Rpc::default(),
            db: DataBase::new(config),
            db_podcast,
            cmd_tx,
            current_track_updated: false,
            subscribers: vec![],
        }
    }
    pub fn toggle_gapless(&mut self) -> bool {
        self.backend.gapless = !self.backend.gapless;
        self.config.player_gapless = self.backend.gapless;
        self.backend.gapless
    }

    pub fn handle_eos(&mut self) {
        if self.playlist.is_empty() {
            self.stop();
            return;
        }
        debug!(
            "current track index: {:?}",
            self.playlist.get_current_track_index()
        );
        self.playlist.clear_current_track();
        self.start_play();
    }

    pub fn handle_play_selected(&mut self) {
        self.player_save_last_position();
        self.playlist.proceed_false();
        self.next();
    }

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
                #[cfg(not(any(feature = "mpv", feature = "gst")))]
                {
                    {
                        let mut t = self.backend.total_duration.lock();
                        *t = self.playlist.next_track_duration();
                    }
                    self.backend.message_on_end();

                    self.add_and_play_mpris_discord();
                }
                return;
            }

            self.current_track_updated = true;
            let wait = async {
                self.add_and_play(&track).await;
            };
            let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
            rt.block_on(wait);
            self.notify_subscribers(DaemonUpdate::ChangedTrack(DaemonUpdateChangedTrack {
                new_track_index: u32::try_from(
                    self.playlist
                        .get_current_track_index()
                        .expect("no current track index"),
                )
                .expect("current track index is larger than a u32"),
            }));

            self.add_and_play_mpris_discord();
            self.player_restore_last_position();
            #[cfg(not(any(feature = "mpv", feature = "gst")))]
            {
                self.backend.message_on_end();
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

    pub fn handle_about_to_finish(&mut self) {
        if !self.playlist.is_empty()
            && !self.playlist.has_next_track()
                && self.config.player_gapless
        {
            self.enqueue_next();
        }
    }

    fn enqueue_next(&mut self) {
        if self.playlist.next_track().is_some() {
            return;
        }

        let track = match self.playlist.fetch_next_track() {
            Some(t) => t.clone(),
            None => return,
        };

        self.playlist.set_next_track(Some(track.clone()));
        if let Some(file) = track.file() {
            #[cfg(not(any(feature = "mpv", feature = "gst")))]
            self.backend.enqueue_next(file);
            // if let Some(d) = self.player.enqueue_next(file) {
            //     self.playlist.set_next_track_duration(d);
            //     // eprintln!("next track queued");
            // }
            #[cfg(all(feature = "gst", not(feature = "mpv")))]
            {
                self.backend.enqueue_next(file);
                // eprintln!("next track queued");
                self.playlist.set_next_track(None);
                // self.playlist.handle_current_track();
            }

            #[cfg(feature = "mpv")]
            {
                self.backend.enqueue_next(file);
                // eprintln!("next track queued");
            }

            // Notify that we have changed track
            // self.playlist.next_track_index is set to the index of `track` inside of
            // fetch_next_track
            self.notify_subscribers(DaemonUpdate::ChangedTrack(DaemonUpdateChangedTrack {
                new_track_index: u32::try_from(self.playlist.next_track_index)
                    .expect("next_track_index is larger than u32"),
            }));
        }
    }

    pub fn next(&mut self) {
        if self.playlist.current_track().is_some() {
            info!("skip route 1 which is in most cases.");
            self.playlist.set_next_track(None);
            self.backend.skip_one();
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
                self.backend.pause();
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
                self.backend.resume();
                if self.config.player_use_mpris {
                    self.mpris.resume();
                }
                if self.config.player_use_discord {
                    let time_pos = self.backend.position.lock();
                    self.discord.resume(*time_pos);
                }
                self.playlist.set_status(Status::Running);
            }
        }
    }
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
        self.backend.seek(offset).expect("Error in player seek.");
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn player_save_last_position(&mut self) {
        match self.config.player_remember_last_played_position {
            LastPosition::Yes => {
                if let Some(track) = self.playlist.current_track() {
                    // let time_pos = self.player.position.lock().unwrap();
                    let time_pos = self.backend.position.lock();
                    match track.media_type {
                        Some(MediaType::Music) => self
                            .db
                            .set_last_position(track, Duration::from_secs(*time_pos as u64)),
                        Some(MediaType::Podcast) => self
                            .db_podcast
                            .set_last_position(track, Duration::from_secs(*time_pos as u64)),
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
                        let time_pos = self.backend.position.lock();
                        match track.media_type {
                            Some(MediaType::Music) => self
                                .db
                                .set_last_position(track, Duration::from_secs(*time_pos as u64)),
                            Some(MediaType::Podcast) => self
                                .db_podcast
                                .set_last_position(track, Duration::from_secs(*time_pos as u64)),
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
                                self.backend.seek_to(last_pos);
                                restored = true;
                            }
                        }

                        Some(MediaType::Podcast) => {
                            if let Ok(last_pos) = self.db_podcast.get_last_position(track) {
                                self.backend.seek_to(last_pos);
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
                                    self.backend.seek_to(last_pos);
                                    restored = true;
                                }
                            }

                            Some(MediaType::Podcast) => {
                                if let Ok(last_pos) = self.db_podcast.get_last_position(track) {
                                    self.backend.seek_to(last_pos);
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

    #[allow(clippy::needless_pass_by_value)]
    fn notify_subscribers(&mut self, update: DaemonUpdate) {
        self.subscribers.retain(|sub| {
            if let Err(e) = sub.send(update.clone()) {
                log::error!("Could not notify subscriber: {e}");
                false
            } else {
                true
            }
        });
    }

    pub fn handle_tick(&mut self, p_tick: &mut termusiclib::types::player::GetProgressResponse) {
        p_tick.status = self.playlist.status().as_u32();
        if self.playlist.status() == Status::Stopped {
            if self.playlist.is_empty() {
                return;
            }
            debug!(
                "current track index: {:?}",
                self.playlist.get_current_track_index()
            );
            self.playlist.clear_current_track();
            self.playlist.proceed_false();
            self.start_play();
            return;
        }
        if let Ok((position, duration)) = self.get_progress() {
            p_tick.position = position as u32;
            p_tick.duration = duration as u32;
            if self.current_track_updated {
                p_tick.current_track_index =
                    self.playlist.get_current_track_index().unwrap() as u32;
                self.current_track_updated = false;
            }
            if let Some(track) = self.playlist.current_track() {
                if let Some(MediaType::LiveRadio) = &track.media_type {
                    #[cfg(not(any(feature = "mpv", feature = "gst")))]
                    {
                        p_tick.radio_title = self.backend.radio_title.lock().clone();
                        p_tick.duration =
                            ((*self.backend.radio_downloaded.lock() as f32 * 44100.0
                              / 1000000.0
                              / 1024.0)
                             * (self.speed() as f32 / 10.0))
                            as u32;
                    }
                    #[cfg(feature = "mpv")]
                    {
                        p_tick.radio_title = self.backend.media_title.lock().clone();
                    }
                    #[cfg(all(feature = "gst", not(feature = "mpv")))]
                    {
                        // p_tick.duration = player.backend.get_buffer_duration();
                        // eprintln!("buffer duration: {}", p_tick.duration);
                        p_tick.duration = position as u32 + 20;
                        p_tick.radio_title = self.backend.radio_title.lock().clone();
                        // eprintln!("radio title: {}", p_tick.radio_title);
                    }
                }
            }
        }
    }

}

#[async_trait]
impl PlayerTrait for GeneralPlayer {
    async fn add_and_play(&mut self, current_track: &Track) {
        self.backend.add_and_play(current_track).await;
    }
    fn volume(&self) -> i32 {
        self.backend.volume()
    }
    fn volume_up(&mut self) {
        self.backend.volume_up();
    }
    fn volume_down(&mut self) {
        self.backend.volume_down();
    }
    fn set_volume(&mut self, volume: i32) {
        self.backend.set_volume(volume);
    }
    fn pause(&mut self) {
        self.playlist.set_status(Status::Paused);
        self.backend.pause();
    }
    fn resume(&mut self) {
        self.playlist.set_status(Status::Running);
        self.backend.resume();
    }
    fn is_paused(&self) -> bool {
        self.playlist.is_paused()
    }
    fn seek(&mut self, secs: i64) -> Result<()> {
        self.backend.seek(secs)
    }
    fn seek_to(&mut self, last_pos: Duration) {
        self.backend.seek_to(last_pos);
    }

    fn set_speed(&mut self, speed: i32) {
        self.backend.set_speed(speed);
    }

    fn speed_up(&mut self) {
        self.backend.speed_up();
    }

    fn speed_down(&mut self) {
        self.backend.speed_down();
    }

    fn speed(&self) -> i32 {
        self.backend.speed()
    }

    fn stop(&mut self) {
        self.playlist.set_status(Status::Stopped);
        self.playlist.set_next_track(None);
        self.playlist.clear_current_track();
        self.backend.stop();
    }

    fn get_progress(&self) -> Result<(i64, i64)> {
        self.backend.get_progress()
    }
}

#[allow(clippy::module_name_repetitions)]
#[async_trait]
pub trait PlayerTrait {
    async fn add_and_play(&mut self, current_track: &Track);
    fn volume(&self) -> i32;
    fn volume_up(&mut self);
    fn volume_down(&mut self);
    fn set_volume(&mut self, volume: i32);
    fn pause(&mut self);
    fn resume(&mut self);
    fn is_paused(&self) -> bool;
    /// # Errors
    ///
    /// Depending on different backend, there could be different errors during seek.
    fn seek(&mut self, secs: i64) -> Result<()>;
    fn seek_to(&mut self, last_pos: Duration);
    /// # Errors
    ///
    /// Depending on different backend, there could be different errors during get progress.
    fn get_progress(&self) -> Result<(i64, i64)>;
    fn set_speed(&mut self, speed: i32);
    fn speed_up(&mut self);
    fn speed_down(&mut self);
    fn speed(&self) -> i32;
    fn stop(&mut self);
}
