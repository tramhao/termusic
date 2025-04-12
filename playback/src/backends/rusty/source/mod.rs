//! Custom rodio sources and extension trait

use rodio::{Sample, Source};

#[cfg(feature = "rusty-soundtouch")]
pub mod soundtouch;

pub mod async_ring;

/// Extension trait for [`Source`] for additional custom modifiers
#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)] // currently only used for "rusty-soundtouch"
pub trait SourceExt: Source
where
    Self::Item: Sample,
{
    /// Modify samples to sound similar as 1.0 speed when sped-up or slowed-down via [`::soundtouch`] (via `libSoundTouch`)
    #[cfg(feature = "rusty-soundtouch")]
    fn soundtouch(self, factor: f32) -> soundtouch::SoundTouchSource<Self>
    where
        Self: Sized,
        Self: Source<Item = f32>,
    {
        soundtouch::soundtouch(self, factor)
    }
}

impl<T> SourceExt for T
where
    Self::Item: Sample,
    T: Iterator,
    T: Source,
{
}
