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
// use mpv::{MpvHandler, MpvHandlerBuilder};
use crate::song::Song;
use anyhow::{anyhow, Result};
use libmpv::*;
use std::marker::{Send, Sync};

pub struct MPVAudioPlayer {
    mpv: Mpv,
}

unsafe impl Send for MPVAudioPlayer {}
unsafe impl Sync for MPVAudioPlayer {}

impl MPVAudioPlayer {
    pub fn new() -> MPVAudioPlayer {
        let mpv = Mpv::new().expect("Couldn't initialize MpvHandlerBuilder");
        mpv.set_property("vo", "null")
            .expect("Couldn't set vo=null in libmpv");
        MPVAudioPlayer { mpv }
    }
}

impl AudioPlayer for MPVAudioPlayer {
    // pub fn queue(&mut self, new: String) {
    //     self.mpv
    //         .command(&"loadfile", &[new.as_ref(), "append-play"])
    //         .expect("Error loading file");
    // }

    fn queue_and_play(&mut self, new: Song) {
        self.mpv
            // .command(&"loadfile", &[new.as_ref(), "replace"])
            .command(
                "loadfile",
                &[&format!("\"{}\"", new.file.unwrap()), "replace"],
            )
            .expect("Error loading file");
    }

    fn volume(&mut self) -> i64 {
        self.mpv
            .get_property("ao-volume")
            .expect("Error adjusting volume")
    }

    fn volume_up(&mut self) {
        let mut volume = self.volume();
        volume += 5;
        if volume > 100 {
            volume = 100;
        }
        self.mpv
            .set_property("ao-volume", volume)
            .expect("Error increase volume")
    }

    fn volume_down(&mut self) {
        let mut volume = self.volume();
        volume -= 5;
        if volume < 0 {
            volume = 0
        }
        self.mpv
            .set_property("ao-volume", volume)
            .expect("Error decrease volume")
    }
    // pub fn stop(&mut self) {
    //     self.mpv.command("stop", &[""]).expect("Error stopping mpv");
    // }

    fn pause(&mut self) {
        self.mpv
            .set_property("pause", true)
            .expect("Toggling pause property");
    }

    fn resume(&mut self) {
        self.mpv
            .set_property("pause", false)
            .expect("Toggling pause property");
    }

    fn is_paused(&mut self) -> bool {
        self.mpv.get_property("pause").expect("wrong paused state")
    }

    fn seek(&mut self, secs: i64) -> Result<()> {
        match self
            .mpv
            .command("seek", &[&format!("\"{}\"", secs), "relative"])
        {
            Ok(r) => Ok(r),
            Err(e) => Err(anyhow!(format!("Error in mpv: {}", e))),
        }
    }

    // pub fn loop_(&mut self) {
    //     let next_loop = match self.mpv.get_property("loop-file") {
    //         Ok(x) => {
    //             if x == "inf" || x == "yes" {
    //                 println!("Toggling loop off");
    //                 "no"
    //             } else if x == "no" || x == "1" {
    //                 println!("Toggling loop on");
    //                 "inf"
    //             } else {
    //                 panic!("Unexpected value for loop-file property")
    //             }
    //         }
    //         Err(e) => panic!(e),
    //     };
    //     self.mpv
    //         .set_property("loop-file", next_loop)
    //         .expect("Toggling loop-file property");
    // }

    // pub fn print_time_remain(&mut self) {
    //     println!(:w
    //
    //         "Time remaining: {:?}",
    //         self.mpv.get_property::<i64>("time-remain").unwrap_or(-9999)
    //     );
    // }
    fn get_progress(&mut self) -> (f64, i64, i64, String) {
        // let percent_pos = self.mpv.get_property::<i64>("percent-pos").unwrap_or(50);
        let title = self
            .mpv
            .get_property::<String>("media-title")
            .unwrap_or_else(|_| "None".to_string());
        let percent_pos = self.mpv.get_property::<i64>("percent-pos").unwrap_or(0);
        let percent = percent_pos as f64 / 100_f64;
        let time_pos = self.mpv.get_property::<i64>("time-pos").unwrap_or(0);
        let duration = self.mpv.get_property::<i64>("duration").unwrap_or(100);
        (percent, time_pos, duration, title)
    }
}
