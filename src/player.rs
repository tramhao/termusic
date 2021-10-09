/**
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
use anyhow::{bail, Result};
use gst::ClockTime;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_pbutils as gst_pbutils;
use gstreamer_player as gst_player;
// use std::sync::Arc;
// use std::thread;
// use std::marker::{Send, Sync};
#[cfg(feature = "mpris")]
use crate::song::Song;
#[cfg(feature = "mpris")]
use crate::souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
#[cfg(feature = "mpris")]
use std::str::FromStr;
#[cfg(feature = "mpris")]
use std::sync::mpsc::{self, Receiver};

pub struct GStreamer {
    player: gst_player::Player,
    paused: bool,
    volume: i32,
    #[cfg(feature = "mpris")]
    controls: MediaControls,
    #[cfg(feature = "mpris")]
    pub rx: Receiver<MediaControlEvent>,
}

// unsafe impl Send for GSTPlayer {}
// unsafe impl Sync for GSTPlayer {}

impl GStreamer {
    pub fn new() -> Self {
        gst::init().expect("Couldn't initialize Gstreamer");
        let dispatcher = gst_player::PlayerGMainContextSignalDispatcher::new(None);
        let player = gst_player::Player::new(None, Some(&dispatcher.upcast::<gst_player::PlayerSignalDispatcher>()));

        #[cfg(feature = "mpris")]
        let config = PlatformConfig {
            dbus_name: "termusic",
            display_name: "Termuisc in Rust",
        };

        #[cfg(feature = "mpris")]
        let mut controls = MediaControls::new(config);

        #[cfg(feature = "mpris")]
        let (tx, rx) = mpsc::sync_channel(32);
        // The closure must be Send and have a static lifetime.
        #[cfg(feature = "mpris")]
        controls
            .attach(move |event: MediaControlEvent| {
                tx.send(event).ok();
            })
            .unwrap();

        Self {
            player,
            paused: false,
            volume: 50,
            #[cfg(feature = "mpris")]
            controls,
            #[cfg(feature = "mpris")]
            rx,
        }
    }

    pub fn duration(song: &str) -> ClockTime {
        let timeout: ClockTime = ClockTime::from_seconds(1);
        let mut duration = ClockTime::from_seconds(0);
        if let Ok(discoverer) = gst_pbutils::Discoverer::new(timeout) {
            if let Ok(info) = discoverer.discover_uri(&format!("file:///{}", song)) {
                if let Some(d) = info.duration() {
                    duration = d;
                }
            }
        }
        duration
    }

    pub fn add_and_play(&mut self, song_str: &str) {
        self.player.set_uri(&format!("file:///{}", song_str));
        self.paused = false;
        self.player.play();

        #[cfg(feature = "mpris")]
        if let Ok(song) = Song::from_str(song_str) {
            self.controls.set_metadata(MediaMetadata {
                title: Some(song.title().unwrap_or("Unknown Title")),
                artist: Some(song.artist().unwrap_or("Unknown Artist")),
                album: Some(song.album().unwrap_or("")),
                ..MediaMetadata::default()
            });
        }
        #[cfg(feature = "mpris")]
        self.controls
            .set_playback(MediaPlayback::Playing { progress: None })
            .ok();
    }

    pub fn volume_up(&mut self) {
        self.volume += 5;
        if self.volume > 100 {
            self.volume = 100;
        }
        self.player.set_volume(f64::from(self.volume) / 100.0);
    }

    pub fn volume_down(&mut self) {
        self.volume -= 5;
        if self.volume < 0 {
            self.volume = 0;
        }
        self.player.set_volume(f64::from(self.volume) / 100.0);
    }

    pub const fn volume(&self) -> i32 {
        self.volume
    }

    pub fn set_volume(&mut self, mut volume: i32) {
        if volume > 100 {
            volume = 100;
        } else if volume < 0 {
            volume = 0;
        }
        self.volume = volume;
        self.player.set_volume(f64::from(volume) / 100.0);
    }

    pub fn pause(&mut self) {
        self.paused = true;
        self.player.pause();

        #[cfg(feature = "mpris")]
        self.controls
            .set_playback(MediaPlayback::Paused { progress: None })
            .ok();
    }

    pub fn resume(&mut self) {
        self.paused = false;
        self.player.play();

        #[cfg(feature = "mpris")]
        self.controls
            .set_playback(MediaPlayback::Playing { progress: None })
            .ok();
    }

    pub fn is_paused(&mut self) -> bool {
        self.paused
    }

    pub fn seek(&mut self, secs: i64) -> Result<()> {
        let (_, time_pos, duration) = self.get_progress();
        let seek_pos: u64;
        if secs >= 0 {
            seek_pos = time_pos + secs.abs() as u64;
        } else if time_pos > secs.abs() as u64 {
            seek_pos = time_pos - secs.abs() as u64;
        } else {
            seek_pos = 0;
        }

        if seek_pos.cmp(&duration) == std::cmp::Ordering::Greater {
            bail! {"exceed max length"};
        }
        self.player.seek(ClockTime::from_seconds(seek_pos as u64));
        Ok(())
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn get_progress(&mut self) -> (f64, u64, u64) {
        let time_pos = match self.player.position() {
            Some(t) => ClockTime::seconds(t),
            None => 0_u64,
        };
        let duration = match self.player.duration() {
            Some(d) => ClockTime::seconds(d),
            None => 119_u64,
        };
        let percent = time_pos as f64 / (duration as f64);
        (percent, time_pos, duration)
    }
}
