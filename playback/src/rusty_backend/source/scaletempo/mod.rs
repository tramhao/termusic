// mod scaletempo_1;
mod sonic;

use super::{Sample, Source};
use std::time::Duration;
// use soundtouch::{Setting, SoundTouch};

#[allow(clippy::cast_sign_loss)]
pub fn tempo_stretch<I>(input: I, factor: f32) -> TempoStretch<I> {
    // let channels = input.channels();
    // let mut st = SoundTouch::new();
    // st.set_channels(u32::from(channels))
    //     .set_sample_rate(input.sample_rate())
    //     // .set_pitch_semitones(semitones)
    //     .set_setting(Setting::UseQuickseek, 1);
    // let min_samples = st.get_setting(Setting::NominalInputSequence) as usize * channels as usize;
    // let initial_latency = st.get_setting(Setting::InitialLatency) as usize * channels as usize;
    // let mut out_buffer = VecDeque::new();
    // out_buffer.resize(initial_latency, 0.0);
    // out_buffer.make_contiguous();
    // let mut initial_input: VecDeque<f32> = input.by_ref().take(initial_latency).collect();
    // st.put_samples(
    //     initial_input.make_contiguous(),
    //     input.sample_rate() as usize / channels as usize,
    // );
    // let read = st.receive_samples(
    //     out_buffer.as_mut_slices().0,
    //     input.sample_rate() as usize / channels as usize,
    // );
    // out_buffer.truncate(read);
    // initial_input.clear();
    TempoStretch {
        input,
        // min_samples,
        // soundtouch: st,
        // out_buffer,
        // in_buffer: initial_input,
        // mix: 1.0,
        factor,
    }
}

#[derive(Clone, Debug)]
pub struct TempoStretch<I> {
    input: I,
    // soundtouch: SoundTouch,
    // min_samples: usize,
    // out_buffer: VecDeque<f32>,
    // in_buffer: VecDeque<f32>,
    // mix: f32,
    factor: f32,
}

#[allow(unused)]
impl<I> Iterator for TempoStretch<I>
where
    I: Source,
    I::Item: Sample,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.input.next()

        // self.soundtouch.set_tempo(self.factor.into());
        // if self.out_buffer.is_empty() {
        //     self.in_buffer.clear();
        //     self.input
        //         .by_ref()
        //         .take(self.min_samples)
        //         .for_each(|x| self.in_buffer.push_back(x));

        //     self.soundtouch.put_samples(
        //         self.in_buffer.make_contiguous(),
        //         self.input.sample_rate() as usize / self.input.channels() as usize,
        //     );

        //     self.out_buffer.resize(self.min_samples, 0.0);
        //     self.out_buffer.make_contiguous();

        //     let read = self.soundtouch.receive_samples(
        //         self.out_buffer.as_mut_slices().0,
        //         self.input.sample_rate() as usize / self.input.channels() as usize,
        //     );

        //     self.out_buffer
        //         .truncate(read * self.input.channels() as usize);
        // }

        // match (
        //     self.out_buffer.pop_front().map(|x| x * self.mix),
        //     self.in_buffer.pop_front().map(|x| x * (1.0 - self.mix)),
        // ) {
        //     (Some(a), Some(b)) => Some(a + b),
        //     (None, None) => None,
        //     (None, Some(v)) | (Some(v), None) => Some(v),
        // }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input.size_hint()
    }
}

#[allow(unused)]
impl<I> TempoStretch<I>
where
    I: Source,
    I::Item: Sample,
{
    /// Modifies the speed factor.
    #[inline]
    pub fn set_factor(&mut self, factor: f32) {
        self.factor = factor;
    }

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

    /// Returns the inner source.
    #[inline]
    pub fn into_inner(self) -> I {
        self.input
    }
}

impl<I> ExactSizeIterator for TempoStretch<I>
where
    I: Source + ExactSizeIterator,
    I::Item: Sample,
{
}

impl<I> Source for TempoStretch<I>
where
    I: Source,
    I::Item: Sample,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.input.channels()
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        (self.input.sample_rate() as f32 * self.factor) as u32
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        // TODO: the crappy API of duration makes this code difficult to write
        self.input.total_duration().map(|duration| {
            let as_ns = duration.as_secs() * 1_000_000_000 + u64::from(duration.subsec_nanos());
            let new_val = (as_ns as f32 / self.factor) as u64;
            Duration::new(new_val / 1_000_000_000, (new_val % 1_000_000_000) as u32)
        })
    }
    #[inline]
    fn elapsed(&mut self) -> Duration {
        self.input.elapsed()
    }
    #[inline]
    fn seek(&mut self, time: Duration) -> Option<Duration> {
        self.input.seek(time)
    }
}
