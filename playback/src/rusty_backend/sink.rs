use parking_lot::{Mutex, RwLock};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

use super::stream::{OutputStreamHandle, PlayError};
use super::{queue, source::Done, PlayerInternalCmd, Sample, Source};
use crate::PlayerCmd;
use cpal::FromSample;

/// Handle to an device that outputs sounds.
///
/// Dropping the `Sink` stops all sounds. You can use `detach` if you want the sounds to continue
/// playing.
pub struct Sink {
    queue_tx: Arc<queue::SourcesQueueInput<f32>>,
    sleep_until_end: Mutex<Option<Receiver<()>>>,
    controls: Arc<Controls>,
    sound_count: Arc<AtomicUsize>,
    detached: bool,
    elapsed: Arc<RwLock<Duration>>,
    message_tx: Sender<PlayerInternalCmd>,
    cmd_tx: Arc<Mutex<UnboundedSender<PlayerCmd>>>,
}

struct Controls {
    pause: AtomicBool,
    volume: Mutex<f32>,
    seek: Mutex<Option<Duration>>,
    stopped: AtomicBool,
    speed: Mutex<f32>,
    to_clear: Mutex<u32>,
}

impl Sink {
    /// Builds a new `Sink`, beginning playback on a stream.
    #[inline]
    pub fn try_new(
        stream: &OutputStreamHandle,
        tx: Sender<PlayerInternalCmd>,
        cmd_tx: Arc<Mutex<UnboundedSender<PlayerCmd>>>,
    ) -> Result<Self, PlayError> {
        let (sink, queue_rx) = Self::new_idle(tx, cmd_tx);
        stream.play_raw(queue_rx)?;
        Ok(sink)
    }
    /// Builds a new `Sink`.
    #[inline]
    pub fn new_idle(
        tx: Sender<PlayerInternalCmd>,
        cmd_tx: Arc<Mutex<UnboundedSender<PlayerCmd>>>,
    ) -> (Self, queue::SourcesQueueOutput<f32>) {
        // pub fn new_idle() -> (Sink, queue::SourcesQueueOutput<f32>) {
        // let (queue_tx, queue_rx) = queue::queue(true);
        let (queue_tx, queue_rx) = queue::queue(true);

        let sink = Sink {
            queue_tx,
            sleep_until_end: Mutex::new(None),
            controls: Arc::new(Controls {
                pause: AtomicBool::new(false),
                volume: Mutex::new(1.0),
                stopped: AtomicBool::new(false),
                seek: Mutex::new(None),
                speed: Mutex::new(1.0),
                to_clear: Mutex::new(0),
            }),
            sound_count: Arc::new(AtomicUsize::new(0)),
            detached: false,
            elapsed: Arc::new(RwLock::new(Duration::from_secs(0))),
            message_tx: tx,
            cmd_tx,
        };
        (sink, queue_rx)
    }

    /// Appends a sound to the queue of sounds to play.
    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub fn append<S>(&self, source: S)
    where
        S: Source + Send + 'static,
        f32: FromSample<S::Item>,
        S::Item: Sample + Send,
    {
        // Wait for queue to flush then resume stopped playback
        if self.controls.stopped.load(Ordering::SeqCst) {
            if self.sound_count.load(Ordering::SeqCst) > 0 {
                self.sleep_until_end();
            }
            self.controls.stopped.store(false, Ordering::SeqCst);
        }

        let controls = self.controls.clone();

        let start_played = AtomicBool::new(false);

        let tx = self.message_tx.clone();
        let elapsed = self.elapsed.clone();
        let source = source
            .speed(1.0)
            .pausable(false)
            .amplify(1.0)
            .skippable()
            .stoppable()
            .periodic_access(Duration::from_millis(500), move |src| {
                let position = src.elapsed().as_secs() as i64;
                tx.send(PlayerInternalCmd::Progress(position)).ok();
            })
            .periodic_access(Duration::from_millis(5), move |src| {
                let src = src.inner_mut();
                if controls.stopped.load(Ordering::SeqCst) {
                    src.stop();
                } else {
                    if let Some(seek_time) = controls.seek.lock().take() {
                        src.seek(seek_time);
                    }
                    *elapsed.write() = src.elapsed();
                    {
                        let mut to_clear = controls.to_clear.lock();
                        if *to_clear > 0 {
                            src.inner_mut().skip();
                            *to_clear -= 1;
                        }
                    }
                    let amp = src.inner_mut().inner_mut();
                    amp.set_factor(*controls.volume.lock());
                    amp.inner_mut()
                        .set_paused(controls.pause.load(Ordering::SeqCst));
                    amp.inner_mut()
                        .inner_mut()
                        .set_factor(*controls.speed.lock());
                    start_played.store(true, Ordering::SeqCst);
                }
            })
            .convert_samples();
        self.sound_count.fetch_add(1, Ordering::Relaxed);
        let source = Done::new(source, self.sound_count.clone());
        *self.sleep_until_end.lock() = Some(self.queue_tx.append_with_signal(source));
    }

    /// Gets the volume of the sound.
    ///
    /// The value `1.0` is the "normal" volume (unfiltered input). Any value other than 1.0 will
    /// multiply each sample by this value.
    #[inline]
    pub fn volume(&self) -> f32 {
        *self.controls.volume.lock()
    }

    /// Changes the volume of the sound.
    ///
    /// The value `1.0` is the "normal" volume (unfiltered input). Any value other than `1.0` will
    /// multiply each sample by this value.
    #[inline]
    pub fn set_volume(&self, value: f32) {
        *self.controls.volume.lock() = value;
    }

    /// Gets the speed of the sound.
    ///
    /// The value `1.0` is the "normal" speed (unfiltered input). Any value other than `1.0` will
    /// change the play speed of the sound.
    #[inline]
    pub fn speed(&self) -> f32 {
        *self.controls.speed.lock()
    }

    /// Changes the speed of the sound.
    ///
    /// The value `1.0` is the "normal" speed (unfiltered input). Any value other than `1.0` will
    /// change the play speed of the sound.
    #[inline]
    pub fn set_speed(&self, value: f32) {
        *self.controls.speed.lock() = value;
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

    /// Gets if a sink is paused
    ///
    /// Sinks can be paused and resumed using `pause()` and `play()`. This returns `true` if the
    /// sink is paused.
    pub fn is_paused(&self) -> bool {
        self.controls.pause.load(Ordering::SeqCst)
    }

    pub fn seek(&self, seek_time: Duration) {
        if self.is_paused() {
            self.play();
        }
        *self.controls.seek.lock() = Some(seek_time);
    }
    /// Toggles playback of the sink
    pub fn toggle_playback(&self) {
        if self.is_paused() {
            self.play();
        } else {
            self.pause();
        }
    }
    /// Removes all currently loaded `Source`s from the `Sink`, and pauses it.
    ///
    /// See `pause()` for information about pausing a `Sink`.
    #[allow(clippy::cast_possible_truncation)]
    pub fn clear(&self) {
        let len = self.sound_count.load(Ordering::SeqCst) as u32;
        *self.controls.to_clear.lock() = len;
        self.sleep_until_end();
        self.pause();
    }

    /// Skips to the next `Source` in the `Sink`
    ///
    /// If there are more `Source`s appended to the `Sink` at the time,
    /// it will play the next one. Otherwise, the `Sink` will finish as if
    /// it had finished playing a `Source` all the way through.
    #[allow(clippy::cast_possible_truncation)]
    pub fn skip_one(&self) {
        let len = self.sound_count.load(Ordering::SeqCst) as u32;
        let mut to_clear = self.controls.to_clear.lock();
        if len > *to_clear {
            *to_clear += 1;
        }
    }

    /// Stops the sink by emptying the queue.
    #[inline]
    pub fn stop(&self) {
        self.controls.stopped.store(true, Ordering::SeqCst);
    }

    /// Destroys the sink without stopping the sounds that are still playing.
    #[inline]
    pub fn detach(mut self) {
        self.detached = true;
    }

    /// Sleeps the current thread until the sound ends.
    #[inline]
    pub fn sleep_until_end(&self) {
        if let Some(sleep_until_end) = self.sleep_until_end.lock().take() {
            let _drop = sleep_until_end.recv();
        }
    }

    /// Returns true if this sink has no more sounds to play.
    #[inline]
    pub fn empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of sounds currently in the queue.
    #[inline]
    pub fn len(&self) -> usize {
        self.sound_count.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        *self.elapsed.read()
    }

    // Spawns a new thread to sleep until the sound ends, and then sends the SoundEnded
    // message through the given Sender.
    pub fn message_on_end(&self) {
        if let Some(sleep_until_end) = self.sleep_until_end.lock().take() {
            let cmd_tx = self.cmd_tx.clone();
            std::thread::Builder::new()
                .name("rusty message_on_end".into())
                .spawn(move || {
                    let _drop = sleep_until_end.recv();
                    if let Err(e) = cmd_tx.lock().send(PlayerCmd::Eos) {
                        error!("Error in message_on_end: {e}");
                    }
                })
                .expect("failed to spawn message_on_end thread");
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
