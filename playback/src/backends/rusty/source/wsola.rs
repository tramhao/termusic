use std::{fmt::Debug, time::Duration};

use rodio::{ChannelCount, SampleRate, Source, source::SeekError};

use super::SampleType;

/// Wrap the `wsola` implementation in a custom source so that all impls align with termusic
pub struct WSola<I: Source<Item = SampleType>>(rodio_wsola::Wsola<I>);

impl<I> Debug for WSola<I>
where
    I: Source<Item = SampleType>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WSola").finish()
    }
}

impl<I> WSola<I>
where
    I: Source<Item = SampleType>,
{
    /// Wrap the `input` source in a wsola tempo scaler.
    #[inline]
    pub fn new(input: I, initial_speed: f32) -> Self {
        WSola(rodio_wsola::Wsola::new(input, initial_speed))
    }

    /// Modifies the speed factor.
    #[inline]
    pub fn set_factor(&mut self, factor: f32) {
        self.0.set_speed(factor);
    }

    /// Returns a reference to the inner source.
    #[inline]
    #[expect(dead_code)]
    pub fn inner(&self) -> &I {
        // TODO: https://github.com/axel10/rodio-wsola/pull/3
        // &self.input
        unimplemented!()
    }

    /// Returns a mutable reference to the inner source.
    #[inline]
    #[expect(dead_code)]
    pub fn inner_mut(&mut self) -> &mut I {
        // TODO: https://github.com/axel10/rodio-wsola/pull/3
        // &mut self.input
        unimplemented!()
    }

    /// Returns the inner source.
    #[inline]
    #[expect(dead_code)]
    pub fn into_inner(self) -> I {
        self.0.into_inner()
    }
}

impl<I> Iterator for WSola<I>
where
    I: Source<Item = SampleType>,
{
    type Item = SampleType;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<I> Source for WSola<I>
where
    I: Source<Item = SampleType>,
{
    #[inline]
    fn current_span_len(&self) -> Option<usize> {
        self.0.current_span_len()
    }

    fn channels(&self) -> ChannelCount {
        self.0.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.0.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.0.total_duration()
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.0.try_seek(pos)
    }
}
