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
use anyhow::Result;
use std::marker::{Send, Sync};
// use std::sync::mpsc::channel;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::io::BufReader;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
// // PlayerState is used to describe the status of player
pub enum PlayerState {
    // StartPlaying,
    // Running, // indicates progress
    Completed,
    // Skipped,
}

pub struct RodioPlayer {
    // sink:Sink,
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sender: Sender<PlayerState>,
    // receiver: Receiver<PlayerState>,
}

unsafe impl Send for RodioPlayer {}
unsafe impl Sync for RodioPlayer {}

impl RodioPlayer {
    pub fn new() -> RodioPlayer {
        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let (tx, _): (Sender<PlayerState>, Receiver<PlayerState>) = mpsc::channel();
        RodioPlayer {
            _stream,
            handle,
            sender: tx,
            // receiver: rx,
        }
    }
}

impl AudioPlayer for RodioPlayer {
    fn queue_and_play(&mut self, song: Song) {
        // Create a media from a file
        // let tx = self.sender.clone();
        // Create a media player
        let local_tx = self.sender.clone();
        if local_tx.send(PlayerState::Completed).is_ok() {}
        let (tx, rx): (Sender<PlayerState>, Receiver<PlayerState>) = mpsc::channel();
        // tx.send(PlayerState::Completed).unwrap();
        self.sender = tx; //.clone();
        let file = std::fs::File::open(song.file.unwrap()).unwrap();
        let sink = Sink::try_new(&self.handle).unwrap();
        sink.set_volume(0.5);
        thread::spawn(move || {
            sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
            // thread::spawn(move || {
            //     sink.sleep_until_end();
            //     match tx.send(PlayerState::Completed) {
            //         Ok(_) => {}
            //         Err(_) => {}
            //     }
            // });
            // sink.detach();
            loop {
                if let Ok(player_state) = rx.try_recv() {
                    match player_state {
                        PlayerState::Completed => {
                            break;
                        } // _ => {}
                    }
                }
                sleep(Duration::from_secs(1));
            }
        });
    }

    fn volume(&mut self) -> i64 {
        // self.vlc.get_volume() as i64
        75
    }

    fn volume_up(&mut self) {
        // let mut volume = self.volume();
        // volume += 5;
        // if volume > 100 {
        //     volume = 100;
        // }
        // self.vlc
        //     .set_volume(volume as i32)
        //     .expect("Error set volume");
    }

    fn volume_down(&mut self) {
        // let mut volume = self.volume();
        // volume -= 5;
        // if volume < 0 {
        //     volume = 0
        // }
        // self.vlc
        //     .set_volume(volume as i32)
        //     .expect("Error set volume");
    }
    // pub fn stop(&mut self) {
    //     self.mpv.command("stop", &[""]).expect("Error stopping mpv");
    // }

    fn pause(&mut self) {
        // self.vlc.pause();
    }

    fn resume(&mut self) {
        // self.vlc.play().expect("Error play");
    }

    fn is_paused(&mut self) -> bool {
        // !self.vlc.is_playing()
        true
    }

    fn seek(&mut self, _secs: i64) -> Result<()> {
        // self.vlc.set_time(secs * 1000);
        Ok(())
    }

    fn get_progress(&mut self) -> (f64, i64, i64, String) {
        // let percent_pos = self.mpv.get_property::<i64>("percent-pos").unwrap_or(50);
        // let vlc = MediaPlayer::new(&self.instance).expect("Couldn't initialize VLCAudioPlayer 2");
        // let md = vlc.get_media().expect("cannot get media");
        // let meta: Meta;
        // md.get_meta(meta);
        let title = String::from("no title");
        // let percent_pos = self.mpv.get_property::<i64>("percent-pos").unwrap_or(0);
        // let percent = percent_pos as f64 / 100_f64;
        // let time_pos = self.mpv.get_property::<i64>("time-pos").unwrap_or(0);
        // let duration = self.mpv.get_property::<i64>("duration").unwrap_or(100);
        let percent = 0.5;
        // let percent = vlc.get_position().unwrap_or(0.3) as f64;
        let time_pos = 2;
        // let duration = md.duration().unwrap_or(100);
        let duration = 100;
        (percent, time_pos, duration, title)
    }
}
