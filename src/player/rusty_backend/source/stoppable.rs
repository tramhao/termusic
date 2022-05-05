use super::{Sample, Source};
use std::time::Duration;

/// Internal function that builds a `Stoppable` object.
pub const fn stoppable<I>(source: I) -> Stoppable<I> {
    Stoppable {
        input: source,
        stopped: false,
    }
}

#[derive(Clone, Debug)]
pub struct Stoppable<I> {
    input: I,
    stopped: bool,
}

#[allow(unused, clippy::missing_const_for_fn)]
impl<I> Stoppable<I> {
    /// Stops the sound.
    #[inline]
    pub fn stop(&mut self) {
        self.stopped = true;
    }

    /// Returns a reference to the inner source.
    #[inline]
    pub const fn inner(&self) -> &I {
        &self.input
    }

    /// Returns a mutable reference to the inner source.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.input
    }

    /// Returns the inner source.
    #[inline]
    pub fn into_inner(self) -> I {
        self.input
    }
}

impl<I> Iterator for Stoppable<I>
where
    I: Source,
    I::Item: Sample,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if self.stopped {
            None
        } else {
            self.input.next()
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input.size_hint()
    }
}

impl<I> Source for Stoppable<I>
where
    I: Source,
    I::Item: Sample,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.input.channels()
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }

    #[inline]
    fn seek(&mut self, time: Duration) -> Option<Duration> {
        self.input.seek(time)
    }

    #[inline]
    fn elapsed(&mut self) -> Duration {
        self.input.elapsed()
    }
}
