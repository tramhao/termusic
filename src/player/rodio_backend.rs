use super::GeneralP;
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
// use std::marker::{Send, Sync};
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
    Pause(bool),
    Progress,
    // Seek(i64),
}

pub struct RodioPlayer {
    sender: Sender<PlayerCommand>,
    progress_receiver: Receiver<i64>,
    current_song: Option<Song>,
    paused: bool,
    volume: i32,
}

// unsafe impl Send for RodioPlayer {}
// unsafe impl Sync for RodioPlayer {}

impl Default for RodioPlayer {
    fn default() -> Self {
        let (tx, rx): (Sender<PlayerCommand>, Receiver<PlayerCommand>) = mpsc::channel();
        let (progress_tx, progress_rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();
        thread::spawn(move || loop {
            let (_stream, handle) = OutputStream::try_default().unwrap();
            let mut sink: Sink;
            sink = Sink::try_new(&handle).unwrap();
            let mut time_pos: i64 = 0;
            let mut paused = false;
            loop {
                if let Ok(player_command) = rx.try_recv() {
                    match player_command {
                        PlayerCommand::Play(song) => {
                            sink = Sink::try_new(&handle).unwrap();
                            sink.set_volume(0.5);
                            let file = std::fs::File::open(song).unwrap();
                            sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
                            time_pos = 0;
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
                        PlayerCommand::Pause(pause_or_resume) => {
                            if pause_or_resume {
                                sink.pause();
                                paused = true;
                            } else {
                                sink.play();
                                paused = false;
                            }
                        }
                        PlayerCommand::Progress => {
                            progress_tx.send(time_pos).unwrap();
                        } // PlayerCommand::Seek(pos) => {}
                    }
                }
                if !paused {
                    time_pos += 1;
                }
                sleep(Duration::from_secs(1));
            }
        });

        Self {
            sender: tx,
            progress_receiver: progress_rx,
            current_song: None,
            paused: false,
            volume: 75,
            // receiver: rx,
        }
    }
}

impl GeneralP for RodioPlayer {
    fn add_and_play(&mut self, song: &str) {
        // Create a media from a file
        // let tx = self.sender.clone();
        // Create a media player
        // self.current_song = Some(song.clone());
        let tx = self.sender.clone();
        if tx.send(PlayerCommand::Stop).is_ok() {}
        // sleep(Duration::from_secs(2));

        if tx.send(PlayerCommand::Play(song.to_string())).is_ok() {}
        // tx.send(PlayerState::Completed).unwrap();
    }

    fn volume(&self) -> i32 {
        self.volume
    }

    fn volume_up(&mut self) {
        if self.sender.send(PlayerCommand::VolumeUp).is_ok() {};
    }

    fn volume_down(&mut self) {
        if self.sender.send(PlayerCommand::VolumeDown).is_ok() {};
    }

    fn set_volume(&mut self, mut volume: i32) {
        if volume > 100 {
            volume = 100;
        } else if volume < 0 {
            volume = 0;
        }
        self.volume = volume;
    }

    fn pause(&mut self) {
        if self.sender.send(PlayerCommand::Pause(true)).is_ok() {
            self.paused = true;
        };
    }

    fn resume(&mut self) {
        if self.sender.send(PlayerCommand::Pause(false)).is_ok() {
            self.paused = false;
        };
    }

    fn is_paused(&mut self) -> bool {
        self.paused
    }

    fn seek(&mut self, _secs: i64) -> Result<()> {
        // if self.sender.send(PlayerCommand::Seek(secs)).is_ok() {};
        Ok(())
    }

    fn get_progress(&mut self) -> Result<(f64, i64, i64)> {
        // match self.current_song.clone() {
        //     Some(song) => {
        //         if self.sender.send(PlayerCommand::Progress).is_ok() {
        //             if let Ok(time_pos) = self.progress_receiver.try_recv() {
        //                 let duration = song.duration.unwrap_or_else(|| Duration::from_secs(100));
        //                 let duration_i64 = duration.as_secs() as i64;
        //                 let percent = time_pos as f64 / duration_i64 as f64;
        //                 (percent, time_pos, duration_i64)
        //             } else {
        //                 (0.9, 0, 100)
        //             }
        //         } else {
        //             (0.9, 0, 100)
        //         }
        //     }
        //     None => (0.9, 0, 100),
        // }
        Ok((0.9, 1, 59))
    }
}
