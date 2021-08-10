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
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
// use std::thread::sleep;
// use std::time::Duration;
use vlc::MediaPlayerAudioEx;
// use vlc::{Event, EventType, State};
use std::thread::sleep;
use std::time::Duration;
// use vlc::Vlm;
use vlc::{Instance, Media, MediaPlayer};

// // PlayerCommand is used to control the player
pub enum PlayerCommand {
    VolumeUp,
    VolumeDown,
    Stop,
    Play(String),
    Pause(bool),
    Progress,
    // Seek(i64),
}

pub struct VLCAudioPlayer {
    // instance: Instance,
    // vlc: MediaPlayer,
    sender: Sender<PlayerCommand>,
    progress_receiver: Receiver<i64>,
    current_song: Option<Song>,
    paused: bool,
}

unsafe impl Send for VLCAudioPlayer {}
unsafe impl Sync for VLCAudioPlayer {}

impl VLCAudioPlayer {
    pub fn new() -> VLCAudioPlayer {
        // Create an instance
        let instance = Instance::new().expect("Couldn't initialize VLCAudioPlayer");
        // Create a media player
        let vlc = MediaPlayer::new(&instance).expect("Couldn't initialize VLCAudioPlayer 2");
        if let Ok(()) = vlc.set_volume(70) {}
        let (tx, rx): (Sender<PlayerCommand>, Receiver<PlayerCommand>) = mpsc::channel();
        let (progress_tx, progress_rx): (Sender<i64>, Receiver<i64>) = mpsc::channel();
        thread::spawn(move || -> Result<()> {
            let mut time_pos: i64 = 0;
            let mut paused = false;
            loop {
                if let Ok(player_command) = rx.try_recv() {
                    match player_command {
                        PlayerCommand::Play(song) => {
                            let md = Media::new_path(&instance, song).unwrap();
                            vlc.set_media(&md);
                            if let Ok(()) = vlc.play() {
                                time_pos = 0
                            }
                        }
                        PlayerCommand::Stop => {
                            // vlc.stop();
                        }
                        PlayerCommand::VolumeUp => {
                            let mut volume = vlc.get_volume();
                            volume += 10;
                            if volume > 100 {
                                volume = 100;
                            }
                            if let Ok(()) = vlc.set_volume(volume) {}
                        }
                        PlayerCommand::VolumeDown => {
                            let mut volume = vlc.get_volume();
                            volume -= 10;
                            if volume < 0 {
                                volume = 0;
                            }
                            if let Ok(()) = vlc.set_volume(volume) {}
                        }
                        PlayerCommand::Pause(pause_or_resume) => match pause_or_resume {
                            true => {
                                vlc.pause();
                                paused = true;
                            }
                            false => {
                                if let Ok(()) = vlc.play() {
                                    paused = false
                                }
                            }
                        },
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

        VLCAudioPlayer {
            sender: tx,
            progress_receiver: progress_rx,
            current_song: None,
            paused: false,
        }
    }
}

impl AudioPlayer for VLCAudioPlayer {
    fn queue_and_play(&mut self, song: Song) {
        self.current_song = Some(song.clone());
        let tx = self.sender.clone();
        if tx.send(PlayerCommand::Stop).is_ok() {}

        if tx.send(PlayerCommand::Play(song.file.unwrap())).is_ok() {}
    }

    fn volume(&mut self) -> i64 {
        75
    }

    fn volume_up(&mut self) {
        if self.sender.send(PlayerCommand::VolumeUp).is_ok() {};
    }

    fn volume_down(&mut self) {
        if self.sender.send(PlayerCommand::VolumeDown).is_ok() {};
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
        Ok(())
    }

    fn get_progress(&mut self) -> (f64, i64, i64) {
        match self.current_song.clone() {
            Some(song) => {
                if self.sender.send(PlayerCommand::Progress).is_ok() {
                    if let Ok(time_pos) = self.progress_receiver.try_recv() {
                        let duration = song.duration.unwrap_or_else(|| Duration::from_secs(100));
                        let duration_i64 = duration.as_secs() as i64;
                        let percent = time_pos as f64 / duration_i64 as f64;
                        (percent, time_pos, duration_i64)
                    } else {
                        (0.9, 0, 100)
                    }
                } else {
                    (0.9, 0, 100)
                }
            }
            None => (0.9, 0, 100),
        }
    }
}
