use crate::ape::ApeFile;
use crate::error::{LoftyError, Result};
use crate::iff::aiff::AiffFile;
use crate::iff::wav::WavFile;
use crate::mp3::header::verify_frame_sync;
use crate::mp3::Mp3File;
use crate::mp4::Mp4File;
use crate::ogg::flac::FlacFile;
use crate::ogg::opus::OpusFile;
use crate::ogg::vorbis::VorbisFile;
use crate::types::file::{AudioFile, FileType, TaggedFile};

use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::Path;

/// A format agnostic reader
///
/// This provides a way to determine the [`FileType`] of a reader, for when a concrete
/// type is not known.
///
/// ## Usage
///
/// When reading from a path, the [`FileType`] will be inferred from the path, rather than the
/// open file.
///
/// ```rust
/// # use lofty::{LoftyError, Probe};
/// # fn main() -> Result<(), LoftyError> {
/// use lofty::FileType;
///
/// let probe = Probe::open("tests/files/assets/a.mp3")?;
///
/// // Inferred from the `mp3` extension
/// assert_eq!(probe.file_type(), Some(FileType::MP3));
/// # Ok(())
/// # }
/// ```
///
/// When a path isn't available, or is unreliable, content-based detection is also possible.
///
/// ```rust
/// # use lofty::{LoftyError, Probe};
/// # fn main() -> Result<(), LoftyError> {
/// use lofty::FileType;
///
/// // Our same path probe with a guessed file type
/// let probe = Probe::open("tests/files/assets/a.mp3")?.guess_file_type()?;
///
/// // Inferred from the `mp3` extension
/// assert_eq!(probe.file_type(), Some(FileType::MP3));
/// # Ok(())
/// # }
/// ```
///
/// Or with another reader
///
/// ```rust
/// # use lofty::{LoftyError, Probe};
/// # fn main() -> Result<(), LoftyError> {
/// use lofty::FileType;
/// use std::io::Cursor;
///
/// static MAC_HEADER: &[u8; 3] = b"MAC";
///
/// let probe = Probe::new(Cursor::new(MAC_HEADER)).guess_file_type()?;
///
/// // Inferred from the MAC header
/// assert_eq!(probe.file_type(), Some(FileType::APE));
/// # Ok(())
/// # }
/// ```
pub struct Probe<R: Read> {
	inner: R,
	f_ty: Option<FileType>,
}

impl<R: Read> Probe<R> {
	/// Create a new `Probe`
	pub fn new(reader: R) -> Self {
		Self {
			inner: reader,
			f_ty: None,
		}
	}

	/// Create a new `Probe` with a specified [`FileType`]
	pub fn with_file_type(reader: R, file_type: FileType) -> Self {
		Self {
			inner: reader,
			f_ty: Some(file_type),
		}
	}

	/// Returns the current [`FileType`]
	pub fn file_type(&self) -> Option<FileType> {
		self.f_ty
	}

	/// Set the [`FileType`] with which to read the file
	pub fn set_file_type(&mut self, file_type: FileType) {
		self.f_ty = Some(file_type)
	}

	/// Extract the reader
	pub fn into_inner(self) -> R {
		self.inner
	}
}

impl Probe<BufReader<File>> {
	/// Opens a file for reading
	///
	/// This will initially guess the [`FileType`] from the path, but
	/// this can be overwritten with [`Probe::guess_file_type`] or [`Probe::set_file_type`]
	///
	/// # Errors
	///
	/// * `path` does not exist
	pub fn open<P>(path: P) -> Result<Self>
	where
		P: AsRef<Path>,
	{
		let path = path.as_ref();

		Ok(Self {
			inner: BufReader::new(File::open(path)?),
			f_ty: FileType::from_path(path).ok(),
		})
	}
}

impl<R: Read + Seek> Probe<R> {
	/// Attempts to get the [`FileType`] based on the data in the reader
	///
	/// On success, the file type will be replaced
	///
	/// # Errors
	///
	/// All errors that occur within this function are [`std::io::Error`].
	/// If an error does occur, there is likely an issue with the provided
	/// reader, and the entire `Probe` should be discarded.
	pub fn guess_file_type(mut self) -> Result<Self> {
		let f_ty = self.guess_inner()?;
		self.f_ty = f_ty.or(self.f_ty);

		Ok(self)
	}

	#[allow(clippy::shadow_unrelated)]
	fn guess_inner(&mut self) -> Result<Option<FileType>> {
		let mut buf = [0; 36];

		let pos = self.inner.seek(SeekFrom::Current(0))?;
		let buf_len = std::io::copy(
			&mut self.inner.by_ref().take(36),
			&mut Cursor::new(&mut buf[..]),
		)? as usize;

		self.inner.seek(SeekFrom::Start(pos))?;

		match FileType::from_buffer_inner(&buf[..buf_len]) {
			Ok((Some(f_ty), _)) => Ok(Some(f_ty)),
			Ok((None, id3_len)) => {
				self.inner
					.seek(SeekFrom::Current(i64::from(10 + id3_len)))?;

				let mut ident = [0; 3];
				let buf_len = std::io::copy(
					&mut self.inner.by_ref().take(3),
					&mut Cursor::new(&mut ident[..]),
				)?;

				self.inner.seek(SeekFrom::Start(pos))?;

				if buf_len < 3 {
					return Err(LoftyError::UnknownFormat);
				}

				if &ident == b"MAC" {
					Ok(Some(FileType::APE))
				} else if verify_frame_sync([ident[0], ident[1]]) {
					Ok(Some(FileType::MP3))
				} else {
					Err(LoftyError::UnknownFormat)
				}
			}
			_ => Ok(None),
		}
	}

	/// Attempts to extract a [`TaggedFile`] from the reader
	///
	/// If `read_properties` is false, the properties will be zeroed out.
	///
	/// # Errors
	///
	/// * No file type
	///     - This expects the file type to have been set already, either with
	///       [`Probe::guess_file_type`] or [`Probe::set_file_type`]. When reading from
	///       paths, this is not necessary.
	/// * The reader contains invalid data
	pub fn read(mut self, read_properties: bool) -> Result<TaggedFile> {
		let reader = &mut self.inner;

		match self.f_ty {
			Some(f_type) => Ok(match f_type {
				FileType::AIFF => AiffFile::read_from(reader, read_properties)?.into(),
				FileType::APE => ApeFile::read_from(reader, read_properties)?.into(),
				FileType::FLAC => FlacFile::read_from(reader, read_properties)?.into(),
				FileType::MP3 => Mp3File::read_from(reader, read_properties)?.into(),
				FileType::Opus => OpusFile::read_from(reader, read_properties)?.into(),
				FileType::Vorbis => VorbisFile::read_from(reader, read_properties)?.into(),
				FileType::WAV => WavFile::read_from(reader, read_properties)?.into(),
				FileType::MP4 => Mp4File::read_from(reader, read_properties)?.into(),
			}),
			None => Err(LoftyError::UnknownFormat),
		}
	}
}

/// Read a [`TaggedFile`] from a [File]
///
/// # Errors
///
/// See:
///
/// * [`Probe::guess_file_type`]
/// * [`Probe::read`]
pub fn read_from(file: &mut File, read_properties: bool) -> Result<TaggedFile> {
	Probe::new(file).guess_file_type()?.read(read_properties)
}

/// Read a [`TaggedFile`] from a path
///
/// NOTE: This will determine the [`FileType`] from the extension
///
/// # Errors
///
/// See:
///
/// * [`Probe::open`]
/// * [`Probe::read`]
pub fn read_from_path<P>(path: P, read_properties: bool) -> Result<TaggedFile>
where
	P: AsRef<Path>,
{
	Probe::open(path)?.read(read_properties)
}
