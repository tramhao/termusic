use crate::{LoftyError, Result};
#[cfg(feature = "id3v2")]
use {crate::id3::v2::util::text_utils::TextEncoding, crate::id3::v2::Id3v2Version};

use std::borrow::Cow;
#[cfg(any(feature = "vorbis_comments", feature = "ape", feature = "id3v2"))]
use std::io::Cursor;
use std::io::Read;
#[cfg(feature = "id3v2")]
use std::io::Write;
#[cfg(any(feature = "vorbis_comments", feature = "ape"))]
use std::io::{Seek, SeekFrom};

#[cfg(any(feature = "vorbis_comments"))]
use byteorder::BigEndian;
#[cfg(any(feature = "vorbis_comments", feature = "id3v2", feature = "ape"))]
use byteorder::ReadBytesExt;
#[cfg(feature = "id3v2")]
use byteorder::WriteBytesExt;

#[cfg(feature = "ape")]
/// Common picture item keys for APE
pub const APE_PICTURE_TYPES: [&str; 21] = [
	"Cover Art (Other)",
	"Cover Art (Png Icon)",
	"Cover Art (Icon)",
	"Cover Art (Front)",
	"Cover Art (Back)",
	"Cover Art (Leaflet)",
	"Cover Art (Media)",
	"Cover Art (Lead Artist)",
	"Cover Art (Artist)",
	"Cover Art (Conductor)",
	"Cover Art (Band)",
	"Cover Art (Composer)",
	"Cover Art (Lyricist)",
	"Cover Art (Recording Location)",
	"Cover Art (During Recording)",
	"Cover Art (During Performance)",
	"Cover Art (Video Capture)",
	"Cover Art (Fish)",
	"Cover Art (Illustration)",
	"Cover Art (Band Logotype)",
	"Cover Art (Publisher Logotype)",
];

/// Mime types for pictures.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MimeType {
	/// PNG image
	Png,
	/// JPEG image
	Jpeg,
	/// TIFF image
	Tiff,
	/// BMP image
	Bmp,
	/// GIF image
	Gif,
	/// Some unknown mimetype
	Unknown(String),
	/// No mimetype
	None,
}

impl ToString for MimeType {
	fn to_string(&self) -> String {
		match self {
			MimeType::Jpeg => "image/jpeg".to_string(),
			MimeType::Png => "image/png".to_string(),
			MimeType::Tiff => "image/tiff".to_string(),
			MimeType::Bmp => "image/bmp".to_string(),
			MimeType::Gif => "image/gif".to_string(),
			MimeType::Unknown(unknown) => unknown.clone(),
			MimeType::None => String::new(),
		}
	}
}

impl MimeType {
	#[allow(clippy::should_implement_trait)]
	/// Get a MimeType from a string
	pub fn from_str(mime_type: &str) -> Self {
		match &*mime_type.to_lowercase() {
			"image/jpeg" => Self::Jpeg,
			"image/png" => Self::Png,
			"image/tiff" => Self::Tiff,
			"image/bmp" => Self::Bmp,
			"image/gif" => Self::Gif,
			"" => Self::None,
			_ => Self::Unknown(mime_type.to_string()),
		}
	}

	/// Get a &str from a MimeType
	pub fn as_str(&self) -> &str {
		match self {
			MimeType::Jpeg => "image/jpeg",
			MimeType::Png => "image/png",
			MimeType::Tiff => "image/tiff",
			MimeType::Bmp => "image/bmp",
			MimeType::Gif => "image/gif",
			MimeType::Unknown(unknown) => &*unknown,
			MimeType::None => "",
		}
	}
}

/// The picture type
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PictureType {
	Other,
	Icon,
	OtherIcon,
	CoverFront,
	CoverBack,
	Leaflet,
	Media,
	LeadArtist,
	Artist,
	Conductor,
	Band,
	Composer,
	Lyricist,
	RecordingLocation,
	DuringRecording,
	DuringPerformance,
	ScreenCapture,
	BrightFish,
	Illustration,
	BandLogo,
	PublisherLogo,
	Undefined(u8),
}

impl PictureType {
	// ID3/OGG specific methods

	/// Get a u8 from a PictureType according to ID3v2 APIC
	#[cfg(any(feature = "id3v2", feature = "vorbis_comments"))]
	pub fn as_u8(&self) -> u8 {
		match self {
			Self::Other => 0,
			Self::Icon => 1,
			Self::OtherIcon => 2,
			Self::CoverFront => 3,
			Self::CoverBack => 4,
			Self::Leaflet => 5,
			Self::Media => 6,
			Self::LeadArtist => 7,
			Self::Artist => 8,
			Self::Conductor => 9,
			Self::Band => 10,
			Self::Composer => 11,
			Self::Lyricist => 12,
			Self::RecordingLocation => 13,
			Self::DuringRecording => 14,
			Self::DuringPerformance => 15,
			Self::ScreenCapture => 16,
			Self::BrightFish => 17,
			Self::Illustration => 18,
			Self::BandLogo => 19,
			Self::PublisherLogo => 20,
			Self::Undefined(i) => *i,
		}
	}

	/// Get a PictureType from a u8 according to ID3v2 APIC
	#[cfg(any(feature = "id3v2", feature = "vorbis_comments"))]
	pub fn from_u8(bytes: u8) -> Self {
		match bytes {
			0 => Self::Other,
			1 => Self::Icon,
			2 => Self::OtherIcon,
			3 => Self::CoverFront,
			4 => Self::CoverBack,
			5 => Self::Leaflet,
			6 => Self::Media,
			7 => Self::LeadArtist,
			8 => Self::Artist,
			9 => Self::Conductor,
			10 => Self::Band,
			11 => Self::Composer,
			12 => Self::Lyricist,
			13 => Self::RecordingLocation,
			14 => Self::DuringRecording,
			15 => Self::DuringPerformance,
			16 => Self::ScreenCapture,
			17 => Self::BrightFish,
			18 => Self::Illustration,
			19 => Self::BandLogo,
			20 => Self::PublisherLogo,
			i => Self::Undefined(i as u8),
		}
	}

	// APE specific methods

	/// Get an APE item key from a PictureType
	#[cfg(feature = "ape")]
	pub fn as_ape_key(&self) -> Option<&str> {
		match self {
			Self::Other => Some("Cover Art (Other)"),
			Self::Icon => Some("Cover Art (Png Icon)"),
			Self::OtherIcon => Some("Cover Art (Icon)"),
			Self::CoverFront => Some("Cover Art (Front)"),
			Self::CoverBack => Some("Cover Art (Back)"),
			Self::Leaflet => Some("Cover Art (Leaflet)"),
			Self::Media => Some("Cover Art (Media)"),
			Self::LeadArtist => Some("Cover Art (Lead Artist)"),
			Self::Artist => Some("Cover Art (Artist)"),
			Self::Conductor => Some("Cover Art (Conductor)"),
			Self::Band => Some("Cover Art (Band)"),
			Self::Composer => Some("Cover Art (Composer)"),
			Self::Lyricist => Some("Cover Art (Lyricist)"),
			Self::RecordingLocation => Some("Cover Art (Recording Location)"),
			Self::DuringRecording => Some("Cover Art (During Recording)"),
			Self::DuringPerformance => Some("Cover Art (During Performance)"),
			Self::ScreenCapture => Some("Cover Art (Video Capture)"),
			Self::BrightFish => Some("Cover Art (Fish)"),
			Self::Illustration => Some("Cover Art (Illustration)"),
			Self::BandLogo => Some("Cover Art (Band Logotype)"),
			Self::PublisherLogo => Some("Cover Art (Publisher Logotype)"),
			Self::Undefined(_) => None,
		}
	}

	/// Get a PictureType from an APE item key
	#[cfg(feature = "ape")]
	pub fn from_ape_key(key: &str) -> Self {
		match key {
			"Cover Art (Other)" => Self::Other,
			"Cover Art (Png Icon)" => Self::Icon,
			"Cover Art (Icon)" => Self::OtherIcon,
			"Cover Art (Front)" => Self::CoverFront,
			"Cover Art (Back)" => Self::CoverBack,
			"Cover Art (Leaflet)" => Self::Leaflet,
			"Cover Art (Media)" => Self::Media,
			"Cover Art (Lead Artist)" => Self::LeadArtist,
			"Cover Art (Artist)" => Self::Artist,
			"Cover Art (Conductor)" => Self::Conductor,
			"Cover Art (Band)" => Self::Band,
			"Cover Art (Composer)" => Self::Composer,
			"Cover Art (Lyricist)" => Self::Lyricist,
			"Cover Art (Recording Location)" => Self::RecordingLocation,
			"Cover Art (During Recording)" => Self::DuringRecording,
			"Cover Art (During Performance)" => Self::DuringPerformance,
			"Cover Art (Video Capture)" => Self::ScreenCapture,
			"Cover Art (Fish)" => Self::BrightFish,
			"Cover Art (Illustration)" => Self::Illustration,
			"Cover Art (Band Logotype)" => Self::BandLogo,
			"Cover Art (Publisher Logotype)" => Self::PublisherLogo,
			_ => Self::Undefined(0),
		}
	}
}

#[cfg(feature = "vorbis_comments")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
/// Information about a [`Picture`]
///
/// This information is necessary for FLAC's `METADATA_BLOCK_PICTURE`.
/// See [`Picture::as_flac_bytes`] for more information.
pub struct PictureInformation {
	/// The picture's width in pixels
	pub width: u32,
	/// The picture's height in pixels
	pub height: u32,
	/// The picture's color depth in bits per pixel
	pub color_depth: u32,
	/// The number of colors used
	pub num_colors: u32,
}

#[cfg(feature = "vorbis_comments")]
impl PictureInformation {
	/// Attempt to extract [`PictureInformation`] from a [`Picture`]
	///
	/// NOTE: Since FLAC only supports PNG and JPEG, this function is
	/// no different.
	///
	/// # Errors
	///
	/// * `picture.data` is less than 8 bytes in length
	/// * `picture.data` contains a format that isn't PNG or JPEG
	/// * See [`PictureInformation::from_png`] and [`PictureInformation::from_jpeg`]
	pub fn from_picture(picture: &Picture) -> Result<Self> {
		let reader = &mut &*picture.data;

		if reader.len() < 8 {
			return Err(LoftyError::NotAPicture);
		}

		match reader[..4] {
			[0x89, b'P', b'N', b'G'] => Ok(Self::from_png(reader).unwrap_or_default()),
			[0xFF, 0xD8, 0xFF, ..] => Ok(Self::from_jpeg(reader).unwrap_or_default()),
			_ => Err(LoftyError::UnsupportedPicture),
		}
	}

	/// Attempt to extract [`PictureInformation`] from a PNG
	///
	/// # Errors
	///
	/// * `reader` does not start with a PNG signature
	/// * `reader` is not a valid PNG
	pub fn from_png(mut data: &[u8]) -> Result<Self> {
		let reader = &mut data;

		let mut sig = [0; 8];
		reader.read_exact(&mut sig)?;

		if sig != [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
			return Err(LoftyError::NotAPicture);
		}

		let mut ihdr = [0; 8];
		reader.read_exact(&mut ihdr)?;

		// Verify the signature is immediately followed by the IHDR chunk
		if !ihdr.ends_with(&[0x49, 0x48, 0x44, 0x52]) {
			return Err(LoftyError::NotAPicture);
		}

		let width = reader.read_u32::<BigEndian>()?;
		let height = reader.read_u32::<BigEndian>()?;
		let mut color_depth = u32::from(reader.read_u8()?);
		let color_type = reader.read_u8()?;

		match color_type {
			2 => color_depth *= 3,
			4 | 6 => color_depth *= 4,
			_ => {}
		}

		// The color type 3 (indexed-color) means there should be
		// a "PLTE" chunk, whose data can be used in the `num_colors`
		// field. It isn't really applicable to other color types.
		if color_type != 3 {
			return Ok(Self {
				width,
				height,
				color_depth,
				num_colors: 0,
			});
		}

		let mut reader = Cursor::new(reader);

		// Skip 7 bytes
		// Compression method (1)
		// Filter method (1)
		// Interlace method (1)
		// CRC (4)
		reader.seek(SeekFrom::Current(7))?;

		let mut num_colors = 0;
		let mut chunk_type = [0; 4];

		while let (Ok(size), Ok(())) = (
			reader.read_u32::<BigEndian>(),
			reader.read_exact(&mut chunk_type),
		) {
			if &chunk_type == b"PLTE" {
				// The PLTE chunk contains 1-256 3-byte entries
				num_colors = size / 3;
				break;
			}

			// Skip the chunk's data (size) and CRC (4 bytes)
			reader.seek(SeekFrom::Current(i64::from(size + 4)))?;
		}

		Ok(Self {
			width,
			height,
			color_depth,
			num_colors,
		})
	}

	/// Attempt to extract [`PictureInformation`] from a JPEG
	///
	/// # Errors
	///
	/// * `reader` is not a JPEG image
	/// * `reader` does not contain a `SOFn` frame
	pub fn from_jpeg(mut data: &[u8]) -> Result<Self> {
		let reader = &mut data;

		let mut frame_marker = [0; 4];
		reader.read_exact(&mut frame_marker)?;

		if !matches!(frame_marker, [0xFF, 0xD8, 0xFF, ..]) {
			return Err(LoftyError::NotAPicture);
		}

		let mut section_len = reader.read_u16::<BigEndian>()?;

		let mut reader = Cursor::new(reader);

		// The length contains itself
		reader.seek(SeekFrom::Current(i64::from(section_len - 2)))?;

		while let Ok(0xFF) = reader.read_u8() {
			let marker = reader.read_u8()?;
			section_len = reader.read_u16::<BigEndian>()?;

			// This marks the SOS (Start of Scan), which is
			// the end of the header
			if marker == 0xDA {
				break;
			}

			// We are looking for a frame with a "SOFn" marker,
			// with `n` either being 0 or 2. Since there isn't a
			// header like PNG, we actually need to search for this
			// frame
			if marker == 0xC0 || marker == 0xC2 {
				let precision = reader.read_u8()?;
				let height = u32::from(reader.read_u16::<BigEndian>()?);
				let width = u32::from(reader.read_u16::<BigEndian>()?);
				let components = reader.read_u8()?;

				return Ok(Self {
					width,
					height,
					color_depth: u32::from(precision * components),
					num_colors: 0,
				});
			}

			reader.seek(SeekFrom::Current(i64::from(section_len - 2)))?;
		}

		Err(LoftyError::NotAPicture)
	}
}

/// Represents a picture.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Picture {
	/// The picture type according to ID3v2 APIC
	pub(crate) pic_type: PictureType,
	/// The picture's mimetype
	pub(crate) mime_type: MimeType,
	/// The picture's description
	pub(crate) description: Option<Cow<'static, str>>,
	/// The binary data of the picture
	pub(crate) data: Cow<'static, [u8]>,
}

impl Picture {
	/// Create a [`Picture`] from a reader
	///
	/// NOTES:
	///
	/// * This is **not** for reading format-specific
	/// pictures, it is for reading picture data only,
	/// from a [`File`](std::fs::File) for example.
	/// * `pic_type` will always be [`PictureType::Other`],
	/// be sure to change it accordingly if writing.
	///
	/// # Errors
	///
	/// * `reader` contains less than 8 bytes
	/// * `reader` does not contain a supported format.
	/// See [`MimeType`] for valid formats
	pub fn from_reader<R>(reader: &mut R) -> Result<Self>
	where
		R: Read,
	{
		let mut data = Vec::new();
		reader.read_to_end(&mut data)?;

		let pic_type = PictureType::Other;
		let description = None;

		let mime_type = match data.get(..8) {
			Some([0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) => MimeType::Png,
			Some([0xFF, 0xD8, ..]) => MimeType::Jpeg,
			Some([0x47, 0x49, 0x46, 0x38, 0x37 | 0x39, 0x61, ..]) => MimeType::Gif,
			Some([b'B', b'M', ..]) => MimeType::Bmp,
			Some([0x49, 0x49, 0x2A, 0x00, ..] | [0x4D, 0x4D, 0x00, 0x2A, ..]) => MimeType::Tiff,
			_ => return Err(LoftyError::NotAPicture),
		};

		Ok(Self {
			pic_type,
			mime_type,
			description,
			data: data.into(),
		})
	}

	/// Create a new `Picture`
	///
	/// NOTE: This will **not** verify `data`'s signature.
	/// This should only be used if all data has been verified
	/// beforehand.
	pub fn new_unchecked(
		pic_type: PictureType,
		mime_type: MimeType,
		description: Option<String>,
		data: Vec<u8>,
	) -> Self {
		Self {
			pic_type,
			mime_type,
			description: description.map(Cow::from),
			data: Cow::from(data),
		}
	}

	/// Returns the [`PictureType`]
	pub fn pic_type(&self) -> PictureType {
		self.pic_type
	}

	/// Sets the [`PictureType`]
	pub fn set_pic_type(&mut self, pic_type: PictureType) {
		self.pic_type = pic_type
	}

	/// Returns the [`MimeType`]
	///
	/// The `mime_type` is determined from the `data`, and
	/// is immutable.
	pub fn mime_type(&self) -> &MimeType {
		&self.mime_type
	}

	/// Returns the description
	pub fn description(&self) -> Option<&str> {
		self.description.as_deref()
	}

	/// Sets the description
	pub fn set_description(&mut self, description: Option<String>) {
		self.description = description.map(Cow::from);
	}

	/// Returns the picture data
	pub fn data(&self) -> &[u8] {
		&self.data
	}

	#[cfg(feature = "id3v2")]
	/// Convert a [`Picture`] to a ID3v2 A/PIC byte Vec
	///
	/// NOTE: This does not include the frame header
	///
	/// # Errors
	///
	/// * Too much data was provided
	///
	/// ID3v2.2:
	///
	/// * The mimetype is not [`MimeType::Png`] or [`MimeType::Jpeg`]
	pub fn as_apic_bytes(
		&self,
		version: Id3v2Version,
		text_encoding: TextEncoding,
	) -> Result<Vec<u8>> {
		let mut data = vec![text_encoding as u8];

		let max_size = if version == Id3v2Version::V2 {
			// ID3v2.2 PIC is pretty limited with formats
			let format = match self.mime_type {
				MimeType::Png => "PNG",
				MimeType::Jpeg => "JPG",
				_ => return Err(LoftyError::BadPictureFormat(self.mime_type.to_string())),
			};

			data.write_all(format.as_bytes())?;

			// ID3v2.2 uses a 24-bit number for sizes
			0xFFFF_FF16_u64
		} else {
			data.write_all(self.mime_type.as_str().as_bytes())?;
			data.write_u8(0)?;

			u64::from(u32::MAX)
		};

		data.write_u8(self.pic_type.as_u8())?;

		match &self.description {
			Some(description) => data.write_all(
				&*crate::id3::v2::util::text_utils::encode_text(description, text_encoding, true),
			)?,
			None => data.write_u8(0)?,
		}

		data.write_all(&*self.data)?;

		let size = data.len();

		if size as u64 > max_size {
			return Err(LoftyError::TooMuchData);
		}

		Ok(data)
	}

	#[cfg(feature = "id3v2")]
	/// Get a [`Picture`] and [`TextEncoding`] from ID3v2 A/PIC bytes:
	///
	/// NOTE: This expects *only* the frame content
	///
	/// # Errors
	///
	/// * There isn't enough data present
	/// * The data isn't a picture
	///
	/// ID3v2.2:
	///
	/// * The format is not "PNG" or "JPG"
	pub fn from_apic_bytes(bytes: &[u8], version: Id3v2Version) -> Result<(Self, TextEncoding)> {
		let mut cursor = Cursor::new(bytes);

		let encoding = match TextEncoding::from_u8(cursor.read_u8()?) {
			Some(encoding) => encoding,
			None => return Err(LoftyError::NotAPicture),
		};

		let mime_type = if version == Id3v2Version::V2 {
			let mut format = [0; 3];
			cursor.read_exact(&mut format)?;

			match format {
				[b'P', b'N', b'G'] => MimeType::Png,
				[b'J', b'P', b'G'] => MimeType::Jpeg,
				_ => {
					return Err(LoftyError::BadPictureFormat(
						String::from_utf8_lossy(&format).to_string(),
					))
				}
			}
		} else {
			(crate::id3::v2::util::text_utils::decode_text(&mut cursor, TextEncoding::UTF8, true)?)
				.map_or(MimeType::None, |mime_type| MimeType::from_str(&*mime_type))
		};

		let picture_type = PictureType::from_u8(cursor.read_u8()?);

		let description =
			crate::id3::v2::util::text_utils::decode_text(&mut cursor, encoding, true)?
				.map(Cow::from);

		let mut data = Vec::new();
		cursor.read_to_end(&mut data)?;

		Ok((
			Picture {
				pic_type: picture_type,
				mime_type,
				description,
				data: Cow::from(data),
			},
			encoding,
		))
	}

	#[cfg(feature = "vorbis_comments")]
	/// Convert a [`Picture`] to a base64 encoded FLAC `METADATA_BLOCK_PICTURE` String
	///
	/// NOTES:
	///
	/// * This does not include a key (Vorbis comments) or METADATA_BLOCK_HEADER (FLAC blocks)
	/// * FLAC blocks have different size requirements than OGG Vorbis/Opus, size is not checked here
	pub fn as_flac_bytes(&self, picture_information: PictureInformation) -> String {
		let mut data = Vec::<u8>::new();

		let picture_type = u32::from(self.pic_type.as_u8()).to_be_bytes();

		let mime_str = self.mime_type.to_string();
		let mime_len = mime_str.len() as u32;

		data.extend(picture_type.iter());
		data.extend(mime_len.to_be_bytes().iter());
		data.extend(mime_str.as_bytes().iter());

		if let Some(desc) = self.description.clone() {
			let desc_str = desc.to_string();
			let desc_len = desc_str.len() as u32;

			data.extend(desc_len.to_be_bytes().iter());
			data.extend(desc_str.as_bytes().iter());
		}

		data.extend(picture_information.width.to_be_bytes().iter());
		data.extend(picture_information.height.to_be_bytes().iter());
		data.extend(picture_information.color_depth.to_be_bytes().iter());
		data.extend(picture_information.num_colors.to_be_bytes().iter());

		let pic_data = &self.data;
		let pic_data_len = pic_data.len() as u32;

		data.extend(pic_data_len.to_be_bytes().iter());
		data.extend(pic_data.iter());

		base64::encode(data)
	}

	#[cfg(feature = "vorbis_comments")]
	/// Get a [`Picture`] from FLAC `METADATA_BLOCK_PICTURE` bytes (can be base64 encoded):
	///
	/// NOTE: This expects *only* the comment's value
	///
	/// # Errors
	///
	/// This function will return [`NotAPicture`][LoftyError::NotAPicture] if
	/// at any point it's unable to parse the data
	pub fn from_flac_bytes(bytes: &[u8]) -> Result<(Self, PictureInformation)> {
		let data = base64::decode(bytes).unwrap_or_else(|_| bytes.to_vec());

		let mut cursor = Cursor::new(data);

		if let Ok(bytes) = cursor.read_u32::<BigEndian>() {
			let picture_type = PictureType::from_u8(bytes as u8);

			if let Ok(mime_len) = cursor.read_u32::<BigEndian>() {
				let mut buf = vec![0; mime_len as usize];
				cursor.read_exact(&mut buf)?;

				if let Ok(mime_type_str) = String::from_utf8(buf) {
					let mime_type = MimeType::from_str(&*mime_type_str);

					let mut description = None;

					if let Ok(desc_len) = cursor.read_u32::<BigEndian>() {
						if cursor.get_ref().len() >= (cursor.position() as u32 + desc_len) as usize
						{
							let mut buf = vec![0; desc_len as usize];
							cursor.read_exact(&mut buf)?;

							if let Ok(desc) = String::from_utf8(buf) {
								description = Some(Cow::from(desc));
							}
						} else {
							cursor.set_position(cursor.position() - 4)
						}
					}

					if let (Ok(width), Ok(height), Ok(color_depth), Ok(num_colors)) = (
						cursor.read_u32::<BigEndian>(),
						cursor.read_u32::<BigEndian>(),
						cursor.read_u32::<BigEndian>(),
						cursor.read_u32::<BigEndian>(),
					) {
						if let Ok(data_len) = cursor.read_u32::<BigEndian>() {
							let mut binary = vec![0; data_len as usize];

							if let Ok(()) = cursor.read_exact(&mut binary) {
								return Ok((
									Self {
										pic_type: picture_type,
										mime_type,
										description,
										data: Cow::from(binary.clone()),
									},
									PictureInformation {
										width,
										height,
										color_depth,
										num_colors,
									},
								));
							}
						}
					}
				}
			}
		}

		Err(LoftyError::NotAPicture)
	}

	#[cfg(feature = "ape")]
	/// Convert a [`Picture`] to an APE Cover Art byte vec:
	///
	/// NOTE: This is only the picture data and description, a
	/// key and terminating null byte will not be prepended.
	/// To map a [`PictureType`] to an APE key see [`PictureType::as_ape_key`]
	pub fn as_ape_bytes(&self) -> Vec<u8> {
		let mut data: Vec<u8> = Vec::new();

		if let Some(desc) = &self.description {
			data.extend(desc.as_bytes().iter());
		}

		data.extend([0].iter());
		data.extend(self.data.iter());

		data
	}

	#[cfg(feature = "ape")]
	/// Get a [`Picture`] from an APEv2 binary item:
	///
	/// NOTE: This function expects `bytes` to contain *only* the APE item data
	///
	/// # Errors
	///
	/// This function will return [`NotAPicture`](LoftyError::NotAPicture)
	/// if at any point it's unable to parse the data
	pub fn from_ape_bytes(key: &str, bytes: &[u8]) -> Result<Self> {
		if !bytes.is_empty() {
			let pic_type = PictureType::from_ape_key(key);

			let mut cursor = Cursor::new(bytes);

			let description = {
				let mut text = String::new();

				while let Ok(ch) = cursor.read_u8() {
					if ch != b'\0' {
						text.push(char::from(ch));
						continue;
					}

					break;
				}

				(!text.is_empty()).then(|| Cow::from(text))
			};

			let mime_type = {
				let mut identifier = [0; 4];
				cursor.read_exact(&mut identifier)?;

				cursor.seek(SeekFrom::Current(-4))?;

				match identifier {
					[0x89, b'P', b'N', b'G'] => MimeType::Png,
					[0xFF, 0xD8, ..] => MimeType::Jpeg,
					[b'G', b'I', b'F', ..] => MimeType::Gif,
					[b'B', b'M', ..] => MimeType::Bmp,
					[b'I', b'I', ..] => MimeType::Tiff,
					_ => return Err(LoftyError::NotAPicture),
				}
			};

			let pos = cursor.position() as usize;
			let data = Cow::from(cursor.into_inner()[pos..].to_vec());

			return Ok(Picture {
				pic_type,
				mime_type,
				description,
				data,
			});
		}

		Err(LoftyError::NotAPicture)
	}
}
