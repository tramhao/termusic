use base64::Engine;
use termusiclib::track::Track;
// use crate::souvlaki::{
//     MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig,
// };

use crate::{GeneralPlayer, PlayerCmd, PlayerTrait, Status};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
// use std::str::FromStr;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;
// use std::sync::{mpsc, Arc, Mutex};
// use std::thread::{self, JoinHandle};

pub struct Mpris {
    controls: MediaControls,
    pub rx: Receiver<MediaControlEvent>,
}
impl Mpris {
    pub fn new(cmd_tx: crate::PlayerCmdSender) -> Self {
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
                picture.mime_type().map_or_else(
                    || {
                        error!(
                            "Unknown mimetype for picture of track {}",
                            track.file().unwrap_or("<unknown file>")
                        );
                        "application/octet-stream"
                    },
                    |v| v.as_str()
                ),
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
            // The "Seek" even seems to currently only be used for windows, mpris uses "SeekBy"
            MediaControlEvent::Seek(direction) => {
                let cmd = match direction {
                    souvlaki::SeekDirection::Forward => PlayerCmd::SeekForward,
                    souvlaki::SeekDirection::Backward => PlayerCmd::SeekBackward,
                };

                // ignore error if sending failed
                self.cmd_tx.lock().send(cmd).ok();
            }
            MediaControlEvent::SetPosition(position) => {
                self.seek_to(position.0);
            }
            MediaControlEvent::OpenUri(_uri) => {
                // let wait = async {
                //     self.player.add_and_play(&uri).await;
                // };
                // let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
                // rt.block_on(wait);
                // TODO: handle "Seek"
                info!("Unimplemented Event: OpenUri");
            }
            MediaControlEvent::SeekBy(direction, duration) => {
                #[allow(clippy::cast_possible_wrap)]
                let as_secs = duration.as_secs().min(i64::MAX as u64) as i64;

                // mpris seeking is in micro-seconds (not milliseconds or seconds)
                if as_secs == 0 {
                    warn!("can only seek in seconds, got less than 0 seconds");
                    return;
                }

                let offset = match direction {
                    souvlaki::SeekDirection::Forward => as_secs,
                    souvlaki::SeekDirection::Backward => -as_secs,
                };

                // make use of "PlayerTrait" impl on "GeneralPlayer"
                // ignore result
                let _ = self.seek(offset);
            }
            MediaControlEvent::SetVolume(volume) => {
                debug!("got souvlaki SetVolume: {:#}", volume);
                // volume can be anything above 0; 1.0 means a sensible max; termusic currently does not support more than 100 volume
                // warn users trying to set higher than max via logging
                if volume > 1.0 {
                    error!("SetVolume above 1.0 will be clamped to 1.0!");
                }
                // convert a 0.0 to 1.0 range to 0 to 100, because that is what termusic uses for volume
                // default float to int casting will truncate values to the decimal point
                #[allow(clippy::cast_possible_truncation)]
                let uvol = (volume.clamp(0.0, 1.0) * 100.0) as i32;
                self.set_volume(uvol);
            }
            MediaControlEvent::Quit => {
                // ignore error if sending failed
                self.cmd_tx.lock().send(PlayerCmd::Quit).ok();
            }
            MediaControlEvent::Stop => {
                // TODO: handle "Stop"
                info!("Unimplemented Event: Stop");
            }
            // explicitly unsupported events
            MediaControlEvent::Raise => {}
        }
    }

    pub fn update_mpris(&mut self) {
        if let Ok(m) = self.mpris.rx.try_recv() {
            self.mpris_handler(m);
        }

        // currently "set_volume" only exists for "linux"(mpris)
        #[cfg(target_os = "linux")]
        {
            // update the reported volume in mpris
            let vol = f64::from(self.volume()) / 100.0;
            self.mpris.controls.set_volume(vol).ok();
        }

        self.mpris.update_progress(
            self.get_progress().ok().map_or(0, |v| v.0),
            self.playlist.status(),
        );
    }
}
