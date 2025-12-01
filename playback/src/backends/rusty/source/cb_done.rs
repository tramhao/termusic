use std::time::Duration;

use rodio::{Source, source::SeekError};

use super::SampleType;

// This type is similar to `rodio::source::Done`, but that type has one function, but we actually accept a function

/// Call a function at the end of the inner source.
#[derive(Debug, Clone)]
pub struct CbDone<I, Fn> {
    input: I,
    fun: Option<Fn>,
}

impl<I, Fn> CbDone<I, Fn>
where
    Fn: FnOnce(),
{
    /// Wrap the `input` source in a Done Callback that calls a function.
    #[inline]
    pub fn new(input: I, fun: Fn) -> CbDone<I, Fn> {
        CbDone {
            input,
            fun: Some(fun),
        }
    }

    /// Returns a reference to the inner source.
    #[inline]
    #[expect(dead_code)]
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
}

impl<I, Fn> Iterator for CbDone<I, Fn>
where
    I: Source<Item = SampleType>,
    Fn: FnOnce(),
{
    type Item = SampleType;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.input.next();
        if next.is_none()
            && let Some(fun) = self.fun.take()
        {
            fun();
        }
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input.size_hint()
    }
}

impl<I, Fn> Source for CbDone<I, Fn>
where
    I: Source<Item = SampleType>,
    Fn: FnOnce(),
{
    #[inline]
    fn current_span_len(&self) -> Option<usize> {
        self.input.current_span_len()
    }

    fn channels(&self) -> u16 {
        self.input.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.input.try_seek(pos)
    }
}
