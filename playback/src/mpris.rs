use std::{
    sync::mpsc::{self, Receiver},
    time::Duration,
};

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
        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        #[cfg(target_os = "windows")]
        let hwnd = {
            let dummy_window =
                windows::DummyWindow::new().expect("Failed to create Windows Dummy window");
            Some(dummy_window.handle.0)
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
    /// Set Mpris metadata based on the given track.
    pub fn set_track(&mut self, track: &Track) {
        let cover_art = match track.get_picture() {
            Ok(v) => v.and_then(|v| {
                let mut path = std::env::temp_dir();
                path.push("termusic-cover");
                if let Some(mime) = v.mime_type()
                    && let Some(ext) = mime.ext()
                {
                    path.set_extension(ext);
                }
                std::fs::write(&path, v.data())
                    .inspect_err(|e| error!("Saving cover to file failed: {e}"))
                    .ok()?;
                Some(format!("file://{}", path.display()))
            }),
            Err(err) => {
                error!("Fetching the cover failed: {err:#?}");
                None
            }
        };

        let album = track.as_track().and_then(|v| v.album());

        if let Err(err) = self.controls.set_metadata(MediaMetadata {
            title: Some(track.title().unwrap_or(UNKNOWN_TITLE)),
            artist: Some(track.artist().unwrap_or(UNKNOWN_ARTIST)),
            album: Some(album.unwrap_or("")),
            cover_url: cover_art.as_deref(),
            duration: track.duration(),
        }) {
            error!("Error setting MPRIS metadata: {err}");
        }
    }

    /// Set the MPRIS metadata to display that playback is paused.
    pub fn pause(&mut self, position: Option<Duration>) {
        self.controls
            .set_playback(MediaPlayback::Paused {
                progress: position.map(souvlaki::MediaPosition),
            })
            .ok();
    }

    /// Set the MPRIS metadata to display that playback is playing.
    pub fn resume(&mut self, position: Option<Duration>) {
        self.controls
            .set_playback(MediaPlayback::Playing {
                progress: position.map(souvlaki::MediaPosition),
            })
            .ok();
    }

    /// Set the MPRIS metadata to display that playback is stopped.
    pub fn stop(&mut self) {
        self.controls.set_playback(MediaPlayback::Stopped).ok();
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
                RunningStatus::Paused => self
                    .controls
                    .set_playback(MediaPlayback::Paused {
                        progress: Some(souvlaki::MediaPosition(position)),
                    })
                    .ok(),
                RunningStatus::Stopped => self.controls.set_playback(MediaPlayback::Stopped).ok(),
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
                let as_secs = duration.as_secs().min(i64::MAX as u64).cast_signed();

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
                // convert a 0.0 to 1.0 range to 0 to 100, because that is what termusic uses for volume.
                let clamp_100 = volume.clamp(0.0, 1.0) * 100.0;
                // Default float to int casting will truncate values to the decimal point,
                // so we round instead.
                // For example "volume: 0.5000" plus "playerctl volume 0.1+" results in "0.15000", but doing
                // "playerctl volume 0.1-" now, results in "0.49999", so if we dont round we would get "Volume: 4".
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let uvol = clamp_100.round() as u16;
                // ignore error if sending failed
                self.cmd_tx.send(PlayerCmd::VolumeSet(uvol)).ok();
            }
            MediaControlEvent::Quit => {
                // ignore error if sending failed
                let _ = self
                    .cmd_tx
                    .send(PlayerCmd::Quit(crate::quit_sources::MPRIS));
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
        if let Some(ref mut mpris) = self.mpris
            && let Ok(m) = mpris.rx.try_recv()
        {
            self.mpris_handler(m);
        }
    }

    /// Update Media-Controls reported Position & Status, if enabled to be reporting
    #[inline]
    pub fn mpris_update_progress(&mut self, progress: &PlayerProgress) {
        if let Some(ref mut mpris) = self.mpris {
            mpris.update_progress(progress.position, self.run_info.read().status());
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

// ref: https://github.com/Sinono3/souvlaki/blob/master/examples/print_events.rs
#[cfg(target_os = "windows")]
#[allow(clippy::cast_possible_truncation, unsafe_code)]
mod windows {
    use std::io::Error;
    use std::mem::size_of;

    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, RegisterClassExW, WINDOW_EX_STYLE, WINDOW_STYLE,
        WNDCLASSEXW,
    };
    use windows::core::w;

    /// A Dummy Window HWND Handle for souvlaki.
    ///
    /// Note that this window is invisible and never dropped until termusic quits.
    pub struct DummyWindow {
        pub handle: HWND,
    }

    impl DummyWindow {
        /// Create a Dummy Window for souvlaki as `ISystemMediaTransportControlsInterop`'s only way to access the Media Controls requires a window / HWND.
        pub fn new() -> Result<DummyWindow, String> {
            let class_name = w!("SimpleTray");

            let handle_result = unsafe {
                let instance = GetModuleHandleW(None)
                    .map_err(|e| format!("Getting module handle failed: {e}"))?;

                let wnd_class = WNDCLASSEXW {
                    cbSize: size_of::<WNDCLASSEXW>() as u32,
                    hInstance: instance.into(),
                    lpszClassName: class_name,
                    lpfnWndProc: Some(Self::wnd_proc),
                    ..Default::default()
                };

                if RegisterClassExW(&raw const wnd_class) == 0 {
                    return Err(format!(
                        "Registering class failed: {}",
                        Error::last_os_error()
                    ));
                }

                let handle = match CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    class_name,
                    w!("Termusic MediaControl Dummy window"),
                    WINDOW_STYLE::default(),
                    0,
                    0,
                    0,
                    0,
                    None,
                    None,
                    Some(instance.into()),
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

            // The following does not seem to be necessary
            // std::thread::Builder::new()
            //     .name("Windows Message Pump".to_string())
            //     .spawn(pump_event_queue)
            //     .expect("Expected to start the windows message queue pump");

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

    // Dont drop the window until termusic exists, as otherwise Media Controls will silently fail after dropping the handle
    // impl Drop for DummyWindow {
    //     fn drop(&mut self) {
    //         unsafe {
    //             DestroyWindow(self.handle).unwrap();
    //         }
    //     }
    // }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
pub mod macos {
    //! macOS `AppKit` integration for media key support.
    //!
    //! `souvlaki`'s macOS backend registers `MPRemoteCommandCenter` handlers for
    //! media key events, but macOS delivers those events through `AppKit`'s main run
    //! loop. Rust CLI binaries don't initialize an `NSApplication` run loop by
    //! default, so the registered handlers never receive events.
    //!
    //! Both [`init_macos_main_thread`] and [`pump_run_loop`] must be called from
    //! the main thread. Apple's [`NSApplicationMain(_:_:)`] documentation
    //! explicitly states: *"You must call this function from the main thread of
    //! your application."* The Thread Safety Summary also notes that the main
    //! thread is *"the one blocked in the `run` method of `NSApplication`"* and
    //! that `NSRunLoop` is not thread-safe ([Thread Safety Summary]). In
    //! practice, `MPRemoteCommandCenter` dispatches callbacks via GCD's main
    //! queue which executes on the main thread, so event delivery breaks without
    //! the main thread pumping the run loop.
    //!
    //! [`NSApplicationMain(_:_:)`]: https://developer.apple.com/documentation/appkit/nsapplicationmain(_:_:)/
    //! [Thread Safety Summary]: https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Multithreading/ThreadSafetySummary/ThreadSafetySummary.html#//apple_ref/doc/uid/10000057i-CH12-SW1
    //!
    //! This module provides:
    //! - [`init_macos_main_thread`]: Initialize `NSApplication` (no Dock icon).
    //! - [`pump_run_loop`]: Pump the main run loop so `AppKit` can dispatch events.
    //! - [`run_with_run_loop`]: Convenience: spawn a closure on a background thread
    //!   while pumping the run loop on the main thread.

    use std::time::Duration;

    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject};

    /// `NSApplicationActivationPolicy.accessory` — run as a background process
    /// with no Dock icon, but still able to receive media key events.
    const NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY: i64 = 1;

    /// Initialize the macOS `AppKit` application on the main thread.
    ///
    /// Must be called from the main thread (the only thread that can run
    /// `AppKit` per Apple's [Thread Safety Summary](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Multithreading/ThreadSafetySummary/ThreadSafetySummary.html))
    /// before any `souvlaki`
    /// `MediaControls` are attached. Sets the activation policy to
    /// `.accessory`, meaning the app appears as a background process (no Dock
    /// icon) but can still receive media key events via
    /// `MPRemoteCommandCenter`.
    ///
    /// # Panics
    ///
    /// Panics if the `NSApplication` class is not available at runtime (should
    /// never happen on a real macOS system).
    pub fn init_macos_main_thread() {
        unsafe {
            let cls = AnyClass::get(c"NSApplication").expect("NSApplication class not found");
            let app: *mut AnyObject = msg_send![cls, sharedApplication];
            let _: bool =
                msg_send![app, setActivationPolicy: NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY];
        }
    }

    /// Pump the main `CFRunLoop` until the sender is dropped or a message is
    /// received (i.e. the background thread finished).
    ///
    /// Each iteration processes any pending events via `runUntilDate:` with
    /// `distantPast` (non-blocking), then sleeps for 50 ms. This loop is
    /// intended to run on the main thread while the Tokio runtime executes on a
    /// background thread. When the background thread completes (or panics), the
    /// sender is dropped and `recv_timeout` returns `Err`, causing this function
    /// to return.
    ///
    /// # Panics
    ///
    /// Panics if the `NSRunLoop` or `NSDate` class is not available at runtime
    /// (should never happen on a real macOS system).
    pub fn pump_run_loop(done: &std::sync::mpsc::Receiver<()>) {
        let rl_cls = AnyClass::get(c"NSRunLoop").expect("NSRunLoop class not found on macOS");
        let date_cls = AnyClass::get(c"NSDate").expect("NSDate class not found on macOS");
        loop {
            match done.recv_timeout(Duration::from_millis(50)) {
                Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            }
            unsafe {
                let rl: *mut AnyObject = msg_send![rl_cls, mainRunLoop];
                let distant_past: *mut AnyObject = msg_send![date_cls, distantPast];
                let _: () = msg_send![rl, runUntilDate: distant_past];
            }
        }
    }

    /// Run a closure on a background thread while pumping the `AppKit` run loop
    /// on the main thread.
    ///
    /// This is the simplest way to use the macOS media key support: call this
    /// from `main()`, passing a closure that sets up and runs the Tokio runtime.
    /// The closure runs on a background thread named "termusic-tokio", and the
    /// main thread pumps `CFRunLoop` until the closure returns. `AppKit` and
    /// `CFRunLoop` must run on the main thread per Apple's
    /// [Thread Safety Summary](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Multithreading/ThreadSafetySummary/ThreadSafetySummary.html).
    ///
    /// # Panics
    ///
    /// Panics if the background thread cannot be spawned or panics.
    pub fn run_with_run_loop<T>(f: impl FnOnce() -> T + Send + 'static) -> T
    where
        T: Send + 'static,
    {
        init_macos_main_thread();

        let (tx, rx) = std::sync::mpsc::channel::<()>();

        let handle = std::thread::Builder::new()
            .name("termusic-tokio".into())
            .spawn(move || {
                let result = f();
                let _ = tx.send(());
                result
            })
            .expect("failed to spawn termusic-tokio thread");

        pump_run_loop(&rx);
        handle.join().expect("termusic-tokio thread panicked")
    }
}
