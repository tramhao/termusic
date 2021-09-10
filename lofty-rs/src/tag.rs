#[allow(clippy::wildcard_imports)]
use crate::components::tags::*;
use crate::{AudioTag, LoftyError, Result};

use byteorder::ReadBytesExt;
use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

/// A builder for `Box<dyn AudioTag>`
#[derive(Default)]
pub struct Tag(Option<TagType>);

impl Tag {
	/// Initiate a new Tag
	pub fn new() -> Self {
		Self::default()
	}

	/// This function can be used to specify a `TagType` to skip the guessing entirely
	#[allow(clippy::unused_self)]
	pub fn with_tag_type(self, tag_type: TagType) -> Self {
		Self(Some(tag_type))
	}

	/// Attempts to get the tag format based on the file extension
	///
	/// NOTE: Since this only looks at the extension, the result could be incorrect.
	///
	///
	/// # Errors
	///
	/// * `path` does not exist
	/// * `path` either has no extension, or the extension is not valid UTF-8
	/// * `path` has an unsupported/unknown extension
	///
	/// # Warning
	/// Using this on a `wav`/`wave`/`riff` file will **always** assume there's an ID3 tag.
	/// [`read_from_path_signature`](Tag::read_from_path_signature) is recommended, in the event that a RIFF INFO list is present instead.
	pub fn read_from_path(&self, path: impl AsRef<Path>) -> Result<Box<dyn AudioTag>> {
		let mut c = Cursor::new(std::fs::read(&path)?);

		let tag_type = self.0.clone().unwrap_or({
			let extension = path
				.as_ref()
				.extension()
				.ok_or(LoftyError::UnknownFileExtension)?;

			let extension_str = extension.to_str().ok_or(LoftyError::UnknownFileExtension)?;

			TagType::try_from_ext(extension_str)?
		});

		_read_from(&mut c, tag_type)
	}

	/// Attempts to get the tag format based on the file signature
	///
	/// NOTE: This is *slightly* slower than reading from extension, but more accurate.
	/// The only times were this would really be necessary is if the file format being read
	/// supports more than one metadata format (ex. RIFF), or there is no file extension.
	///
	/// # Errors
	///
	/// * `path` does not exist
	/// * The tag is non-existent/invalid/unknown
	///
	/// # Warning
	/// In the event that a riff file contains both an ID3 tag *and* a RIFF INFO chunk, the ID3 tag will **always** be chosen.
	pub fn read_from_path_signature(&self, path: impl AsRef<Path>) -> Result<Box<dyn AudioTag>> {
		let mut c = Cursor::new(std::fs::read(&path)?);
		let tag_type = self.0.clone().unwrap_or(TagType::try_from_sig(&mut c)?);

		_read_from(&mut c, tag_type)
	}

	/// Attempts to get the tag format based on the data in the reader
	///
	/// See [`read_from_path_signature`][Tag::read_from_path_signature] for important notes, errors, and warnings.
	///
	/// # Errors
	///
	/// Same as [`read_from_path_signature`][Tag::read_from_path_signature]
	pub fn read_from<R>(&self, reader: &mut R) -> Result<Box<dyn AudioTag>>
	where
		R: Read + Seek,
	{
		let tag_type = self.0.clone().unwrap_or(TagType::try_from_sig(reader)?);

		_read_from(reader, tag_type)
	}

	/// Attempts to remove the tag from a path
	///
	/// NOTE: It is not an error if the file doesn't contain a tag
	///
	/// # Errors
	///
	/// * `path` does not exist
	pub fn remove_from_path(self, path: impl AsRef<Path>) -> Result<()> {
		self.remove_from(&mut OpenOptions::new().read(true).write(true).open(path)?)
	}

	/// Attempts to remove the tag from a [`File`][std::fs::File]
	///
	/// NOTE: It is not an error if the file doesn't contain a tag
	///
	/// # Errors
	///
	/// * The file contains invalid data
	pub fn remove_from(self, file: &mut File) -> Result<()> {
		let tag_type = self.0.unwrap_or(TagType::try_from_sig(file)?);

		_remove_from(file, &tag_type)
	}
}

fn _remove_from(file: &mut File, tag_type: &TagType) -> Result<()> {
	match tag_type {
		#[cfg(feature = "format-ape")]
		TagType::Ape => ApeTag::remove_from(file),
		#[cfg(feature = "format-id3")]
		TagType::Id3v2(_) => Id3v2Tag::remove_from(file),
		#[cfg(feature = "format-mp4")]
		TagType::Mp4 => Mp4Tag::remove_from(file),
		#[cfg(feature = "format-riff")]
		TagType::RiffInfo => RiffTag::remove_from(file),
		#[cfg(any(
			feature = "format-vorbis",
			feature = "format-flac",
			feature = "format-opus"
		))]
		TagType::Ogg(ref format) => OggTag::remove_from(file, format),
		#[cfg(feature = "format-aiff")]
		TagType::AiffText => AiffTag::remove_from(file),
	}
}

fn _read_from<R>(reader: &mut R, tag_type: TagType) -> Result<Box<dyn AudioTag>>
where
	R: Read + Seek,
{
	match tag_type {
		#[cfg(feature = "format-ape")]
		TagType::Ape => Ok(Box::new(ApeTag::read_from(reader)?)),
		#[cfg(feature = "format-id3")]
		TagType::Id3v2(format) => Ok(Box::new(Id3v2Tag::read_from(reader, format)?)),
		#[cfg(feature = "format-mp4")]
		TagType::Mp4 => Ok(Box::new(Mp4Tag::read_from(reader)?)),
		#[cfg(feature = "format-riff")]
		TagType::RiffInfo => Ok(Box::new(RiffTag::read_from(reader)?)),
		#[cfg(any(
			feature = "format-vorbis",
			feature = "format-flac",
			feature = "format-opus"
		))]
		TagType::Ogg(ref format) => Ok(Box::new(OggTag::read_from(reader, format)?)),
		#[cfg(feature = "format-aiff")]
		TagType::AiffText => Ok(Box::new(AiffTag::read_from(reader)?)),
	}
}

/// The tag type, based on the file extension.
#[derive(Clone, Debug, PartialEq)]
pub enum TagType {
	#[cfg(feature = "format-ape")]
	/// Common file extensions: `.ape`
	Ape,
	#[cfg(feature = "format-id3")]
	/// Represents multiple formats, see [`Id3Format`](Id3Format) for extensions.
	Id3v2(Id3Format),
	#[cfg(feature = "format-mp4")]
	/// Common file extensions: `.mp4, .m4a, .m4p, .m4b, .m4r, .m4v`
	Mp4,
	#[cfg(any(
		feature = "format-vorbis",
		feature = "format-opus",
		feature = "format-flac"
	))]
	/// Represents multiple formats, see [`OggFormat`](OggFormat) for extensions.
	Ogg(OggFormat),
	#[cfg(feature = "format-riff")]
	/// Metadata stored in a RIFF INFO chunk
	/// Common file extensions: `.wav, .wave, .riff`
	RiffInfo,
	#[cfg(feature = "format-aiff")]
	/// Metadata stored in AIFF text chunks
	/// Common file extensions: `.aiff, .aif`
	AiffText,
}

#[cfg(any(
	feature = "format-vorbis",
	feature = "format-opus",
	feature = "format-flac"
))]
#[derive(Clone, Debug, PartialEq)]
/// File formats using vorbis comments
pub enum OggFormat {
	#[cfg(feature = "format-vorbis")]
	/// Common file extensions:  `.ogg, .oga`
	Vorbis,
	#[cfg(feature = "format-opus")]
	/// Common file extensions: `.opus`
	Opus,
	#[cfg(feature = "format-flac")]
	/// Common file extensions: `.flac`
	Flac,
}

#[cfg(feature = "format-id3")]
#[derive(Clone, Debug, PartialEq)]
/// ID3 tag's underlying format
pub enum Id3Format {
	/// MP3
	Mp3,
	/// AIFF
	Aiff,
	/// RIFF/WAV/WAVE
	Riff,
}

impl TagType {
	fn try_from_ext(ext: &str) -> Result<Self> {
		match ext {
			#[cfg(feature = "format-ape")]
			"ape" => Ok(Self::Ape),
			#[cfg(feature = "format-id3")]
			"aiff" | "aif" => Ok(Self::Id3v2(Id3Format::Aiff)),
			#[cfg(feature = "format-id3")]
			"mp3" => Ok(Self::Id3v2(Id3Format::Mp3)),
			#[cfg(all(feature = "format-riff", feature = "format-id3"))]
			"wav" | "wave" | "riff" => Ok(Self::Id3v2(Id3Format::Riff)),
			#[cfg(feature = "format-opus")]
			"opus" => Ok(Self::Ogg(OggFormat::Opus)),
			#[cfg(feature = "format-flac")]
			"flac" => Ok(Self::Ogg(OggFormat::Flac)),
			#[cfg(feature = "format-vorbis")]
			"ogg" | "oga" => Ok(Self::Ogg(OggFormat::Vorbis)),
			#[cfg(feature = "format-mp4")]
			"m4a" | "m4b" | "m4p" | "m4v" | "isom" | "mp4" => Ok(Self::Mp4),
			_ => Err(LoftyError::UnsupportedFormat(ext.to_string())),
		}
	}

	fn try_from_sig<R>(data: &mut R) -> Result<Self>
	where
		R: Read + Seek,
	{
		#[cfg(feature = "format-id3")]
		use crate::components::logic::{id3::decode_u32, mpeg::header::verify_frame_sync};

		if data.seek(SeekFrom::End(0))? == 0 {
			return Err(LoftyError::EmptyFile);
		}

		data.seek(SeekFrom::Start(0))?;

		let mut sig = vec![0; 10];
		data.read_exact(&mut sig)?;

		data.seek(SeekFrom::Start(0))?;

		match sig.first().unwrap() {
			#[cfg(feature = "format-ape")]
			77 if sig.starts_with(b"MAC") => Ok(Self::Ape),
			#[cfg(feature = "format-id3")]
			_ if verify_frame_sync(sig[0], sig[1])
				|| ((sig.starts_with(b"ID3") || sig.starts_with(b"id3")) && {
					let size = decode_u32(u32::from_be_bytes(
						sig[6..10]
							.try_into()
							.map_err(|_| LoftyError::UnknownFormat)?,
					));

					data.seek(SeekFrom::Start(u64::from(10 + size)))?;

					let b1 = data.read_u8()?;
					let b2 = data.read_u8()?;

					data.seek(SeekFrom::Start(0))?;

					verify_frame_sync(b1, b2)
				}) =>
			{
				Ok(Self::Id3v2(Id3Format::Mp3))
			},
			#[cfg(any(feature = "format-id3", feature = "format-aiff"))]
			70 if sig.starts_with(b"FORM") => {
				data.seek(SeekFrom::Start(8))?;

				let mut id = [0; 4];
				data.read_exact(&mut id)?;

				if &id == b"AIFF" || &id == b"AIFC" {
					#[cfg(feature = "format-id3")]
					{
						use byteorder::{BigEndian, LittleEndian};

						let mut found_id3 = false;

						while let (Ok(fourcc), Ok(size)) = (
							data.read_u32::<LittleEndian>(),
							data.read_u32::<BigEndian>(),
						) {
							if &fourcc.to_le_bytes()[..3] == b"ID3"
								|| &fourcc.to_le_bytes()[..3] == b"id3"
							{
								found_id3 = true;
								break;
							}

							data.seek(SeekFrom::Current(i64::from(u32::from_be_bytes(
								size.to_be_bytes(),
							))))?;
						}

						data.seek(SeekFrom::Start(0))?;

						if found_id3 {
							return Ok(Self::Id3v2(Id3Format::Aiff));
						}
					}

					#[cfg(feature = "format-aiff")]
					return Ok(Self::AiffText);
				}

				Err(LoftyError::UnknownFormat)
			},
			#[cfg(feature = "format-flac")]
			102 if sig.starts_with(b"fLaC") => Ok(Self::Ogg(OggFormat::Flac)),
			#[cfg(any(feature = "format-vorbis", feature = "format-opus"))]
			79 if sig.starts_with(b"OggS") => {
				data.seek(SeekFrom::Start(28))?;

				let mut ident_sig = vec![0; 8];
				data.read_exact(&mut ident_sig)?;

				data.seek(SeekFrom::Start(0))?;

				#[cfg(feature = "format-vorbis")]
				if &ident_sig[1..7] == b"vorbis" {
					return Ok(Self::Ogg(OggFormat::Vorbis));
				}

				#[cfg(feature = "format-opus")]
				if &ident_sig[..] == b"OpusHead" {
					return Ok(Self::Ogg(OggFormat::Opus));
				}

				Err(LoftyError::UnknownFormat)
			},
			#[cfg(feature = "format-riff")]
			82 if sig.starts_with(b"RIFF") => {
				#[cfg(feature = "format-id3")]
				{
					use byteorder::LittleEndian;

					data.seek(SeekFrom::Start(12))?;

					let mut found_id3 = false;

					while let (Ok(fourcc), Ok(size)) = (
						data.read_u32::<LittleEndian>(),
						data.read_u32::<LittleEndian>(),
					) {
						if &fourcc.to_le_bytes()[..3] == b"ID3"
							|| &fourcc.to_le_bytes()[..3] == b"id3"
						{
							found_id3 = true;
							break;
						}

						data.seek(SeekFrom::Current(i64::from(size)))?;
					}

					data.seek(SeekFrom::Start(0))?;

					if found_id3 {
						return Ok(Self::Id3v2(Id3Format::Riff));
					}
				}

				Ok(Self::RiffInfo)
			},
			#[cfg(feature = "format-mp4")]
			_ if &sig[4..8] == b"ftyp" => Ok(Self::Mp4),
			_ => Err(LoftyError::UnknownFormat),
		}
	}
}
