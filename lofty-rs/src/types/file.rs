use super::properties::FileProperties;
use super::tag::{Tag, TagType};
use crate::error::{LoftyError, Result};

use std::convert::TryInto;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek};
use std::path::Path;

/// Provides various methods for interaction with a file
pub trait AudioFile {
	/// The struct the file uses for audio properties
	///
	/// Not all formats can use [`FileProperties`] since they may contain additional information
	type Properties;

	/// Read a file from a reader
	///
	/// # Errors
	///
	/// Errors depend on the file and tags being read. See [`LoftyError`]
	fn read_from<R>(reader: &mut R, read_properties: bool) -> Result<Self>
	where
		R: Read + Seek,
		Self: Sized;
	/// Returns a reference to the file's properties
	fn properties(&self) -> &Self::Properties;
	/// Checks if the file contains any tags
	fn contains_tag(&self) -> bool;
	/// Checks if the file contains the given [`TagType`]
	fn contains_tag_type(&self, tag_type: &TagType) -> bool;
}

/// A generic representation of a file
///
/// This is used when the [`FileType`] has to be guessed
pub struct TaggedFile {
	/// The file's type
	pub(crate) ty: FileType,
	/// The file's audio properties
	pub(crate) properties: FileProperties,
	/// A collection of the file's tags
	pub(crate) tags: Vec<Tag>,
}

#[cfg(any(
	feature = "id3v1",
	feature = "riff_info_list",
	feature = "aiff_text_chunks",
	feature = "vorbis_comments",
	feature = "id3v2",
	feature = "mp4_ilst",
	feature = "ape"
))]
impl TaggedFile {
	/// Gets the file's "Primary tag", or the one most likely to be used in the target format
	///
	/// | [`FileType`]             | [`TagType`]      |
	/// |--------------------------|------------------|
	/// | `AIFF`, `MP3`, `WAV`     | `Id3v2`          |
	/// | `APE`                    | `Ape`            |
	/// | `FLAC`, `Opus`, `Vorbis` | `VorbisComments` |
	/// | `MP4`                    | `Mp4Ilst`        |
	pub fn primary_tag(&self) -> Option<&Tag> {
		self.tag(&self.primary_tag_type())
	}

	/// Gets a mutable reference to the file's "Primary tag"
	///
	/// See [`primary_tag`](Self::primary_tag) for an explanation
	pub fn primary_tag_mut(&mut self) -> Option<&mut Tag> {
		self.tag_mut(&self.primary_tag_type())
	}

	#[allow(unreachable_patterns, clippy::match_same_arms)]
	/// Returns the file type's "primary" [`TagType`]
	///
	/// See [`primary_tag`](Self::primary_tag) for an explanation
	pub fn primary_tag_type(&self) -> TagType {
		match self.ty {
			#[cfg(all(not(feature = "id3v2"), feature = "aiff_text_chunks"))]
			FileType::AIFF => TagType::AiffText,
			#[cfg(all(not(feature = "id3v2"), feature = "riff_info_list"))]
			FileType::WAV => TagType::RiffInfo,
			#[cfg(all(not(feature = "id3v2"), feature = "id3v1"))]
			FileType::MP3 => TagType::Id3v1,
			#[cfg(all(not(feature = "id3v2"), not(feature = "id3v1"), feature = "ape"))]
			FileType::MP3 => TagType::Ape,
			FileType::AIFF | FileType::MP3 | FileType::WAV => TagType::Id3v2,
			#[cfg(all(not(feature = "ape"), feature = "id3v1"))]
			FileType::MP3 => TagType::Id3v1,
			FileType::APE => TagType::Ape,
			FileType::FLAC | FileType::Opus | FileType::Vorbis => TagType::VorbisComments,
			FileType::MP4 => TagType::Mp4Ilst,
		}
	}

	/// Determines whether the file supports the given [`TagType`]
	pub fn supports_tag_type(&self, tag_type: TagType) -> bool {
		self.ty.supports_tag_type(&tag_type)
	}

	/// Returns all tags
	pub fn tags(&self) -> &[Tag] {
		self.tags.as_slice()
	}

	/// Gets the first tag, if there are any
	pub fn first_tag(&self) -> Option<&Tag> {
		self.tags.first()
	}

	/// Gets a mutable reference to the first tag, if there are any
	pub fn first_tag_mut(&mut self) -> Option<&mut Tag> {
		self.tags.first_mut()
	}

	/// Get a reference to a specific [`TagType`]
	pub fn tag(&self, tag_type: &TagType) -> Option<&Tag> {
		self.tags.iter().find(|i| i.tag_type() == tag_type)
	}

	/// Get a mutable reference to a specific [`TagType`]
	pub fn tag_mut(&mut self, tag_type: &TagType) -> Option<&mut Tag> {
		self.tags.iter_mut().find(|i| i.tag_type() == tag_type)
	}

	/// Inserts a [`Tag`]
	///
	/// If a tag is replaced, it will be returned
	pub fn insert_tag(&mut self, tag: Tag) -> Option<Tag> {
		let tag_type = *tag.tag_type();

		if self.supports_tag_type(tag_type) {
			let ret = self.remove_tag(tag_type);
			self.tags.push(tag);

			return ret;
		}

		None
	}

	/// Removes a specific [`TagType`]
	///
	/// This will return the tag if it is removed
	pub fn remove_tag(&mut self, tag_type: TagType) -> Option<Tag> {
		self.tags
			.iter()
			.position(|t| t.tag_type() == &tag_type)
			.map(|pos| self.tags.remove(pos))
	}

	/// Attempts to write all tags to a path
	///
	/// # Errors
	///
	/// See [TaggedFile::save_to]
	pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<()> {
		self.save_to(&mut OpenOptions::new().read(true).write(true).open(path)?)
	}

	/// Attempts to write all tags to a file
	///
	/// # Errors
	///
	/// See [`Tag::save_to`], however this is applicable to every tag in the `TaggedFile`.
	pub fn save_to(&self, file: &mut File) -> Result<()> {
		for tag in &self.tags {
			tag.save_to(file)?;
		}

		Ok(())
	}
}

impl TaggedFile {
	/// Returns the file's [`FileType`]
	pub fn file_type(&self) -> &FileType {
		&self.ty
	}

	/// Returns a reference to the file's [`FileProperties`]
	pub fn properties(&self) -> &FileProperties {
		&self.properties
	}
}

#[derive(PartialEq, Copy, Clone, Debug)]
#[allow(missing_docs)]
/// The type of file read
pub enum FileType {
	AIFF,
	APE,
	FLAC,
	MP3,
	MP4,
	Opus,
	Vorbis,
	WAV,
}

impl FileType {
	/// Returns if the target FileType supports a [`TagType`]
	pub fn supports_tag_type(&self, tag_type: &TagType) -> bool {
		match self {
			#[cfg(feature = "id3v2")]
			FileType::AIFF | FileType::APE | FileType::MP3 | FileType::WAV
				if tag_type == &TagType::Id3v2 =>
			{
				true
			},
			#[cfg(feature = "aiff_text_chunks")]
			FileType::AIFF if tag_type == &TagType::AiffText => true,
			#[cfg(feature = "id3v1")]
			FileType::APE | FileType::MP3 if tag_type == &TagType::Id3v1 => true,
			#[cfg(feature = "ape")]
			FileType::APE | FileType::MP3 if tag_type == &TagType::Ape => true,
			#[cfg(feature = "vorbis_comments")]
			FileType::Opus | FileType::FLAC | FileType::Vorbis => tag_type == &TagType::VorbisComments,
			#[cfg(feature = "mp4_ilst")]
			FileType::MP4 => tag_type == &TagType::Mp4Ilst,
			#[cfg(feature = "riff_info_list")]
			FileType::WAV => tag_type == &TagType::RiffInfo,
			_ => false,
		}
	}

	/// Attempts to extract a [`FileType`] from an extension
	pub fn from_ext<E>(ext: E) -> Option<Self>
	where
		E: AsRef<OsStr>,
	{
		let ext = ext.as_ref().to_str()?.to_ascii_lowercase();

		match ext.as_str() {
			"ape" => Some(Self::APE),
			"aiff" | "aif" => Some(Self::AIFF),
			"mp3" => Some(Self::MP3),
			"wav" | "wave" => Some(Self::WAV),
			"opus" => Some(Self::Opus),
			"flac" => Some(Self::FLAC),
			"ogg" => Some(Self::Vorbis),
			"mp4" | "m4a" | "m4b" | "m4p" | "m4r" | "m4v" | "3gp" => Some(Self::MP4),
			_ => None,
		}
	}

	/// Attempts to extract a [`FileType`] from a path
	///
	/// # Errors
	///
	/// This will return [`LoftyError::BadExtension`] if the extension didn't map to a `FileType`
	pub fn from_path<P>(path: P) -> Result<Self>
	where
		P: AsRef<Path>,
	{
		let ext = path.as_ref().extension();

		ext.and_then(Self::from_ext).map_or_else(
			|| {
				let ext_err = match ext {
					Some(ext) => ext.to_string_lossy().into_owned(),
					None => String::new(),
				};

				Err(LoftyError::BadExtension(ext_err))
			},
			Ok,
		)
	}

	/// Attempts to extract a [`FileType`] from a buffer
	///
	/// NOTE: This is for use in [`Probe::guess_file_type`](crate::Probe::guess_file_type), it
	/// is recommended to use it that way
	pub fn from_buffer(buf: &[u8]) -> Option<Self> {
		match Self::from_buffer_inner(buf) {
			Ok((Some(f_ty), _)) => Some(f_ty),
			_ => None,
		}
	}

	pub(crate) fn from_buffer_inner(buf: &[u8]) -> Result<(Option<Self>, u32)> {
		use crate::id3::v2::unsynch_u32;

		if buf.is_empty() {
			return Err(LoftyError::EmptyFile);
		}

		match Self::quick_type_guess(buf) {
			Some(f_ty) => Ok((Some(f_ty), 0)),
			// Special case for ID3, gets checked in `Probe::guess_file_type`
			None if buf.starts_with(b"ID3") && buf.len() >= 11 => {
				let size = unsynch_u32(u32::from_be_bytes(
					buf[6..10]
						.try_into()
						.map_err(|_| LoftyError::UnknownFormat)?,
				));

				Ok((None, size))
			},
			None => Err(LoftyError::UnknownFormat),
		}
	}

	fn quick_type_guess(buf: &[u8]) -> Option<Self> {
		use crate::mp3::header::verify_frame_sync;

		match buf.first().unwrap() {
			77 if buf.starts_with(b"MAC") => Some(Self::APE),
			_ if verify_frame_sync([buf[0], buf[1]]) => Some(Self::MP3),
			70 if buf.starts_with(b"FORM") => {
				if buf.len() >= 12 {
					let id = &[buf[8], buf[9], buf[10], buf[11]];

					if id == b"AIFF" || id == b"AIFC" {
						return Some(Self::AIFF);
					}
				}

				None
			},
			79 if buf.starts_with(b"OggS") => {
				if buf.len() >= 36 {
					if &buf[29..35] == b"vorbis" {
						return Some(Self::Vorbis);
					} else if &buf[28..36] == b"OpusHead" {
						return Some(Self::Opus);
					}
				}

				None
			},
			102 if buf.starts_with(b"fLaC") => Some(Self::FLAC),
			82 if buf.starts_with(b"RIFF") => {
				if buf.len() >= 12 && &buf[8..12] == b"WAVE" {
					return Some(Self::WAV);
				}

				None
			},
			_ if buf.len() >= 8 && &buf[4..8] == b"ftyp" => Some(Self::MP4),
			_ => None,
		}
	}
}
