use std::time::Duration;

use super::Delay;
use super::Sample;

use super::Source;

pub trait MixSource: Source
where
    Self::Item: Sample,
{
    fn set_mix(&mut self, mix: f32);
}

impl<S> Source for Box<dyn MixSource<Item = S>>
where
    S: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        (**self).current_frame_len()
    }

    fn channels(&self) -> u16 {
        (**self).channels()
    }

    fn sample_rate(&self) -> u32 {
        (**self).sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        (**self).total_duration()
    }

    #[inline]
    fn elapsed(&mut self) -> Duration {
        (**self).elapsed()
    }
    #[inline]
    fn seek(&mut self, time: Duration) -> Option<Duration> {
        (**self).seek(time)
    }
}

impl<S> Source for Box<dyn MixSource<Item = S> + Send>
where
    S: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        (**self).current_frame_len()
    }

    fn channels(&self) -> u16 {
        (**self).channels()
    }

    fn sample_rate(&self) -> u32 {
        (**self).sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        (**self).total_duration()
    }

    #[inline]
    fn elapsed(&mut self) -> Duration {
        (**self).elapsed()
    }
    #[inline]
    fn seek(&mut self, time: Duration) -> Option<Duration> {
        (**self).seek(time)
    }
}

impl<S> Source for Box<dyn MixSource<Item = S> + Send + Sync>
where
    S: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        (**self).current_frame_len()
    }

    fn channels(&self) -> u16 {
        (**self).channels()
    }

    fn sample_rate(&self) -> u32 {
        (**self).sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        (**self).total_duration()
    }

    #[inline]
    fn elapsed(&mut self) -> Duration {
        (**self).elapsed()
    }
    #[inline]
    fn seek(&mut self, time: Duration) -> Option<Duration> {
        (**self).seek(time)
    }
}

impl<S> MixSource for Box<dyn MixSource<Item = S>>
where
    S: Sample,
{
    fn set_mix(&mut self, mix: f32) {
        (**self).set_mix(mix);
    }
}

impl<S> MixSource for Box<dyn MixSource<Item = S> + Send>
where
    S: Sample,
{
    fn set_mix(&mut self, mix: f32) {
        (**self).set_mix(mix);
    }
}

impl<S> MixSource for Box<dyn MixSource<Item = S> + Send + Sync>
where
    S: Sample,
{
    fn set_mix(&mut self, mix: f32) {
        (**self).set_mix(mix);
    }
}

pub struct NoMix<I: Source<Item = D>, D: Sample>(pub I);

impl<I, D> Iterator for NoMix<I, D>
where
    I: Source<Item = D>,
    D: Sample,
{
    type Item = D;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<I, D> Source for NoMix<I, D>
where
    I: Source<Item = D>,
    D: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.0.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.0.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.0.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.0.total_duration()
    }

    #[inline]
    fn elapsed(&mut self) -> Duration {
        self.0.elapsed()
    }
    #[inline]
    fn seek(&mut self, time: Duration) -> Option<Duration> {
        self.0.seek(time)
    }
}

impl<I, D> MixSource for NoMix<I, D>
where
    I: Source<Item = D>,
    D: Sample,
{
    fn set_mix(&mut self, _mix: f32) {}
}
impl<I, D> MixSource for Delay<I>
where
    I: MixSource<Item = D>,
    D: Sample,
{
    fn set_mix(&mut self, mix: f32) {
        self.inner_mut().set_mix(mix);
    }
}
