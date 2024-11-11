// mod sonic;

use std::collections::VecDeque;

use super::MixSource;
use super::Source;
use soundtouch::{Setting, SoundTouch};

#[allow(clippy::cast_sign_loss)]
pub fn tempo_stretch<I>(mut input: I, rate: f32) -> TempoStretch<I>
where
    I: Source<Item = f32>,
{
    let channels = input.channels();
    let mut st = SoundTouch::new();
    st.set_channels(u32::from(channels));
    st.set_sample_rate(input.sample_rate());
    st.set_tempo(f64::from(rate));
    let min_samples = st.get_setting(Setting::NominalInputSequence) as usize * channels as usize;
    let initial_latency = st.get_setting(Setting::InitialLatency) as usize * channels as usize;
    let mut out_buffer = VecDeque::new();
    out_buffer.resize(initial_latency, 0.0);
    out_buffer.make_contiguous();
    let mut initial_input: VecDeque<f32> = input.by_ref().take(initial_latency).collect();
    let num_samples = initial_input.len() / channels as usize;
    st.put_samples(initial_input.make_contiguous(), num_samples);
    let read = st.receive_samples(out_buffer.as_mut_slices().0, num_samples);
    out_buffer.truncate(read);
    initial_input.clear();
    TempoStretch {
        input,
        min_samples,
        soundtouch: st,
        out_buffer,
        in_buffer: initial_input,
        mix: 1.0,
        factor: 1.0,
    }
}

pub struct TempoStretch<I> {
    input: I,
    soundtouch: SoundTouch,
    min_samples: usize,
    out_buffer: VecDeque<f32>,
    in_buffer: VecDeque<f32>,
    mix: f32,
    factor: f64,
}

impl<I> Iterator for TempoStretch<I>
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

            let len_input = self.in_buffer.len() / self.input.channels() as usize;
            self.soundtouch
                .put_samples(self.in_buffer.make_contiguous(), len_input);

            self.out_buffer.resize(self.min_samples, 0.0);
            self.out_buffer.make_contiguous();

            let len_output = self.in_buffer.len() / self.input.channels() as usize;
            let read = self
                .soundtouch
                .receive_samples(self.out_buffer.as_mut_slices().0, len_output);

            self.out_buffer
                .truncate(read * self.input.channels() as usize);
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

impl<I> ExactSizeIterator for TempoStretch<I> where I: Source<Item = f32> + ExactSizeIterator {}

impl<I> Source for TempoStretch<I>
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

    fn seek(&mut self, time: std::time::Duration) -> Option<std::time::Duration> {
        self.input.seek(time)
    }

    fn elapsed(&mut self) -> std::time::Duration {
        self.input.elapsed()
    }
}

impl<I> MixSource for TempoStretch<I>
where
    I: Source<Item = f32>,
{
    fn set_mix(&mut self, mix: f32) {
        self.mix = mix;
    }
}

#[allow(unused)]
impl<I> TempoStretch<I>
where
    I: Source<Item = f32>,
{
    /// Modifies the speed factor.
    #[inline]
    pub fn set_factor(&mut self, factor: f64) {
        self.factor = factor;
    }
}
