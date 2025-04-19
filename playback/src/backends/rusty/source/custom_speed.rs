use std::time::Duration;

use rodio::Source;

use super::SampleType;

#[allow(clippy::needless_return)]
pub fn custom_speed<I>(input: I, initial_speed: f32) -> CustomSpeed<I>
where
    I: Source<Item = SampleType>,
{
    #[cfg(not(feature = "rusty-soundtouch"))]
    return CustomSpeed::Rodio(input.speed(initial_speed));
    #[cfg(feature = "rusty-soundtouch")]
    return CustomSpeed::SoundTouch(super::soundtouch::soundtouch(input, initial_speed));
}

/// A custom [`Source`] implementation to abstract away which speed module gets chosen.
#[derive(Debug)]
#[allow(dead_code)]
pub enum CustomSpeed<I> {
    Rodio(rodio::source::Speed<I>),
    #[cfg(feature = "rusty-soundtouch")]
    SoundTouch(super::soundtouch::SoundTouchSource<I>),
}

impl<I> Iterator for CustomSpeed<I>
where
    I: Source<Item = SampleType>,
{
    type Item = SampleType;

    fn next(&mut self) -> Option<Self::Item> {
        self.as_source_mut().next()
    }
}

impl<I> ExactSizeIterator for CustomSpeed<I> where I: Source<Item = SampleType> + ExactSizeIterator {}

impl<I> Source for CustomSpeed<I>
where
    I: Source<Item = SampleType>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.as_source().current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.as_source().channels()
    }

    fn sample_rate(&self) -> u32 {
        self.as_source().sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.as_source().total_duration()
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.as_source_mut().try_seek(pos)
    }
}

impl<I> CustomSpeed<I>
where
    I: Source<Item = SampleType>,
{
    #[inline]
    fn as_source(&self) -> &dyn Source<Item = SampleType> {
        match self {
            CustomSpeed::Rodio(speed) => speed,
            #[cfg(feature = "rusty-soundtouch")]
            CustomSpeed::SoundTouch(soundtouch) => soundtouch,
        }
    }

    #[inline]
    fn as_source_mut(&mut self) -> &mut dyn Source<Item = SampleType> {
        match self {
            CustomSpeed::Rodio(speed) => speed,
            #[cfg(feature = "rusty-soundtouch")]
            CustomSpeed::SoundTouch(soundtouch) => soundtouch,
        }
    }

    /// Returns a reference to the inner source.
    #[inline]
    pub fn inner(&self) -> &I {
        match self {
            CustomSpeed::Rodio(speed) => speed.inner(),
            #[cfg(feature = "rusty-soundtouch")]
            CustomSpeed::SoundTouch(soundtouch) => soundtouch.inner(),
        }
    }

    /// Returns a mutable reference to the inner source.
    #[inline]
    #[expect(dead_code)]
    pub fn inner_mut(&mut self) -> &mut I {
        match self {
            CustomSpeed::Rodio(speed) => speed.inner_mut(),
            #[cfg(feature = "rusty-soundtouch")]
            CustomSpeed::SoundTouch(soundtouch) => soundtouch.inner_mut(),
        }
    }

    /// Modifies the speed factor.
    #[inline]
    pub fn set_factor(&mut self, factor: f32) {
        match self {
            CustomSpeed::Rodio(speed) => speed.set_factor(factor),
            #[cfg(feature = "rusty-soundtouch")]
            CustomSpeed::SoundTouch(soundtouch) => soundtouch.set_factor(f64::from(factor)),
        }
    }
}
