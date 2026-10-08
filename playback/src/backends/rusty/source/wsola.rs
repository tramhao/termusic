use std::{
    collections::VecDeque,
    fmt::Debug,
    num::{NonZeroU16, NonZeroU32},
    time::Duration,
};

use rodio::{ChannelCount, SampleRate, Source, source::SeekError};
use wsola::TimeStretch;

use super::SampleType;

/// How many MS to buffer in the Timestretcher.
/// This affects how fast a speed (or sample rate or channel change) will be reflected.
const BLOCK_MS: usize = 128;
const FLOAT_CMP_ERROR: f32 = 0.000_000_005;

/// Wrap the `wsola` implementation in a custom source so that all impls align with termusic
#[derive(Debug, Clone)]
pub struct WSola<I> {
    /// The inner source where we get the original samples from
    input: I,
    /// The Soundtouch instance where we input all values and get converted values out of
    ts: TimeStretch,

    /// Already processed samples that still need to be output
    out_buffer: VecDeque<f32>,
    /// Samples we input to be processed
    in_buffer: Vec<f32>,
    /// The timescale factor. `1.0` means no change from the source.
    factor: f32,

    done: bool,
}

impl<I> WSola<I>
where
    I: Source<Item = SampleType>,
{
    /// Wrap the `input` source in a wsola tempo scaler.
    #[inline]
    pub fn new(input: I, initial_speed: f32) -> Self {
        debug!("Using wsola as TimeStretcher");

        let sample_rate = input.sample_rate().get();
        let channels = input.channels().get();
        let ts = Self::create_ts(sample_rate, channels, initial_speed);

        let block_size = Self::get_block_size(sample_rate, channels);

        Self {
            input,
            ts,
            out_buffer: VecDeque::with_capacity(block_size),
            in_buffer: Vec::with_capacity(block_size),
            factor: initial_speed,

            done: false,
        }
    }

    /// Get the size of how many samples to buffer for a single processing step.
    fn get_block_size(sample_rate: u32, channels: u16) -> usize {
        (sample_rate as usize / BLOCK_MS) * channels as usize
    }

    /// Create a new `TimeStretch` instance.
    fn create_ts(sample_rate: u32, channels: u16, factor: f32) -> TimeStretch {
        let mut ts = TimeStretch::new(sample_rate, channels)
            .expect("Expected TimeStretch to successfully be created");
        ts.set_tempo(factor);

        ts
    }

    /// Modifies the speed factor.
    #[inline]
    pub fn set_factor(&mut self, factor: f32) {
        self.factor = factor;
    }

    /// Returns a reference to the inner source.
    #[inline]
    pub fn inner(&self) -> &I {
        &self.input
    }

    /// Returns a mutable reference to the inner source.
    #[inline]
    #[expect(dead_code)]
    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.input
    }

    /// Returns the inner source.
    #[inline]
    #[expect(dead_code)]
    pub fn into_inner(self) -> I {
        self.input
    }

    /// Refill the output buffer with the Timestretcher.
    fn refill(&mut self) {
        if self.ts.available() > 0 {
            let res = self.ts.pull(self.ts.available());
            self.out_buffer.extend(res);

            return;
        }

        // make sure the timestretch is aligned with the new data format (rodio supports sample rate and channels changing)
        let sample_rate = self.input.sample_rate().get();
        let channels = self.input.channels().get();
        let block_size = Self::get_block_size(sample_rate, channels);

        if sample_rate != self.ts.sample_rate() || channels != self.ts.channels() {
            // this path should practically not happen in normal audio, but rodio supports it, so do we
            self.ts = Self::create_ts(sample_rate, channels, self.factor);

            self.in_buffer.reserve(block_size);
            self.out_buffer.reserve(block_size);
        } else if (self.factor - self.ts.tempo()).abs() > FLOAT_CMP_ERROR {
            self.ts.set_tempo(self.factor);
        }

        // fill the input buffer
        self.in_buffer.extend(self.input.by_ref().take(block_size));

        // EOF
        if self.in_buffer.is_empty() {
            let res = self.ts.flush();
            self.out_buffer.extend(res);
            self.done = true;
            return;
        }

        // Ensure that we give a full frame to the stretcher, in case the underlying source somehow does not.
        // This is a cold path.
        if !self.in_buffer.len().is_multiple_of(channels as usize) {
            error!("Input source gave incomplete frame at EOS!");
            let len = self.in_buffer.len();
            self.in_buffer
                .truncate(len.saturating_sub(len % channels as usize));
        }

        self.ts.push(&self.in_buffer);
        self.in_buffer.clear();

        let res = self.ts.pull(usize::MAX);
        self.out_buffer.extend(res);
    }
}

impl<I> Iterator for WSola<I>
where
    I: Source<Item = SampleType>,
{
    type Item = SampleType;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // TODO: normal tempo bypass stretcher.

        loop {
            if let Some(sample) = self.out_buffer.pop_front() {
                return Some(sample);
            }
            if self.done {
                return None;
            }

            self.refill();
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input.size_hint()
    }
}

impl<I> Source for WSola<I>
where
    I: Source<Item = SampleType>,
{
    #[inline]
    fn current_span_len(&self) -> Option<usize> {
        Some(self.out_buffer.len())
    }

    fn channels(&self) -> ChannelCount {
        // This should never panic as the channels are initialized by rodio's values.
        NonZeroU16::new(self.ts.channels()).unwrap()
    }

    fn sample_rate(&self) -> SampleRate {
        // This should never panic as the sample_rate are initialized by rodio's values.
        NonZeroU32::new(self.ts.sample_rate()).unwrap()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.in_buffer.clear();
        self.out_buffer.clear();
        self.ts.reset();
        self.done = false;

        self.input.try_seek(pos)
    }
}
