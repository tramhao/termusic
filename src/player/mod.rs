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
pub mod mpv;
use crate::song::Song;
use anyhow::Result;

pub trait AudioPlayer {
    // pub fn queue(&mut self, new: String) {
    //     self.mpv
    //         .command(&"loadfile", &[new.as_ref(), "append-play"])
    //         .expect("Error loading file");
    // }

    fn queue_and_play(&mut self, new: Song);
    fn volume(&mut self) -> i64;
    fn volume_up(&mut self);
    fn volume_down(&mut self); // pub fn stop(&mut self) {
                               //     self.mpv.command("stop", &[""]).expect("Error stopping mpv");
                               // }

    fn pause(&mut self);
    fn resume(&mut self);
    fn is_paused(&mut self) -> bool;
    fn seek(&mut self, secs: i64) -> Result<()>;
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
    fn get_progress(&mut self) -> (f64, i64, i64, String);
}
