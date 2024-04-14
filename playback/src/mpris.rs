use base64::Engine;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use std::sync::mpsc::{self, Receiver};
use termusiclib::track::Track;

use crate::{GeneralPlayer, PlayerCmd, PlayerTimeUnit, PlayerTrait, Status, Volume};

pub struct Mpris {
    controls: MediaControls,
    pub rx: Receiver<MediaControlEvent>,
}

impl Mpris {
    pub fn new(cmd_tx: crate::PlayerCmdSender) -> Self {
        // #[cfg(not(target_os = "windows"))]
        // let hwnd = None;

        // #[cfg(target_os = "windows")]
        // let hwnd = {
        //     use raw_window_handle::windows::WindowsHandle;

        //     let handle: WindowsHandle = unimplemented!();
        //     Some(handle.hwnd)
        // };

        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        #[cfg(target_os = "windows")]
        let (hwnd, _dummy_window) = {
            let dummy_window = windows::DummyWindow::new().unwrap();
            let handle = Some(dummy_window.handle.0 as _);
            (handle, dummy_window)
        };

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
                cmd_tx.send(PlayerCmd::Tick).ok();
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
    pub fn update_progress(&mut self, position: Option<PlayerTimeUnit>, playlist_status: Status) {
        if let Some(position) = position {
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

    /// Update the Volume reported by Media-Controls
    ///
    /// currently only does something on linux (mpris)
    pub fn update_volume(&mut self, volume: Volume) {
        // currently "set_volume" only exists for "linux"(mpris)
        #[cfg(target_os = "linux")]
        {
            // update the reported volume in mpris
            let vol = f64::from(volume) / 100.0;
            let _ = self.controls.set_volume(vol);
        }
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
                self.cmd_tx.send(cmd).ok();
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
                // TODO: handle "OpenUri"
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
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let uvol = (volume.clamp(0.0, 1.0) * 100.0) as u16;
                self.set_volume(uvol);
            }
            MediaControlEvent::Quit => {
                // ignore error if sending failed
                self.cmd_tx.send(PlayerCmd::Quit).ok();
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

        if let Some(progress) = self.get_progress() {
            self.mpris
                .update_progress(progress.position, self.playlist.status());
        }
    }

    /// Update Media-Controls reported volume, if enabled to be reporting
    #[inline]
    pub fn mpris_volume_update(&mut self) {
        if self.config.read().player_use_mpris {
            self.mpris.update_volume(self.volume());
        }
    }
}

// demonstrates how to make a minimal window to allow use of media keys on the command line
// ref: https://github.com/Sinono3/souvlaki/blob/master/examples/print_events.rs
#[cfg(target_os = "windows")]
#[allow(clippy::cast_possible_truncation)]
mod windows {
    use std::io::Error;
    use std::mem;

    use windows::core::w;
    // use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetAncestor,
        IsDialogMessageW, PeekMessageW, RegisterClassExW, TranslateMessage, GA_ROOT, MSG,
        PM_REMOVE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_QUIT, WNDCLASSEXW,
    };

    pub struct DummyWindow {
        pub handle: HWND,
    }

    impl DummyWindow {
        pub fn new() -> Result<DummyWindow, String> {
            let class_name = w!("SimpleTray");

            let handle_result = unsafe {
                let instance = GetModuleHandleW(None)
                    .map_err(|e| (format!("Getting module handle failed: {e}")))?;

                let wnd_class = WNDCLASSEXW {
                    cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                    hInstance: instance.into(),
                    lpszClassName: class_name,
                    lpfnWndProc: Some(Self::wnd_proc),
                    ..Default::default()
                };

                if RegisterClassExW(&wnd_class) == 0 {
                    return Err(format!(
                        "Registering class failed: {}",
                        Error::last_os_error()
                    ));
                }

                let handle = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    class_name,
                    w!(""),
                    WINDOW_STYLE::default(),
                    0,
                    0,
                    0,
                    0,
                    None,
                    None,
                    instance,
                    None,
                );

                if handle.0 == 0 {
                    Err(format!(
                        "Message only window creation failed: {}",
                        Error::last_os_error()
                    ))
                } else {
                    Ok(handle)
                }
            };

            handle_result.map(|handle| DummyWindow { handle })
        }
        extern "system" fn wnd_proc(
            hwnd: HWND,
            msg: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT {
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
    }

    impl Drop for DummyWindow {
        fn drop(&mut self) {
            unsafe {
                DestroyWindow(self.handle).unwrap();
            }
        }
    }

    #[allow(dead_code)]
    pub fn pump_event_queue() -> bool {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            let mut has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
            while msg.message != WM_QUIT && has_message {
                if !IsDialogMessageW(GetAncestor(msg.hwnd, GA_ROOT), &msg).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
            }

            msg.message == WM_QUIT
        }
    }
}
