use std::{collections::VecDeque, time::Duration};

use rodio::Source;
use soundtouch::{Setting, SoundTouch};

pub fn soundtouch<I>(mut input: I, rate: f32) -> SoundTouchSource<I>
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
    let initial_latency =
        u32::try_from(st.get_setting(Setting::InitialLatency)).unwrap() * channels;
    let initial_latency = usize::try_from(initial_latency).unwrap();

    let mut out_buffer = VecDeque::with_capacity(initial_latency);
    out_buffer.resize(initial_latency, 0.0);

    let mut initial_input: VecDeque<f32> = input.by_ref().take(initial_latency).collect();
    let num_samples = initial_input.len() / usize::try_from(channels).unwrap();
    st.put_samples(initial_input.make_contiguous(), num_samples);

    let read = st.receive_samples(out_buffer.make_contiguous(), num_samples);
    out_buffer.truncate(read);
    initial_input.clear();

    SoundTouchSource {
        input,
        min_samples,
        soundtouch: st,
        out_buffer,
        in_buffer: initial_input,
        mix: 1.0,
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
    /// unused
    mix: f32,
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
            return self.input.next();
        }

        self.soundtouch.set_tempo(self.factor);
        if self.out_buffer.is_empty() {
            self.in_buffer.clear();
            self.input
                .by_ref()
                .take(self.min_samples)
                .for_each(|x| self.in_buffer.push_back(x));

            let channels = usize::from(self.input.channels());

            let len_input = self.in_buffer.len() / channels;
            self.soundtouch
                .put_samples(self.in_buffer.make_contiguous(), len_input);

            self.out_buffer.resize(self.min_samples, 0.0);

            let len_output = self.in_buffer.len() / channels;
            let read = self
                .soundtouch
                .receive_samples(self.out_buffer.make_contiguous(), len_output);

            self.out_buffer.truncate(read * channels);
        }

        match (
            self.out_buffer.pop_front().map(|x| x * self.mix),
            self.in_buffer.pop_front().map(|x| x * (1.0 - self.mix)),
        ) {
            (Some(a), Some(b)) => Some(a + b),
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
    fn current_frame_len(&self) -> Option<usize> {
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

#[allow(dead_code)]
impl<I> SoundTouchSource<I>
where
    I: Source<Item = f32>,
{
    /// Modifies the speed factor.
    #[inline]
    pub fn set_factor(&mut self, factor: f64) {
        self.factor = factor;
    }

    #[inline]
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix;
    }
}
