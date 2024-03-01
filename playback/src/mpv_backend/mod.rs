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
mod libmpv;

use super::{PlayerCmd, PlayerProgress, PlayerTimeUnit, PlayerTrait};
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
use termusiclib::config::Settings;
use termusiclib::track::Track;

pub struct MpvBackend {
    // player: Mpv,
    volume: i32,
    speed: i32,
    pub gapless: bool,
    command_tx: Sender<PlayerInternalCmd>,
    pub position: Arc<Mutex<i64>>,
    // TODO: this should likely be a Option
    pub duration: Arc<Mutex<i64>>,
    pub media_title: Arc<Mutex<String>>,
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
    SeekAbsolute(i64),
    Speed(i32),
    Stop,
    Volume(i64),
}

impl MpvBackend {
    #[allow(clippy::too_many_lines, clippy::cast_precision_loss)]
    pub fn new(config: &Settings, cmd_tx: crate::PlayerCmdSender) -> Self {
        let (command_tx, command_rx): (Sender<PlayerInternalCmd>, Receiver<PlayerInternalCmd>) =
            mpsc::channel();
        let volume = config.player_volume;
        let speed = config.player_speed;
        let gapless = config.player_gapless;
        let position = Arc::new(Mutex::new(0_i64));
        let duration = Arc::new(Mutex::new(0_i64));
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
                    .observe_property("duration", Format::Int64, 0)
                    .expect("failed to watch duration");
                ev_ctx
                    .observe_property("time-pos", Format::Int64, 1)
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
                                    if let PropertyData::Int64(c) = change {
                                        *duration_inside.lock() = c;
                                    }
                                }
                                "time-pos" => {
                                    if let PropertyData::Int64(time_pos) = change {
                                        *position_inside.lock() = time_pos;

                                        // About to finish signal is a simulation of gstreamer, and used for gapless
                                        let dur = duration_inside.lock();
                                        let progress = time_pos as f64 / *dur as f64;
                                        if progress >= 0.5 && (*dur - time_pos) < 2 {
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
                                *duration_inside.lock() = 0;
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
                                mpv.set_property("volume", volume).ok();
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
                            PlayerInternalCmd::SeekAbsolute(secs) => {
                                mpv.pause().ok();
                                while mpv
                                    .command("seek", &[&format!("\"{secs}\""), "absolute"])
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

    pub fn enqueue_next(&mut self, next: &str) {
        self.command_tx
            .send(PlayerInternalCmd::QueueNext(next.to_string()))
            .ok();
    }

    fn queue_and_play(&mut self, new: &Track) {
        if let Some(file) = new.file() {
            self.command_tx
                .send(PlayerInternalCmd::Play(file.to_string()))
                .expect("failed to queue and play");
        }
    }

    pub fn skip_one(&mut self) {
        self.command_tx.send(PlayerInternalCmd::Eos).ok();
    }
}

#[async_trait]
impl PlayerTrait for MpvBackend {
    async fn add_and_play(&mut self, current_item: &Track) {
        self.queue_and_play(current_item);
    }

    fn volume(&self) -> i32 {
        self.volume
    }

    fn volume_up(&mut self) {
        self.volume = cmp::min(self.volume + 5, 100);
        self.set_volume(self.volume);
    }

    fn volume_down(&mut self) {
        self.volume = cmp::max(self.volume - 5, 0);
        self.set_volume(self.volume);
    }
    fn set_volume(&mut self, volume: i32) {
        self.volume = volume.clamp(0, 100);
        self.command_tx
            .send(PlayerInternalCmd::Volume(i64::from(self.volume)))
            .ok();
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
    fn seek_to(&mut self, last_pos: Duration) {
        self.command_tx
            .send(PlayerInternalCmd::SeekAbsolute(last_pos.as_secs() as i64))
            .ok();
    }
    fn speed(&self) -> i32 {
        self.speed
    }

    fn set_speed(&mut self, speed: i32) {
        self.speed = speed;
        self.command_tx
            .send(PlayerInternalCmd::Speed(self.speed))
            .ok();
    }

    fn speed_up(&mut self) {
        let mut speed = self.speed + 1;
        if speed > 30 {
            speed = 30;
        }
        self.set_speed(speed);
    }

    fn speed_down(&mut self) {
        let mut speed = self.speed - 1;
        if speed < 1 {
            speed = 1;
        }
        self.set_speed(speed);
    }
    fn stop(&mut self) {
        self.command_tx.send(PlayerInternalCmd::Stop).ok();
    }

    fn get_progress(&self) -> PlayerProgress {
        PlayerProgress {
            position: *self.position.lock(),
            total_duration: Some(*self.duration.lock()),
        }
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

    fn position_lock(&self) -> parking_lot::MutexGuard<'_, PlayerTimeUnit> {
        self.position.lock()
    }

    fn enqueue_next(&mut self, file: &str) {
        self.enqueue_next(file);
    }
}
