use std::time::Duration;
use symphonia::{
    core::{
        audio::{AudioBufferRef, SampleBuffer, SignalSpec},
        codecs::{Decoder, DecoderOptions},
        errors::Error,
        formats::{FormatReader, SeekMode, SeekTo},
        io::MediaSourceStream,
        // meta::MetadataOptions,
        probe::Hint,
        units::{Time, TimeBase},
    },
    default::get_probe,
};

use super::DecoderError;
use super::Source;

#[allow(clippy::module_name_repetitions)]
pub struct SymphoniaDecoder {
    decoder: Box<dyn Decoder>,
    current_frame_offset: usize,
    format: Box<dyn FormatReader>,
    buffer: SampleBuffer<i16>,
    spec: SignalSpec,
    total_duration: Duration,
    elapsed: Duration,
}

#[allow(unused)]
impl SymphoniaDecoder {
    pub fn new(mss: MediaSourceStream, extension: Option<&str>) -> Result<Self, DecoderError> {
        match Self::init(mss, extension) {
            Err(e) => match e {
                Error::IoError(e) => Err(DecoderError::IoError(e.to_string())),
                Error::DecodeError(e) => Err(DecoderError::DecodeError(e)),
                Error::SeekError(_) => {
                    unreachable!("Seek errors should not occur during initialization")
                }
                Error::Unsupported(_) => Err(DecoderError::UnrecognizedFormat),
                Error::LimitError(e) => Err(DecoderError::LimitError(e)),
                Error::ResetRequired => Err(DecoderError::ResetRequired),
            },
            Ok(Some(decoder)) => Ok(decoder),
            Ok(None) => Err(DecoderError::NoStreams),
        }
    }

    pub fn into_inner(self) -> MediaSourceStream {
        self.format.into_inner()
    }

    fn init(
        mss: MediaSourceStream,
        extension: Option<&str>,
    ) -> symphonia::core::errors::Result<Option<Self>> {
        let mut hint = Hint::new();
        if let Some(ext) = extension {
            hint.with_extension(ext);
        }
        let format_opts = symphonia::core::formats::FormatOptions::default();
        let metadata_opts = symphonia::core::meta::MetadataOptions::default();
        let mut probed = get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;

        let stream = match probed.format.default_track() {
            Some(stream) => stream,
            None => return Ok(None),
        };

        let mut decoder = symphonia::default::get_codecs()
            .make(&stream.codec_params, &DecoderOptions { verify: true })?;

        //Calculate total duraiton
        let tb = stream.codec_params.time_base;
        let dur = stream
            .codec_params
            .n_frames
            .map(|frames| stream.codec_params.start_ts + frames);

        let total_duration = dur.map_or_else(
            || Duration::from_secs(0),
            |dur| {
                tb.map_or_else(
                    || Duration::from_secs(0),
                    |tb| {
                        let d = tb.calc_time(dur);
                        Duration::from_secs(d.seconds) + Duration::from_secs_f64(d.frac)
                    },
                )
            },
        );

        let current_frame = probed.format.next_packet()?;
        let decoded_result = decoder.decode(&current_frame)?;
        let spec = *decoded_result.spec();
        let buffer = Self::get_buffer(decoded_result, &spec);

        Ok(Some(Self {
            decoder,
            current_frame_offset: 0,
            format: probed.format,
            buffer,
            spec,
            total_duration,
            elapsed: Duration::from_secs(0),
        }))
    }

    #[inline]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn get_buffer(decoded: AudioBufferRef, spec: &SignalSpec) -> SampleBuffer<i16> {
        let duration = decoded.capacity() as u64;
        let mut buffer = SampleBuffer::<i16>::new(duration, *spec);
        buffer.copy_interleaved_ref(decoded);
        buffer
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
impl Source for SymphoniaDecoder {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.buffer.samples().len())
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.spec.channels.count() as u16
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.spec.rate
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        Some(self.total_duration)
    }

    #[inline]
    fn elapsed(&mut self) -> Duration {
        self.elapsed
    }

    #[inline]
    fn seek(&mut self, time: Duration) -> Result<Duration, ()> {
        let nanos_per_sec = 1_000_000_000.0;
        match self.format.seek(
            SeekMode::Coarse,
            SeekTo::Time {
                time: Time::new(
                    time.as_secs(),
                    f64::from(time.subsec_nanos()) / nanos_per_sec,
                ),
                track_id: None,
            },
        ) {
            Ok(seeked_to) => {
                let base = TimeBase::new(1, self.sample_rate());
                let time = base.calc_time(seeked_to.actual_ts);

                Ok(Duration::from_millis(
                    time.seconds * 1000 + ((time.frac * 60. * 1000.).round() as u64),
                ))
            }
            Err(_) => Err(()),
        }
    }
}

impl Iterator for SymphoniaDecoder {
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if self.current_frame_offset == self.buffer.len() {
            match self.format.next_packet() {
                Ok(packet) => match self.decoder.decode(&packet) {
                    Ok(decoded) => {
                        self.spec = *decoded.spec();
                        self.buffer = Self::get_buffer(decoded, &self.spec);

                        let ts = packet.ts();
                        let tb = self
                            .format
                            .tracks()
                            .first()
                            .unwrap()
                            .codec_params
                            .time_base
                            .unwrap();

                        let t = tb.calc_time(ts);

                        self.elapsed =
                            Duration::from_secs(t.seconds) + Duration::from_secs_f64(t.frac);
                    }
                    Err(_) => return None,
                },
                Err(_) => return None,
            }
            self.current_frame_offset = 0;
        }

        let sample = self.buffer.samples()[self.current_frame_offset];
        self.current_frame_offset += 1;

        Some(sample)
    }
}
