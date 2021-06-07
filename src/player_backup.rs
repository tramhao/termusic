use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

use rodio::source::Source;
use rodio::Sample;
use tui::widgets::ListState;

pub struct Player {
    pub list_state: ListState,
    _stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    sink: rodio::Sink,
    volume: f32,
    playing: Vec<PathBuf>,
    remaining: Arc<AtomicUsize>,
}

impl Player {
    pub fn new(volume: f32) -> Result<Player, rodio::StreamError> {
        let list_state = ListState::default();

        let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
        let (sink, _) = rodio::Sink::new_idle();

        let playing = Vec::new();
        let remaining = Arc::new(AtomicUsize::new(0));

        Ok(Player {
            list_state,
            _stream,
            stream_handle,
            sink,
            volume,
            playing,
            remaining,
        })
    }

    fn reset_sink(&mut self) {
        // FIXME: actually handle the error instead of just expecting
        self.sink = rodio::Sink::try_new(&self.stream_handle).expect("error opening sink");
        self.sink.set_volume(self.volume);
    }

    pub fn play_song(&mut self, p: PathBuf) -> io::Result<Receiver<usize>> {
        self.play_songs(0, vec![p])
    }

    pub fn play_songs(&mut self, start: usize, dir: Vec<PathBuf>) -> io::Result<Receiver<usize>> {
        self.reset_sink();
        let remaining = &self.remaining;
        remaining.store(dir.len() - start, Ordering::Relaxed);
        self.playing = dir.clone();

        let (sender, receiver) = channel::<usize>();
        for path in dir[start..].to_vec() {
            let f = File::open(path)?;
            let source = rodio::Decoder::new(BufReader::new(f)).expect("error decoding file");
            self.sink
                .append(Signal::new(source, self.remaining.clone(), sender.clone()));
        }

        Ok(receiver)
    }

    pub fn playing(&self) -> &Vec<PathBuf> {
        &self.playing
    }

    pub fn index(&self) -> usize {
        self.playing.len() - self.remaining.load(Ordering::Relaxed)
    }

    pub fn toggle_pause(&self) {
        if self.sink.is_paused() {
            self.sink.play()
        } else {
            self.sink.pause()
        }
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, v: f32) {
        self.volume = if v < 0f32 {
            0f32
        } else if v > 1f32 {
            1f32
        } else {
            v
        };
        self.sink.set_volume(self.volume);
    }
}

/// Send a message on the given Sender and decrement an AtomicUsize when the inner Source is empty.
/// Like rodio's built in Done, but with a channel.
pub struct Signal<I> {
    input: I,
    num: Arc<AtomicUsize>,
    sender: Sender<usize>,
    sent: bool,
}

impl<I> Signal<I> {
    pub fn new(input: I, num: Arc<AtomicUsize>, sender: Sender<usize>) -> Signal<I> {
        Signal {
            input,
            num,
            sender,
            sent: false,
        }
    }
}

impl<I: Source> Iterator for Signal<I>
where
    I: Source,
    I::Item: Sample,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        let next = self.input.next();
        if !self.sent && next.is_none() {
            // with Ordering::Relaxed these might happen out of order, but idk xd
            let n = self.num.fetch_sub(1, Ordering::Relaxed);
            let result = self.sender.send(self.num.load(Ordering::Relaxed));
            if n != 1 && result.is_err() {
                eprintln!("error writing to channel: {}", result.err().unwrap());
            }
            self.sent = true;
        }
        next
    }
}

impl<I> Source for Signal<I>
where
    I: Source,
    I::Item: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.input.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}
