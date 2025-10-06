use std::{fmt, num::NonZeroU64, sync::LazyLock, time::Duration};

use symphonia::{
    core::{
        audio::{AudioBufferRef, SampleBuffer, SignalSpec},
        codecs::{self, CODEC_TYPE_NULL, CodecParameters, CodecRegistry},
        errors::Error,
        formats::{FormatOptions, FormatReader, SeekMode, SeekTo, Track},
        io::MediaSourceStream,
        meta::{MetadataOptions, MetadataRevision, StandardTagKey, Value},
        probe::{Hint, ProbeResult, ProbedMetadata},
        units::TimeBase,
    },
    default::{get_probe, register_enabled_codecs},
};
use tokio::sync::mpsc;

use super::{Source, source::SampleType};

pub mod buffered_source;
pub mod read_seek_source;

fn is_codec_null(track: &Track) -> bool {
    track.codec_params.codec == CODEC_TYPE_NULL
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
    registry.register_all::<symphonia_adapter_libopus::OpusDecoder>();
    registry
});

pub struct Symphonia {
    decoder: Box<dyn codecs::Decoder>,
    current_frame_offset: usize,
    probed: ProbeResult,
    buffer: SampleBuffer<SampleType>,
    spec: SignalSpec,
    duration: Option<Duration>,
    track_id: u32,
    time_base: Option<TimeBase>,
    seek_required_ts: Option<NonZeroU64>,

    media_title_tx: MediaTitleTxWrap,
}

impl Symphonia {
    /// Create a new symphonia decoder.
    ///
    /// The returned `Option<MediaTitleRx>` is always `Some` if parameter `media_title` is `true`.
    pub fn new(
        mss: MediaSourceStream,
        gapless: bool,
        media_title: bool,
    ) -> Result<(Self, Option<MediaTitleRx>), SymphoniaDecoderError> {
        match Self::init(mss, gapless, media_title) {
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
            Ok(Some((decoder, rx))) => Ok((decoder, rx)),
            Ok(None) => Err(SymphoniaDecoderError::NoStreams),
        }
    }

    fn init(
        mss: MediaSourceStream,
        gapless: bool,
        media_title: bool,
    ) -> symphonia::core::errors::Result<Option<(Self, Option<MediaTitleRx>)>> {
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

        let mut decoder = CODEC_REGISTRY.make(
            &track.codec_params,
            &codecs::DecoderOptions { verify: true },
        )?;

        let duration = Self::get_duration(&track.codec_params);
        let track_id = track.id;
        let time_base = track.codec_params.time_base;
        let mut media_title_tx = MediaTitleTxWrap::new();

        let media_title_rx = if media_title {
            Some(media_title_tx.media_title_channel())
        } else {
            None
        };

        // decode the first part, to get the spec and initial buffer
        let mut buffer = None;
        let DecodeLoopResult { spec } = decode_loop(
            &mut *probed.format,
            &mut *decoder,
            BufferInputType::New(&mut buffer),
            track_id,
            time_base,
            &mut media_title_tx,
            &mut probed.metadata,
            &mut None,
        )?;
        // safe to unwrap because "decode_loop" ensures it will be set
        let buffer = buffer.unwrap();

        Ok(Some((
            Self {
                decoder,
                current_frame_offset: 0,
                probed,
                buffer,
                spec,
                duration,
                track_id,
                time_base,
                seek_required_ts: None,

                media_title_tx,
            },
            media_title_rx,
        )))
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
    fn get_buffer_new(decoded: AudioBufferRef<'_>) -> SampleBuffer<SampleType> {
        let duration = decoded.capacity() as u64;
        let mut buffer = SampleBuffer::<SampleType>::new(duration, *decoded.spec());
        buffer.copy_interleaved_ref(decoded);
        buffer
    }

    /// Copy passed [`AudioBufferRef`] into the existing [`SampleBuffer`], if possible, otherwise create a new
    #[inline]
    fn maybe_reuse_buffer(buffer: &mut SampleBuffer<SampleType>, decoded: AudioBufferRef<'_>) {
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

    /// Run a potential decode, if the buffer is exhausted.
    pub fn decode_once(&mut self) -> Option<()> {
        if self.exhausted_buffer() {
            let DecodeLoopResult { spec } = decode_loop(
                &mut *self.probed.format,
                &mut *self.decoder,
                BufferInputType::Existing(&mut self.buffer),
                self.track_id,
                self.time_base,
                &mut self.media_title_tx,
                &mut self.probed.metadata,
                &mut self.seek_required_ts,
            )
            .ok()?;

            self.spec = spec;

            self.current_frame_offset = 0;
        }

        if self.buffer.samples().is_empty() {
            return None;
        }

        Some(())
    }

    /// Get whether the current buffer is used up.
    pub fn exhausted_buffer(&self) -> bool {
        self.buffer.samples().is_empty() || self.current_frame_offset == self.buffer.len()
    }

    /// Increase the offset from which to read the buffer from.
    pub fn advance_offset(&mut self, by: usize) {
        self.current_frame_offset += by;
    }

    /// Get the current spec plus frame length.
    pub fn get_spec(&self) -> (SignalSpec, usize) {
        (self.spec, self.current_span_len().unwrap())
    }

    /// Get the current buffer interpreted as u8(bytes) in native encoding.
    pub fn get_buffer_u8(&self) -> &[u8] {
        #[allow(unsafe_code)]
        unsafe {
            // re-interpret the SampleType slice as a u8 slice with the same byte-length.
            let len = size_of_val(self.buffer.samples());
            std::slice::from_raw_parts(
                self.buffer.samples()[self.current_frame_offset..]
                    .as_ptr()
                    .cast::<u8>(),
                len,
            )
        }
    }

    /// Get the current buffer, but only the part has not been read yet.
    pub fn get_buffer(&self) -> &[SampleType] {
        &self.buffer.samples()[self.current_frame_offset..]
    }
}

impl Source for Symphonia {
    #[inline]
    fn current_span_len(&self) -> Option<usize> {
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
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        match self.probed.format.seek(
            SeekMode::Coarse,
            SeekTo::Time {
                time: pos.into(),
                track_id: Some(self.track_id),
            },
        ) {
            Ok(seeked_to) => {
                // clear sample buffer after seek
                self.current_frame_offset = 0;
                self.buffer.clear();

                // Coarse seeking may seek (slightly) beyond the requested ts, so it may not actually need to be set
                if seeked_to.required_ts > seeked_to.actual_ts {
                    // the unwrap should never fail as "(0 > 0) == false" and "(0 > 1(or higher)) == false"
                    self.seek_required_ts = Some(NonZeroU64::new(seeked_to.required_ts).unwrap());
                }

                // some decoders need to be reset after a seek, but not all can be reset without unexpected behavior (like mka seeking to 0 again)
                // see https://github.com/pdeljanov/Symphonia/issues/274
                if self.decoder.codec_params().codec == codecs::CODEC_TYPE_MP3 {
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

/// Resulting values from the decode loop
#[derive(Debug)]
struct DecodeLoopResult {
    spec: SignalSpec,
}

// is there maybe a better option for this?
enum BufferInputType<'a> {
    /// Allocate a new [`SampleBuffer`] in the specified location (without unsafe)
    New(&'a mut Option<SampleBuffer<SampleType>>),
    /// Try to re-use the provided [`SampleBuffer`]
    Existing(&'a mut SampleBuffer<SampleType>),
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
fn decode_loop(
    format: &mut dyn FormatReader,
    decoder: &mut dyn codecs::Decoder,
    buffer: BufferInputType<'_>,
    track_id: u32,
    time_base: Option<TimeBase>,
    media_title_tx: &mut MediaTitleTxWrap,
    probed: &mut ProbedMetadata,
    seek_required_ts: &mut Option<NonZeroU64>,
) -> Result<DecodeLoopResult, symphonia::core::errors::Error> {
    let (audio_buf, elapsed) = loop {
        let packet = format.next_packet()?;

        // Skip all packets that are not the selected track
        if packet.track_id() != track_id {
            continue;
        }

        // seeking in symphonia can only be done to the nearest packet in the format reader
        // so we need to also seek until the actually required_ts in the decoder
        if let Some(dur) = seek_required_ts {
            if packet.ts() < dur.get() {
                continue;
            }
            // else, remove the value as we are now at or beyond that point
            seek_required_ts.take();
        }

        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                let ts = packet.ts();
                let elapsed = time_base.map(|tb| Duration::from(tb.calc_time(ts)));
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
            do_container_metdata(media_title_tx, format, probed);
        } else if !format.metadata().is_latest() {
            // only execute it once if there is a new metadata iteration
            do_inline_metdata(media_title_tx, format);
        }
    }

    let spec = *audio_buf.spec();

    match buffer {
        BufferInputType::New(buffer) => {
            *buffer = Some(Symphonia::get_buffer_new(audio_buf));
        }
        BufferInputType::Existing(buffer) => {
            Symphonia::maybe_reuse_buffer(buffer, audio_buf);
        }
    }

    Ok(DecodeLoopResult { spec })
}

/// Do container metadata / track start metadata
///
/// No optimizations for when [`MediaTitleTxWrap`] is [`None`], should be done outside of this function
#[inline]
fn do_container_metdata(
    media_title_tx: &mut MediaTitleTxWrap,
    format: &mut dyn FormatReader,
    probed: &mut ProbedMetadata,
) {
    // prefer standard container tags over non-standard
    let title = if let Some(metadata_rev) = format.metadata().current() {
        // tags that are from the container standard (like mkv)
        find_title_metadata(metadata_rev).cloned()
    } else if let Some(metadata_rev) = probed.get().as_ref().and_then(|m| m.current()) {
        // tags that are not from the container standard (like mp3)
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
    if let Some(metadata_rev) = format.metadata().skip_to_latest() {
        if let Some(title) = find_title_metadata(metadata_rev).cloned() {
            media_title_tx.media_title_send(MediaTitleType::Value(title));
        }
    }
}

#[inline]
fn find_title_metadata(metadata: &MetadataRevision) -> Option<&String> {
    metadata
        .tags()
        .iter()
        .find(|v| v.std_key.is_some_and(|v| v == StandardTagKey::TrackTitle))
        .and_then(|v| {
            if let Value::String(ref v) = v.value {
                Some(v)
            } else {
                None
            }
        })
}
