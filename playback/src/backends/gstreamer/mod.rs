use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use glib::value::FromValue;
use glib::{ControlFlow, FlagsClass};
use gstreamer::bus::BusWatchGuard;
use gstreamer::prelude::*;
use gstreamer::{self as gst, ResourceError, StreamError};
use gstreamer::{ClockTime, StateChangeError, StateChangeSuccess};
use gstreamer::{Element, SeekFlags, SeekType, event::Seek};
use parking_lot::Mutex;
use termusiclib::config::ServerOverlay;
use termusiclib::track::{MediaTypes, Track};
use tokio::sync::mpsc;

use crate::{MediaInfo, PlayerCmd, PlayerErrorType, PlayerProgress, PlayerTrait, Speed, Volume};

/// This trait allows for easy conversion of a path to a URI for gstreamer
trait PathToURI {
    fn to_uri(&self) -> String;
}

impl PathToURI for Path {
    /// Returns `self` as a URI. Panics in case of an error.
    fn to_uri(&self) -> String {
        glib::filename_to_uri(self, None)
            .expect("Error converting path to URI")
            .to_string()
    }
}

/// Wrapper for a playbin gst element with common functions and some type safety
#[derive(Debug, Clone)]
struct PlaybinWrap(Element);

impl PlaybinWrap {
    #[inline]
    fn new(playbin: Element) -> Self {
        Self(playbin)
    }

    /// Send a Seek Event that sets the speed.
    ///
    /// Returns `false` if sending the event failed or was not handled, otherwise `true` if handled
    fn set_speed(&self, speed: i32) -> bool {
        let rate = f64::from(speed) / 10.0;
        // Obtain the current position, needed for the seek event
        let position = self.get_position();

        // Create the seek event
        let seek_event = if rate > 0. {
            Seek::new(
                rate,
                SeekFlags::FLUSH | SeekFlags::ACCURATE,
                SeekType::Set,
                position,
                SeekType::None,
                position,
            )
        } else {
            Seek::new(
                rate,
                SeekFlags::FLUSH | SeekFlags::ACCURATE,
                SeekType::Set,
                position,
                SeekType::Set,
                position,
            )
        };

        // If we have not done so, obtain the sink through which we will send the seek events
        if let Some(sink) = self.0.property::<Option<Element>>("audio-sink") {
            // Send the event
            let send_event = sink.send_event(seek_event);
            if !send_event {
                warn!("Speed event was *NOT* handled!");
            }
            send_event
        } else {
            false
        }
    }

    /// Get a abitrary property.
    #[inline]
    fn get_prop<V: for<'b> FromValue<'b> + 'static>(&self, key: &str) -> V {
        self.0.property(key)
    }

    #[inline]
    fn get_position(&self) -> Option<ClockTime> {
        self.0.query_position::<ClockTime>()
    }

    #[inline]
    fn get_duration(&self) -> Option<ClockTime> {
        self.0.query_duration::<ClockTime>()
    }

    #[inline]
    fn set_volume(&self, volume: f64) {
        self.0.set_property("volume", volume);
    }

    #[inline]
    fn set_uri(&self, url: impl Into<glib::Value>) {
        self.0.set_property("uri", url);
    }

    // #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    // pub fn get_buffer_duration(&self) -> u32 {
    //     self.0.property::<i64>("buffer-duration") as u32
    // }

    #[inline]
    fn set_state(&self, state: gst::State) -> Result<StateChangeSuccess, StateChangeError> {
        self.0.set_state(state)
    }

    #[inline]
    fn current_state(&self) -> gst::State {
        self.0.current_state()
    }

    #[inline]
    fn seek_simple(
        &self,
        seek_flags: gst::SeekFlags,
        seek_pos: impl FormattedValue,
    ) -> Result<(), glib::BoolError> {
        self.0.seek_simple(seek_flags, seek_pos)
    }

    #[inline]
    fn connect_about_to_finish<F>(&self, cb: F)
    where
        F: Fn(&[glib::Value]) -> Option<glib::Value> + Send + Sync + 'static,
    {
        self.0.connect("about-to-finish", false, cb);
    }

    #[inline]
    fn pause(&self) -> Result<StateChangeSuccess, StateChangeError> {
        self.0.set_state(gst::State::Paused)
    }

    #[inline]
    fn play(&self) -> Result<StateChangeSuccess, StateChangeError> {
        self.0.set_state(gst::State::Playing)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerInternalCmd {
    Eos,
    Error(PlayerErrorType),
    AboutToFinish,
    SkipNext,
    ReloadSpeed,
    MetadataChanged,
}

pub struct GStreamerBackend {
    playbin: PlaybinWrap,
    volume: u16,
    speed: i32,
    gapless: bool,
    icmd_tx: mpsc::Sender<PlayerInternalCmd>,
    media_title: Arc<Mutex<String>>,
    _bus_watch_guard: BusWatchGuard,
}

impl GStreamerBackend {
    #[allow(clippy::too_many_lines)]
    pub fn new(config: &ServerOverlay, cmd_tx: crate::PlayerCmdSender) -> Self {
        gst::init().expect("Couldn't initialize Gstreamer");
        let ctx = glib::MainContext::default();
        let _guard = ctx.acquire();
        let mainloop = glib::MainLoop::new(Some(&ctx), false);

        // store whether a "cmd_tx Eos" signal has been send already for the last track
        // this is necessary because gstreamer seemingly only sends a EOS when nothing new is queued up when "gapless" is enabled
        // if "gapless" is disabled, gstreamer will send a EOS correctly
        // false = not had EOS; true = had EOS
        // starting with a "true", because there was no previous stream
        let eos_watcher: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));

        let eos_watcher_clone = eos_watcher.clone();

        // store whether a "About to finish" message had already been sent, to only give one instead spamming
        // but they need to be reset on many occasions, like seek or stream start
        let send_atf_watcher: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let send_atf_watcher_clone = send_atf_watcher.clone();

        // Asynchronous channel to communicate internal events
        let (icmd_tx, icmd_rx) = mpsc::channel(3);

        tokio::spawn(async move {
            Self::channel_proxy_task(
                icmd_rx,
                &cmd_tx,
                &eos_watcher_clone,
                &send_atf_watcher_clone,
            )
            .await;
        });

        let playbin = Box::new(gst::ElementFactory::make("playbin3"))
            .build()
            .expect("playbin3 make error");

        let tempo = gst::ElementFactory::make("scaletempo")
            .name("tempo")
            .build()
            .expect("make scaletempo error");

        let sink = gst::ElementFactory::make("autoaudiosink")
            .name("audiosink")
            .build()
            .expect("make audio sink error");

        let bin = gst::Bin::with_name("audiosink");
        bin.add_many([&tempo, &sink]).expect("add many failed");
        gst::Element::link_many([&tempo, &sink]).expect("link many failed");
        tempo.sync_state_with_parent().expect("sync state failed");

        let pad = tempo
            .static_pad("sink")
            .expect("Failed to get a static pad from equalizer.");

        let ghost_pad = gst::GhostPad::with_target(&pad).expect("make ghost_pad failed");

        ghost_pad
            .set_active(true)
            .expect("ghostpad set active failed");
        bin.add_pad(&ghost_pad).expect("bin add pad failed");
        playbin.set_property("audio-sink", &bin);

        // Set flags to show Audio and Video but ignore Subtitles
        let flags = playbin.property_value("flags");
        let flags_class = FlagsClass::with_type(flags.type_()).unwrap();

        let flags = flags_class
            .builder_with_value(flags)
            .unwrap()
            .set_by_nick("audio")
            .set_by_nick("download")
            .unset_by_nick("video")
            .unset_by_nick("text")
            .build()
            .unwrap();
        playbin.set_property_from_value("flags", &flags);

        // Handle messages from GStreamer bus

        let media_title = Arc::new(Mutex::new(String::new()));
        let media_title_internal = media_title.clone();
        let playbin = PlaybinWrap::new(playbin);
        let playbin_clone = playbin.clone();
        let main_tx_watcher = icmd_tx.clone();
        // deduplicate errors, as gstreamer spams a bunch of the same error; stores the current-uri
        let error_watcher: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let bus_watch = playbin
            .0
            .bus()
            .expect("Failed to get GStreamer message bus")
            .add_watch(move |_bus, msg| {
                Self::watch_fn(
                    msg,
                    &playbin_clone,
                    &main_tx_watcher,
                    &eos_watcher,
                    &send_atf_watcher,
                    &media_title_internal,
                    &error_watcher,
                )
            })
            .expect("Failed to connect to GStreamer message bus");

        // extra thread to run the glib mainloop on
        std::thread::Builder::new()
            .name("gst glib mainloop".into())
            .spawn(move || {
                mainloop.run();
            })
            .expect("failed to start gstreamer mainloop thread");

        let volume = config.settings.player.volume;
        let speed = config.settings.player.speed;
        let gapless = config.settings.player.gapless;
        let icmd_tx_c = icmd_tx.clone();

        let mut this = Self {
            playbin,
            volume,
            speed,
            gapless,
            icmd_tx: icmd_tx_c,
            media_title,
            _bus_watch_guard: bus_watch,
        };

        this.set_volume(volume);
        // this.set_speed(speed);

        // Send a signal to enqueue the next media before the current finished
        this.playbin.connect_about_to_finish(move |_| {
            debug!("Sending playbin AboutToFinish");
            icmd_tx
                .blocking_send(PlayerInternalCmd::AboutToFinish)
                .unwrap();
            None
        });

        this
    }

    /// Check the given events(Messages) and process them
    fn watch_fn(
        msg: &gst::Message,
        playbin: &PlaybinWrap,
        main_tx: &mpsc::Sender<PlayerInternalCmd>,
        eos_watcher: &Arc<AtomicBool>,
        send_atf_watcher: &Arc<AtomicBool>,
        media_title: &Arc<Mutex<String>>,
        error_watcher: &Arc<Mutex<Option<String>>>,
    ) -> ControlFlow {
        match msg.view() {
            gst::MessageView::Eos(_) => {
                // debug tracking, as gapless interferes with it
                debug!("gstreamer message EOS");
                main_tx
                    .blocking_send(PlayerInternalCmd::Eos)
                    .expect("Unable to send message to main()");
                eos_watcher.store(true, std::sync::atomic::Ordering::SeqCst);

                // clear stored title on end
                media_title.lock().clear();
                // let _ = main_tx.blocking_send(PlayerInternalCmd::MetadataChanged);
            }
            gst::MessageView::StreamStart(_e) => {
                if !eos_watcher.load(std::sync::atomic::Ordering::SeqCst) {
                    trace!("Sending EOS because it was not sent since last StreamStart");
                    main_tx
                        .blocking_send(PlayerInternalCmd::Eos)
                        .expect("Unable to send message to main()");
                }

                send_atf_watcher.store(false, Ordering::SeqCst);
                eos_watcher.store(false, std::sync::atomic::Ordering::SeqCst);

                // clear stored title on stream start (should work without conflicting in ::Tag)
                media_title.lock().clear();
                // let _ = main_tx.blocking_send(PlayerInternalCmd::MetadataChanged);

                // HACK: gstreamer does not handle seek events before some undocumented time, see other note in main_rx handler
                let _ = main_tx.blocking_send(PlayerInternalCmd::ReloadSpeed);
            }
            gst::MessageView::Error(e) => {
                let err = e.error();
                error!("GStreamer Error: {err:#?}");

                // KNOWN ISSUES:
                // This implementation does not work when working with enqueuement, due to gstreamer somehow re-setting "uri" to "current-uri"
                // instead of still being the error-ing manually set "uri"; this means that we dont know if the currently still playing
                // track had a error (like decode or something) and should be aborted, or if it was because the new track that was enqueued
                // had a error and should be skipped instead. Additionally, when this happens and we implement outside "next-uri" tracking, "EOS" will
                // never get triggered for whatever reason. And finally, Errors themself contain no context about what triggered it.
                // This effectively means that playback with gst will get stuck on "about to finish" enqueuement if the next track is something like "NotFound".

                // https://gstreamer.freedesktop.org/documentation/gstreamer/gsterror.html?gi-language=c#GstResourceError
                // this has to be done this way instead of a simple "matches!" call, due to glib not providing a way to get the "code"
                if err.matches(ResourceError::NotFound)
                    || err.matches(StreamError::Decode)
                    || err.matches(StreamError::Demux)
                    || err.matches(StreamError::CodecNotFound)
                {
                    let current_uri: String = playbin.get_prop("current-uri");
                    let next_uri: String = playbin.get_prop("uri");
                    trace!("current uri: {current_uri:#?}; next uri: {next_uri:#?}");

                    // only send a error event if both uri's match as otherwise it is unclear which uri the error is for
                    // though note that at least with gstreamer@0.23.5 they are basically always the same in this path for some reason
                    if current_uri == next_uri {
                        let mut lock = error_watcher.lock();

                        // only send a error event if none for the current uri have already been sent
                        if lock.as_ref().is_none_or(|v| v != &current_uri) {
                            info!("Recoverable Error, sending Event");
                            eos_watcher.store(true, std::sync::atomic::Ordering::SeqCst);

                            let _ = main_tx
                                .blocking_send(PlayerInternalCmd::Error(PlayerErrorType::Current));

                            *lock = Some(current_uri);
                        }
                    }
                }
            }
            gst::MessageView::Tag(tag) => {
                if let Some(title) = tag.tags().get::<gst::tags::Title>() {
                    info!("Title: {}", title.get());
                    *media_title.lock() = title.get().into();
                    let _ = main_tx.blocking_send(PlayerInternalCmd::MetadataChanged);
                }
                // if let Some(artist) = tag.tags().get::<gst::tags::Artist>() {
                //     info!("Artist: {}", artist.get());
                //     // *media_title.lock() = artist.get().to_string();
                // }
                // if let Some(album) = tag.tags().get::<gst::tags::Album>() {
                //     info!("Album: {}", album.get());
                //     // *media_title.lock() = album.get().to_string();
                // }
            }
            gst::MessageView::Buffering(buffering) => {
                // let (mode,_, _, left) = buffering.buffering_stats();
                // info!("mode is: {mode:?}, and left is: {left}");
                let percent = buffering.percent();
                // according to the documentation, the application (we) need to set the playbin state according to the buffering state
                // see https://gstreamer.freedesktop.org/documentation/playback/playbin.html?gi-language=c#buffering
                if percent < 100 {
                    let _ = playbin.pause();
                } else {
                    let _ = playbin.play();
                }
                // Left for debug
                // let msg = buffering.message();
                // info!("message is: {msg:?}");
            }
            gst::MessageView::Warning(warning) => {
                info!("GStreamer Warning: {}", warning.error());
            }
            // Left for debug
            // msg => {
            //     info!("msg: {msg:?}");
            // }
            _ => (),
        }
        glib::ControlFlow::Continue
    }

    /// Code for the Event Proxy Task / Thread.
    ///
    /// This extra event proxy is necessary as the playbin itself may need to some extra events done (like `ReloadSpeed`)
    /// or set some extra values like on `SkipNext`.
    ///
    /// TODO: This extra proxy could likely be avoided.
    async fn channel_proxy_task(
        mut main_rx: mpsc::Receiver<PlayerInternalCmd>,
        cmd_tx: &crate::PlayerCmdSender,
        eos_watcher: &Arc<AtomicBool>,
        send_atf_watcher: &Arc<AtomicBool>,
    ) {
        while let Some(msg) = main_rx.recv().await {
            match msg {
                PlayerInternalCmd::Eos => {
                    send_atf_watcher.store(false, Ordering::Relaxed);
                    if let Err(e) = cmd_tx.send(PlayerCmd::Eos) {
                        error!("error in sending Eos: {e}");
                    }
                }
                PlayerInternalCmd::Error(ty) => {
                    if let Err(e) = cmd_tx.send(PlayerCmd::Error(ty)) {
                        error!("error in sending Error: {e}");
                    }
                }
                PlayerInternalCmd::AboutToFinish => {
                    info!("about to finish received by gstreamer internal");
                    if !send_atf_watcher.swap(true, Ordering::Relaxed) {
                        if let Err(e) = cmd_tx.send(PlayerCmd::AboutToFinish) {
                            error!("error in sending AboutToFinish: {e}");
                        }
                    }
                }
                PlayerInternalCmd::SkipNext => {
                    // store it here, as there will be no EOS event send by gst
                    eos_watcher.store(true, std::sync::atomic::Ordering::SeqCst);
                    send_atf_watcher.store(false, Ordering::Relaxed);
                    if let Err(e) = cmd_tx.send(PlayerCmd::Eos) {
                        error!("error in sending SkipNext: {e}");
                    }
                }
                PlayerInternalCmd::ReloadSpeed => {
                    // HACK: currently gstreamer does not have any internal events to be send, and there is no global "re-apply speed property", this also means that if using max speed, it will not actually use full-speed
                    let _ = cmd_tx.send(PlayerCmd::SpeedUp);
                    let _ = cmd_tx.send(PlayerCmd::SpeedDown);
                }
                PlayerInternalCmd::MetadataChanged => {
                    let _ = cmd_tx.send(PlayerCmd::MetadataChanged);
                }
            }
        }
    }
}

#[async_trait]
impl PlayerTrait for GStreamerBackend {
    async fn add_and_play(&mut self, track: &Track) {
        self.playbin
            .set_state(gst::State::Ready)
            .expect("set gst state ready error");
        set_uri_from_track(&self.playbin, track);
        // state change can fail if for example the current file does not exist
        let _ = self.playbin.set_state(gst::State::Playing);
    }

    fn volume(&self) -> Volume {
        self.volume
    }

    fn set_volume(&mut self, volume: Volume) -> Volume {
        let volume = volume.min(100);
        self.volume = volume;
        self.playbin.set_volume(f64::from(volume) / 100.0);

        volume
    }

    fn pause(&mut self) {
        // state change can fail if for example the current file does not exist
        let _ = self.playbin.pause();
    }

    fn resume(&mut self) {
        self.playbin
            .play()
            .expect("set gst state playing error in resume");
    }

    fn is_paused(&self) -> bool {
        match self.playbin.current_state() {
            gst::State::Playing => false,
            gst::State::Paused => true,
            state => {
                debug!("Bad GStreamer state {state:#?}");
                // fallback to saying it is paused, even in other states
                true
            }
        }
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_wrap)]
    fn seek(&mut self, secs: i64) -> Result<()> {
        if let Some(time_pos) = self.playbin.get_position() {
            if let Some(duration) = self.playbin.get_duration() {
                let time_pos = time_pos.seconds() as i64;
                let duration = duration.seconds() as i64;

                let mut seek_pos = time_pos + secs;
                if seek_pos < 0 {
                    seek_pos = 0;
                }
                if seek_pos > duration - 6 {
                    seek_pos = duration - 6;
                }

                let seek_pos_clock = ClockTime::from_seconds(seek_pos as u64);
                self.playbin.seek_simple(
                    gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                    seek_pos_clock,
                )?;
                // add this sleep to get progress feedback
                std::thread::sleep(Duration::from_millis(50));
            }
        }
        Ok(())
    }

    fn seek_to(&mut self, position: Duration) {
        // expect should be fine here, as this function does not allow erroring and any duration more than u64::MAX is unlikely
        let seek_pos_clock =
            ClockTime::try_from(position).expect("Duration(u128) did not fit into ClockTime(u64)");
        self.playbin.set_volume(0.0);
        while self
            .playbin
            .seek_simple(gst::SeekFlags::FLUSH, seek_pos_clock)
            .is_err()
        {
            std::thread::sleep(Duration::from_millis(100));
        }
        self.playbin.set_volume(f64::from(self.volume) / 100.0);
    }
    fn speed(&self) -> Speed {
        self.speed
    }

    fn set_speed(&mut self, speed: Speed) -> Speed {
        self.speed = speed;
        self.playbin.set_speed(speed);

        self.speed
    }

    fn stop(&mut self) {
        let _ = self.playbin.set_state(gst::State::Null);
    }

    fn get_progress(&self) -> Option<PlayerProgress> {
        let position = Some(self.playbin.get_position()?.into());
        let total_duration = Some(self.playbin.get_duration()?.into());
        Some(PlayerProgress {
            position,
            total_duration,
        })
    }

    fn gapless(&self) -> bool {
        self.gapless
    }

    fn set_gapless(&mut self, to: bool) {
        self.gapless = to;
    }

    fn skip_one(&mut self) {
        let _ = self.icmd_tx.blocking_send(PlayerInternalCmd::SkipNext);
    }

    fn enqueue_next(&mut self, track: &Track) {
        set_uri_from_track(&self.playbin, track);
    }

    fn media_info(&self) -> MediaInfo {
        let media_title_r = self.media_title.lock();
        if media_title_r.is_empty() {
            MediaInfo::default()
        } else {
            MediaInfo {
                media_title: Some(media_title_r.clone()),
            }
        }
    }
}

impl Drop for GStreamerBackend {
    /// Cleans up `GStreamer` pipeline when `Backend` is dropped.
    fn drop(&mut self) {
        self.playbin
            .set_state(gst::State::Null)
            .expect("Unable to set the pipeline to the `Null` state");
    }
}

/// Helper function to consistently set the `uri` on `playbin` from a [`Track`]
fn set_uri_from_track(playbin: &PlaybinWrap, track: &Track) {
    match track.inner() {
        MediaTypes::Track(track_data) => playbin.set_uri(track_data.path().to_uri()),
        MediaTypes::Radio(radio_track_data) => playbin.set_uri(radio_track_data.url()),
        MediaTypes::Podcast(podcast_track_data) => playbin.set_uri(podcast_track_data.url()),
    }
}
