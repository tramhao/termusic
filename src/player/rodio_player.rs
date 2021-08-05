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
use rodio::{OutputStream, Sink};
use std::io::BufReader;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
// // PlayerState is used to describe the status of player
pub enum PlayerCommand {
    VolumeUp,
    VolumeDown,
    Stop,
    Play(String),
}

pub struct RodioPlayer {
    sender: Sender<PlayerCommand>,
}

unsafe impl Send for RodioPlayer {}
unsafe impl Sync for RodioPlayer {}

impl RodioPlayer {
    pub fn new() -> RodioPlayer {
        let (tx, rx): (Sender<PlayerCommand>, Receiver<PlayerCommand>) = mpsc::channel();
        thread::spawn(move || loop {
            let (_stream, handle) = OutputStream::try_default().unwrap();
            let mut sink: Sink;
            sink = Sink::try_new(&handle).unwrap();
            loop {
                if let Ok(player_command) = rx.try_recv() {
                    match player_command {
                        PlayerCommand::Play(song) => {
                            sink = Sink::try_new(&handle).unwrap();
                            sink.set_volume(0.5);
                            let file = std::fs::File::open(song).unwrap();
                            sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
                        }
                        PlayerCommand::Stop => {
                            sink.stop();
                        }
                        PlayerCommand::VolumeUp => {
                            let mut volume = sink.volume();
                            volume += 0.05;
                            if volume > 1.0 {
                                volume = 1.0;
                            }
                            sink.set_volume(volume);
                        }
                        PlayerCommand::VolumeDown => {
                            let mut volume = sink.volume();
                            volume -= 0.05;
                            if volume < 0.0 {
                                volume = 0.0;
                            }
                            sink.set_volume(volume);
                        }
                    }
                }
                sleep(Duration::from_secs(1));
            }
        });

        RodioPlayer {
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
        let tx = self.sender.clone();
        if tx.send(PlayerCommand::Stop).is_ok() {}
        // sleep(Duration::from_secs(2));

        if tx.send(PlayerCommand::Play(song.file.unwrap())).is_ok() {}
        // tx.send(PlayerState::Completed).unwrap();
    }

    fn volume(&mut self) -> i64 {
        // self.vlc.get_volume() as i64
        // let sink = Sink::try_new(&self.handle).unwrap();
        // let volume_f32 = sink.volume();
        // (volume_f32 * 100 as f32) as i64
        75
    }

    fn volume_up(&mut self) {
        if self.sender.send(PlayerCommand::VolumeUp).is_ok() {};
    }

    fn volume_down(&mut self) {
        if self.sender.send(PlayerCommand::VolumeDown).is_ok() {};
    }

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
