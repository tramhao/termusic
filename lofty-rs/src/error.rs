use std::fmt::{Display, Formatter};

use ogg_pager::PageError;

/// Alias for `Result<T, LoftyError>`
pub type Result<T> = std::result::Result<T, LoftyError>;

/// Errors that could occur within Lofty
#[derive(Debug)]
pub enum LoftyError {
	// File extension/format related errors
	/// Unsupported file extension
	BadExtension(String),
	/// Unable to guess the format
	UnknownFormat,

	// File data related errors
	/// Provided an empty file
	EmptyFile,
	/// Attempting to read/write an abnormally large amount of data
	TooMuchData,

	// Picture related errors
	#[cfg(feature = "id3v2")]
	/// Arises when an invalid picture format is parsed. Only applicable to [`Id3v2Version::V2`](crate::id3::v2::Id3v2Version)
	BadPictureFormat(String),
	/// Provided an invalid picture
	NotAPicture,
	/// Attempted to write a picture that the format does not support
	UnsupportedPicture,

	// Tag related errors
	/// Arises when writing a tag to a file type that doesn't support it
	UnsupportedTag,
	/// Arises when a tag is expected (Ex. found an "ID3 " chunk in a WAV file), but isn't found
	FakeTag,
	/// Errors that arise while decoding text
	TextDecode(&'static str),
	/// Errors that arise while reading/writing ID3v2 tags
	Id3v2(&'static str),
	/// Arises when an invalid ID3v2 version is found
	BadId3v2Version(u8, u8),
	#[cfg(feature = "id3v2")]
	/// Arises when [`std::str::from_utf8`] fails to parse a frame ID
	BadFrameID,
	#[cfg(feature = "id3v2")]
	/// Arises when a frame doesn't have enough data
	BadFrameLength,
	#[cfg(feature = "id3v2")]
	/// Arises when invalid data is encountered while reading an ID3v2 synchronized text frame
	BadSyncText,
	/// Arises when an atom contains invalid data
	BadAtom(&'static str),

	// File specific errors
	/// Errors that arise while reading/writing to WAV files
	Wav(&'static str),
	/// Errors that arise while reading/writing to AIFF files
	Aiff(&'static str),
	/// Errors that arise while reading/writing to FLAC files
	Flac(&'static str),
	/// Errors that arise while reading/writing to OPUS files
	Opus(&'static str),
	/// Errors that arise while reading/writing to OGG Vorbis files
	Vorbis(&'static str),
	/// Errors that arise while reading/writing to OGG files
	Ogg(&'static str),
	/// Errors that arise while reading/writing to MP3 files
	Mp3(&'static str),
	/// Errors that arise while reading/writing to MP4 files
	Mp4(&'static str),
	/// Errors that arise while reading/writing to APE files
	Ape(&'static str),

	// Conversions for external errors
	/// Errors that arise while parsing OGG pages
	OggPage(ogg_pager::PageError),
	/// Unable to convert bytes to a String
	FromUtf8(std::string::FromUtf8Error),
	/// Represents all cases of [`std::io::Error`].
	Io(std::io::Error),
}

impl Display for LoftyError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			// Conversions
			LoftyError::OggPage(ref err) => write!(f, "{}", err),
			LoftyError::FromUtf8(ref err) => write!(f, "{}", err),
			LoftyError::Io(ref err) => write!(f, "{}", err),

			LoftyError::BadExtension(ext) => write!(f, "Found unknown file extension \"{}\"", ext),
			LoftyError::UnknownFormat => {
				write!(f, "No format could be determined from the provided file")
			},
			LoftyError::EmptyFile => write!(f, "File contains no data"),
			LoftyError::TooMuchData => write!(
				f,
				"An abnormally large amount of data was provided, and an overflow occurred"
			),
			LoftyError::NotAPicture => write!(f, "Picture: Encountered invalid data"),
			LoftyError::UnsupportedPicture => {
				write!(f, "Picture: attempted to write an unsupported picture")
			},
			LoftyError::UnsupportedTag => write!(
				f,
				"Attempted to write a tag to a format that does not support it"
			),
			LoftyError::FakeTag => write!(f, "Reading: Expected a tag, found invalid data"),
			#[cfg(feature = "id3v2")]
			LoftyError::BadPictureFormat(format) => {
				write!(f, "Picture: Found unexpected format \"{}\"", format)
			},
			LoftyError::TextDecode(message) => write!(f, "Text decoding: {}", message),
			LoftyError::Id3v2(message) => write!(f, "ID3v2: {}", message),
			LoftyError::BadId3v2Version(major, minor) => write!(
				f,
				"ID3v2: Found an invalid version (v{}.{}), expected any major revision in: (2, 3, \
				 4)",
				major, minor
			),
			#[cfg(feature = "id3v2")]
			LoftyError::BadFrameID => write!(f, "ID3v2: Failed to parse a frame ID"),
			#[cfg(feature = "id3v2")]
			LoftyError::BadFrameLength => write!(
				f,
				"ID3v2: Frame isn't long enough to extract the necessary information"
			),
			#[cfg(feature = "id3v2")]
			LoftyError::BadSyncText => write!(f, "ID3v2: Encountered invalid data in SYLT frame"),
			LoftyError::BadAtom(message) => write!(f, "MP4 Atom: {}", message),

			// Files
			LoftyError::Wav(message) => write!(f, "WAV: {}", message),
			LoftyError::Aiff(message) => write!(f, "AIFF: {}", message),
			LoftyError::Flac(message) => write!(f, "FLAC: {}", message),
			LoftyError::Opus(message) => write!(f, "Opus: {}", message),
			LoftyError::Vorbis(message) => write!(f, "OGG Vorbis: {}", message),
			LoftyError::Ogg(message) => write!(f, "OGG: {}", message),
			LoftyError::Mp3(message) => write!(f, "MP3: {}", message),
			LoftyError::Mp4(message) => write!(f, "MP4: {}", message),
			LoftyError::Ape(message) => write!(f, "APE: {}", message),
		}
	}
}

impl std::error::Error for LoftyError {}

impl From<ogg_pager::PageError> for LoftyError {
	fn from(input: PageError) -> Self {
		LoftyError::OggPage(input)
	}
}

impl From<std::io::Error> for LoftyError {
	fn from(input: std::io::Error) -> Self {
		LoftyError::Io(input)
	}
}

impl From<std::string::FromUtf8Error> for LoftyError {
	fn from(input: std::string::FromUtf8Error) -> Self {
		LoftyError::FromUtf8(input)
	}
}
