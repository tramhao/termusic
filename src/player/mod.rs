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

#[cfg(all(feature = "gst", not(feature = "mpv")))]
mod gstreamer_backend;
#[cfg(feature = "mpv")]
mod mpv_backend;
mod playlist;
#[cfg(not(any(feature = "mpv", feature = "gst")))]
mod rusty_backend;
use crate::config::Settings;
use anyhow::Result;
#[cfg(feature = "mpv")]
use mpv_backend::MpvBackend;
pub use playlist::{Loop, Playlist, Status};
use std::sync::mpsc::{self, Receiver, Sender};
// #[cfg(not(any(feature = "mpv", feature = "gst")))]
use std::time::Duration;

#[allow(clippy::module_name_repetitions)]
pub enum PlayerMsg {
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    CacheStart,
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    CacheEnd,
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    Duration(u64),
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    DurationNext(u64),
    Eos,
    AboutToFinish,
    CurrentTrackUpdated,
    Progress(i64, i64),
}

#[allow(clippy::module_name_repetitions)]
pub struct GeneralPlayer {
    #[cfg(all(feature = "gst", not(feature = "mpv")))]
    player: gstreamer_backend::GStreamer,
    #[cfg(feature = "mpv")]
    player: MpvBackend,
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    pub player: rusty_backend::Player,
    pub message_tx: Sender<PlayerMsg>,
    pub message_rx: Receiver<PlayerMsg>,
    pub playlist: Playlist,
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
        if let Ok(p) = Playlist::new(config) {
            playlist = p;
        }
        Self {
            player,
            message_tx,
            message_rx,
            playlist,
        }
    }
    pub fn toggle_gapless(&mut self) -> bool {
        self.player.gapless = !self.player.gapless;
        self.player.gapless
    }

    pub fn start_play(&mut self) {
        if self.playlist.is_stopped() | self.playlist.is_paused() {
            self.playlist.set_status(Status::Running);
            // self.resume();
            if self.playlist.current_track().is_none() {
                self.playlist.handle_current_track();
            }
        }

        if let Some(file) = self.playlist.get_current_track() {
            if self.playlist.has_next_track() {
                self.playlist.set_next_track(None);
                // eprintln!("next track played");
                #[cfg(not(any(feature = "mpv", feature = "gst")))]
                {
                    self.player.total_duration = Some(self.playlist.next_track_duration());
                    self.player.message_on_end();
                    self.message_tx
                        .send(PlayerMsg::CurrentTrackUpdated)
                        .expect("fail to send track updated signal");
                }
                return;
            }

            self.add_and_play(&file);
            // eprintln!("completely new track added");
            #[cfg(not(any(feature = "mpv", feature = "gst")))]
            {
                self.player.message_on_end();
                self.message_tx
                    .send(PlayerMsg::CurrentTrackUpdated)
                    .expect("fail to send track updated signal");
            }
        }
    }

    pub fn enqueue_next(&mut self) {
        if self.playlist.next_track().is_some() {
            return;
        }

        let track = match self.playlist.fetch_next_track() {
            Some(t) => t.clone(),
            None => return,
        };

        self.playlist.set_next_track(Some(&track));
        if let Some(file) = track.file() {
            #[cfg(not(any(feature = "mpv", feature = "gst")))]
            self.player.enqueue_next(file);
            // if let Some(d) = self.player.enqueue_next(file) {
            //     self.playlist.set_next_track_duration(d);
            //     // eprintln!("next track queued");
            // }
            #[cfg(all(feature = "gst", not(feature = "mpv")))]
            {
                self.player.enqueue_next(file);
                // eprintln!("next track queued");
                self.playlist.set_next_track(None);
                // self.playlist.handle_current_track();
            }

            #[cfg(feature = "mpv")]
            {
                self.player.enqueue_next(file);
                // eprintln!("next track queued");
            }
        }
    }

    pub fn skip(&mut self) {
        if self.playlist.current_track().is_some() {
            self.playlist.set_next_track(None);
            self.player.skip_one();
        } else {
            self.message_tx.send(PlayerMsg::Eos).ok();
        }
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
        self.playlist.set_status(Status::Paused);
        self.player.pause();
    }
    fn resume(&mut self) {
        self.playlist.set_status(Status::Running);
        self.player.resume();
    }
    fn is_paused(&self) -> bool {
        self.playlist.is_paused()
    }
    fn seek(&mut self, secs: i64) -> Result<()> {
        self.player.seek(secs)
    }
    fn seek_to(&mut self, last_pos: Duration) {
        self.player.seek_to(last_pos);
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
        self.playlist.set_status(Status::Stopped);
        self.playlist.set_next_track(None);
        self.playlist.set_current_track(None);
        self.player.stop();
    }
}

#[allow(clippy::module_name_repetitions)]
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
    fn seek_to(&mut self, last_pos: Duration);
    // fn get_progress(&self) -> Result<()>;
    fn set_speed(&mut self, speed: i32);
    fn speed_up(&mut self);
    fn speed_down(&mut self);
    fn speed(&self) -> i32;
    fn stop(&mut self);
}
