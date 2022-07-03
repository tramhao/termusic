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
use super::{PlayerMsg, PlayerTrait};
use crate::config::Settings;
use anyhow::Result;
use libmpv::Mpv;
use libmpv::{
    events::{Event, PropertyData},
    Format,
};
use std::cmp;
use std::sync::mpsc::{self, Receiver, Sender};

pub struct MpvBackend {
    // player: Mpv,
    volume: i32,
    speed: i32,
    pub gapless: bool,
    message_tx: Sender<PlayerMsg>,
    command_tx: Sender<PlayerCmd>,
}

enum PlayerCmd {
    Play(String),
    Pause,
    QueueNext(String),
    Resume,
    Seek(i64),
    Speed(i32),
    Stop,
    Volume(i64),
}

impl MpvBackend {
    pub fn new(config: &Settings, tx: Sender<PlayerMsg>) -> Self {
        let (command_tx, command_rx): (Sender<PlayerCmd>, Receiver<PlayerCmd>) = mpsc::channel();
        let volume = config.volume;
        let speed = config.speed;
        let gapless = config.gapless;
        let message_tx = tx.clone();

        let mpv = Mpv::new().expect("Couldn't initialize MpvHandlerBuilder");
        mpv.set_property("vo", "null")
            .expect("Couldn't set vo=null in libmpv");

        mpv.set_property("volume", i64::from(volume))
            .expect("Error setting volume");
        mpv.set_property("speed", speed as f64 / 10.0).ok();
        let gapless_setting = if gapless { "yes" } else { "no" };
        mpv.set_property("gapless-audio", gapless_setting)
            .expect("gapless setting failed");

        // crossbeam::scope(|scope| {
        // scope.spawn(|_| {
        let mut duration: i64 = 0;
        std::thread::spawn(move || {
            let mut ev_ctx = mpv.create_event_context();
            ev_ctx
                .disable_deprecated_events()
                .expect("failed to disable deprecated events.");
            // ev_ctx
            //     .observe_property("volume", Format::Int64, 0)
            //     .expect("failed to watch volume");
            // ev_ctx
            //     .observe_property("pause", Format::Flag, 0)
            //     .expect("failed to watch volume");
            ev_ctx
                .observe_property("duration", Format::Int64, 0)
                .expect("failed to watch volume");
            ev_ctx
                .observe_property("time-pos", Format::Int64, 0)
                .expect("failed to watch volume");
            // ev_ctx
            //     .observe_property("eof-reached", Format::Flag, 0)
            //     .expect("failed to watch volume");
            loop {
                if let Ok(cmd) = command_rx.try_recv() {
                    match cmd {
                        // PlayerCmd::Eos => message_tx.send(PlayerMsg::Eos).unwrap(),
                        PlayerCmd::Play(new) => {
                            duration = 0;
                            mpv.command("loadfile", &[&format!("\"{}\"", new), "replace"])
                                .expect("Error loading file");
                            eprintln!("add and play {} ok", new);
                        }
                        PlayerCmd::QueueNext(next) => {
                            mpv.command("loadfile", &[&format!("\"{}\"", next), "append"])
                                .expect("Error loading file");
                        }
                        PlayerCmd::Volume(volume) => {
                            mpv.set_property("volume", volume)
                                .expect("Error increase volume");
                        }
                        PlayerCmd::Pause => {
                            mpv.set_property("pause", true)
                                .expect("Toggling pause property");
                        }
                        PlayerCmd::Resume => {
                            mpv.set_property("pause", false)
                                .expect("Toggling pause property");
                        }
                        PlayerCmd::Speed(speed) => {
                            mpv.set_property("speed", speed as f64 / 10.0).ok();
                        }
                        PlayerCmd::Stop => {
                            mpv.command("stop", &[""]).expect("Error stop mpv player");
                        }
                        PlayerCmd::Seek(secs) => {
                            mpv.command("seek", &[&format!("\"{}\"", secs), "relative"])
                                .expect("Seek error");
                        }
                    }
                }

                // This is important to keep the mpv running, otherwise it cannot play.
                std::thread::sleep(std::time::Duration::from_millis(200));

                // if let Some(ev) = ev_ctx.wait_event(600.) {
                if let Some(ev) = ev_ctx.wait_event(0.0) {
                    match ev {
                        Ok(Event::EndFile(e)) => {
                            eprintln!("event end file {:?} received", e);
                            if e == 0 {
                                message_tx.send(PlayerMsg::PlayNextStart).unwrap();
                            }
                        }
                        Ok(Event::PropertyChange {
                            name,
                            change,
                            reply_userdata,
                        }) => match name {
                            "duration" => {
                                if let PropertyData::Int64(c) = change {
                                    duration = c;
                                }
                            }
                            "time-pos" => {
                                if let PropertyData::Int64(c) = change {
                                    message_tx.send(PlayerMsg::Progress(c, duration)).ok();
                                }
                            }
                            &_ => {
                                eprintln!(
                                    "Event not handled {:?}",
                                    Event::PropertyChange {
                                        name,
                                        change,
                                        reply_userdata
                                    }
                                )
                            }
                        },
                        Ok(e) => eprintln!("Event triggered: {:?}", e),
                        Err(e) => eprintln!("Event errored: {:?}", e),
                    }
                }
            }
            // })
        });

        Self {
            volume,
            speed,
            gapless,
            message_tx: tx,
            command_tx,
        }
    }

    pub fn enqueue_next(&mut self, next: &str) {
        self.command_tx
            .send(PlayerCmd::QueueNext(next.to_string()))
            .ok();
    }

    fn queue_and_play(&mut self, new: &str) {
        self.command_tx
            .send(PlayerCmd::Play(new.to_string()))
            .expect("failed to queue and play");
    }

    pub fn skip_one(&mut self) {
        self.message_tx.send(PlayerMsg::Eos).unwrap();
    }
}

impl PlayerTrait for MpvBackend {
    fn add_and_play(&mut self, current_item: &str) {
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
    fn set_volume(&mut self, mut volume: i32) {
        if volume > 100 {
            volume = 100;
        } else if volume < 0 {
            volume = 0;
        }
        self.volume = volume;
        self.command_tx
            .send(PlayerCmd::Volume(i64::from(self.volume)))
            .ok();
    }

    fn pause(&mut self) {
        self.command_tx.send(PlayerCmd::Pause).ok();
    }

    fn resume(&mut self) {
        self.command_tx.send(PlayerCmd::Resume).ok();
    }

    fn is_paused(&self) -> bool {
        // self.player
        //     .get_property("pause")
        //     .expect("wrong paused state")
        true
    }

    fn seek(&mut self, secs: i64) -> Result<()> {
        self.command_tx.send(PlayerCmd::Seek(secs)).ok();
        Ok(())
    }

    fn get_progress(&mut self) -> Result<(f64, i64, i64)> {
        // let percent_pos = self
        //     .player
        //     .get_property::<f64>("percent-pos")
        //     .unwrap_or(0.0);
        // // let percent = percent_pos / 100_f64;
        // let time_pos = self.player.get_property::<i64>("time-pos").unwrap_or(0);
        // let duration = self.player.get_property::<i64>("duration").unwrap_or(0);
        // Ok((percent_pos, time_pos, duration))
        Ok((0.5, 1, 100))
    }

    fn speed(&self) -> i32 {
        self.speed
    }

    fn set_speed(&mut self, speed: i32) {
        self.speed = speed;
        self.command_tx.send(PlayerCmd::Speed(self.speed)).ok();
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
        self.command_tx.send(PlayerCmd::Stop).ok();
    }
}
