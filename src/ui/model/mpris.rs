use super::Status;
use crate::song::Song;
use crate::souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig,
};
use crate::ui::model::Model;
use std::str::FromStr;
use std::sync::mpsc::{self, Receiver};
// use std::sync::{mpsc, Arc, Mutex};
// use std::thread::{self, JoinHandle};

pub struct Mpris {
    controls: MediaControls,
    pub rx: Receiver<MediaControlEvent>,
}
impl Default for Mpris {
    fn default() -> Self {
        let config = PlatformConfig {
            dbus_name: "termusic",
            display_name: "Termuisc in Rust",
        };

        let mut controls = MediaControls::new(config);

        let (tx, rx) = mpsc::sync_channel(32);
        // The closure must be Send and have a static lifetime.
        controls
            .attach(move |event: MediaControlEvent| {
                tx.send(event).ok();
            })
            .unwrap();

        Self { controls, rx }
    }
}

impl Mpris {
    pub fn add_and_play(&mut self, song_str: &str) {
        if let Ok(song) = Song::from_str(song_str) {
            self.controls.set_metadata(MediaMetadata {
                title: Some(song.title().unwrap_or("Unknown Title")),
                artist: Some(song.artist().unwrap_or("Unknown Artist")),
                album: Some(song.album().unwrap_or("")),
                ..MediaMetadata::default()
            });
        }
        self.controls
            .set_playback(MediaPlayback::Playing { progress: None })
            .ok();
    }
    pub fn pause(&mut self) {
        self.controls
            .set_playback(MediaPlayback::Paused { progress: None })
            .ok();
    }
    pub fn resume(&mut self) {
        self.controls
            .set_playback(MediaPlayback::Playing { progress: None })
            .ok();
    }
}

impl Model {
    pub fn mpris_handler(&mut self, e: MediaControlEvent) {
        match e {
            MediaControlEvent::Next => {
                self.player_next();
            }
            MediaControlEvent::Previous => {
                self.player_previous();
            }
            MediaControlEvent::Pause => {
                self.player.pause();
            }
            MediaControlEvent::Toggle => {
                if self.player.is_paused() {
                    self.status = Some(Status::Running);
                    self.player.resume();
                } else {
                    self.status = Some(Status::Paused);
                    self.player.pause();
                }
            }
            MediaControlEvent::Play => {
                self.player.resume();
            }
            // MediaControlEvent::Seek(x) => match x {
            //     SeekDirection::Forward => activity.player.seek(5).ok(),
            //     SeekDirection::Backward => activity.player.seek(-5).ok(),
            // },
            // MediaControlEvent::SetPosition(position) => {
            //     let _position = position. / 1000;
            // }
            MediaControlEvent::OpenUri(uri) => {
                self.player.add_and_play(&uri);
            }
            _ => {}
        }
    }

    pub fn update_mpris(&mut self) {
        if let Ok(m) = self.mpris.rx.try_recv() {
            self.mpris_handler(m);
        }
    }
}
