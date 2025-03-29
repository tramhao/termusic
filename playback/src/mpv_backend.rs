//! SPDX-License-Identifier: MIT

use super::{PlayerCmd, PlayerProgress, PlayerTrait};
use crate::{MediaInfo, Speed, Volume};
use anyhow::Result;
use async_trait::async_trait;
use libmpv::Mpv;
use libmpv::{
    events::{Event, PropertyData},
    Format,
};
use parking_lot::Mutex;
use std::cmp;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;
use termusiclib::config::ServerOverlay;
use termusiclib::track::Track;

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
    #[allow(clippy::too_many_lines, clippy::cast_precision_loss)]
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
        mpv.set_property("speed", f64::from(speed) / 10.0).ok();
        let gapless_setting = if gapless { "yes" } else { "no" };
        mpv.set_property("gapless-audio", gapless_setting)
            .expect("gapless setting failed");

        let cmd_tx_inside = command_tx.clone();
        // let mut time_pos: i64 = 0;
        std::thread::Builder::new()
            .name("mpv event loop".into())
            .spawn(move || {
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
                    // if let Some(ev) = ev_ctx.wait_event(600.) {
                    if let Some(ev) = ev_ctx.wait_event(0.0) {
                        match ev {
                            Ok(Event::EndFile(e)) => {
                                // error!("event end file {:?} received", e);
                                if e == 0 {
                                    cmd_tx_inside.send(PlayerInternalCmd::Eos).ok();
                                }

                                // clear stored title on end
                                media_title_inside.lock().clear();
                            }
                            Ok(Event::StartFile) => {
                                // message_tx.send(PlayerMsg::CurrentTrackUpdated).ok();
                            }
                            Ok(Event::PropertyChange {
                                name,
                                change,
                                reply_userdata: _,
                            }) => match name {
                                "duration" => {
                                    if let PropertyData::Double(dur) = change {
                                        // using "dur.max" because mpv *may* return a negative number
                                        *duration_inside.lock() =
                                            Duration::from_secs_f64(dur.max(0.0));
                                    }
                                }
                                "time-pos" => {
                                    if let PropertyData::Double(time_pos) = change {
                                        // using "dur.max" because mpv *may* return a negative number
                                        let time_pos = Duration::from_secs_f64(time_pos.max(0.0));
                                        *position_inside.lock() = time_pos;

                                        // About to finish signal is a simulation of gstreamer, and used for gapless
                                        let dur = duration_inside.lock();
                                        let progress = time_pos.as_secs_f64() / dur.as_secs_f64();
                                        if progress >= 0.5
                                            && (*dur - time_pos) < Duration::from_secs(2)
                                        {
                                            if let Err(e) = cmd_tx.send(PlayerCmd::AboutToFinish) {
                                                error!("command AboutToFinish sent failed: {e}");
                                            }
                                        }
                                    }
                                }
                                "media-title" => {
                                    if let PropertyData::Str(title) = change {
                                        *media_title_inside.lock() = title.to_string();
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
                            Ok(_e) => {}  //error!("Event triggered: {:?}", e),
                            Err(_e) => {} //error!("Event errored: {:?}", e),
                        }
                    }

                    if let Ok(cmd) = command_rx.try_recv() {
                        match cmd {
                            // PlayerCmd::Eos => message_tx.send(PlayerMsg::Eos).unwrap(),
                            PlayerInternalCmd::Play(new) => {
                                *duration_inside.lock() = Duration::default();
                                mpv.command("loadfile", &[&format!("\"{new}\""), "replace"])
                                    .ok();
                                // .expect("Error loading file");
                                // error!("add and play {} ok", new);
                            }
                            PlayerInternalCmd::QueueNext(next) => {
                                mpv.command("loadfile", &[&format!("\"{next}\""), "append"])
                                    .ok();
                                // .expect("Error loading file");
                            }
                            PlayerInternalCmd::Volume(volume) => {
                                mpv.set_property("volume", i64::from(volume)).ok();
                                // .expect("Error increase volume");
                            }
                            PlayerInternalCmd::Pause => {
                                mpv.set_property("pause", true).ok();
                            }
                            PlayerInternalCmd::Resume => {
                                mpv.set_property("pause", false).ok();
                            }
                            PlayerInternalCmd::Speed(speed) => {
                                mpv.set_property("speed", f64::from(speed) / 10.0).ok();
                            }
                            PlayerInternalCmd::Stop => {
                                mpv.command("stop", &[""]).ok();
                            }
                            PlayerInternalCmd::Seek(secs) => {
                                let time_pos_seek =
                                    mpv.get_property::<i64>("time-pos").unwrap_or(0);
                                let duration_seek =
                                    mpv.get_property::<i64>("duration").unwrap_or(100);
                                let mut absolute_secs = secs + time_pos_seek;
                                absolute_secs = cmp::max(absolute_secs, 0);
                                absolute_secs = cmp::min(absolute_secs, duration_seek - 5);
                                mpv.pause().ok();
                                mpv.command("seek", &[&format!("\"{absolute_secs}\""), "absolute"])
                                    .ok();
                                mpv.unpause().ok();
                                // message_tx
                                //     .send(PlayerMsg::Progress(time_pos_seek, duration_seek))
                                //     .ok();
                            }
                            PlayerInternalCmd::SeekAbsolute(position) => {
                                mpv.pause().ok();
                                while mpv
                                    .command("seek", &[&format_duration(position), "absolute"])
                                    .is_err()
                                {
                                    // This is because we need to wait until the file is fully loaded.
                                    std::thread::sleep(Duration::from_millis(100));
                                }
                                mpv.unpause().ok();
                                // message_tx.send(PlayerMsg::Progress(secs, duration)).ok();
                            }
                            PlayerInternalCmd::Eos => {
                                if let Err(e) = cmd_tx.send(PlayerCmd::Eos) {
                                    error!("error sending eos: {e}");
                                }
                            }
                        }
                    }

                    // This is important to keep the mpv running, otherwise it cannot play.
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
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
}

/// Format a duration in "SS.mm" format
///
/// Note that mpv supports "HH:MM:SS.mmmm" format, but only the second and millisecond part is used
fn format_duration(dur: Duration) -> String {
    let secs = dur.as_secs();
    let milli = dur.subsec_millis();

    format!("{secs}.{milli}")
}

#[async_trait]
impl PlayerTrait for MpvBackend {
    async fn add_and_play(&mut self, track: &Track) {
        if let Some(file) = track.file() {
            self.command_tx
                .send(PlayerInternalCmd::Play(file.to_string()))
                .expect("failed to queue and play");
        }
    }

    fn volume(&self) -> Volume {
        self.volume
    }

    fn set_volume(&mut self, volume: Volume) -> Volume {
        self.volume = volume.min(100);
        self.command_tx
            .send(PlayerInternalCmd::Volume(self.volume))
            .ok();

        self.volume
    }

    fn pause(&mut self) {
        self.command_tx.send(PlayerInternalCmd::Pause).ok();
    }

    fn resume(&mut self) {
        self.command_tx.send(PlayerInternalCmd::Resume).ok();
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
        self.command_tx
            .send(PlayerInternalCmd::SeekAbsolute(position))
            .ok();
    }
    fn speed(&self) -> Speed {
        self.speed
    }

    fn set_speed(&mut self, speed: Speed) -> Speed {
        self.speed = speed;
        self.command_tx
            .send(PlayerInternalCmd::Speed(self.speed))
            .ok();

        self.speed
    }

    fn stop(&mut self) {
        self.command_tx.send(PlayerInternalCmd::Stop).ok();
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
        self.command_tx.send(PlayerInternalCmd::Eos).ok();
    }

    fn enqueue_next(&mut self, track: &Track) {
        let Some(file) = track.file() else {
            error!("Got track, but cant handle it without a file!");
            return;
        };

        self.command_tx
            .send(PlayerInternalCmd::QueueNext(file.to_string()))
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
