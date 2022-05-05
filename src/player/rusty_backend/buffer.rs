//! A simple source of samples coming from a buffer.
//!
//! The `SamplesBuffer` struct can be used to treat a list of values as a `Source`.
//!
//! # Example
//!
//! ```
//! use rodio::buffer::SamplesBuffer;
//! let _ = SamplesBuffer::new(1, 44100, vec![1i16, 2, 3, 4, 5, 6]);
//! ```
//!

use std::time::Duration;
use std::vec::IntoIter as VecIntoIter;

use super::{Sample, Source};

/// A buffer of samples treated as a source.
#[allow(clippy::module_name_repetitions)]
pub struct SamplesBuffer<S> {
    data: VecIntoIter<S>,
    channels: u16,
    sample_rate: u32,
    duration: Duration,
}

#[allow(unused, clippy::use_self)]
impl<S> SamplesBuffer<S>
where
    S: Sample,
{
    /// Builds a new `SamplesBuffer`.
    ///
    /// # Panic
    ///
    /// - Panics if the number of channels is zero.
    /// - Panics if the samples rate is zero.
    /// - Panics if the length of the buffer is larger than approximately 16 billion elements.
    ///   This is because the calculation of the duration would overflow.
    ///
    #[allow(clippy::cast_possible_truncation)]
    pub fn new<D>(channels: u16, sample_rate: u32, data: D) -> SamplesBuffer<S>
    where
        D: Into<Vec<S>>,
    {
        assert!(channels != 0);
        assert!(sample_rate != 0);

        let data = data.into();
        let duration_ns = 1_000_000_000_u64.checked_mul(data.len() as u64).unwrap()
            / u64::from(sample_rate)
            / u64::from(channels);
        let duration = Duration::new(
            duration_ns / 1_000_000_000,
            (duration_ns % 1_000_000_000) as u32,
        );

        SamplesBuffer {
            data: data.into_iter(),
            channels,
            sample_rate,
            duration,
        }
    }
}

#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
impl<S> Source for SamplesBuffer<S>
where
    S: Sample,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.channels
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        Some(self.duration)
    }

    #[inline]
    fn elapsed(&mut self) -> Duration {
        Duration::from_secs(0)
    }

    fn seek(&mut self, seek_time: Duration) -> Option<Duration> {
        let iters =
            (self.sample_rate as f32 / 1000.0 * seek_time.as_millis() as f32).round() as u32;
        for i in 0..iters {
            self.data.next().ok_or(i).unwrap();
        }
        Some(seek_time)
    }
}

impl<S> Iterator for SamplesBuffer<S>
where
    S: Sample,
{
    type Item = S;

    #[inline]
    fn next(&mut self) -> Option<S> {
        self.data.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.data.size_hint()
    }
}
