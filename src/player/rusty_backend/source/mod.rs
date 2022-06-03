//! Sources of sound and various filters.

use std::time::Duration;

use super::Sample;

pub use self::amplify::Amplify;
pub use self::done::Done;
pub use self::empty::Empty;
pub use self::fadein::FadeIn;
pub use self::pausable::Pausable;
pub use self::periodic::PeriodicAccess;
pub use self::samples_converter::SamplesConverter;
pub use self::skippable::Skippable;
pub use self::speed::Speed;
pub use self::stoppable::Stoppable;
pub use self::take::TakeDuration;
pub use self::uniform::UniformSourceIterator;
pub use self::zero::Zero;

mod amplify;
mod done;
mod empty;
mod fadein;
mod pausable;
mod periodic;
mod samples_converter;
mod skippable;
mod speed;
mod stoppable;
mod take;
mod uniform;
mod zero;

/// A source of samples.
///
/// # A quick lesson about sounds
///
/// ## Sampling
///
/// A sound is a vibration that propagates through air and reaches your ears. This vibration can
/// be represented as an analog signal.
///
/// In order to store this signal in the computer's memory or on the disk, we perform what is
/// called *sampling*. The consists in choosing an interval of time (for example 20µs) and reading
/// the amplitude of the signal at each interval (for example, if the interval is 20µs we read the
/// amplitude every 20µs). By doing so we obtain a list of numerical values, each value being
/// called a *sample*.
///
/// Therefore a sound can be represented in memory by a frequency and a list of samples. The
/// frequency is expressed in hertz and corresponds to the number of samples that have been
/// read per second. For example if we read one sample every 20µs, the frequency would be
/// 50000 Hz. In reality, common values for the frequency are 44100, 48000 and 96000.
///
/// ## Channels
///
/// But a frequency and a list of values only represent one signal. When you listen to a sound,
/// your left and right ears don't receive exactly the same signal. In order to handle this,
/// we usually record not one but two different signals: one for the left ear and one for the right
/// ear. We say that such a sound has two *channels*.
///
/// Sometimes sounds even have five or six channels, each corresponding to a location around the
/// head of the listener.
///
/// The standard in audio manipulation is to *interleave* the multiple channels. In other words,
/// in a sound with two channels the list of samples contains the first sample of the first
/// channel, then the first sample of the second channel, then the second sample of the first
/// channel, then the second sample of the second channel, and so on. The same applies if you have
/// more than two channels. The rodio library only supports this schema.
///
/// Therefore in order to represent a sound in memory in fact we need three characteristics: the
/// frequency, the number of channels, and the list of samples.
///
/// ## The `Source` trait
///
/// A Rust object that represents a sound should implement the `Source` trait.
///
/// The three characteristics that describe a sound are provided through this trait:
///
/// - The number of channels can be retrieved with `channels`.
/// - The frequency can be retrieved with `sample_rate`.
/// - The list of values can be retrieved by iterating on the source. The `Source` trait requires
///   that the `Iterator` trait be implemented as well.
///
/// # Frames
///
/// The samples rate and number of channels of some sound sources can change by itself from time
/// to time.
///
/// > **Note**: As a basic example, if you play two audio files one after the other and treat the
/// > whole as a single source, then the channels and samples rate of that source may change at the
/// > transition between the two files.
///
/// However, for optimization purposes rodio supposes that the number of channels and the frequency
/// stay the same for long periods of time and avoids calling `channels()` and
/// `sample_rate` too frequently.
///
/// In order to properly handle this situation, the `current_frame_len()` method should return
/// the number of samples that remain in the iterator before the samples rate and number of
/// channels can potentially change.
///
pub trait Source: Iterator
where
    Self::Item: Sample,
{
    /// Returns the number of samples before the current frame ends. `None` means "infinite" or
    /// "until the sound ends".
    /// Should never return 0 unless there's no more data.
    ///
    /// After the engine has finished reading the specified number of samples, it will check
    /// whether the value of `channels()` and/or `sample_rate()` have changed.
    fn current_frame_len(&self) -> Option<usize>;

    /// Returns the number of channels. Channels are always interleaved.
    fn channels(&self) -> u16;

    /// Returns the rate at which the source should be played. In number of samples per second.
    fn sample_rate(&self) -> u32;

    /// Returns the total duration of this source, if known.
    ///
    /// `None` indicates at the same time "infinite" or "unknown".
    fn total_duration(&self) -> Option<Duration>;

    fn seek(&mut self, time: Duration) -> Option<Duration>;

    fn elapsed(&mut self) -> Duration;

    /// Takes a certain duration of this source and then stops.
    #[inline]
    fn take_duration(self, duration: Duration) -> TakeDuration<Self>
    where
        Self: Sized,
    {
        take::take_duration(self, duration)
    }

    /// Immediately skips a certain duration of this source.
    ///
    /// If the specified duration is longer than the source itself, `skip_duration` will skip to the end of the source.

    /// Amplifies the sound by the given value.
    #[inline]
    fn amplify(self, value: f32) -> Amplify<Self>
    where
        Self: Sized,
    {
        amplify::amplify(self, value)
    }

    /// Fades in the sound.
    #[inline]
    fn fade_in(self, duration: Duration) -> FadeIn<Self>
    where
        Self: Sized,
    {
        fadein::fadein(self, duration)
    }

    /// Calls the `access` closure on `Self` the first time the source is iterated and every
    /// time `period` elapses.
    ///
    /// Later changes in either `sample_rate()` or `channels_count()` won't be reflected in
    /// the rate of access.
    ///
    /// The rate is based on playback speed, so both the following will call `access` when the
    /// same samples are reached:
    /// `periodic_access(Duration::from_secs(1), ...).speed(2.0)`
    /// `speed(2.0).periodic_access(Duration::from_secs(2), ...)`
    #[inline]
    fn periodic_access<F>(self, period: Duration, access: F) -> PeriodicAccess<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut Self),
    {
        periodic::periodic(self, period, access)
    }

    /// Converts the samples of this source to another type.
    #[inline]
    fn convert_samples<D>(self) -> SamplesConverter<Self, D>
    where
        Self: Sized,
        D: Sample,
    {
        SamplesConverter::new(self)
    }

    /// Makes the sound pausable.
    // TODO: add example
    #[inline]
    fn pausable(self, initially_paused: bool) -> Pausable<Self>
    where
        Self: Sized,
    {
        pausable::pausable(self, initially_paused)
    }

    /// Makes the sound stoppable.
    #[inline]
    fn stoppable(self) -> Stoppable<Self>
    where
        Self: Sized,
    {
        stoppable::stoppable(self)
    }

    #[inline]
    fn skippable(self) -> Skippable<Self>
    where
        Self: Sized,
    {
        skippable::skippable(self)
    }
    /// Changes the play speed of the sound. Does not adjust the samples, only the play speed.
    #[inline]
    fn speed(self, ratio: f32) -> Speed<Self>
    where
        Self: Sized,
    {
        speed::speed(self, ratio)
    }
}
