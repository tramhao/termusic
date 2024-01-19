use base64::Engine;
use parking_lot::Mutex;
use termusiclib::track::Track;
use tokio::sync::mpsc::UnboundedSender;
// use crate::souvlaki::{
//     MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig,
// };

use crate::{GeneralPlayer, PlayerCmd};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
// use std::str::FromStr;
use std::sync::{
    mpsc::{self, Receiver},
    Arc,

};
// use std::sync::{mpsc, Arc, Mutex};
// use std::thread::{self, JoinHandle};

pub struct Mpris {
    controls: MediaControls,
    pub rx: Receiver<MediaControlEvent>,
}
impl Mpris {
    pub fn new(cmd_tx: Arc<Mutex<UnboundedSender<PlayerCmd>>>) -> Self {
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
                // immediately process any mpris commands, current update is inside PlayerCmd::Tick
                // TODO: this should likely be refactored
                cmd_tx.lock().send(PlayerCmd::Tick).ok();
            })
            .ok();

        Self { controls, rx }
    }
}

impl Mpris {
    pub fn add_and_play(&mut self, track: &Track) {
        // This is to fix a bug that the first track is not updated
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.controls
            .set_playback(MediaPlayback::Playing { progress: None })
            .ok();

        let cover_art = track.picture().map(|picture| {
            format!(
                "data:{};base64,{}",
                picture.mime_type().as_str(),
                base64::engine::general_purpose::STANDARD_NO_PAD.encode(picture.data())
            )
        });

        self.controls
            .set_metadata(MediaMetadata {
                title: Some(track.title().unwrap_or("Unknown Title")),
                artist: Some(track.artist().unwrap_or("Unknown Artist")),
                album: Some(track.album().unwrap_or("")),
                cover_url: cover_art.as_deref(),
                duration: Some(track.duration()),
            })
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

    /// Update Track position / progress, requires `playlist_status` because [`MediaControls`] only allows `set_playback`, not `set_position` or `get_playback`
    pub fn update_progress(&mut self, position: i64, playlist_status: Status) {
        // safe cast because of "max(0)"
        #[allow(clippy::cast_sign_loss)]
        let position = Duration::from_secs(position.max(0) as u64);
        match playlist_status {
            Status::Running => self
                .controls
                .set_playback(MediaPlayback::Playing {
                    progress: Some(souvlaki::MediaPosition(position)),
                })
                .ok(),
            Status::Paused | Status::Stopped => self
                .controls
                .set_playback(MediaPlayback::Paused {
                    progress: Some(souvlaki::MediaPosition(position)),
                })
                .ok(),
        };
    }
}

impl GeneralPlayer {
    pub fn mpris_handler(&mut self, e: MediaControlEvent) {
        match e {
            MediaControlEvent::Next => {
                self.next();
            }
            MediaControlEvent::Previous => {
                self.previous();
            }
            MediaControlEvent::Pause => {
                self.pause();
            }
            MediaControlEvent::Toggle => {
                self.toggle_pause();
            }
            MediaControlEvent::Play => {
                self.play();
            }
            // MediaControlEvent::Seek(x) => match x {
            //     SeekDirection::Forward => activity.player.seek(5).ok(),
            //     SeekDirection::Backward => activity.player.seek(-5).ok(),
            // },
            // MediaControlEvent::SetPosition(position) => {
            //     let _position = position. / 1000;
            // }
            MediaControlEvent::OpenUri(_uri) => {
                // let wait = async {
                //     self.player.add_and_play(&uri).await;
                // };
                // let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
                // rt.block_on(wait);
            }
            _ => {}
        }
    }

    pub fn update_mpris(&mut self) {
        if let Ok(m) = self.mpris.rx.try_recv() {
            self.mpris_handler(m);
        }

        self.mpris.update_progress(
            self.get_progress().ok().map_or(0, |v| v.0),
            self.playlist.status(),
        );
    }
}
