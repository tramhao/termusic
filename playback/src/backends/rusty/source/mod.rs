//! Custom rodio sources and extension trait

pub use custom_speed::SpecificType;
use rodio::{Sample, Source};

#[cfg(feature = "rusty-soundtouch")]
pub mod soundtouch;

pub mod async_ring;
mod cb_done;
mod custom_speed;

/// Our sample type we choose to use across all places
pub type SampleType = f32;

/// Extension trait for [`Source`] for additional custom modifiers
#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)] // currently only used for "rusty-soundtouch"
pub trait SourceExt: Source
where
    Self::Item: Sample,
{
    /// A custom [`Source`] implementation to abstract away which speed module gets chosen.
    fn custom_speed(
        self,
        initial_speed: f32,
        specific: SpecificType,
    ) -> custom_speed::CustomSpeed<Self>
    where
        Self: Sized,
        Self: Source<Item = f32>,
    {
        custom_speed::custom_speed(self, initial_speed, specific)
    }

    /// Run a function once at the end of a source.
    fn cbdone<Fn: FnOnce()>(self, fun: Fn) -> cb_done::CbDone<Self, Fn>
    where
        Self: Sized,
    {
        cb_done::CbDone::new(self, fun)
    }
}

impl<T> SourceExt for T
where
    Self::Item: Sample,
    T: Iterator,
    T: Source,
{
}
