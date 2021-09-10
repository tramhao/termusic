/// Errors that could occur within Lofty.
#[derive(thiserror::Error, Debug)]
pub enum LoftyError {
	// File extension/format related errors
	/// Unknown file extension.
	#[error("Failed to guess the metadata format based on the file extension.")]
	UnknownFileExtension,
	/// Unsupported file extension
	#[error("Unsupported format: {0}")]
	UnsupportedFormat(String),
	/// Unable to guess the format
	#[error("No format could be determined from the provided file.")]
	UnknownFormat,

	// File data related errors
	/// Provided an empty file
	#[error("File contains no data")]
	EmptyFile,
	/// Attempting to write an abnormally large amount of data
	#[error("An abnormally large amount of data was provided, and an overflow occurred")]
	TooMuchData,

	// Picture related errors
	/// Picture has an unsupported mime type
	#[error("Unsupported mime type: {0}")]
	UnsupportedMimeType(String),
	/// Provided an invalid picture
	#[error("Picture contains invalid data")]
	NotAPicture,

	// Tag related errors
	#[cfg(feature = "format-id3")]
	/// Any error from [`id3`]
	#[error(transparent)]
	Id3Tag(#[from] id3::Error),
	#[cfg(feature = "format-mp4")]
	/// Any error from [`mp4ameta`]
	#[error(transparent)]
	Mp4Tag(#[from] mp4ameta::Error),
	/// Errors that arrist while parsing OGG pages
	#[cfg(any(
		feature = "format-opus",
		feature = "format-vorbis",
		feature = "format-flac"
	))]
	#[error(transparent)]
	OggPage(#[from] ogg_pager::PageError),
	#[cfg(feature = "format-riff")]
	/// Errors that arise while reading/writing to RIFF files
	#[error("Riff: {0}")]
	Riff(&'static str),
	#[cfg(feature = "format-aiff")]
	/// Errors that arise while reading/writing to AIFF files
	#[error("Aiff: {0}")]
	Aiff(&'static str),
	#[cfg(feature = "format-flac")]
	/// Errors that arise while reading/writing to FLAC files
	#[error("Flac: {0}")]
	Flac(&'static str),
	#[cfg(feature = "format-opus")]
	/// Errors that arise while reading/writing to OPUS files
	#[error("Opus: {0}")]
	Opus(&'static str),
	#[cfg(feature = "format-vorbis")]
	/// Errors that arise while reading/writing to OGG Vorbis files
	#[error("Vorbis: {0}")]
	Vorbis(&'static str),
	#[cfg(any(
		feature = "format-opus",
		feature = "format-vorbis",
		feature = "format-flac"
	))]
	/// Errors that arise while reading/writing to OGG files
	#[error("OGG: {0}")]
	Ogg(&'static str),
	// TODO: feature
	/// Errors that arise while reading/writing to MPEG files
	#[error("MPEG: {0}")]
	Mpeg(&'static str),
	#[cfg(feature = "format-ape")]
	/// Errors that arise while reading/writing to APE files
	#[error("APE: {0}")]
	Ape(&'static str),

	// Conversions for std Errors
	/// Unable to convert bytes to a String
	#[error(transparent)]
	FromUtf8(#[from] std::string::FromUtf8Error),
	/// Represents all cases of `std::io::Error`.
	#[error(transparent)]
	Io(#[from] std::io::Error),
}

/// Result of tag operations.
pub type Result<T> = std::result::Result<T, LoftyError>;
