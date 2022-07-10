use super::Source;
use std::{fmt, fs::File, time::Duration};
use symphonia::{
    core::{
        audio::{AudioBufferRef, SampleBuffer, SignalSpec},
        codecs::{self, CodecParameters},
        errors::Error,
        formats::{FormatOptions, FormatReader, SeekMode, SeekTo},
        io::{MediaSourceStream, MediaSourceStreamOptions},
        meta::MetadataOptions,
        probe::Hint,
        units::{Time, TimeBase},
    },
    default::get_probe,
};
// Decoder errors are not considered fatal.
// The correct action is to just get a new packet and try again.
// But a decode error in more than 3 consecutive packets is fatal.
const MAX_DECODE_ERRORS: usize = 3;

pub struct Symphonia {
    decoder: Box<dyn codecs::Decoder>,
    current_frame_offset: usize,
    format: Box<dyn FormatReader>,
    buffer: SampleBuffer<i16>,
    spec: SignalSpec,
    duration: Duration,
    elapsed: Duration,
}

impl Symphonia {
    pub fn new(file: File, gapless: bool) -> Result<Self, SymphoniaDecoderError> {
        let source = Box::new(file);

        let mss = MediaSourceStream::new(source, MediaSourceStreamOptions::default());
        match Self::init(mss, gapless) {
            Err(e) => match e {
                Error::IoError(e) => Err(SymphoniaDecoderError::IoError(e.to_string())),
                Error::DecodeError(e) => Err(SymphoniaDecoderError::DecodeError(e)),
                Error::SeekError(_) => {
                    unreachable!("Seek errors should not occur during initialization")
                }
                Error::Unsupported(_) => Err(SymphoniaDecoderError::UnrecognizedFormat),
                Error::LimitError(e) => Err(SymphoniaDecoderError::LimitError(e)),
                Error::ResetRequired => Err(SymphoniaDecoderError::ResetRequired),
            },
            Ok(Some(decoder)) => Ok(decoder),
            Ok(None) => Err(SymphoniaDecoderError::NoStreams),
        }
    }

    fn init(
        mss: MediaSourceStream,
        gapless: bool,
    ) -> symphonia::core::errors::Result<Option<Self>> {
        let mut probed = get_probe().format(
            &Hint::default(),
            mss,
            &FormatOptions {
                prebuild_seek_index: true,
                seek_index_fill_rate: 10,
                enable_gapless: gapless,
                // enable_gapless: false,
            },
            &MetadataOptions::default(),
        )?;

        let track = match probed.format.default_track() {
            Some(stream) => stream,
            None => return Ok(None),
        };

        let mut decoder = symphonia::default::get_codecs().make(
            &track.codec_params,
            &codecs::DecoderOptions { verify: true },
        )?;

        let duration = Self::get_duration(&track.codec_params);

        let mut decode_errors: usize = 0;
        let decode_result = loop {
            let current_frame = probed.format.next_packet()?;
            match decoder.decode(&current_frame) {
                Ok(result) => break result,
                Err(e) => match e {
                    Error::DecodeError(_) => {
                        decode_errors += 1;
                        if decode_errors > MAX_DECODE_ERRORS {
                            return Err(e);
                        }
                    }
                    _ => return Err(e),
                },
            }
        };
        let spec = *decode_result.spec();
        let buffer = Self::get_buffer(decode_result, spec);

        Ok(Some(Self {
            decoder,
            current_frame_offset: 0,
            format: probed.format,
            buffer,
            spec,
            duration,
            elapsed: Duration::from_secs(0),
        }))
    }

    fn get_duration(params: &CodecParameters) -> Duration {
        // if let Some(n_frames) = params.n_frames {
        //     if let Some(tb) = params.time_base {
        //         let time = tb.calc_time(n_frames);
        //         Duration::from_secs(time.seconds) + Duration::from_secs_f64(time.frac)
        //     } else {
        //         panic!("no time base?");
        //     }
        // } else {
        //     panic!("no n_frames");
        // }

        params.n_frames.map_or_else(
            || {
                // panic!("no n_frames");
                Duration::from_secs(99)
            },
            |n_frames| {
                params.time_base.map_or_else(
                    || {
                        // panic!("no time base?");
                        Duration::from_secs(199)
                    },
                    |tb| {
                        let time = tb.calc_time(n_frames);
                        Duration::from_secs(time.seconds) + Duration::from_secs_f64(time.frac)
                    },
                )
            },
        )
    }

    #[inline]
    fn get_buffer(decoded: AudioBufferRef<'_>, spec: SignalSpec) -> SampleBuffer<i16> {
        let duration = decoded.capacity() as u64;
        let mut buffer = SampleBuffer::<i16>::new(duration, spec);
        buffer.copy_interleaved_ref(decoded);
        buffer
    }
}

impl Source for Symphonia {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.buffer.samples().len())
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn channels(&self) -> u16 {
        self.spec.channels.count() as u16
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.spec.rate
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        Some(self.duration)
    }

    #[inline]
    fn elapsed(&mut self) -> Duration {
        self.elapsed
    }

    #[inline]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn seek(&mut self, time: Duration) -> Option<Duration> {
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

                Some(Duration::from_millis(
                    time.seconds * 1000 + ((time.frac * 60. * 1000.).round() as u64),
                ))
            }
            Err(_) => None,
        }
    }
}

impl Iterator for Symphonia {
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if self.current_frame_offset == self.buffer.len() {
            let mut decode_errors: usize = 0;
            let decoded = loop {
                match self.format.next_packet() {
                    Ok(packet) => match self.decoder.decode(&packet) {
                        Ok(decoded) => {
                            let ts = packet.ts();
                            if let Some(track) = self.format.default_track() {
                                if let Some(tb) = track.codec_params.time_base {
                                    let t = tb.calc_time(ts);
                                    self.elapsed = Duration::from_secs(t.seconds)
                                        + Duration::from_secs_f64(t.frac);
                                }
                            }
                            break decoded;
                        }
                        Err(e) => match e {
                            Error::DecodeError(_) => {
                                decode_errors += 1;
                                if decode_errors > MAX_DECODE_ERRORS {
                                    return None;
                                }
                            }
                            _ => return None,
                        },
                    },
                    Err(_) => return None,
                }
            };
            self.spec = *decoded.spec();
            self.buffer = Self::get_buffer(decoded, self.spec);
            self.current_frame_offset = 0;
        }

        if self.buffer.samples().is_empty() {
            return None;
        }

        let sample = self.buffer.samples()[self.current_frame_offset];
        self.current_frame_offset += 1;

        Some(sample)
    }
}

/// Error that can happen when creating a decoder.
#[derive(Debug, Clone)]
pub enum SymphoniaDecoderError {
    /// The format of the data has not been recognized.
    UnrecognizedFormat,

    /// An IO error occured while reading, writing, or seeking the stream.
    IoError(String),

    /// The stream contained malformed data and could not be decoded or demuxed.
    DecodeError(&'static str),

    /// A default or user-defined limit was reached while decoding or demuxing the stream. Limits
    /// are used to prevent denial-of-service attacks from malicious streams.
    LimitError(&'static str),

    /// The demuxer or decoder needs to be reset before continuing.
    ResetRequired,

    /// No streams were found by the decoder
    NoStreams,
}

impl fmt::Display for SymphoniaDecoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::UnrecognizedFormat => "Unrecognized format",
            Self::IoError(msg) => &msg[..],
            Self::DecodeError(msg) | Self::LimitError(msg) => msg,
            Self::ResetRequired => "Reset required",
            Self::NoStreams => "No streams",
        };
        write!(f, "{}", text)
    }
}
impl std::error::Error for SymphoniaDecoderError {}
