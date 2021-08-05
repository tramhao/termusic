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
use std::sync::mpsc;
// use std::sync::mpsc::{Receiver, Sender};
use std::thread;
// use std::thread::sleep;
// use std::time::Duration;
use vlc::MediaPlayerAudioEx;
use vlc::{Event, EventType, State};
// use vlc::Vlm;
use vlc::{Instance, Media, MediaPlayer};

// // PlayerState is used to describe the status of player
// pub enum PlayerState {
//     // StartPlaying,
//     // Running, // indicates progress
//     Completed,
//     // Skipped,
// }

pub struct VLCAudioPlayer {
    instance: Instance,
    vlc: MediaPlayer,
    // sender: Sender<PlayerState>,
    // receiver: Receiver<PlayerState>,
}

unsafe impl Send for VLCAudioPlayer {}
unsafe impl Sync for VLCAudioPlayer {}

impl VLCAudioPlayer {
    pub fn new() -> VLCAudioPlayer {
        // Create an instance
        let instance = Instance::new().expect("Couldn't initialize VLCAudioPlayer");
        // Create a media player
        let vlc = MediaPlayer::new(&instance).expect("Couldn't initialize VLCAudioPlayer 2");
        // let (tx, rx): (Sender<PlayerState>, Receiver<PlayerState>) = mpsc::channel();
        VLCAudioPlayer {
            instance,
            vlc,
            // sender: tx,
            // receiver: rx,
        }
    }
}

impl AudioPlayer for VLCAudioPlayer {
    fn queue_and_play(&mut self, song: Song) {
        // Create a media from a file
        // let tx = self.sender.clone();
        // Create a media player

        // let (tx, rx): (Sender<PlayerState>, Receiver<PlayerState>) = mpsc::channel();
        let (tx, rx) = mpsc::channel::<()>();
        // let instance = Instance::new().expect("Couldn't initialize VLCAudioPlayer");
        // Start playing
        let vlc = MediaPlayer::new(&self.instance).expect("Couldn't initialize VLCAudioPlayer 2");
        // let vlc = self.vlc.

        let md = Media::new_path(&self.instance, song.file.unwrap()).unwrap();
        vlc.set_media(&md);

        thread::spawn(move || {
            let em = md.event_manager();
            #[allow(clippy::single_match)]
            let _ = em.attach(EventType::MediaStateChanged, move |e, _| match e {
                Event::MediaStateChanged(s) => {
                    if s == State::Ended {
                        //}|| s == State::Error {
                        tx.send(()).unwrap();
                    }
                }
                _ => (),
            });

            vlc.play().unwrap();
            rx.recv().unwrap();
        });
        // loop {
        //     if let Ok(player_state) = rx.try_recv() {
        //         match player_state {
        //             PlayerState::Completed => {
        //                 break;
        //             }
        //             _ => {}
        //         }
        //     }

        // self.instance
        //     .play_media(song.file.unwrap().as_str())
        //     .expect("play media failed");
        // }
        // instance.wait();
    }

    fn volume(&mut self) -> i64 {
        self.vlc.get_volume() as i64
    }

    fn volume_up(&mut self) {
        let mut volume = self.volume();
        volume += 5;
        if volume > 100 {
            volume = 100;
        }
        self.vlc
            .set_volume(volume as i32)
            .expect("Error set volume");
    }

    fn volume_down(&mut self) {
        let mut volume = self.volume();
        volume -= 5;
        if volume < 0 {
            volume = 0
        }
        self.vlc
            .set_volume(volume as i32)
            .expect("Error set volume");
    }
    // pub fn stop(&mut self) {
    //     self.mpv.command("stop", &[""]).expect("Error stopping mpv");
    // }

    fn pause(&mut self) {
        self.vlc.pause();
    }

    fn resume(&mut self) {
        self.vlc.play().expect("Error play");
    }

    fn is_paused(&mut self) -> bool {
        !self.vlc.is_playing()
    }

    fn seek(&mut self, secs: i64) -> Result<()> {
        self.vlc.set_time(secs * 1000);
        Ok(())
    }

    fn get_progress(&mut self) -> (f64, i64, i64) {
        // let percent_pos = self.mpv.get_property::<i64>("percent-pos").unwrap_or(50);
        let vlc = MediaPlayer::new(&self.instance).expect("Couldn't initialize VLCAudioPlayer 2");
        // let md = vlc.get_media().expect("cannot get media");
        // let meta: Meta;
        // md.get_meta(meta);
        // let percent_pos = self.mpv.get_property::<i64>("percent-pos").unwrap_or(0);
        // let percent = percent_pos as f64 / 100_f64;
        // let time_pos = self.mpv.get_property::<i64>("time-pos").unwrap_or(0);
        // let duration = self.mpv.get_property::<i64>("duration").unwrap_or(100);
        // let percent = 0.5;
        let percent = vlc.get_position().unwrap_or(0.3) as f64;
        let time_pos = 2;
        // let duration = md.duration().unwrap_or(100);
        let duration = 100;
        (percent, time_pos, duration)
    }
}
