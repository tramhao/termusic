//! SPDX-License-Identifier: MIT

use std::cmp;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use libmpv::Mpv;
use libmpv::{
    events::{Event, PropertyData},
    Format,
};
use parking_lot::Mutex;
use termusiclib::config::ServerOverlay;
use termusiclib::new_track::{MediaTypes, Track};

use crate::{MediaInfo, PlayerCmd, PlayerProgress, PlayerTrait, Speed, Volume};

pub struct MpvBackend {
    // player: Mpv,
    volume: u16,
    speed: i32,
    gapless: bool,
    command_tx: Sender<PlayerInternalCmd>,
    position: Arc<Mutex<Duration>>,
    // TODO: this should likely be a Option
    duration: Arc<Mutex<Duration>>,
    media_title: Arc<Mutex<String>>,
    // cmd_tx: crate::PlayerCmdSender,
}

enum PlayerInternalCmd {
    Eos,
    Pause,
    // GetProgress,
    Play(String),
    QueueNext(String),
    Resume,
    Seek(i64),
    SeekAbsolute(Duration),
    Speed(i32),
    Stop,
    Volume(u16),
}

impl MpvBackend {
    pub fn new(config: &ServerOverlay, cmd_tx: crate::PlayerCmdSender) -> Self {
        let (command_tx, command_rx): (Sender<PlayerInternalCmd>, Receiver<PlayerInternalCmd>) =
            mpsc::channel();
        let volume = config.settings.player.volume;
        let speed = config.settings.player.speed;
        let gapless = config.settings.player.gapless;
        let position = Arc::new(Mutex::new(Duration::default()));
        let duration = Arc::new(Mutex::new(Duration::default()));
        let media_title = Arc::new(Mutex::new(String::new()));
        let position_inside = position.clone();
        let duration_inside = duration.clone();
        let media_title_inside = media_title.clone();

        let mpv = Mpv::new().expect("Couldn't initialize MpvHandlerBuilder");
        mpv.set_property("vo", "null")
            .expect("Couldn't set vo=null in libmpv");

        #[cfg(target_os = "linux")]
        mpv.set_property("ao", "pulse")
            .expect("Couldn't set ao=pulse in libmpv");

        mpv.set_property("volume", i64::from(volume))
            .expect("Error setting volume");
        let _ = mpv.set_property("speed", f64::from(speed) / 10.0);
        let gapless_setting = if gapless { "yes" } else { "no" };
        mpv.set_property("gapless-audio", gapless_setting)
            .expect("gapless setting failed");

        let icmd_tx = command_tx.clone();
        std::thread::Builder::new()
            .name("mpv event loop".into())
            .spawn(move || {
                Self::event_handler(
                    &mpv,
                    &icmd_tx,
                    &command_rx,
                    &cmd_tx,
                    &media_title_inside,
                    &position_inside,
                    &duration_inside,
                );
            })
            .expect("failed to start mpv event loop thread");

        Self {
            volume,
            speed,
            gapless,
            command_tx,
            position,
            duration,
            media_title,
        }
    }

    /// The Event handler thread code
    fn event_handler(
        mpv: &Mpv,
        icmd_tx: &Sender<PlayerInternalCmd>,
        icmd_rx: &Receiver<PlayerInternalCmd>,
        cmd_tx: &crate::PlayerCmdSender,
        media_title: &Arc<Mutex<String>>,
        position: &Arc<Mutex<Duration>>,
        duration: &Arc<Mutex<Duration>>,
    ) {
        let mut ev_ctx = mpv.create_event_context();
        ev_ctx
            .disable_deprecated_events()
            .expect("failed to disable deprecated events.");
        ev_ctx
            .observe_property("duration", Format::Double, 0)
            .expect("failed to watch duration");
        ev_ctx
            .observe_property("time-pos", Format::Double, 1)
            .expect("failed to watch time_pos");
        ev_ctx
            .observe_property("media-title", Format::String, 2)
            .expect("failed to watch media-title");
        loop {
            if let Some(ev) = ev_ctx.wait_event(0.0) {
                match ev {
                    Ok(ev) => {
                        Self::handle_mpv_event(
                            ev,
                            icmd_tx,
                            cmd_tx,
                            media_title,
                            position,
                            duration,
                        );
                    }
                    Err(err) => {
                        error!("Event Error: {err:?}");
                        continue;
                    }
                }
            }

            if let Ok(cmd) = icmd_rx.try_recv() {
                Self::handle_internal_cmd(cmd, mpv, duration, cmd_tx);
            }

            // This is important to keep the mpv running, otherwise it cannot play.
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    /// Handle a given [`Event`].
    fn handle_mpv_event(
        ev: Event<'_>,
        icmd_tx: &Sender<PlayerInternalCmd>,
        cmd_tx: &crate::PlayerCmdSender,
        media_title: &Arc<Mutex<String>>,
        position: &Arc<Mutex<Duration>>,
        duration: &Arc<Mutex<Duration>>,
    ) {
        match ev {
            Event::EndFile(e) => {
                // error!("event end file {:?} received", e);
                if e == 0 {
                    let _ = icmd_tx.send(PlayerInternalCmd::Eos);
                }

                // clear stored title on end
                media_title.lock().clear();
            }
            Event::PropertyChange {
                name,
                change,
                reply_userdata: _,
            } => match name {
                "duration" => {
                    if let PropertyData::Double(dur) = change {
                        // using "dur.max" because mpv *may* return a negative number
                        *duration.lock() = Duration::from_secs_f64(dur.max(0.0));
                    }
                }
                "time-pos" => {
                    if let PropertyData::Double(time_pos) = change {
                        // using "dur.max" because mpv *may* return a negative number
                        let time_pos = Duration::from_secs_f64(time_pos.max(0.0));
                        *position.lock() = time_pos;

                        // About to finish signal is a simulation of gstreamer, and used for gapless
                        let dur = duration.lock();
                        let progress = time_pos.as_secs_f64() / dur.as_secs_f64();
                        if progress >= 0.5 && (*dur - time_pos) < Duration::from_secs(2) {
                            if let Err(e) = cmd_tx.send(PlayerCmd::AboutToFinish) {
                                error!("command AboutToFinish sent failed: {e}");
                            }
                        }
                    }
                }
                "media-title" => {
                    if let PropertyData::Str(title) = change {
                        *media_title.lock() = title.to_string();
                    }
                }
                &_ => {
                    // left for debug
                    // error!(
                    //     "Event not handled {:?}",
                    //     Event::PropertyChange {
                    //         name,
                    //         change,
                    //         reply_userdata
                    //     }
                    // )
                }
            },
            _ev => {
                // debug!("Event triggered: {:?}", ev),
            }
        }
    }

    /// Handle a given [`PlayerInternalCmd`].
    fn handle_internal_cmd(
        cmd: PlayerInternalCmd,
        mpv: &Mpv,
        duration: &Arc<Mutex<Duration>>,
        cmd_tx: &crate::PlayerCmdSender,
    ) {
        match cmd {
            // PlayerCmd::Eos => message_tx.send(PlayerMsg::Eos).unwrap(),
            PlayerInternalCmd::Play(new) => {
                *duration.lock() = Duration::default();
                let _ = mpv.command("loadfile", &[&format!("\"{new}\""), "replace"]);
                // error!("add and play {} ok", new);
            }
            PlayerInternalCmd::QueueNext(next) => {
                let _ = mpv.command("loadfile", &[&format!("\"{next}\""), "append"]);
            }
            PlayerInternalCmd::Volume(volume) => {
                let _ = mpv.set_property("volume", i64::from(volume));
            }
            PlayerInternalCmd::Pause => {
                let _ = mpv.pause();
            }
            PlayerInternalCmd::Resume => {
                let _ = mpv.unpause();
            }
            PlayerInternalCmd::Speed(speed) => {
                let _ = mpv.set_property("speed", f64::from(speed) / 10.0);
            }
            PlayerInternalCmd::Stop => {
                let _ = mpv.command("stop", &[""]);
            }
            PlayerInternalCmd::Seek(secs) => {
                let time_pos_seek = mpv.get_property::<i64>("time-pos").unwrap_or(0);
                let duration_seek = mpv.get_property::<i64>("duration").unwrap_or(100);
                let mut absolute_secs = secs + time_pos_seek;
                absolute_secs = cmp::max(absolute_secs, 0);
                absolute_secs = cmp::min(absolute_secs, duration_seek - 5);
                let _ = mpv.pause();
                let _ = mpv.command("seek", &[&format!("\"{absolute_secs}\""), "absolute"]);
                let _ = mpv.unpause();
                // let _ = message_tx
                //     .send(PlayerMsg::Progress(time_pos_seek, duration_seek));
            }
            PlayerInternalCmd::SeekAbsolute(position) => {
                let _ = mpv.pause();
                while mpv
                    .command("seek", &[&format_duration(position), "absolute"])
                    .is_err()
                {
                    // This is because we need to wait until the file is fully loaded.
                    std::thread::sleep(Duration::from_millis(100));
                }
                let _ = mpv.unpause();
                // let _ = message_tx.send(PlayerMsg::Progress(secs, duration));
            }
            PlayerInternalCmd::Eos => {
                if let Err(e) = cmd_tx.send(PlayerCmd::Eos) {
                    error!("error sending eos: {e}");
                }
            }
        }
    }
}

/// Format a duration in "SS.mm" format
///
/// Note that mpv supports "HH:MM:SS.mmmm" format, but only the second and millisecond part is used
fn format_duration(dur: Duration) -> String {
    let secs = dur.as_secs();
    let milli = dur.subsec_millis();

    format!("{secs}.{milli}")
}

fn track_to_string(track: &Track) -> String {
    match track.inner() {
        MediaTypes::Track(track_data) => track_data.path().to_string_lossy().to_string(),
        MediaTypes::Radio(radio_track_data) => radio_track_data.url().to_string(),
        MediaTypes::Podcast(podcast_track_data) => podcast_track_data.url().to_string(),
    }
}

#[async_trait]
impl PlayerTrait for MpvBackend {
    async fn add_and_play(&mut self, track: &Track) {
        let file = track_to_string(track);

        self.command_tx
            .send(PlayerInternalCmd::Play(file))
            .expect("failed to queue and play");
    }

    fn volume(&self) -> Volume {
        self.volume
    }

    fn set_volume(&mut self, volume: Volume) -> Volume {
        self.volume = volume.min(100);
        let _ = self.command_tx.send(PlayerInternalCmd::Volume(self.volume));

        self.volume
    }

    fn pause(&mut self) {
        let _ = self.command_tx.send(PlayerInternalCmd::Pause);
    }

    fn resume(&mut self) {
        let _ = self.command_tx.send(PlayerInternalCmd::Resume);
    }

    fn is_paused(&self) -> bool {
        true
    }

    fn seek(&mut self, secs: i64) -> Result<()> {
        self.command_tx.send(PlayerInternalCmd::Seek(secs))?;
        Ok(())
    }

    #[allow(clippy::cast_possible_wrap)]
    fn seek_to(&mut self, position: Duration) {
        let _ = self
            .command_tx
            .send(PlayerInternalCmd::SeekAbsolute(position));
    }
    fn speed(&self) -> Speed {
        self.speed
    }

    fn set_speed(&mut self, speed: Speed) -> Speed {
        self.speed = speed;
        let _ = self.command_tx.send(PlayerInternalCmd::Speed(self.speed));

        self.speed
    }

    fn stop(&mut self) {
        let _ = self.command_tx.send(PlayerInternalCmd::Stop);
    }

    fn get_progress(&self) -> Option<PlayerProgress> {
        Some(PlayerProgress {
            position: Some(*self.position.lock()),
            total_duration: Some(*self.duration.lock()),
        })
    }

    fn gapless(&self) -> bool {
        self.gapless
    }

    fn set_gapless(&mut self, to: bool) {
        self.gapless = to;
    }

    fn skip_one(&mut self) {
        let _ = self.command_tx.send(PlayerInternalCmd::Eos);
    }

    fn enqueue_next(&mut self, track: &Track) {
        let file = track_to_string(track);

        self.command_tx
            .send(PlayerInternalCmd::QueueNext(file))
            .expect("failed to queue next");
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
