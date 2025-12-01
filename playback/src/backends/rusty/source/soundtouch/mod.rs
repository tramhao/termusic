use std::{collections::VecDeque, time::Duration};

use rodio::Source;
use soundtouch::{Setting, SoundTouch};

/// Modify samples to sound similar as 1.0 speed when sped-up or slowed-down via [`::soundtouch`] (via `libSoundTouch`)
pub fn soundtouch<I>(input: I, rate: f32) -> SoundTouchSource<I>
where
    I: Source<Item = f32>,
{
    let mut st = SoundTouch::new();

    let channels = u32::from(input.channels());
    st.set_channels(channels);
    st.set_sample_rate(input.sample_rate());
    st.set_tempo(f64::from(rate));

    let min_samples =
        u32::try_from(st.get_setting(Setting::NominalInputSequence)).unwrap() * channels;
    let min_samples = usize::try_from(min_samples).unwrap();

    SoundTouchSource {
        input,
        min_samples,
        soundtouch: st,
        out_buffer: VecDeque::new(),
        in_buffer: VecDeque::new(),
        factor: 1.0,
    }
}

#[derive(Debug)]
pub struct SoundTouchSource<I> {
    /// The inner source where we get the original samples from
    input: I,
    /// The Soundtouch instance where we input all values and get converted values out of
    soundtouch: SoundTouch,
    /// The approximate minimal amount of samples necessary to get new processed samples
    min_samples: usize,
    /// Already processed samples that still need to be output
    out_buffer: VecDeque<f32>,
    /// Samples we input to be processed
    in_buffer: VecDeque<f32>,
    /// The timescale factor. `1.0` means no change from the source.
    factor: f64,
}

impl<I> Iterator for SoundTouchSource<I>
where
    I: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // This is to skip calculation if speed is not changed
        if (self.factor - 1.0).abs() < 0.05 {
            // use the samples from the in_buffer, otherwise we could be dropping samples without actually playing them
            // when quickly changing between 1.0 and other speeds, there may still be a audible drop, but this lowers it
            if !self.in_buffer.is_empty() {
                let _ = self.out_buffer.pop_front();
                return self.in_buffer.pop_front();
            }

            return self.input.next();
        }

        if self.out_buffer.is_empty() {
            self.get_new_samples();
        }

        match (self.out_buffer.pop_front(), self.in_buffer.pop_front()) {
            (Some(a), Some(_b)) => Some(a),
            (None, None) => None,
            (None, Some(v)) | (Some(v), None) => Some(v),
        }
    }
}

impl<I> ExactSizeIterator for SoundTouchSource<I> where I: Source<Item = f32> + ExactSizeIterator {}

impl<I> Source for SoundTouchSource<I>
where
    I: Source<Item = f32>,
{
    fn current_span_len(&self) -> Option<usize> {
        Some(self.min_samples)
    }

    fn channels(&self) -> u16 {
        self.input.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.input.total_duration()
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.input.try_seek(pos)
    }
}

impl<I> SoundTouchSource<I>
where
    I: Source<Item = f32>,
{
    /// Returns a reference to the inner source.
    #[inline]
    pub fn inner(&self) -> &I {
        &self.input
    }

    /// Returns a mutable reference to the inner source.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.input
    }

    /// Modifies the speed factor.
    #[inline]
    pub fn set_factor(&mut self, factor: f64) {
        self.factor = factor;
    }

    /// Get new samples into the `out_buffer`.
    ///
    /// Will completely overwrite the `in_buffer` & `out_buffer`.
    fn get_new_samples(&mut self) {
        let channels = u32::from(self.input.channels());

        // in rodio and symphonia, any of these factors could have changed since the last time
        self.soundtouch.set_tempo(self.factor);
        self.soundtouch.set_channels(channels);
        self.soundtouch.set_sample_rate(self.input.sample_rate());

        let min_samples = u32::try_from(self.soundtouch.get_setting(Setting::NominalInputSequence))
            .unwrap()
            * channels;
        self.min_samples = usize::try_from(min_samples).unwrap();

        self.in_buffer.clear();

        let mut take_samples = self.min_samples;

        // if the input buffer has not been allocated yet, we can safely assume this is the first time non-1.0 speed is called
        // and we need to first allocate some things and likely take more samples too
        if self.in_buffer.capacity() == 0 {
            let initial_latency =
                u32::try_from(self.soundtouch.get_setting(Setting::InitialLatency)).unwrap()
                    * channels;
            let initial_latency = usize::try_from(initial_latency).unwrap();

            self.out_buffer.resize(initial_latency, 0.0);
            self.in_buffer.reserve(initial_latency);

            // Soundtouch may need a different amount for the initial batch of samples
            take_samples = initial_latency;
        }

        let channels = usize::try_from(channels).unwrap();

        self.input
            .by_ref()
            .take(take_samples)
            .for_each(|x| self.in_buffer.push_back(x));

        let len_input = self.in_buffer.len() / channels;
        self.soundtouch
            .put_samples(self.in_buffer.make_contiguous(), len_input);

        // this could only mean the inner source has ended
        if self.in_buffer.len() < self.min_samples {
            // soundtouch may not output anything if there are not at least "min_samples", unless "flush" is called, which fills with empty samples
            self.soundtouch.flush();
        }

        self.out_buffer.resize(self.min_samples, 0.0);

        let len_output = self.in_buffer.len() / channels;
        let read = self
            .soundtouch
            .receive_samples(self.out_buffer.make_contiguous(), len_output);

        // The following check is basically just debug, but if this should ever happen, it is not fatal (hence no assert)
        // but it would be good to know
        if self.in_buffer.len() < self.min_samples && self.soundtouch.is_empty() != 0 {
            error!("Soundtouch was not empty!");
        }

        self.out_buffer.truncate(read * channels);
    }
}
