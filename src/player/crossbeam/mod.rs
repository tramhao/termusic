// mod mp3;
mod symphonia_decoder;
use crate::player::GeneralP;
use anyhow::Result;
use std::cell::Cell;
use std::cmp;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
// use std::time::Duration;

use crossbeam::sync::SegQueue;
use futures::sync::mpsc::UnboundedSender;
use futures::{AsyncSink, Sink};
use pulse_simple::Playback;

// use self::Action::*;
// use mp3::Decoder;
use symphonia_decoder::SymphoniaDecoder;

const BUFFER_SIZE: usize = 1000;
const DEFAULT_RATE: u32 = 44100;

enum Action {
    Load(PathBuf),
    // Stop,
}

#[derive(Clone)]
pub enum PlayerMsg {
    Play,
    Stop,
    Time(u64),
}

#[derive(Clone)]
struct EventLoop {
    condition_variable: Arc<(Mutex<bool>, Condvar)>,
    queue: Arc<SegQueue<Action>>,
    playing: Arc<Mutex<bool>>,
}

impl Default for EventLoop {
    fn default() -> Self {
        Self {
            condition_variable: Arc::new((Mutex::new(false), Condvar::new())),
            queue: Arc::new(SegQueue::new()),
            playing: Arc::new(Mutex::new(false)),
        }
    }
}

pub struct Player {
    event_loop: EventLoop,
    paused: Cell<bool>,
    // rx: UnboundedReceiver<PlayerMsg>,
    tx: UnboundedSender<PlayerMsg>,
    volume: i32,
}

impl Player {
    pub fn load<P: AsRef<Path>>(&self, path: P) {
        let pathbuf = path.as_ref().to_path_buf();
        self.emit(Action::Load(pathbuf));
        self.set_playing(true);
    }

    pub fn pause(&mut self) {
        self.paused.set(true);
        self.send(PlayerMsg::Stop);
        self.set_playing(false);
    }

    pub fn resume(&mut self) {
        self.paused.set(false);
        self.send(PlayerMsg::Play);
        self.set_playing(true);
    }

    fn emit(&self, action: Action) {
        self.event_loop.queue.push(action);
    }

    fn send(&mut self, msg: PlayerMsg) {
        send(&mut self.tx, msg);
    }

    fn set_playing(&self, playing: bool) {
        *self.event_loop.playing.lock().unwrap() = playing;
        let (ref lock, ref condition_variable) = *self.event_loop.condition_variable;
        let mut started = lock.lock().unwrap();
        *started = playing;
        if playing {
            condition_variable.notify_one();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused.get()
    }

    // pub fn stop(&mut self) {
    //     self.paused.set(false);
    //     self.send(PlayerMsg::Time(0));
    //     self.send(PlayerMsg::Stop);
    //     self.emit(Action::Stop);
    //     self.set_playing(false);
    // }
}

impl Default for Player {
    fn default() -> Self {
        let (tx, _rx) = futures::sync::mpsc::unbounded();
        let event_loop = EventLoop::default();

        {
            // let mut tx = tx.clone();
            let event_loop = event_loop.clone();
            let condition_variable = event_loop.condition_variable.clone();
            thread::spawn(move || -> ! {
                let block = || {
                    let (ref lock, ref condition_variable) = *condition_variable;
                    let mut started = lock.lock().unwrap();
                    *started = false;
                    while !*started {
                        started = condition_variable.wait(started).unwrap();
                    }
                };

                let mut buffer = [[0; 2]; BUFFER_SIZE];
                let mut playback = Playback::new("MP3", "MP3 Playback", None, DEFAULT_RATE);
                let mut source = None;
                loop {
                    if let Some(action) = event_loop.queue.try_pop() {
                        match action {
                            Action::Load(path) => {
                                let file = File::open(path).unwrap();
                                source = Some(Decoder::new(BufReader::new(file)).unwrap());
                                let rate = source
                                    .as_ref()
                                    .map_or(DEFAULT_RATE, mp3::Decoder::samples_rate);
                                playback = Playback::new("MP3", "MP3 Playback", None, rate);
                                // send(&mut tx, PlayerMsg::Play);
                            } // Action::Stop => {
                              //     source = None;
                              // }
                        }
                    } else if *event_loop.playing.lock().unwrap() {
                        let mut written = false;
                        if let Some(ref mut source) = source {
                            let size = iter_to_buffer(source, &mut buffer);
                            if size > 0 {
                                // send(&mut tx, PlayerMsg::Time(source.current_time()));
                                playback.write(&buffer[..size]);
                                written = true;
                            }
                        }

                        if !written {
                            // send(&mut tx, PlayerMsg::Stop);
                            *event_loop.playing.lock().unwrap() = false;
                            source = None;
                            block();
                        }
                    } else {
                        block();
                    }
                }
            });
        }

        Self {
            event_loop,
            paused: Cell::new(false),
            tx,
            // rx,
            volume: 50,
        }
    }
}
fn iter_to_buffer<I: Iterator<Item = i16>>(
    iter: &mut I,
    buffer: &mut [[i16; 2]; BUFFER_SIZE],
) -> usize {
    let mut iter = iter.take(BUFFER_SIZE);
    let mut index = 0;
    while let Some(sample1) = iter.next() {
        if let Some(sample2) = iter.next() {
            buffer[index][0] = sample1;
            buffer[index][1] = sample2;
        }
        index += 1;
    }
    index
}

fn send(tx: &mut UnboundedSender<PlayerMsg>, msg: PlayerMsg) {
    if let Ok(AsyncSink::Ready) = tx.start_send(msg) {
        tx.poll_complete().unwrap();
    } else {
        eprintln!("Unable to send message to sender");
    }
}
impl GeneralP for Player {
    fn add_and_play(&mut self, song: &str) {
        // self.sender.send(PlayerCommand::Play(song.to_string())).ok();
        let path = PathBuf::from(song);
        self.load(path);
    }

    fn volume(&self) -> i32 {
        // self.volume
        75
    }

    fn volume_up(&mut self) {
        self.volume = cmp::min(self.volume + 5, 100);
    }

    fn volume_down(&mut self) {
        self.volume = cmp::max(self.volume - 5, 0);
    }
    fn set_volume(&mut self, mut volume: i32) {
        if volume > 100 {
            volume = 100;
        } else if volume < 0 {
            volume = 0;
        }
        self.volume = volume;
    }

    fn pause(&mut self) {}

    fn resume(&mut self) {}

    fn is_paused(&mut self) -> bool {
        false
    }

    fn seek(&mut self, _secs: i64) -> Result<()> {
        Ok(())
    }

    fn get_progress(&mut self) -> Result<(f64, i64, i64)> {
        Ok((0.9, 0, 100))
    }
}
