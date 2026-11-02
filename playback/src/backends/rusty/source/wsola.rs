use std::{
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
const FLOAT_CMP_ERROR: f32 = 5e-9;

/// Wrap the `wsola` implementation in a custom source so that all impls align with termusic
#[derive(Debug, Clone)]
pub struct WSola<I> {
    /// The inner source where we get the original samples from
    input: I,
    /// The Soundtouch instance where we input all values and get converted values out of
    ts: TimeStretch,

    /// Already processed samples that still need to be output
    out_buffer: Box<[f32]>,
    /// The length of the actual samples in `out_buffer`
    out_write_idx: usize,
    /// The next index to read from `out_buffer`.
    out_read_idx: usize,
    /// Samples we input to be processed
    in_buffer: Vec<f32>,
    /// The next index to read from `in_buffer`.
    in_read_idx: usize,
    /// The timescale factor. `1.0` means no change from the source.
    factor: f32,

    /// Determines if we hit the end on `input`.
    done: bool,
}

impl<I> WSola<I>
where
    I: Source<Item = SampleType>,
{
    /// Wrap the `input` source in a wsola tempo scaler.
    #[inline]
    pub fn new(input: I, initial_speed: f32) -> Self {
        let sample_rate = input.sample_rate();
        let channels = input.channels();
        let ts = Self::create_ts(sample_rate, channels, initial_speed);

        let block_size = Self::get_block_size(sample_rate.get(), channels.get());
        let out_buffer = vec![0.0f32; ts.min_output_buff_size() * 4].into_boxed_slice();

        Self {
            input,
            ts,
            out_buffer,
            out_read_idx: 0,
            out_write_idx: 0,
            in_buffer: Vec::with_capacity(block_size),
            in_read_idx: 0,
            factor: initial_speed,

            done: false,
        }
    }

    /// Get the size of how many samples to buffer for a single processing step.
    fn get_block_size(sample_rate: u32, channels: u16) -> usize {
        (sample_rate as usize / BLOCK_MS) * channels as usize
    }

    /// Create a new `TimeStretch` instance.
    fn create_ts(sample_rate: NonZeroU32, channels: NonZeroU16, factor: f32) -> TimeStretch {
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

    /// Process samples with the Timestretcher.
    fn process_stretch(&mut self) {
        // make sure the timestretch is aligned with the new data format (rodio supports sample rate and channels changing)
        let sample_rate = self.input.sample_rate();
        let channels = self.input.channels();
        let block_size = Self::get_block_size(sample_rate.get(), channels.get());

        if sample_rate != self.ts.sample_rate() || channels != self.ts.channels() {
            // We dont want to immediately reset the Timestretcher and so lose samples, instead we want to first drain it
            // and only *then* change the instance.
            if self.ts.buffered() > 0 {
                self.ts.set_draining();
            } else {
                // This path should practically not happen in normal audio, but rodio supports it, so do we.
                self.ts = Self::create_ts(sample_rate, channels, self.factor);

                self.in_buffer.reserve(block_size);
            }
        } else if (self.factor - self.ts.tempo()).abs() > FLOAT_CMP_ERROR {
            self.ts.set_tempo(self.factor);
        }

        // Fill the input buffer.
        // This works for both the Timestretcher and the bypass.
        self.in_buffer.extend(self.input.by_ref().take(block_size));
        self.in_read_idx = 0;

        if self.in_buffer.is_empty() {
            // EOF
            self.done = true;
            self.ts.set_draining();
        } else if !self.ts.is_draining() {
            // This branch should never run if the Timestretcher is set to be draining, as we dont want to add new data.
            let channels = channels.get();

            // Ensure that we give a full frame to the Timestretcher, in case the underlying source somehow does not.
            // This is a cold path.
            if !self.in_buffer.len().is_multiple_of(channels as usize) {
                error!("Input source gave incomplete frame at EOS!");
                let len = self.in_buffer.len();
                self.in_buffer
                    .truncate(len.saturating_sub(len % channels as usize));
            }

            self.ts.push(&self.in_buffer);
            // We dont need to set it again, as it is added and processed and cleared in the same function.
            // self.in_read_idx = 0;
            self.in_buffer.clear();
        }

        if let Ok(written) = self.ts.pull(&mut self.out_buffer) {
            self.out_write_idx = written;
            self.out_read_idx = 0;
        }
    }

    /// Dont process samples and just gather in a fixed block size.
    /// Assumes the input buffer has been fully exhausted.
    fn process_bypass(&mut self) {
        self.in_buffer.clear();
        self.in_read_idx = 0;
        // The following shouldnt need to be reset, but just to have consistent debug state.
        self.out_write_idx = 0;
        self.out_read_idx = 0;

        let sample_rate = self.input.sample_rate();
        let channels = self.input.channels();
        let block_size = Self::get_block_size(sample_rate.get(), channels.get());
        if sample_rate != self.ts.sample_rate() || channels != self.ts.channels() {
            // This path should practically not happen in normal audio, but rodio supports it, so do we.
            self.ts = Self::create_ts(sample_rate, channels, self.factor);

            self.in_buffer.reserve(block_size);
        }

        self.in_buffer.extend(self.input.by_ref().take(block_size));

        if self.in_buffer.is_empty() {
            self.done = true;
        }
    }
}

impl<I> Iterator for WSola<I>
where
    I: Source<Item = SampleType>,
{
    type Item = SampleType;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Bypass tempo strech for normal speed.
        // This is done as the Timestrecher will add unncessary CPU and latency overhead when not necessary.
        // We also want to drain the input buffer before transitioning to the Timestretcher, as otherwise we could output the wrong channels
        if ((self.factor - 1.0).abs() < FLOAT_CMP_ERROR || self.in_read_idx < self.in_buffer.len())
            && self.out_read_idx >= self.out_write_idx
            // Buffered data can only be 0 if nothing has ever been pushed, or after draining.
            && self.ts.buffered() == 0
        {
            loop {
                if self.in_read_idx < self.in_buffer.len() {
                    let idx = self.in_read_idx;
                    self.in_read_idx += 1;
                    return Some(self.in_buffer[idx]);
                }

                if self.done {
                    return None;
                }

                self.process_bypass();
            }
        }
        // The following commented-out code makes it possible to go back to BYPASS mode once a speed had been non-1.0
        // But due to how the wsola algorithm works, it has audible changes at the beginning and end, so we want to minimize
        // the audible issues.
        // Also it does not consume too much extra CPU if it is always optimized, see
        // https://github.com/jhheider/pdcst/tree/d74c46c9457770372a4f4c9d61fe6de9202d84f6/crates/wsola#performance-in-debug-profile

        // // At this point we know that we still either have something buffered in the Output buffer or the Timestretcher,
        // // But also know that the new factor is normal tempo again, so we just flush out the remaining samples in the Timestretcher
        // // so that we can bypass it again.
        // if (self.factor - 1.0).abs() < FLOAT_CMP_ERROR && !self.ts.is_draining() && self.ts.buffered() > 0 {
        //     trace!("Transition to 1.0 speed again!");
        //     self.ts.set_draining();
        // }
        loop {
            if self.out_read_idx < self.out_write_idx {
                let idx = self.out_read_idx;
                self.out_read_idx += 1;
                return Some(self.out_buffer[idx]);
            }

            if self.done {
                return None;
            }

            self.process_stretch();
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
        self.ts.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.ts.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.in_buffer.clear();
        self.out_read_idx = 0;
        self.out_write_idx = 0;
        self.in_read_idx = 0;
        self.ts.reset();
        self.done = false;

        self.input.try_seek(pos)
    }
}
