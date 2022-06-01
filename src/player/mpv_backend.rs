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
use crate::config::Termusic;
use anyhow::{anyhow, Result};
use libmpv::Mpv as MpvBackend;
use std::cmp;

pub struct Mpv {
    player: MpvBackend,
    volume: i32,
    speed: f32,
    pub gapless: bool,
    current_item: Option<String>,
    next_item: Option<String>,
}

impl Mpv {
    pub fn new(config: &Termusic) -> Self {
        let mpv = MpvBackend::new().expect("Couldn't initialize MpvHandlerBuilder");
        mpv.set_property("vo", "null")
            .expect("Couldn't set vo=null in libmpv");

        let volume = config.volume;
        mpv.set_property("volume", i64::from(volume))
            .expect("Error setting volume");
        let speed = config.speed;
        mpv.set_property("speed", f64::from(speed)).ok();
        let gapless = config.gapless;
        let gapless_setting = if gapless { "yes" } else { "no" };
        mpv.set_property("gapless-audio", gapless_setting)
            .expect("gapless setting failed");
        Self {
            player: mpv,
            volume,
            speed,
            gapless,
            current_item: None,
            next_item: None,
        }
    }

    fn queue(&mut self, new: &str) {
        self.player
            .command("loadfile", &[&format!("\"{}\"", new), "append-play"])
            .expect("Error loading file");
    }

    fn queue_and_play(&mut self, new: &str) {
        self.player
            .command("loadfile", &[&format!("\"{}\"", new), "replace"])
            .expect("Error loading file");
    }
}

impl GeneralP for Mpv {
    fn add_and_play(&mut self, current_item: &str, next_item: Option<&str>) {
        if self.next_item.is_none() {
            self.stop();
        }
        self.current_item = Some(current_item.to_string());
        if self.current_item == self.next_item {
            // This is for gapless playback
            if let Some(next) = next_item {
                self.next_item = Some(next.to_string());
                self.queue(next);
            }
            self.player
                .command("playlist_next", &["weak"])
                .expect("fail to go to next track");
            return;
        }

        self.queue_and_play(current_item);
    }

    fn volume(&self) -> i32 {
        self.volume
    }

    fn volume_up(&mut self) {
        self.volume = cmp::min(self.volume + 5, 100);
        self.player
            .set_property("volume", i64::from(self.volume))
            .expect("Error increase volume");
    }

    fn volume_down(&mut self) {
        self.volume = cmp::max(self.volume - 5, 0);
        self.player
            .set_property("volume", i64::from(self.volume))
            .expect("Error decrease volume");
    }
    fn set_volume(&mut self, mut volume: i32) {
        if volume > 100 {
            volume = 100;
        } else if volume < 0 {
            volume = 0;
        }
        self.volume = volume;
        self.player
            .set_property("volume", i64::from(self.volume))
            .expect("Error setting volume");
    }

    fn pause(&mut self) {
        self.player
            .set_property("pause", true)
            .expect("Toggling pause property");
    }

    fn resume(&mut self) {
        self.player
            .set_property("pause", false)
            .expect("Toggling pause property");
    }

    fn is_paused(&self) -> bool {
        self.player
            .get_property("pause")
            .expect("wrong paused state")
    }

    fn seek(&mut self, secs: i64) -> Result<()> {
        match self
            .player
            .command("seek", &[&format!("\"{}\"", secs), "relative"])
        {
            Ok(r) => Ok(r),
            Err(e) => Err(anyhow!(format!("Error in mpv: {}", e))),
        }
    }

    fn get_progress(&mut self) -> Result<(f64, i64, i64)> {
        let percent_pos = self
            .player
            .get_property::<f64>("percent-pos")
            .unwrap_or(0.0);
        // let percent = percent_pos / 100_f64;
        let time_pos = self.player.get_property::<i64>("time-pos").unwrap_or(0);
        let duration = self.player.get_property::<i64>("duration").unwrap_or(0);
        Ok((percent_pos, time_pos, duration))
    }

    fn speed(&self) -> f32 {
        self.speed
    }

    fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
        self.player.set_property("speed", f64::from(speed)).ok();
    }

    fn speed_up(&mut self) {
        let mut speed = self.speed + 0.1;
        if speed > 3.0 {
            speed = 3.0;
        }
        self.set_speed(speed);
    }

    fn speed_down(&mut self) {
        let mut speed = self.speed - 0.1;
        if speed < 0.1 {
            speed = 0.1;
        }
        self.set_speed(speed);
    }
    fn stop(&mut self) {
        self.player
            .command("stop", &[""])
            .expect("Error stop mpv player");
    }
}
