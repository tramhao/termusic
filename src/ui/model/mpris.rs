use super::Status;
use crate::player::GeneralP;
use crate::song::Song;
// use crate::souvlaki::{
//     MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig,
// };
use crate::ui::model::Model;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
// use std::str::FromStr;
use std::sync::mpsc::{self, Receiver};
// use std::sync::{mpsc, Arc, Mutex};
// use std::thread::{self, JoinHandle};

pub struct Mpris {
    controls: MediaControls,
    pub rx: Receiver<MediaControlEvent>,
}
impl Default for Mpris {
    fn default() -> Self {
        // #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        // #[cfg(target_os = "windows")]
        // let hwnd = {
        //     use raw_window_handle::windows::WindowsHandle;

        //     let handle: WindowsHandle = unimplemented!();
        //     Some(handle.hwnd)
        // };

        let config = PlatformConfig {
            dbus_name: "termusic",
            display_name: "Termuisc in Rust",
            hwnd,
        };

        let mut controls = MediaControls::new(config).unwrap();

        let (tx, rx) = mpsc::sync_channel(32);
        // The closure must be Send and have a static lifetime.
        controls
            .attach(move |event: MediaControlEvent| {
                tx.send(event).ok();
            })
            .ok();

        Self { controls, rx }
    }
}

impl Mpris {
    pub fn add_and_play(&mut self, song_str: &str) {
        if let Ok(song) = Song::read_from_path(song_str) {
            self.controls
                .set_metadata(MediaMetadata {
                    title: Some(song.title().unwrap_or("Unknown Title")),
                    artist: Some(song.artist().unwrap_or("Unknown Artist")),
                    album: Some(song.album().unwrap_or("")),
                    ..MediaMetadata::default()
                })
                .ok();
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
                    self.status = Status::Running;
                    self.player.resume();
                } else {
                    self.status = Status::Paused;
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
