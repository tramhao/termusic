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
use super::{PlayerCmd, PlayerProgress, PlayerTrait};
use crate::{MediaInfo, Speed, Volume};
use anyhow::Result;
use async_trait::async_trait;
use glib::FlagsClass;
use gst::bus::BusWatchGuard;
use gst::{event::Seek, Element, SeekFlags, SeekType};
use gst::{ClockTime, StateChangeError, StateChangeSuccess};
use gstreamer as gst;
use gstreamer::prelude::*;
use parking_lot::Mutex;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use termusiclib::config::ServerOverlay;
use termusiclib::track::{MediaType, Track};

/// This trait allows for easy conversion of a path to a URI
pub trait PathToURI {
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
    AboutToFinish,
    SkipNext,
    ReloadSpeed,
}

pub struct GStreamerBackend {
    playbin: PlaybinWrap,
    volume: u16,
    speed: i32,
    gapless: bool,
    message_tx: async_channel::Sender<PlayerInternalCmd>,
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

        // Asynchronous channel to communicate with main() with
        let (main_tx, main_rx) = async_channel::bounded(3);
        let message_tx = main_tx.clone();
        std::thread::Builder::new()
            .name("gstreamer event loop".into())
            .spawn(move || loop {
                if let Ok(msg) = main_rx.try_recv() {
                    match msg {
                        PlayerInternalCmd::Eos => {
                            if let Err(e) = cmd_tx.send(PlayerCmd::Eos) {
                                error!("error in sending Eos: {e}");
                            }
                        }
                        PlayerInternalCmd::AboutToFinish => {
                            info!("about to finish received by gstreamer internal !!!!!");
                            if let Err(e) = cmd_tx.send(PlayerCmd::AboutToFinish) {
                                error!("error in sending AboutToFinish: {e}");
                            }
                        }
                        PlayerInternalCmd::SkipNext => {
                            // store it here, as there will be no EOS event send by gst
                            eos_watcher_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                            if let Err(e) = cmd_tx.send(PlayerCmd::Eos) {
                                error!("error in sending SkipNext: {e}");
                            }
                        }
                        PlayerInternalCmd::ReloadSpeed => {
                            // HACK: currently gstreamer does not have any internal events to be send, and there is no global "re-apply speed property", this also means that if using max speed, it will not actually use full-speed
                            let _ = cmd_tx.send(PlayerCmd::SpeedUp);
                            let _ = cmd_tx.send(PlayerCmd::SpeedDown);
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            })
            .expect("failed to start gstreamer event loop thread");
        let playbin = Box::new(gst::ElementFactory::make("playbin3"))
            .build()
            .expect("playbin3 make error");

        let tempo = gst::ElementFactory::make("scaletempo")
            .name("tempo")
            .build()
            .expect("make scaletempo error");

        // let sink = gst::ElementFactory::make_with_name("autoaudiosink",
        // Some("autoaudiosink")).unwrap();
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

        // let sink = gst::ElementFactory::make("autoaudiosink")
        //     .build()
        //     .expect("audio sink make error");

        // playbin.set_property("audio-sink", &sink);
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
        let bus_watch = playbin
            .0
            .bus()
            .expect("Failed to get GStreamer message bus")
            .add_watch(glib::clone!(
                #[strong]
                main_tx,
                move |_bus, msg| {
                    match msg.view() {
                        gst::MessageView::Eos(_) => {
                            // debug tracking, as gapless interferes with it
                            debug!("gstreamer message EOS");
                            main_tx
                                .send_blocking(PlayerInternalCmd::Eos)
                                .expect("Unable to send message to main()");
                            eos_watcher.store(true, std::sync::atomic::Ordering::SeqCst);

                            // clear stored title on end
                            media_title_internal.lock().clear();
                        }
                        gst::MessageView::StreamStart(_e) => {
                            if !eos_watcher.load(std::sync::atomic::Ordering::SeqCst) {
                                trace!(
                                    "Sending EOS because it was not sent since last StreamStart"
                                );
                                main_tx
                                    .send_blocking(PlayerInternalCmd::Eos)
                                    .expect("Unable to send message to main()");
                            }

                            eos_watcher.store(false, std::sync::atomic::Ordering::SeqCst);

                            // clear stored title on stream start (should work without conflicting in ::Tag)
                            media_title_internal.lock().clear();

                            // HACK: gstreamer does not handle seek events before some undocumented time, see other note in main_rx handler
                            let _ = main_tx.send_blocking(PlayerInternalCmd::ReloadSpeed);
                        }
                        gst::MessageView::Error(e) => error!("GStreamer Error: {}", e.error()),
                        gst::MessageView::Tag(tag) => {
                            if let Some(title) = tag.tags().get::<gst::tags::Title>() {
                                info!("  Title: {}", title.get());
                                *media_title_internal.lock() = title.get().into();
                            }
                            // if let Some(artist) = tag.tags().get::<gst::tags::Artist>() {
                            //     info!("  Artist: {}", artist.get());
                            //     // *media_title_internal.lock() = artist.get().to_string();
                            // }
                            // if let Some(album) = tag.tags().get::<gst::tags::Album>() {
                            //     info!("  Album: {}", album.get());
                            //     // *media_title_internal.lock() = album.get().to_string();
                            // }
                        }
                        gst::MessageView::Buffering(buffering) => {
                            // let (mode,_, _, left) = buffering.buffering_stats();
                            // info!("mode is: {mode:?}, and left is: {left}");
                            let percent = buffering.percent();
                            // according to the documentation, the application (we) need to set the playbin state according tothe buffering state
                            // see https://gstreamer.freedesktop.org/documentation/playback/playbin.html?gi-language=c#buffering
                            if percent < 100 {
                                let _ = playbin_clone.pause();
                            } else {
                                let _ = playbin_clone.play();
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
            ))
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

        let mut this = Self {
            playbin,
            volume,
            speed,
            gapless,
            message_tx,
            media_title,
            _bus_watch_guard: bus_watch,
        };

        this.set_volume(volume);
        // this.set_speed(speed);

        // Send a signal to enqueue the next media before the current finished
        this.playbin.connect_about_to_finish(move |_| {
            debug!("Sending playbin AboutToFinish");
            main_tx
                .send_blocking(PlayerInternalCmd::AboutToFinish)
                .unwrap();
            None
        });

        this
    }
}

#[async_trait]
impl PlayerTrait for GStreamerBackend {
    async fn add_and_play(&mut self, track: &Track) {
        self.playbin
            .set_state(gst::State::Ready)
            .expect("set gst state ready error");
        set_uri_from_track(&self.playbin, track);
        self.playbin
            .set_state(gst::State::Playing)
            .expect("set gst state playing error");
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
        self.playbin.pause().expect("set gst state paused error");
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
        self.playbin.set_state(gst::State::Null).ok();
    }

    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_wrap)]
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
        self.message_tx
            .send_blocking(PlayerInternalCmd::SkipNext)
            .ok();
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
    match track.media_type {
        MediaType::Music => {
            if let Some(file) = track.file() {
                let path = Path::new(file);
                playbin.set_uri(path.to_uri());
            }
        }
        MediaType::Podcast | MediaType::LiveRadio => {
            if let Some(url) = track.file() {
                playbin.set_uri(url);
            }
        }
    }
}
