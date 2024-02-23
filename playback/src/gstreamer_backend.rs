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
use super::{PlayerCmd, PlayerTrait};
use anyhow::Result;
use async_trait::async_trait;
use glib::FlagsClass;
use gst::bus::BusWatchGuard;
use gst::ClockTime;
use gst::{event::Seek, Element, SeekFlags, SeekType};
use gstreamer as gst;
use gstreamer::prelude::*;
use parking_lot::Mutex;
use std::cmp;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;
use termusiclib::config::Settings;
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

pub struct GStreamerBackend {
    playbin: Element,
    paused: bool,
    volume: i32,
    speed: i32,
    pub gapless: bool,
    pub message_tx: Sender<PlayerCmd>,
    pub position: Arc<Mutex<i64>>,
    pub duration: Arc<Mutex<i64>>,
    pub radio_title: Arc<Mutex<String>>,
    _bus_watch_guard: BusWatchGuard,
}

#[allow(clippy::cast_lossless)]
impl GStreamerBackend {
    #[allow(clippy::too_many_lines)]
    pub fn new(config: &Settings, cmd_tx: crate::PlayerCmdSender) -> Self {
        gst::init().expect("Couldn't initialize Gstreamer");
        let ctx = glib::MainContext::default();
        let _guard = ctx.acquire();
        let mainloop = glib::MainLoop::new(Some(&ctx), false);

        let (message_tx, message_rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("gstreamer event loop".into())
            .spawn(move || loop {
                if let Ok(msg) = message_rx.try_recv() {
                    match msg {
                        PlayerCmd::Eos => {
                            if let Err(e) = cmd_tx.send(PlayerCmd::Eos) {
                                error!("error in sending eos: {e}");
                            }
                        }
                        PlayerCmd::AboutToFinish => {
                            info!("about to finish received by gstreamer internal !!!!!");
                            if let Err(e) = cmd_tx.send(PlayerCmd::AboutToFinish) {
                                error!("error in sending eos: {e}");
                            }
                        }
                        _ => {}
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            })
            .expect("failed to start gstreamer event loop thread");
        let playbin = Box::new(gst::ElementFactory::make("playbin3"))
            .build()
            .expect("playbin3 make error");

        let sink = gst::ElementFactory::make("autoaudiosink")
            .build()
            .expect("audio sink make error");

        playbin.set_property("audio-sink", &sink);
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

        let duration = Arc::new(Mutex::new(0_i64));

        // Asynchronous channel to communicate with main() with
        // let (main_tx, main_rx) = MainContext::channel(glib::Priority::default());
        let (main_tx, main_rx) = async_channel::bounded(3);
        // Handle messages from GSTreamer bus

        let radio_title = Arc::new(Mutex::new(String::new()));
        let radio_title_internal = radio_title.clone();
        let bus_watch = playbin
            .bus()
            .expect("Failed to get GStreamer message bus")
            .add_watch(glib::clone!(@strong main_tx=> move |_bus, msg| {
                match msg.view() {
                    gst::MessageView::Eos(_) =>
                        main_tx.send_blocking(PlayerCmd::Eos)
                        .expect("Unable to send message to main()"),
                    gst::MessageView::StreamStart(_) => {}
                    gst::MessageView::Error(e) =>
                        error!("GStreamer Error: {}", e.error()),
                    gst::MessageView::Tag(tag) => {
                        if let Some(title) = tag.tags().get::<gst::tags::Title>() {
                            info!("  Title: {}", title.get());
                            *radio_title_internal.lock() = format!("Current Playing: {}",title.get()).to_string();
                        }
                        // if let Some(artist) = tag.tags().get::<gst::tags::Artist>() {
                        //     info!("  Artist: {}", artist.get());
                        //     // *radio_title_internal.lock() = artist.get().to_string();
                        // }
                        // if let Some(album) = tag.tags().get::<gst::tags::Album>() {
                        //     info!("  Album: {}", album.get());
                        //     // *radio_title_internal.lock() = album.get().to_string();
                        // }
                    }
                    gst::MessageView::Buffering(buffering) => {
                        // let (mode,_, _, left) = buffering.buffering_stats();
                        // info!("mode is: {mode:?}, and left is: {left}");
                        let percent = buffering.percent();
                        // info!("Buffering ({}%)\r", percent);
                        if percent < 100 {
                            let _ = main_tx.send_blocking(PlayerCmd::Pause);
                        } else {
                            let _ = main_tx.send_blocking(PlayerCmd::Play);
                        }
                        // Left for debug
                        // let msg = buffering.message();
                        // info!("message is: {msg:?}");
                    }
                    gst::MessageView::Warning(warning) => {
                        info!("GStreamer Warning: {}", warning.error());
                    }
                    // gst::MessageView::DurationChanged(dur) => {
                    // }
                    // Left for debug
                    // msg => {
                    //     info!("msg: {msg:?}");
                    // }
                    _ => (),
                }
                 glib::ControlFlow::Continue
            }))
            .expect("Failed to connect to GStreamer message bus");

        let tx = message_tx.clone();
        std::thread::spawn(move || {
            // main_rx.attach(
            //     None,
            //     glib::clone!(@strong mainloop => move |msg| {
            //         tx.send(msg).ok();
            //         glib::ControlFlow::Continue
            //     }),
            // );
            mainloop.run();
        });

        // Spawn an async task on the main context to handle the channel messages
        // let main_context = glib::MainContext::default();

        // let self_ = self.downgrade();
        // main_context.spawn_local(async move {
        glib::spawn_future(async move {
            while let Ok(msg) = main_rx.recv().await {
                info!("{:?} received!!", msg);
                tx.send(msg).ok();
                // let Some(self_) = self_.upgrade() else {
                //     break;
                // };

                // self_.do_action(action);
            }
        });

        // glib::spawn_future(glib::clone!(@strong mainloop => async move{
        //     while let Ok(msg) = main_rx.recv().await {
        //         info!("{:?} received!!",msg);
        //         tx.send(msg).ok();
        //         // glib::ControlFlow::Continue;
        //     }
        // }));

        let volume = config.player_volume;
        let speed = config.player_speed;
        let gapless = config.player_gapless;

        let mut this = Self {
            playbin,
            paused: false,
            volume,
            speed,
            gapless,
            message_tx,
            position: Arc::new(Mutex::new(0_i64)),
            duration,
            radio_title,
            _bus_watch_guard: bus_watch,
        };

        this.set_volume(volume);
        this.set_speed(speed);

        // Switch to next song when reaching end of current track
        // let tx = main_tx;
        // this.playbin.connect(
        //     "about-to-finish",
        //     false,
        //     glib::clone!(@strong this => move |_args| {
        //        tx.send(PlayerMsg::AboutToFinish).unwrap();
        //        None
        //     }),
        // );

        this.playbin.connect("about-to-finish", false, move |_| {
            info!("about to finish generated!");
            main_tx.send_blocking(PlayerCmd::AboutToFinish).unwrap();
            info!("about to finish sent by playbin!");
            None
        });

        this
    }
    pub fn skip_one(&mut self) {
        self.message_tx.send(PlayerCmd::Eos).ok();
    }
    pub fn enqueue_next(&mut self, next_track: &str) {
        self.playbin
            .set_state(gst::State::Ready)
            .expect("set gst state ready error.");

        if next_track.starts_with("http") {
            self.playbin.set_property("uri", next_track);
        } else {
            let path = Path::new(next_track);
            self.playbin.set_property("uri", path.to_uri());
        }
        self.playbin
            .set_state(gst::State::Playing)
            .expect("set gst state playing error");
    }
    fn set_volume_inside(&mut self, volume: f64) {
        self.playbin.set_property("volume", volume);
    }

    fn get_position(&self) -> ClockTime {
        match self.playbin.query_position::<ClockTime>() {
            Some(pos) => pos,
            None => ClockTime::from_seconds(0),
        }
    }

    fn get_duration(&self) -> ClockTime {
        match self.playbin.query_duration::<ClockTime>() {
            Some(pos) => pos,
            None => ClockTime::from_seconds(99_u64),
        }
    }

    fn send_seek_event(&mut self, rate: i32) -> bool {
        self.speed = rate;
        let rate = rate as f64 / 10.0;
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
        if let Some(sink) = self.playbin.property::<Option<Element>>("audio-sink") {
            // try_property::<Option<Element>>("audio-sink") {
            // Send the event
            sink.send_event(seek_event)
        } else {
            false
        }
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn get_buffer_duration(&self) -> u32 {
        self.playbin.property::<i64>("buffer-duration") as u32
        // if let Some(duration) = self.playbin.property::<i64>("buffer-duration") {
        //     duration as u64
        // } else {
        //     120_u64
        // }
    }
}

#[async_trait]
impl PlayerTrait for GStreamerBackend {
    async fn add_and_play(&mut self, track: &Track) {
        self.playbin
            .set_state(gst::State::Ready)
            .expect("set gst state ready error.");
        match track.media_type {
            Some(MediaType::Music) => {
                if let Some(file) = track.file() {
                    let path = Path::new(file);
                    self.playbin.set_property("uri", path.to_uri());
                }
            }
            Some(MediaType::Podcast | MediaType::LiveRadio) => {
                if let Some(url) = track.file() {
                    self.playbin.set_property("uri", url);
                }
            }
            None => error!("no media type found for track"),
        }
        self.playbin
            .set_state(gst::State::Playing)
            .expect("set gst state playing error");
    }

    fn volume_up(&mut self) {
        self.volume = cmp::min(self.volume + 5, 100);
        self.set_volume_inside(f64::from(self.volume) / 100.0);
    }

    fn volume_down(&mut self) {
        self.volume = cmp::max(self.volume - 5, 0);
        self.set_volume_inside(f64::from(self.volume) / 100.0);
    }

    fn volume(&self) -> i32 {
        self.volume
    }

    fn set_volume(&mut self, mut volume: i32) {
        volume = volume.clamp(0, 100);
        self.volume = volume;
        self.set_volume_inside(f64::from(volume) / 100.0);
    }

    fn pause(&mut self) {
        self.paused = true;
        // self.player.pause();
        self.playbin
            .set_state(gst::State::Paused)
            .expect("set gst state paused error");
    }

    fn resume(&mut self) {
        self.paused = false;
        // self.player.play();
        self.playbin
            .set_state(gst::State::Playing)
            .expect("set gst state playing error in resume");
    }

    fn is_paused(&self) -> bool {
        self.playbin.current_state() == gst::State::Paused
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_wrap)]
    fn seek(&mut self, secs: i64) -> Result<()> {
        let time_pos = self.get_position().seconds() as i64;
        let duration = self.get_duration().seconds() as i64;
        let mut seek_pos = time_pos + secs;
        if seek_pos < 0 {
            seek_pos = 0;
        }
        if seek_pos > duration - 6 {
            seek_pos = duration - 6;
        }

        let seek_pos_clock = ClockTime::from_seconds(seek_pos as u64);
        self.set_volume_inside(0.0);
        self.playbin
            .seek_simple(gst::SeekFlags::FLUSH, seek_pos_clock)?; // ignore any errors
        self.set_volume_inside(f64::from(self.volume) / 100.0);
        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_wrap)]
    fn seek_to(&mut self, last_pos: Duration) {
        let seek_pos = last_pos.as_secs() as i64;
        // let duration = self.get_duration().seconds() as i64;

        let seek_pos_clock = ClockTime::from_seconds(seek_pos as u64);
        self.set_volume_inside(0.0);
        while self
            .playbin
            .seek_simple(gst::SeekFlags::FLUSH, seek_pos_clock)
            .is_err()
        {
            std::thread::sleep(Duration::from_millis(100));
        }
        self.set_volume_inside(f64::from(self.volume) / 100.0);
    }
    fn speed(&self) -> i32 {
        self.speed
    }

    fn set_speed(&mut self, speed: i32) {
        self.send_seek_event(speed);
    }

    fn speed_up(&mut self) {
        let mut speed = self.speed + 1;
        if speed > 30 {
            speed = 30;
        }
        if !self.send_seek_event(speed) {
            error!("error set speed");
        }
    }

    fn speed_down(&mut self) {
        let mut speed = self.speed - 1;
        if speed < 1 {
            speed = 1;
        }
        self.set_speed(speed);
    }
    fn stop(&mut self) {
        self.playbin.set_state(gst::State::Null).ok();
    }

    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_wrap)]
    fn get_progress(&self) -> Result<(i64, i64)> {
        let time_pos = self.get_position().seconds() as i64;
        let duration = self.get_duration().seconds() as i64;
        *self.position.lock() = time_pos;
        *self.duration.lock() = duration;
        Ok((time_pos, duration))
    }

    fn gapless(&self) -> bool {
        self.gapless
    }

    fn set_gapless(&mut self, to: bool) {
        self.gapless = to;
    }

    fn skip_one(&mut self) {
        self.skip_one();
    }

    fn position_lock(&self) -> parking_lot::MutexGuard<'_, i64> {
        self.position.lock()
    }

    fn enqueue_next(&mut self, file: &str) {
        self.enqueue_next(file);
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
