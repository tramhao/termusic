use std::marker::PhantomData;
use std::time::Duration;

use super::{Sample, Source};

/// An infinite source that produces zero.
#[derive(Clone, Debug)]
pub struct Zero<S> {
    channels: u16,
    sample_rate: u32,
    marker: PhantomData<S>,
}

impl<S> Zero<S> {
    #[inline]
    pub const fn new(channels: u16, sample_rate: u32) -> Self {
        Self {
            channels,
            sample_rate,
            marker: PhantomData,
        }
    }
}

impl<S> Iterator for Zero<S>
where
    S: Sample,
{
    type Item = S;

    #[inline]
    fn next(&mut self) -> Option<S> {
        Some(S::zero_value())
    }
}

impl<S> Source for Zero<S>
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
        None
    }
    #[inline]
    fn elapsed(&mut self) -> Duration {
        Duration::from_secs(0)
    }
    fn seek(&mut self, time: Duration) -> Option<Duration> {
        Some(time)
    }
}
