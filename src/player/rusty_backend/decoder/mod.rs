use std::error::Error;
use std::fmt;
use std::io::{Read, Seek};
use std::marker::Sync;
use std::str::FromStr;
use std::time::Duration;

use super::Source;

use self::{read_seek_source::ReadSeekSource, symphonia::SymphoniaDecoder};
use ::symphonia::core::io::{MediaSource, MediaSourceStream};
mod read_seek_source;
mod symphonia;

pub struct Decoder {
    decoder: SymphoniaDecoder,
}

impl Decoder {
    /// Builds a new decoder.
    ///
    /// Attempts to automatically detect the format of the source of data.
    pub fn new_decoder<R: Read + Seek + Send + Sync + 'static>(
        data: R,
    ) -> Result<SymphoniaDecoder, DecoderError> {
        let mss = MediaSourceStream::new(
            Box::new(ReadSeekSource::new(data)) as Box<dyn MediaSource>,
            ::symphonia::core::io::MediaSourceStreamOptions::default(),
        );

        match symphonia::SymphoniaDecoder::new(mss, None) {
            Err(e) => Err(e),
            Ok(decoder) => Ok(decoder),
        }
    }
}

#[derive(Debug)]
pub enum Mp4Type {
    Mp4,
    M4a,
    M4p,
    M4b,
    M4r,
    M4v,
    Mov,
}

impl FromStr for Mp4Type {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match &input.to_lowercase()[..] {
            "mp4" => Ok(Self::Mp4),
            "m4a" => Ok(Self::M4a),
            "m4p" => Ok(Self::M4p),
            "m4b" => Ok(Self::M4b),
            "m4r" => Ok(Self::M4r),
            "m4v" => Ok(Self::M4v),
            "mov" => Ok(Self::Mov),
            _ => Err(format!("{} is not a valid mp4 extension", input)),
        }
    }
}

impl fmt::Display for Mp4Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Mp4Type::Mp4 => "mp4",
            Mp4Type::M4a => "m4a",
            Mp4Type::M4p => "m4p",
            Mp4Type::M4b => "m4b",
            Mp4Type::M4r => "m4r",
            Mp4Type::M4v => "m4v",
            Mp4Type::Mov => "mov",
        };
        write!(f, "{}", text)
    }
}

impl Iterator for Decoder {
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        self.decoder.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.decoder.size_hint()
    }
}

impl Source for Decoder {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        self.decoder.current_frame_len()
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.decoder.channels()
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.decoder.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.decoder.total_duration()
    }

    #[inline]
    fn elapsed(&mut self) -> Duration {
        self.decoder.elapsed()
    }

    #[inline]
    fn seek(&mut self, time: Duration) -> Result<Duration, ()> {
        self.decoder.seek(time)
    }
}

/// Error that can happen when creating a decoder.
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub enum DecoderError {
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

impl fmt::Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            DecoderError::UnrecognizedFormat => "Unrecognized format",
            DecoderError::IoError(msg) => &msg[..],
            DecoderError::DecodeError(msg) | DecoderError::LimitError(msg) => msg,
            DecoderError::ResetRequired => "Reset required",
            DecoderError::NoStreams => "No streams",
        };
        write!(f, "{}", text)
    }
}
impl Error for DecoderError {}
