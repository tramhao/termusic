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
            .command(&"loadfile", &[new.as_ref()])
            .expect("Error loading file");
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
    //     println!(
    //         "Time remaining: {:?}",
    //         self.mpv.get_property::<i64>("time-remain").unwrap_or(-9999)
    //     );
    // }
}
