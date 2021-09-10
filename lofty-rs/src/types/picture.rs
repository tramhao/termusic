use crate::{LoftyError, Result};

use std::borrow::Cow;
use std::convert::TryFrom;
#[cfg(any(
	feature = "format-id3",
	feature = "format-opus",
	feature = "format-vorbis",
	feature = "format-flac",
	feature = "format-ape",
))]
use std::io::{Cursor, Read};

#[cfg(any(
	feature = "format-id3",
	feature = "format-opus",
	feature = "format-vorbis",
	feature = "format-flac",
))]
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Seek, SeekFrom};

#[cfg(feature = "format-ape")]
pub const APE_PICTYPES: [&str; 21] = [
	"Other",
	"Png Icon",
	"Icon",
	"Front",
	"Back",
	"Leaflet",
	"Media",
	"Lead Artist",
	"Artist",
	"Conductor",
	"Band",
	"Composer",
	"Lyricist",
	"Recording Location",
	"During Recording",
	"During Performance",
	"Video Capture",
	"Fish",
	"Illustration",
	"Band Logotype",
	"Publisher Logotype",
];

/// Mime types for covers.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
}

impl MimeType {
	#[cfg(feature = "format-ape")]
	/// Converts the `MimeType` to an ape str
	pub fn as_ape(self) -> &'static [u8; 4] {
		match self {
			MimeType::Png => b"PNG\0",
			MimeType::Jpeg => b"JPEG",
			MimeType::Tiff => b"TIFF",
			MimeType::Bmp => b"BMP\0",
			MimeType::Gif => b"GIF\0",
		}
	}
}

impl TryFrom<&str> for MimeType {
	type Error = LoftyError;

	fn try_from(inp: &str) -> Result<Self> {
		Ok(match inp {
			"image/jpeg" => MimeType::Jpeg,
			"image/png" => MimeType::Png,
			"image/tiff" => MimeType::Tiff,
			"image/bmp" => MimeType::Bmp,
			"image/gif" => MimeType::Gif,
			_ => return Err(LoftyError::UnsupportedMimeType(inp.to_string())),
		})
	}
}

impl From<MimeType> for &'static str {
	fn from(mt: MimeType) -> Self {
		match mt {
			MimeType::Jpeg => "image/jpeg",
			MimeType::Png => "image/png",
			MimeType::Tiff => "image/tiff",
			MimeType::Bmp => "image/bmp",
			MimeType::Gif => "image/gif",
		}
	}
}

impl From<MimeType> for String {
	fn from(mt: MimeType) -> Self {
		<MimeType as Into<&'static str>>::into(mt).to_owned()
	}
}

pub trait PicType {
	#[cfg(any(
		feature = "format-id3",
		feature = "format-vorbis",
		feature = "format-opus",
		feature = "format-flac"
	))]
	fn as_u32(&self) -> u32;
	#[cfg(any(
		feature = "format-id3",
		feature = "format-vorbis",
		feature = "format-opus",
		feature = "format-flac"
	))]
	fn from_u32(bytes: u32) -> PictureType;
	#[cfg(feature = "format-ape")]
	fn as_ape_key(&self) -> &str;
	#[cfg(feature = "format-ape")]
	fn from_ape_key(key: &str) -> PictureType;
}

/// The picture type
#[cfg(not(feature = "format-id3"))]
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

/// Alias for PictureType
#[cfg(feature = "format-id3")]
pub type PictureType = id3::frame::PictureType;

impl PicType for PictureType {
	// ID3/OGG specific methods

	#[cfg(any(
		feature = "format-id3",
		feature = "format-vorbis",
		feature = "format-opus",
		feature = "format-flac"
	))]
	fn as_u32(&self) -> u32 {
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
			Self::Undefined(i) => u32::from(i.to_owned()),
		}
	}

	#[cfg(any(
		feature = "format-id3",
		feature = "format-vorbis",
		feature = "format-opus",
		feature = "format-flac"
	))]
	fn from_u32(bytes: u32) -> Self {
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

	#[cfg(feature = "format-ape")]
	fn as_ape_key(&self) -> &str {
		match self {
			Self::Other => "Cover Art (Other)",
			Self::Icon => "Cover Art (Png Icon)",
			Self::OtherIcon => "Cover Art (Icon)",
			Self::CoverFront => "Cover Art (Front)",
			Self::CoverBack => "Cover Art (Back)",
			Self::Leaflet => "Cover Art (Leaflet)",
			Self::Media => "Cover Art (Media)",
			Self::LeadArtist => "Cover Art (Lead Artist)",
			Self::Artist => "Cover Art (Artist)",
			Self::Conductor => "Cover Art (Conductor)",
			Self::Band => "Cover Art (Band)",
			Self::Composer => "Cover Art (Composer)",
			Self::Lyricist => "Cover Art (Lyricist)",
			Self::RecordingLocation => "Cover Art (Recording Location)",
			Self::DuringRecording => "Cover Art (During Recording)",
			Self::DuringPerformance => "Cover Art (During Performance)",
			Self::ScreenCapture => "Cover Art (Video Capture)",
			Self::BrightFish => "Cover Art (Fish)",
			Self::Illustration => "Cover Art (Illustration)",
			Self::BandLogo => "Cover Art (Band Logotype)",
			Self::PublisherLogo => "Cover Art (Publisher Logotype)",
			Self::Undefined(_) => "",
		}
	}

	#[cfg(feature = "format-ape")]
	fn from_ape_key(key: &str) -> Self {
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

/// Represents a picture, with its data and mime type.
///
/// NOTE: The width, height, color_depth, and num_color fields can only be read from formats supporting FLAC pictures
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Picture {
	/// The picture type
	pub pic_type: PictureType,
	/// The picture's mimetype
	pub mime_type: MimeType,
	/// The picture's description
	pub description: Option<Cow<'static, str>>,
	/// The picture's width in pixels
	pub width: u32,
	/// The picture's height in pixels
	pub height: u32,
	/// The picture's color depth in bits per pixel
	pub color_depth: u32,
	/// The number of colors used
	pub num_colors: u32,
	/// The binary data of the picture
	pub data: Cow<'static, [u8]>,
}

impl Picture {
	/// Create a new `Picture`
	pub fn new(
		pic_type: PictureType,
		mime_type: MimeType,
		description: Option<String>,
		dimensions: (u32, u32),
		color_depth: u32,
		num_colors: u32,
		data: Vec<u8>,
	) -> Self {
		Self {
			pic_type,
			mime_type,
			description: description.map(Cow::from),
			width: dimensions.0,
			height: dimensions.1,
			color_depth,
			num_colors,
			data: Cow::from(data),
		}
	}

	#[cfg(any(
		feature = "format-opus",
		feature = "format-vorbis",
		feature = "format-flac"
	))]
	/// Convert the [`Picture`] back to an APIC byte vec:
	///
	/// * FLAC METADATA_BLOCK_PICTURE
	pub fn as_apic_bytes(&self) -> Vec<u8> {
		let mut data: Vec<u8> = Vec::new();

		let picture_type = self.pic_type.as_u32().to_be_bytes();

		let mime_str = String::from(self.mime_type);
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

		data.extend(self.width.to_be_bytes().iter());
		data.extend(self.height.to_be_bytes().iter());
		data.extend(self.color_depth.to_be_bytes().iter());
		data.extend(self.num_colors.to_be_bytes().iter());

		let pic_data = &self.data;
		let pic_data_len = pic_data.len() as u32;

		data.extend(pic_data_len.to_be_bytes().iter());
		data.extend(pic_data.iter());

		data
	}

	#[cfg(any(
		feature = "format-opus",
		feature = "format-vorbis",
		feature = "format-flac"
	))]
	/// Get a [`Picture`] from APIC bytes:
	///
	/// * FLAC METADATA_BLOCK_PICTURE
	///
	/// # Errors
	///
	/// This function will return [`NotAPicture`][LoftyError::NotAPicture] if at any point it's unable to parse the data
	pub fn from_apic_bytes(bytes: &[u8]) -> Result<Self> {
		let data = match base64::decode(bytes) {
			Ok(o) => o,
			Err(_) => bytes.to_vec(),
		};

		let mut cursor = Cursor::new(data);

		if let Ok(bytes) = cursor.read_u32::<BigEndian>() {
			let picture_type = PictureType::from_u32(bytes);

			if let Ok(mime_len) = cursor.read_u32::<BigEndian>() {
				let mut buf = vec![0; mime_len as usize];
				cursor.read_exact(&mut buf)?;

				if let Ok(mime_type_str) = String::from_utf8(buf) {
					if let Ok(mime_type) = MimeType::try_from(&*mime_type_str) {
						let mut description = None;

						if let Ok(desc_len) = cursor.read_u32::<BigEndian>() {
							if cursor.get_ref().len()
								>= (cursor.position() as u32 + desc_len) as usize
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
									return Ok(Self {
										pic_type: picture_type,
										mime_type,
										description,
										width,
										height,
										color_depth,
										num_colors,
										data: Cow::from(binary.clone()),
									});
								}
							}
						}
					}
				}
			}
		}

		Err(LoftyError::NotAPicture)
	}

	#[cfg(feature = "format-ape")]
	/// Convert the [`Picture`] back to an APEv2 byte vec:
	///
	/// * APEv2 Cover Art
	pub fn as_ape_bytes(&self) -> Vec<u8> {
		let mut data: Vec<u8> = Vec::new();

		if let Some(desc) = &self.description {
			data.extend(desc.as_bytes().iter());
		}

		data.extend([0].iter());
		data.extend(self.data.iter());

		data
	}

	#[cfg(feature = "format-ape")]
	/// Get a [`Picture`] from an APEv2 binary item:
	///
	/// * APEv2 Cover Art
	///
	/// NOTES:
	///
	/// * This function expects the key and its trailing null byte to have been removed
	/// * Since APE tags only store the binary data, the width, height, color_depth, and num_colors fields will be zero.
	///
	/// # Errors
	///
	/// This function will return [`NotAPicture`][LoftyError::NotAPicture] if at any point it's unable to parse the data
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
					_ if [0xFF, 0xD8] == identifier[..2] => MimeType::Jpeg,
					_ if b"GIF" == &identifier[..3] => MimeType::Gif,
					_ if b"BM" == &identifier[..2] => MimeType::Bmp,
					_ if b"II" == &identifier[..2] => MimeType::Tiff,
					_ => return Err(LoftyError::NotAPicture),
				}
			};

			let pos = cursor.position() as usize;
			let data = Cow::from(cursor.into_inner()[pos..].to_vec());

			return Ok(Picture {
				pic_type,
				mime_type,
				description,
				width: 0,
				height: 0,
				color_depth: 0,
				num_colors: 0,
				data,
			});
		}

		Err(LoftyError::NotAPicture)
	}
}
