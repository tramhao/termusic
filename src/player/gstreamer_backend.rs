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
use super::GeneralP;
use anyhow::{anyhow, bail, Result};
use gst::ClockTime;
use gstreamer as gst;
use gstreamer::prelude::*;
// use gstreamer_pbutils as gst_pbutils;
use gstreamer_player as gst_player;
use std::cmp;
// use std::sync::Arc;
// use std::thread;
// use std::marker::{Send, Sync};

pub struct GStreamer {
    player: gst_player::Player,
    paused: bool,
    volume: i32,
}

// unsafe impl Send for GSTPlayer {}
// unsafe impl Sync for GSTPlayer {}
impl Default for GStreamer {
    fn default() -> Self {
        gst::init().expect("Couldn't initialize Gstreamer");
        let dispatcher = gst_player::PlayerGMainContextSignalDispatcher::new(None);
        let player = gst_player::Player::new(
            None,
            Some(&dispatcher.upcast::<gst_player::PlayerSignalDispatcher>()),
        );

        Self {
            player,
            paused: false,
            volume: 50,
        }
    }
}

impl GeneralP for GStreamer {
    // #[allow(unused)]
    // pub fn duration(song: &str) -> ClockTime {
    //     let timeout: ClockTime = ClockTime::from_seconds(1);
    //     let mut duration = ClockTime::from_seconds(0);
    //     if let Ok(discoverer) = gst_pbutils::Discoverer::new(timeout) {
    //         if let Ok(info) = discoverer.discover_uri(&format!("file:///{}", song)) {
    //             if let Some(d) = info.duration() {
    //                 duration = d;
    //             }
    //         }
    //     }
    //     duration
    // }

    fn add_and_play(&mut self, song_str: &str) {
        self.player.set_uri(Some(&format!("file:///{}", song_str)));
        self.paused = false;
        self.player.play();
    }

    fn volume_up(&mut self) {
        self.volume = cmp::min(self.volume + 5, 100);
        self.player.set_volume(f64::from(self.volume) / 100.0);
    }

    fn volume_down(&mut self) {
        self.volume = cmp::max(self.volume - 5, 0);
        self.player.set_volume(f64::from(self.volume) / 100.0);
    }

    fn volume(&self) -> i32 {
        self.volume
    }

    fn set_volume(&mut self, mut volume: i32) {
        if volume > 100 {
            volume = 100;
        } else if volume < 0 {
            volume = 0;
        }
        self.volume = volume;
        self.player.set_volume(f64::from(volume) / 100.0);
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

    #[allow(clippy::cast_sign_loss)]
    fn seek(&mut self, secs: i64) -> Result<()> {
        if let Ok((_, time_pos, duration)) = self.get_progress() {
            let mut seek_pos = time_pos + secs;
            if seek_pos < 0 {
                seek_pos = 0;
            }

            if seek_pos.cmp(&duration) == std::cmp::Ordering::Greater {
                bail! {"exceed max length"};
            }
            self.player.seek(ClockTime::from_seconds(seek_pos as u64));
        }
        Ok(())
    }

    #[allow(clippy::cast_precision_loss)]
    fn get_progress(&mut self) -> Result<(f64, i64, i64)> {
        let time_pos = match self.player.position() {
            Some(t) => ClockTime::seconds(t).try_into().unwrap_or(0),
            None => 0_i64,
        };
        let duration = match self.player.duration() {
            Some(d) => ClockTime::seconds(d).try_into().unwrap_or(0),
            None => 0_i64,
        };
        let mut percent = (time_pos * 100)
            .checked_div(duration)
            .ok_or_else(|| anyhow!("divide error"))?;
        if percent > 100 {
            percent = 100;
        }
        Ok((percent as f64, time_pos, duration))
    }
}
