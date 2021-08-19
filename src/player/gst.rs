use super::AudioPlayer;
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
use crate::song::Song;
use anyhow::Result;
use gst::ClockTime;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_pbutils as gst_pbutils;
use gstreamer_player as gst_player;
use std::marker::{Send, Sync};

pub struct GSTPlayer {
    player: gst_player::Player,
    paused: bool,
}

unsafe impl Send for GSTPlayer {}
unsafe impl Sync for GSTPlayer {}

impl GSTPlayer {
    pub fn new() -> GSTPlayer {
        // gst::init().expect("Couldn't initialize Gstreamer");
        let dispatcher = gst_player::PlayerGMainContextSignalDispatcher::new(None);
        let player = gst_player::Player::new(
            None,
            Some(&dispatcher.upcast::<gst_player::PlayerSignalDispatcher>()),
        );
        player.set_volume(0.5);
        GSTPlayer {
            player,
            paused: false,
        }
    }

    pub fn duration_m4a(song: &str) -> u64 {
        let timeout: gst::ClockTime = gst::ClockTime::from_seconds(1);
        let mut duration: u64 = 0;
        if let Ok(discoverer) = gst_pbutils::Discoverer::new(timeout) {
            if let Ok(info) = discoverer.discover_uri(&format!("file:///{}", song)) {
                if let Some(d) = info.duration() {
                    duration = d.seconds();
                }
            }
        }
        duration
    }
}

impl AudioPlayer for GSTPlayer {
    fn queue_and_play(&mut self, song: Song) {
        if let Some(file) = song.file {
            self.player.set_uri(&format!("file:///{}", file));
            self.paused = false;
            self.player.play();
        }
    }

    fn volume(&mut self) -> i64 {
        75
    }

    fn volume_up(&mut self) {
        let mut volume = self.player.volume();
        volume += 0.05;
        if volume > 1.0 {
            volume = 1.0
        }
        self.player.set_volume(volume);
    }

    fn volume_down(&mut self) {
        let mut volume = self.player.volume();
        volume -= 0.05;
        if volume < 0.0 {
            volume = 0.0
        }
        self.player.set_volume(volume);
    }

    fn pause(&mut self) {
        self.paused = true;
        self.player.pause();
    }

    fn resume(&mut self) {
        self.paused = false;
        self.player.play();
    }

    fn is_paused(&mut self) -> bool {
        self.paused
    }

    fn seek(&mut self, secs: i64) -> Result<()> {
        let (_, time_pos, duration) = self.get_progress();
        let mut seek_pos = time_pos + secs;
        if seek_pos < 0 {
            seek_pos = 0;
        } else if seek_pos > duration {
            return Ok(());
        }
        self.player.seek(ClockTime::from_seconds(seek_pos as u64));
        Ok(())
    }

    fn get_progress(&mut self) -> (f64, i64, i64) {
        let time_pos = match self.player.position() {
            Some(t) => ClockTime::seconds(t) as i64,
            None => 0_i64,
        };
        let duration = match self.player.duration() {
            Some(d) => ClockTime::seconds(d) as i64,
            None => 119_i64,
        };
        let percent = time_pos as f64 / (duration as f64);
        (percent, time_pos, duration)
    }
}
