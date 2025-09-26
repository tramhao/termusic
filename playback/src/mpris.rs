use std::sync::mpsc::{self, Receiver};

use base64::Engine;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use termusiclib::{
    common::const_unknown::{UNKNOWN_ARTIST, UNKNOWN_TITLE},
    track::Track,
};

use crate::{
    GeneralPlayer, PlayerCmd, PlayerProgress, PlayerTimeUnit, PlayerTrait, RunningStatus, Volume,
};

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
            let handle = Some(dummy_window.handle.0);
            (handle, dummy_window)
        };

        let config = PlatformConfig {
            dbus_name: "termusic",
            display_name: "Termusic in Rust",
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

        let cover_art = match track.get_picture() {
            Ok(v) => v.map(|v| {
                format!(
                    "data:{};base64,{}",
                    v.mime_type().map_or_else(
                        || {
                            error!("Unknown mimetype for picture of track {track:#?}");
                            "application/octet-stream"
                        },
                        |v| v.as_str()
                    ),
                    base64::engine::general_purpose::STANDARD_NO_PAD.encode(v.data())
                )
            }),
            Err(err) => {
                error!("Fetching the cover failed: {err:#?}");
                None
            }
        };

        let album = track.as_track().and_then(|v| v.album());

        self.controls
            .set_metadata(MediaMetadata {
                title: Some(track.title().unwrap_or(UNKNOWN_TITLE)),
                artist: Some(track.artist().unwrap_or(UNKNOWN_ARTIST)),
                album: Some(album.unwrap_or("")),
                cover_url: cover_art.as_deref(),
                duration: track.duration(),
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
    pub fn update_progress(
        &mut self,
        position: Option<PlayerTimeUnit>,
        playlist_status: RunningStatus,
    ) {
        if let Some(position) = position {
            match playlist_status {
                RunningStatus::Running => self
                    .controls
                    .set_playback(MediaPlayback::Playing {
                        progress: Some(souvlaki::MediaPosition(position)),
                    })
                    .ok(),
                RunningStatus::Paused | RunningStatus::Stopped => self
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
    #[allow(unused_variables, clippy::unused_self)] // non-linux targets will complain about unused parameters
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
                debug!("got souvlaki SetVolume: {volume:#}");
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

    /// Handle Media-Controls events, if enabled to be used
    pub fn mpris_handle_events(&mut self) {
        if let Some(ref mut mpris) = self.mpris {
            if let Ok(m) = mpris.rx.try_recv() {
                self.mpris_handler(m);
            }
        }
    }

    /// Update Media-Controls reported Position & Status, if enabled to be reporting
    #[inline]
    pub fn mpris_update_progress(&mut self, progress: &PlayerProgress) {
        if let Some(ref mut mpris) = self.mpris {
            mpris.update_progress(progress.position, self.playlist.read_recursive().status());
        }
    }

    /// Update Media-Controls reported volume, if enabled to be reporting
    #[inline]
    pub fn mpris_volume_update(&mut self) {
        let volume = self.volume();
        if let Some(ref mut mpris) = self.mpris {
            mpris.update_volume(volume);
        }
    }
}

// demonstrates how to make a minimal window to allow use of media keys on the command line
// ref: https://github.com/Sinono3/souvlaki/blob/master/examples/print_events.rs
#[cfg(target_os = "windows")]
#[allow(clippy::cast_possible_truncation, unsafe_code)]
mod windows {
    use std::io::Error;
    use std::mem;

    use windows::core::w;
    // use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassExW, WINDOW_EX_STYLE,
        WINDOW_STYLE, WNDCLASSEXW,
    };

    pub struct DummyWindow {
        pub handle: HWND,
    }

    impl DummyWindow {
        pub fn new() -> Result<DummyWindow, String> {
            let class_name = w!("SimpleTray");

            let handle_result = unsafe {
                let instance = GetModuleHandleW(None)
                    .map_err(|e| format!("Getting module handle failed: {e}"))?;

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

                let handle = match CreateWindowExW(
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
                ) {
                    Ok(v) => v,
                    Err(err) => {
                        return Err(format!("{err}"));
                    }
                };

                if handle.is_invalid() {
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

    // #[allow(dead_code)]
    // pub fn pump_event_queue() -> bool {
    //     unsafe {
    //         let mut msg: MSG = std::mem::zeroed();
    //         let mut has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
    //         while msg.message != WM_QUIT && has_message {
    //             if !IsDialogMessageW(GetAncestor(msg.hwnd, GA_ROOT), &msg).as_bool() {
    //                 TranslateMessage(&msg);
    //                 DispatchMessageW(&msg);
    //             }

    //             has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
    //         }

    //         msg.message == WM_QUIT
    //     }
    // }
}
