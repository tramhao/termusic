use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
// use std::{
//     collections::VecDeque,
//     sync::atomic::{AtomicBool, AtomicUsize, Ordering},
// };
use crate::player::PlayerMsg;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;

use super::{queue, source::Done, Sample, Source};
use super::{OutputStreamHandle, PlayError};

/// Handle to an device that outputs sounds.
///
/// Dropping the `Sink` stops all sounds. You can use `detach` if you want the sounds to continue
/// playing.
pub struct Sink {
    queue_tx: Arc<queue::SourcesQueueInput<f32>>,
    // sleep_until_end: Mutex<VecDeque<Receiver<()>>>,
    sleep_until_end: Mutex<Option<Receiver<()>>>,

    controls: Arc<Controls>,
    sound_count: Arc<AtomicUsize>,

    detached: bool,

    elapsed: Arc<RwLock<Duration>>,
    message_tx: Sender<PlayerMsg>,
}

struct Controls {
    pause: AtomicBool,
    volume: Mutex<f32>,
    seek: Mutex<Option<Duration>>,
    stopped: AtomicBool,
    speed: Mutex<f32>,
    do_skip: AtomicBool,
}

#[allow(unused)]
impl Sink {
    /// Builds a new `Sink`, beginning playback on a stream.
    #[inline]
    pub fn try_new(
        stream: &OutputStreamHandle,
        gapless_playback: bool,
        tx: Sender<PlayerMsg>,
    ) -> Result<Self, PlayError> {
        let (sink, queue_rx) = Self::new_idle(gapless_playback, tx);
        stream.play_raw(queue_rx)?;
        Ok(sink)
    }

    /// Builds a new `Sink`.
    #[inline]
    pub fn new_idle(
        gapless_playback: bool,
        tx: Sender<PlayerMsg>,
    ) -> (Self, queue::SourcesQueueOutput<f32>) {
        let (queue_tx, queue_rx) = queue::queue(true, gapless_playback);

        let sink = Self {
            queue_tx,
            sleep_until_end: Mutex::new(None),
            controls: Arc::new(Controls {
                pause: AtomicBool::new(false),
                volume: Mutex::new(1.0),
                stopped: AtomicBool::new(false),
                seek: Mutex::new(None),
                speed: Mutex::new(1.0),
                do_skip: AtomicBool::new(false),
            }),
            sound_count: Arc::new(AtomicUsize::new(0)),
            detached: false,
            elapsed: Arc::new(RwLock::new(Duration::from_secs(0))),
            message_tx: tx,
        };
        (sink, queue_rx)
    }

    /// Appends a sound to the queue of sounds to play.
    #[inline]
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub fn append<S>(&self, source: S)
    where
        S: Source + Send + 'static,
        S::Item: Sample + Send,
    {
        let controls = self.controls.clone();

        let tx = self.message_tx.clone();
        let elapsed = self.elapsed.clone();
        let source = source
            .speed(1.0)
            .pausable(false)
            .amplify(1.0)
            .skippable()
            .stoppable()
            .periodic_access(Duration::from_secs(1), move |src| {
                let position = src.elapsed().as_secs() as i64;
                let duration = src
                    .total_duration()
                    .map_or(99.0, |duration| duration.as_secs_f64() - 0.29)
                    as i64;
                tx.send(PlayerMsg::Progress(position, duration)).ok();
            })
            .periodic_access(Duration::from_millis(50), move |src| {
                let mut src = src.inner_mut();
                if controls.stopped.load(Ordering::SeqCst) {
                    src.stop();
                } else if controls.do_skip.load(Ordering::SeqCst) {
                    src.inner_mut().skip();
                    controls.do_skip.store(false, Ordering::SeqCst);
                } else {
                    if let Some(seek_time) = controls.seek.lock().unwrap().take() {
                        src.seek(seek_time).unwrap();
                        // while src.seek(seek_time).is_none() {
                        //     std::thread::sleep(Duration::from_millis(100));
                        // }
                        // src.seek(seek_time);
                    }
                    *elapsed.write().unwrap() = src.elapsed();

                    // src.inner_mut().set_factor(*controls.volume.lock().unwrap());
                    // Workaround for buffer underrun issue
                    // If song is started while volume is set to 0, it causes a buffer underrun on alsa
                    let mut new_factor = *controls.volume.lock().unwrap();
                    if new_factor < 0.0001 {
                        new_factor = 0.0001;
                    }
                    src.inner_mut().inner_mut().set_factor(new_factor);
                    src.inner_mut()
                        .inner_mut()
                        .inner_mut()
                        .set_paused(controls.pause.load(Ordering::SeqCst));
                    src.inner_mut()
                        .inner_mut()
                        .inner_mut()
                        .inner_mut()
                        .set_factor(*controls.speed.lock().unwrap());
                }
            })
            .convert_samples();
        self.sound_count.fetch_add(1, Ordering::Relaxed);
        let source = Done::new(source, self.sound_count.clone());
        *self.sleep_until_end.lock().unwrap() = Some(self.queue_tx.append_with_signal(source));
    }

    /// Gets the volume of the sound.
    ///
    /// The value `1.0` is the "normal" volume (unfiltered input). Any value other than 1.0 will
    /// multiply each sample by this value.
    #[inline]
    pub fn volume(&self) -> f32 {
        *self.controls.volume.lock().unwrap()
    }

    /// Changes the volume of the sound.
    ///
    /// The value `1.0` is the "normal" volume (unfiltered input). Any value other than `1.0` will
    /// multiply each sample by this value.
    #[inline]
    pub fn set_volume(&self, value: f32) {
        *self.controls.volume.lock().unwrap() = value;
    }

    /// Resumes playback of a paused sink.
    ///
    /// No effect if not paused.
    #[inline]
    pub fn play(&self) {
        self.controls.pause.store(false, Ordering::SeqCst);
    }

    /// Pauses playback of this sink.
    ///
    /// No effect if already paused.
    ///
    /// A paused sink can be resumed with `play()`.
    pub fn pause(&self) {
        self.controls.pause.store(true, Ordering::SeqCst);
    }

    /// Toggles playback of the sink
    pub fn toggle_playback(&self) {
        if self.is_paused() {
            self.play();
        } else {
            self.pause();
        }
    }

    pub fn seek(&self, seek_time: Duration) {
        if self.is_paused() {
            self.play();
        }
        *self.controls.seek.lock().unwrap() = Some(seek_time);
    }

    /// Gets if a sink is paused
    ///
    /// Sinks can be paused and resumed using `pause()` and `play()`. This returns `true` if the
    /// sink is paused.
    pub fn is_paused(&self) -> bool {
        self.controls.pause.load(Ordering::SeqCst)
    }

    /// Destroys the sink without stopping the sounds that are still playing.
    #[inline]
    pub fn detach(mut self) {
        self.detached = true;
    }

    /// Sleeps the current thread until the sound ends.
    #[inline]
    pub fn sleep_until_end(&self) {
        if let Some(sleep_until_end) = self.sleep_until_end.lock().unwrap().take() {
            let _ = sleep_until_end.recv();
        }
    }

    // pub fn get_current_receiver(&self) -> Option<Receiver<()>> {
    //     self.sleep_until_end.lock().unwrap().pop_front()
    // }
    /// Returns true if this sink has no more sounds to play.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of sounds currently in the queue.
    #[inline]
    pub fn len(&self) -> usize {
        self.sound_count.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        *self.elapsed.read().unwrap()
    }

    /// Gets the speed of the sound.
    ///
    /// The value `1.0` is the "normal" speed (unfiltered input). Any value other than `1.0` will
    /// change the play speed of the sound.
    #[inline]
    pub fn speed(&self) -> f32 {
        *self.controls.speed.lock().unwrap()
    }

    /// Changes the speed of the sound.
    ///
    /// The value `1.0` is the "normal" speed (unfiltered input). Any value other than `1.0` will
    /// change the play speed of the sound.
    #[inline]
    pub fn set_speed(&self, value: f32) {
        *self.controls.speed.lock().unwrap() = value;
    }

    /// Removes all currently loaded `Source`s from the `Sink`, and pauses it.
    ///
    /// See `pause()` for information about pausing a `Sink`.
    pub fn clear(&self) {
        let len = self.queue_tx.clear();
        self.sound_count.fetch_sub(len, Ordering::SeqCst);
        self.pause();
    }

    /// Skips to the next `Source` in the `Sink`
    ///
    /// If there are more `Source`s appended to the `Sink` at the time,
    /// it will play the next one. Otherwise, the `Sink` will finish as if
    /// it had finished playing a `Source` all the way through.
    pub fn skip_one(&self) {
        self.controls.do_skip.store(true, Ordering::SeqCst);
    }

    // Spawns a new thread to sleep until the sound ends, and then sends the SoundEnded
    // message through the given Sender.
    pub fn message_on_end(&self) {
        // let tx1 = Sender::clone(&self.message_tx);
        let tx1 = self.message_tx.clone();
        if let Some(sleep_until_end) = self.sleep_until_end.lock().unwrap().take() {
            std::thread::spawn(move || {
                let _ = sleep_until_end.recv();
                tx1.send(PlayerMsg::Eos).ok();
                // if let Err(e) = tx1.send(PlayerMsg::Eos) {
                //     eprintln!("Error is: {}", e);
                // }
            });
        }
    }
}

impl Drop for Sink {
    #[inline]
    fn drop(&mut self) {
        self.queue_tx.set_keep_alive_if_empty(false);

        if !self.detached {
            self.controls.stopped.store(true, Ordering::Relaxed);
        }
    }
}
