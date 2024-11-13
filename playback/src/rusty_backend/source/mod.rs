//! Custom rodio sources and extension trait

use rodio::{Sample, Source};

#[cfg(feature = "rusty-soundtouch")]
pub use self::scaletempo::TempoStretch;

#[cfg(feature = "rusty-soundtouch")]
pub mod scaletempo;

/// Extension trait for [`Source`] for additional custom modifiers
#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)] // currently only used for "rusty-soundtouch"
pub trait SourceExt: Source
where
    Self::Item: Sample,
{
    #[cfg(feature = "rusty-soundtouch")]
    fn tempo_stretch(self, factor: f32) -> TempoStretch<Self>
    where
        Self: Sized,
        Self: Source<Item = f32>,
    {
        scaletempo::tempo_stretch(self, factor)
    }
}

impl<T> SourceExt for T
where
    Self::Item: Sample,
    T: Iterator,
    T: Source,
{
}
