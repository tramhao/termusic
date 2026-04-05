use std::{fmt, sync::LazyLock, time::Duration};

use rodio::{ChannelCount, SampleRate};
use symphonia::{
    core::{
        audio::{AudioSpec, GenericAudioBufferRef},
        codecs::{self, CodecParameters, audio::CODEC_ID_NULL_AUDIO, registry::CodecRegistry},
        errors::Error,
        formats::{FormatOptions, FormatReader, SeekMode, SeekTo, Track, TrackType, probe::Hint},
        io::MediaSourceStream,
        meta::{MetadataOptions, MetadataRevision, StandardTag},
        units::{self, TimeBase},
    },
    default::{get_probe, register_enabled_codecs},
};
use tokio::sync::mpsc;

use super::{Source, source::SampleType};

pub mod buffered_source;
pub mod read_seek_source;

fn is_codec_null(track: &Track) -> bool {
    let Some(CodecParameters::Audio(audio_codec_params)) = track.codec_params.as_ref() else {
        return true;
    };

    audio_codec_params.codec == CODEC_ID_NULL_AUDIO
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaTitleType {
    /// Command to instruct storage to clear / reset
    Reset,
    /// Command to provide a new value
    Value(String),
}

pub type MediaTitleRx = mpsc::UnboundedReceiver<MediaTitleType>;
// not public as the transmitter never leaves this module
type MediaTitleTx = mpsc::UnboundedSender<MediaTitleType>;

/// A simple `NewType` to provide extra function for the [`Option<MediaTitleTx>`] type
#[derive(Debug)]
struct MediaTitleTxWrap(Option<MediaTitleTx>);

impl MediaTitleTxWrap {
    pub fn new() -> Self {
        Self(None)
    }

    /// Open and store a new channel
    #[inline]
    pub fn media_title_channel(&mut self) -> MediaTitleRx {
        // unbounded channel so that sending *never* blocks
        let (tx, rx) = mpsc::unbounded_channel();
        self.0 = Some(tx);

        rx
    }

    /// Send a command, without caring if [`None`] or the commands result
    #[inline]
    pub fn media_title_send(&self, cmd: MediaTitleType) {
        if let Some(ref tx) = self.0 {
            let _ = tx.send(cmd);
        }
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    /// Helper Shorthand to send [`MediaTitleType::Reset`] command
    #[inline]
    pub fn send_reset(&mut self) {
        self.media_title_send(MediaTitleType::Reset);
    }
}

/// Custom registry for termusic, which might use extensions
static CODEC_REGISTRY: LazyLock<CodecRegistry> = LazyLock::new(|| {
    let mut registry = CodecRegistry::new();
    register_enabled_codecs(&mut registry);
    #[cfg(feature = "rusty-libopus")]
    registry.register_audio_decoder::<symphonia_adapter_libopus::OpusDecoder>();
    registry
});

pub struct Symphonia {
    decoder: Box<dyn codecs::audio::AudioDecoder>,
    current_frame_offset: usize,
    probed: Box<dyn FormatReader>,
    buffer: Vec<SampleType>,
    buffer_frame_len: usize,
    spec: AudioSpec,
    duration: Option<Duration>,
    track_id: u32,
    time_base: Option<TimeBase>,
    seek_required_ts: Option<units::Timestamp>,

    media_title_tx: MediaTitleTxWrap,
}

impl Symphonia {
    /// Create a new symphonia decoder.
    ///
    /// The returned `Option<MediaTitleRx>` is always `Some` if parameter `media_title` is `true`.
    #[inline]
    pub fn new(
        mss: MediaSourceStream<'static>,
        gapless: bool,
        media_title: bool,
    ) -> Result<(Self, Option<MediaTitleRx>), SymphoniaDecoderError> {
        Self::init(mss, gapless, media_title)
    }

    fn init(
        mss: MediaSourceStream<'static>,
        gapless: bool,
        media_title: bool,
    ) -> Result<(Self, Option<MediaTitleRx>), SymphoniaDecoderError> {
        let mut probed = get_probe().probe(
            &Hint::default(),
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )?;

        // see https://github.com/pdeljanov/Symphonia/issues/258
        // TL;DR: "default_track" may choose a video track or a unknown codec, which will fail, this chooses the first non-NULL codec
        // because currently the only way to detect *something* is by comparing the codec_type to NULL
        let track = probed
            .default_track(TrackType::Audio)
            .and_then(|v| if is_codec_null(v) { None } else { Some(v) })
            .or_else(|| probed.tracks().iter().find(|v| !is_codec_null(v)))
            .ok_or(SymphoniaDecoderError::NoStreams)?;

        let Some(CodecParameters::Audio(audio_codec_params)) = track.codec_params.as_ref() else {
            return Err(SymphoniaDecoderError::NoStreams);
        };

        info!(
            "Found supported container with trackid {} and codectype {}",
            track.id, audio_codec_params.codec
        );

        let mut decoder = CODEC_REGISTRY.make_audio_decoder(
            audio_codec_params,
            &codecs::audio::AudioDecoderOptions::default()
                .verify(true)
                .gapless(gapless),
        )?;

        let duration = Self::get_duration(track);
        let track_id = track.id;
        let time_base = track.time_base;
        let mut media_title_tx = MediaTitleTxWrap::new();

        let media_title_rx = if media_title {
            Some(media_title_tx.media_title_channel())
        } else {
            None
        };

        // decode the first part, to get the spec and initial buffer
        let mut buffer = None;
        let DecodeLoopResult { spec } = decode_loop(
            &mut *probed,
            &mut *decoder,
            BufferInputType::New(&mut buffer),
            track_id,
            time_base,
            &mut media_title_tx,
            // &mut probed.metadata,
            &mut None,
        )?
        .ok_or(SymphoniaDecoderError::UnexpectedEOFInit)?;

        // safe to unwrap because "decode_loop" ensures it will be set
        let (buffer, buffer_frame_len) = buffer.unwrap();

        Ok((
            Self {
                decoder,
                current_frame_offset: 0,
                probed,
                buffer,
                buffer_frame_len,
                spec,
                duration,
                track_id,
                time_base,
                seek_required_ts: None,

                media_title_tx,
            },
            media_title_rx,
        ))
    }

    fn get_duration(track: &Track) -> Option<Duration> {
        track.num_frames.map(units::Duration::new).and_then(|dur| {
            track.time_base.map(|tb| {
                let ts = units::Timestamp::ZERO.saturating_add(dur);
                let (secs, nanos) = tb.calc_time_saturating(ts).parts();
                Duration::new(u64::try_from(secs).unwrap_or(0), nanos)
            })
        })
    }

    /// Copy passed [`GenericAudioBufferRef`] into a new Buffer
    ///
    /// also see [`Self::maybe_reuse_buffer`]
    #[inline]
    #[allow(clippy::needless_pass_by_value)]
    fn get_buffer_new(decoded: GenericAudioBufferRef<'_>) -> (Vec<SampleType>, usize) {
        let mut buffer = Vec::<SampleType>::with_capacity(decoded.capacity());
        decoded.copy_to_vec_interleaved(&mut buffer);
        (buffer, decoded.frames())
    }

    /// Copy passed [`GenericAudioBufferRef`] into the existing Buffer, if possible, otherwise create a new
    #[inline]
    fn maybe_reuse_buffer(
        buffer: (&mut Vec<SampleType>, &mut usize),
        decoded: GenericAudioBufferRef<'_>,
    ) {
        // calculate what capacity the Buffer will need
        let required_capacity = decoded.byte_len_as::<SampleType>();
        // avoid a allocation if not actually necessary
        // this also covers the case if the spec changed from the buffer and decoded
        if required_capacity <= buffer.0.capacity() {
            decoded.copy_to_vec_interleaved(buffer.0);
            *buffer.1 = decoded.frames();
        } else {
            (*buffer.0, *buffer.1) = Self::get_buffer_new(decoded);
        }
    }

    /// Run a potential decode, if the buffer is exhausted.
    ///
    /// No-op if buffer is not exhausted.
    ///
    /// `None` means End-of-File (EOF/EOS).
    pub fn decode_once(&mut self) -> Option<()> {
        if self.exhausted_buffer() {
            let DecodeLoopResult { spec } = decode_loop(
                &mut *self.probed,
                &mut *self.decoder,
                BufferInputType::Existing((&mut self.buffer, &mut self.buffer_frame_len)),
                self.track_id,
                self.time_base,
                &mut self.media_title_tx,
                &mut self.seek_required_ts,
            )
            .inspect_err(|err| warn!("Error while decoding: {err:#?}"))
            .ok()??;

            self.spec = spec;

            self.current_frame_offset = 0;
        }

        Some(())
    }

    /// Get whether the current buffer is used up.
    pub fn exhausted_buffer(&self) -> bool {
        self.buffer.is_empty() || self.current_frame_offset == self.buffer.len()
    }

    /// Increase the offset from which to read the buffer from.
    #[inline]
    pub fn advance_offset(&mut self, by: usize) {
        self.current_frame_offset += by;
    }

    /// Get the current spec plus frame length.
    pub fn get_spec(&self) -> (AudioSpec, usize) {
        (self.spec.clone(), self.current_span_len().unwrap())
    }

    /// Get the current buffer interpreted as u8(bytes) in native encoding.
    pub fn get_buffer_u8(&self) -> &[u8] {
        #[allow(unsafe_code)]
        unsafe {
            // re-interpret the SampleType slice as a u8 slice with the same byte-length.
            let len = size_of_val(self.buffer.as_slice());
            std::slice::from_raw_parts(
                self.buffer.as_slice()[self.current_frame_offset..]
                    .as_ptr()
                    .cast::<u8>(),
                len,
            )
        }
    }

    /// Get the current buffer, but only the part has not been read yet.
    pub fn get_buffer(&self) -> &[SampleType] {
        &self.buffer.as_slice()[self.current_frame_offset..]
    }
}

impl Source for Symphonia {
    #[inline]
    fn current_span_len(&self) -> Option<usize> {
        Some(self.buffer_frame_len)
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn channels(&self) -> ChannelCount {
        u16::try_from(self.spec.channels().count())
            .ok()
            .and_then(|v| v.try_into().ok())
            .expect("Valid non-zero channel count")
    }

    #[inline]
    fn sample_rate(&self) -> SampleRate {
        self.spec
            .rate()
            .try_into()
            .expect("Valid non-zero sample rate")
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.duration
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        let pos = units::Time::try_new(
            i64::try_from(pos.as_secs()).unwrap_or(i64::MAX),
            pos.subsec_nanos(),
        )
        .expect("Unexpected nano seconds");
        match self.probed.seek(
            SeekMode::Coarse,
            SeekTo::Time {
                time: pos,
                track_id: Some(self.track_id),
            },
        ) {
            Ok(seeked_to) => {
                // clear sample buffer after seek
                self.current_frame_offset = 0;
                self.buffer.clear();

                // Coarse seeking may seek (slightly) beyond the requested ts, so it may not actually need to be set
                if seeked_to.required_ts > seeked_to.actual_ts {
                    self.seek_required_ts = Some(seeked_to.required_ts);
                }

                // some decoders need to be reset after a seek, but not all can be reset without unexpected behavior (like mka seeking to 0 again)
                // see https://github.com/pdeljanov/Symphonia/issues/274
                if self.decoder.codec_params().codec == codecs::audio::well_known::CODEC_ID_MP3 {
                    self.decoder.reset();
                }

                Ok(())
            }
            Err(_) => Ok(()),
        }
    }
}

impl Iterator for Symphonia {
    type Item = SampleType;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.decode_once()?;

        let sample = *self.buffer.get(self.current_frame_offset)?;
        self.advance_offset(1);

        Some(sample)
    }
}

/// Error that can happen when creating a decoder.
#[derive(Debug, Clone)]
pub enum SymphoniaDecoderError {
    /// The format of the data has not been recognized.
    UnrecognizedFormat,

    /// An IO error occurred while reading, writing, or seeking the stream.
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

    /// The track unexpectedly ended before giving the first audio data.
    UnexpectedEOFInit,
}

impl fmt::Display for SymphoniaDecoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::UnrecognizedFormat => "Unrecognized format",
            Self::IoError(msg) => &msg[..],
            Self::DecodeError(msg) | Self::LimitError(msg) => msg,
            Self::ResetRequired => "Reset required",
            Self::NoStreams => "No streams",
            Self::UnexpectedEOFInit => "Unexpected EOF before first audio data",
        };
        write!(f, "{text}")
    }
}

impl std::error::Error for SymphoniaDecoderError {}

impl From<symphonia::core::errors::Error> for SymphoniaDecoderError {
    fn from(value: symphonia::core::errors::Error) -> Self {
        match value {
            Error::IoError(e) => Self::IoError(e.to_string()),
            Error::DecodeError(e) => Self::DecodeError(e),
            Error::SeekError(_) => {
                unreachable!("Seek errors should not occur during initialization")
            }
            Error::Unsupported(_) => Self::UnrecognizedFormat,
            Error::LimitError(e) => Self::LimitError(e),
            Error::ResetRequired => Self::ResetRequired,
            _ => unimplemented!("Unimplemented mapping"), // TODO: fix this?
        }
    }
}

/// Resulting values from the decode loop
#[derive(Debug)]
struct DecodeLoopResult {
    spec: AudioSpec,
}

// is there maybe a better option for this?
enum BufferInputType<'a> {
    /// Allocate a new Buffer in the specified location (without unsafe)
    New(&'a mut Option<(Vec<SampleType>, usize)>),
    /// Try to reuse the provided Buffer
    Existing((&'a mut Vec<SampleType>, &'a mut usize)),
}

impl std::fmt::Debug for BufferInputType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::New(_) => f.debug_tuple("New").finish(),
            Self::Existing(_) => f.debug_tuple("Existing").finish(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
/// Decode until finding a valid packet and get the samples from it
///
/// If [`BufferInputType::New`] is used, it is guaranteed to be [`Some`] if function result is [`Ok`].
///
/// If `Ok(None)` is returned, it means End-of-File (EOF/EOS).
fn decode_loop(
    format: &mut dyn FormatReader,
    decoder: &mut dyn codecs::audio::AudioDecoder,
    buffer: BufferInputType<'_>,
    track_id: u32,
    time_base: Option<TimeBase>,
    media_title_tx: &mut MediaTitleTxWrap,
    seek_required_ts: &mut Option<units::Timestamp>,
) -> Result<Option<DecodeLoopResult>, symphonia::core::errors::Error> {
    let (audio_buf, elapsed) = loop {
        let Some(packet) = format.next_packet()? else {
            return Ok(None);
        };

        // Skip all packets that are not the selected track
        if packet.track_id() != track_id {
            continue;
        }

        // seeking in symphonia can only be done to the nearest packet in the format reader
        // so we need to also seek until the actually required_ts in the decoder
        if let Some(dur) = seek_required_ts {
            if packet.dts() < *dur {
                continue;
            }
            // else, remove the value as we are now at or beyond that point
            seek_required_ts.take();
        }

        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                // if we got 0 frames, we got 0 samples, so this packet likely did not contain audio data
                // lets try the next packet.
                // re https://github.com/pdeljanov/Symphonia/issues/403
                // prevents 0-length sample buffers before EOF.
                if audio_buf.frames() == 0 {
                    trace!("Decoded Audio, but got 0 frames of samples; continuing");
                    continue;
                }

                let elapsed = time_base
                    .map(|tb| tb.calc_time_saturating(packet.pts()))
                    .map(|time| {
                        let (secs, nanos) = time.parts();
                        Duration::new(u64::try_from(secs).unwrap_or(0), nanos)
                    });

                break (audio_buf, elapsed);
            }
            Err(Error::DecodeError(err)) => {
                info!("Non-fatal Decoder Error: {err}");
            }
            Err(err) => return Err(err),
        }
    };

    if !media_title_tx.is_none() {
        // run container metadata if new and on seek to 0
        // this maybe not be 100% reliable, where for example there is no "time_base", but i dont know of a case yet where that happens
        if elapsed.as_ref().is_some_and(Duration::is_zero) {
            trace!("Time is 0, doing container metadata");
            media_title_tx.send_reset();
            do_container_metdata(media_title_tx, format);
        } else if !format.metadata().is_latest() {
            // only execute it once if there is a new metadata iteration
            do_inline_metdata(media_title_tx, format);
        }
    }

    let spec = audio_buf.spec().clone();

    match buffer {
        BufferInputType::New(buffer) => {
            *buffer = Some(Symphonia::get_buffer_new(audio_buf));
        }
        BufferInputType::Existing(buffer) => {
            Symphonia::maybe_reuse_buffer(buffer, audio_buf);
        }
    }

    Ok(Some(DecodeLoopResult { spec }))
}

/// Do container metadata / track start metadata
///
/// No optimizations for when [`MediaTitleTxWrap`] is [`None`], should be done outside of this function
#[inline]
fn do_container_metdata(media_title_tx: &mut MediaTitleTxWrap, format: &mut dyn FormatReader) {
    // prefer standard container tags over non-standard
    let title = if let Some(metadata_rev) = format.metadata().current() {
        // tags that are from the container standard (like mkv)
        find_title_metadata(metadata_rev).cloned()
    } else {
        trace!("Did not find any metadata in either format or probe!");
        None
    };

    // TODO: maybe change things if https://github.com/pdeljanov/Symphonia/issues/273 should not get unified into metadata

    if let Some(title) = title {
        media_title_tx.media_title_send(MediaTitleType::Value(title));
    }
}

/// Some containers support updating / setting metadata as a frame somewhere inside the track, which could be used for live-streams
///
/// No optimizations for when [`MediaTitleTxWrap`] is [`None`], should be done outside of this function
#[inline]
fn do_inline_metdata(media_title_tx: &mut MediaTitleTxWrap, format: &mut dyn FormatReader) {
    if let Some(metadata_rev) = format.metadata().skip_to_latest()
        && let Some(title) = find_title_metadata(metadata_rev).cloned()
    {
        media_title_tx.media_title_send(MediaTitleType::Value(title));
    }
}

#[inline]
fn find_title_metadata(metadata: &MetadataRevision) -> Option<&String> {
    metadata.per_track.iter().find_map(|v| {
        v.metadata.tags.iter().find_map(|v| {
            let Some(StandardTag::TrackTitle(title)) = &v.std else {
                return None;
            };
            Some(&**title)
        })
    })
}
