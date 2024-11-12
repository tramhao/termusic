//! Custom rodio sources and extension trait

use rodio::{Sample, Source};

#[cfg(feature = "rusty-soundtouch")]
#[allow(clippy::module_name_repetitions)]
pub use self::mix_source::MixSource;
#[cfg(feature = "rusty-soundtouch")]
pub use self::scaletempo::TempoStretch;

// mod amplify;
// #[cfg(feature = "rusty-soundtouch")]
// mod delay;
// mod done;
// mod empty;
#[cfg(feature = "rusty-soundtouch")]
mod mix_source;
// mod pausable;
// mod periodic;
// mod samples_converter;
#[cfg(feature = "rusty-soundtouch")]
pub mod scaletempo;
// mod skippable;
// mod speed;
// mod stoppable;
// mod uniform;
// mod zero;

// pub trait Source: Iterator
// where
//     Self::Item: Sample,
// {
//     fn seek(&mut self, time: Duration) -> Option<Duration>;

//     fn elapsed(&mut self) -> Duration;

//     #[cfg(feature = "rusty-soundtouch")]
//     fn tempo_stretch(self, factor: f32) -> TempoStretch<Self>
//     where
//         Self: Sized,
//         Self: Source<Item = f32>,
//     {
//         scaletempo::tempo_stretch(self, factor)
//     }
// }

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
