// use mpv::{MpvHandler, MpvHandlerBuilder};
use libmpv::*;
use std::marker::{Send, Sync};

pub struct AudioPlayer {
    mpv: Mpv,
}

unsafe impl Send for AudioPlayer {}
unsafe impl Sync for AudioPlayer {}

impl AudioPlayer {
    pub fn new() -> AudioPlayer {
        let mpv = Mpv::new().expect("Couldn't initialize MpvHandlerBuilder");
        // mpv.("ytdl", "yes")
        //     .expect("Couldn't enable ytdl in libmpv");
        mpv.set_property("vo", "null")
            .expect("Couldn't set vo=null in libmpv");
        AudioPlayer { mpv }
    }

    // pub fn queue(&mut self, new: String) {
    //     self.mpv
    //         .command(&"loadfile", &[new.as_ref(), "append-play"])
    //         .expect("Error loading file");
    // }

    pub fn queue_and_play(&mut self, new: String) {
        self.mpv
            // .command(&"loadfile", &[new.as_ref(), "replace"])
            .command(&"loadfile", &[&format!("\"{}\"", new), "replace"])
            .expect("Error loading file");
    }

    pub fn volume(&mut self) -> i64 {
        self.mpv
            .get_property("ao-volume")
            .expect("Error adjusting volume")
    }

    pub fn volume_up(&mut self) {
        let mut volume = self.volume();
        volume += 5;
        if volume > 100 {
            volume = 100;
        }
        self.mpv
            .set_property("ao-volume", volume)
            .expect("Error increase volume")
    }

    pub fn volume_down(&mut self) {
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

    pub fn pause(&mut self) {
        self.mpv
            .set_property("pause", true)
            .expect("Toggling pause property");
    }

    pub fn resume(&mut self) {
        self.mpv
            .set_property("pause", false)
            .expect("Toggling pause property");
    }

    pub fn is_paused(&mut self) -> bool {
        self.mpv.get_property("pause").expect("wrong paused state")
    }

    pub fn seek(&mut self, second: i64) -> Result<()> {
        self.mpv.command(&"seek", &[&format!("\"{}\"", second)])
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
    pub fn get_progress(&mut self) -> (f64, i64, i64, String) {
        // let percent_pos = self.mpv.get_property::<i64>("percent-pos").unwrap_or(50);
        let title = self
            .mpv
            .get_property::<String>("media-title")
            .unwrap_or("None".to_string());
        let percent_pos = self.mpv.get_property::<i64>("percent-pos").unwrap_or(0);
        let percent = percent_pos as f64 / 100 as f64;
        let time_pos = self.mpv.get_property::<i64>("time-pos").unwrap_or(0);
        let duration = self.mpv.get_property::<i64>("duration").unwrap_or(100);
        (percent, time_pos, duration, title)
    }
}
