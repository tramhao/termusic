pub mod buffered_source;
pub mod read_seek_source;

use super::Source;
use std::{fmt, time::Duration};
use symphonia::{
    core::{
        audio::{AudioBufferRef, SampleBuffer, SignalSpec},
        codecs::{self, CodecParameters, CODEC_TYPE_NULL},
        errors::Error,
        formats::{FormatOptions, FormatReader, SeekMode, SeekTo, Track},
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::Hint,
        units::TimeBase,
    },
    default::get_probe,
};

fn is_codec_null(track: &Track) -> bool {
    track.codec_params.codec == CODEC_TYPE_NULL
}

pub struct Symphonia {
    decoder: Box<dyn codecs::Decoder>,
    current_frame_offset: usize,
    format: Box<dyn FormatReader>,
    buffer: SampleBuffer<i16>,
    spec: SignalSpec,
    duration: Option<Duration>,
    elapsed: Duration,
    track_id: u32,
    time_base: Option<TimeBase>,
}

impl Symphonia {
    pub fn new(mss: MediaSourceStream, gapless: bool) -> Result<Self, SymphoniaDecoderError> {
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
                // prebuild_seek_index: true,
                // seek_index_fill_rate: 10,
                enable_gapless: gapless,
                ..Default::default() // enable_gapless: false,
            },
            &MetadataOptions::default(),
        )?;

        // see https://github.com/pdeljanov/Symphonia/issues/258
        // TL;DR: "default_track" may choose a video track or a unknown codec, which will fail, this chooses the first non-NULL codec
        // because currently the only way to detect *something* is by comparing the codec_type to NULL
        let track = probed
            .format
            .default_track()
            .and_then(|v| if is_codec_null(v) { None } else { Some(v) })
            .or_else(|| probed.format.tracks().iter().find(|v| !is_codec_null(v)));

        let Some(track) = track else {
            return Ok(None);
        };

        info!(
            "Found supported container with trackid {} and codectype {}",
            track.id, track.codec_params.codec
        );

        let mut decoder = symphonia::default::get_codecs().make(
            &track.codec_params,
            &codecs::DecoderOptions { verify: true },
        )?;

        let duration = Self::get_duration(&track.codec_params);
        let track_id = track.id;
        let time_base = track.codec_params.time_base;

        let decode_result = loop {
            let packet = probed.format.next_packet()?;

            // Skip all packets that are not the selected track
            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(result) => break result,
                Err(Error::DecodeError(err)) => {
                    info!("Non-fatal Decoder Error: {}", err);
                }
                Err(e) => return Err(e),
            }
        };
        let spec = *decode_result.spec();
        let buffer = Self::get_buffer_new(decode_result);

        Ok(Some(Self {
            decoder,
            current_frame_offset: 0,
            format: probed.format,
            buffer,
            spec,
            duration,
            elapsed: Duration::from_secs(0),
            track_id,
            time_base,
        }))
    }

    fn get_duration(params: &CodecParameters) -> Option<Duration> {
        params.n_frames.and_then(|n_frames| {
            params.time_base.map(|tb| {
                let time = tb.calc_time(n_frames);
                time.into()
            })
        })
    }

    /// Copy passed [`AudioBufferRef`] into a new [`SampleBuffer`]
    ///
    /// also see [`Self::maybe_reuse_buffer`]
    #[inline]
    fn get_buffer_new(decoded: AudioBufferRef<'_>) -> SampleBuffer<i16> {
        let duration = decoded.capacity() as u64;
        let mut buffer = SampleBuffer::<i16>::new(duration, *decoded.spec());
        buffer.copy_interleaved_ref(decoded);
        buffer
    }

    /// Copy passed [`AudioBufferRef`] into the existing [`SampleBuffer`], if possible, otherwise create a new
    #[inline]
    fn maybe_reuse_buffer(buffer: &mut SampleBuffer<i16>, decoded: AudioBufferRef<'_>) {
        // calculate what capacity the SampleBuffer will need (as per SampleBuffer internals)
        let required_capacity = decoded.frames() * decoded.spec().channels.count();
        // avoid a allocation if not actually necessary
        // this also covers the case if the spec changed from the buffer and decoded
        if required_capacity <= buffer.capacity() {
            buffer.copy_interleaved_ref(decoded);
        } else {
            *buffer = Self::get_buffer_new(decoded);
        }
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
        self.duration
    }

    #[inline]
    fn elapsed(&mut self) -> Duration {
        self.elapsed
    }

    #[inline]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn seek(&mut self, time: Duration) -> Option<Duration> {
        match self.format.seek(
            SeekMode::Coarse,
            SeekTo::Time {
                time: time.into(),
                track_id: Some(self.track_id),
            },
        ) {
            Ok(seeked_to) => {
                let base = TimeBase::new(1, self.sample_rate());
                let time = base.calc_time(seeked_to.actual_ts);

                Some(time.into())
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
            let decoded = loop {
                let packet = self.format.next_packet().ok()?;

                // Skip all packets that are not the selected track
                if packet.track_id() != self.track_id {
                    continue;
                }

                match self.decoder.decode(&packet) {
                    Ok(decoded) => {
                        let ts = packet.ts();
                        if let Some(tb) = self.time_base {
                            self.elapsed = tb.calc_time(ts).into();
                        }
                        break decoded;
                    }
                    Err(Error::DecodeError(err)) => {
                        info!("Non-fatal Decoder Error: {}", err);
                    }
                    _ => return None,
                }
            };
            self.spec = *decoded.spec();
            Self::maybe_reuse_buffer(&mut self.buffer, decoded);
            self.current_frame_offset = 0;
        }

        if self.buffer.samples().is_empty() {
            return None;
        }

        let sample = *self.buffer.samples().get(self.current_frame_offset)?;
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
        write!(f, "{text}")
    }
}
impl std::error::Error for SymphoniaDecoderError {}
