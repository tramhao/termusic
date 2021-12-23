use super::atom_info::{AtomIdent, AtomInfo};
use super::read::nested_atom;
use super::read::skip_unneeded;
use super::trak::Trak;
use crate::error::{LoftyError, Result};
use crate::types::properties::FileProperties;

use std::io::{Cursor, Read, Seek, SeekFrom};
use std::time::Duration;

use byteorder::{BigEndian, ReadBytesExt};

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
/// An MP4 file's audio codec
pub enum Mp4Codec {
	AAC,
	ALAC,
	Unknown(String),
}

impl Default for Mp4Codec {
	fn default() -> Self {
		Self::Unknown(String::new())
	}
}

#[derive(Debug, Clone, PartialEq, Default)]
/// An MP4 file's audio properties
pub struct Mp4Properties {
	codec: Mp4Codec,
	duration: Duration,
	overall_bitrate: u32,
	audio_bitrate: u32,
	sample_rate: u32,
	channels: u8,
}

impl From<Mp4Properties> for FileProperties {
	fn from(input: Mp4Properties) -> Self {
		Self {
			duration: input.duration,
			overall_bitrate: Some(input.overall_bitrate),
			audio_bitrate: Some(input.audio_bitrate),
			sample_rate: Some(input.sample_rate),
			channels: Some(input.channels),
		}
	}
}

impl Mp4Properties {
	/// Creates a new [`Mp4Properties`]
	pub const fn new(
		codec: Mp4Codec,
		duration: Duration,
		overall_bitrate: u32,
		audio_bitrate: u32,
		sample_rate: u32,
		channels: u8,
	) -> Self {
		Self {
			codec,
			duration,
			overall_bitrate,
			audio_bitrate,
			sample_rate,
			channels,
		}
	}

	/// Duration
	pub fn duration(&self) -> Duration {
		self.duration
	}

	/// Overall bitrate (kbps)
	pub fn overall_bitrate(&self) -> u32 {
		self.overall_bitrate
	}

	/// Audio bitrate (kbps)
	pub fn audio_bitrate(&self) -> u32 {
		self.audio_bitrate
	}

	/// Sample rate (Hz)
	pub fn sample_rate(&self) -> u32 {
		self.sample_rate
	}

	/// Channel count
	pub fn channels(&self) -> u8 {
		self.channels
	}

	/// Audio codec
	pub fn codec(&self) -> &Mp4Codec {
		&self.codec
	}
}

pub(crate) fn read_properties<R>(
	data: &mut R,
	traks: &[Trak],
	file_length: u64,
) -> Result<Mp4Properties>
where
	R: Read + Seek,
{
	// We need the mdhd and minf atoms from the audio track
	let mut audio_track = false;
	let mut mdhd = None;
	let mut minf = None;

	// We have to search through the traks with a mdia atom to find the audio track
	for mdia in traks.iter().filter_map(|trak| trak.mdia.as_ref()) {
		if audio_track {
			break;
		}

		data.seek(SeekFrom::Start(mdia.start + 8))?;

		let mut read = 8;

		while read < mdia.len {
			let atom = AtomInfo::read(data)?;

			if let AtomIdent::Fourcc(fourcc) = atom.ident {
				match &fourcc {
					b"mdhd" => {
						skip_unneeded(data, atom.extended, atom.len)?;
						mdhd = Some(atom)
					},
					b"hdlr" => {
						// The hdlr atom is followed by 8 zeros
						data.seek(SeekFrom::Current(8))?;

						let mut handler_type = [0; 4];
						data.read_exact(&mut handler_type)?;

						if &handler_type == b"soun" {
							audio_track = true
						}

						skip_unneeded(data, atom.extended, atom.len - 12)?;
					},
					b"minf" => minf = Some(atom),
					_ => {
						skip_unneeded(data, atom.extended, atom.len)?;
						read += atom.len;
					},
				}

				continue;
			}

			skip_unneeded(data, atom.extended, atom.len)?;
			read += atom.len;
		}
	}

	if !audio_track {
		return Err(LoftyError::Mp4("File contains no audio tracks"));
	}

	let duration = match mdhd {
		Some(mdhd) => {
			data.seek(SeekFrom::Start(mdhd.start + 8))?;

			let version = data.read_u8()?;
			let _flags = data.read_uint::<BigEndian>(3)?;

			let (timescale, duration) = if version == 1 {
				// We don't care about these two values
				let _creation_time = data.read_u64::<BigEndian>()?;
				let _modification_time = data.read_u64::<BigEndian>()?;

				let timescale = data.read_u32::<BigEndian>()?;
				let duration = data.read_u64::<BigEndian>()?;

				(timescale, duration)
			} else {
				let _creation_time = data.read_u32::<BigEndian>()?;
				let _modification_time = data.read_u32::<BigEndian>()?;

				let timescale = data.read_u32::<BigEndian>()?;
				let duration = data.read_u32::<BigEndian>()?;

				(timescale, u64::from(duration))
			};

			Duration::from_millis(duration * 1000 / u64::from(timescale))
		},
		None => return Err(LoftyError::BadAtom("Expected atom \"trak.mdia.mdhd\"")),
	};

	// We create the properties here, since it is possible the other information isn't available
	let mut properties = Mp4Properties {
		codec: Mp4Codec::Unknown(String::new()),
		duration,
		overall_bitrate: 0,
		audio_bitrate: 0,
		sample_rate: 0,
		channels: 0,
	};

	if let Some(minf) = minf {
		data.seek(SeekFrom::Start(minf.start + 8))?;

		if let Some(stbl) = nested_atom(data, minf.len, b"stbl")? {
			if let Some(stsd) = nested_atom(data, stbl.len, b"stsd")? {
				let mut stsd = vec![0; (stsd.len - 8) as usize];
				data.read_exact(&mut stsd)?;

				let mut stsd_reader = Cursor::new(&*stsd);

				// Skipping 8 bytes
				// Version (1)
				// Flags (3)
				// Number of entries (4)
				stsd_reader.seek(SeekFrom::Current(8))?;

				let atom = AtomInfo::read(&mut stsd_reader)?;

				if let AtomIdent::Fourcc(ref fourcc) = atom.ident {
					match fourcc {
						b"mp4a" => mp4a_properties(&mut stsd_reader, &mut properties, file_length)?,
						b"alac" => alac_properties(&mut stsd_reader, &mut properties)?,
						unknown => {
							if let Ok(codec) = std::str::from_utf8(unknown) {
								properties.codec = Mp4Codec::Unknown(codec.to_string())
							}
						},
					}
				}
			}
		}
	}

	Ok(properties)
}

fn mp4a_properties<R>(stsd: &mut R, properties: &mut Mp4Properties, file_length: u64) -> Result<()>
where
	R: Read + Seek,
{
	properties.codec = Mp4Codec::AAC;

	// Skipping 16 bytes
	// Reserved (6)
	// Data reference index (2)
	// Version (2)
	// Revision level (2)
	// Vendor (4)
	stsd.seek(SeekFrom::Current(16))?;

	properties.channels = stsd.read_u16::<BigEndian>()? as u8;

	// Skipping 4 bytes
	// Sample size (2)
	// Compression ID (2)
	stsd.seek(SeekFrom::Current(4))?;

	properties.sample_rate = stsd.read_u32::<BigEndian>()?;

	stsd.seek(SeekFrom::Current(2))?;

	// This information is often followed by an esds (elementary stream descriptor) atom containing the bitrate
	if let Ok(esds) = AtomInfo::read(stsd) {
		// There are 4 bytes we expect to be zeroed out
		// Version (1)
		// Flags (3)
		if esds.ident == AtomIdent::Fourcc(*b"esds") && stsd.read_u32::<BigEndian>()? == 0 {
			let mut descriptor = [0; 4];
			stsd.read_exact(&mut descriptor)?;

			// [0x03, 0x80, 0x80, 0x80] marks the start of the elementary stream descriptor.
			// 0x03 being the object descriptor
			if descriptor == [0x03, 0x80, 0x80, 0x80] {
				// Skipping 4 bytes
				// Descriptor length (1)
				// Elementary stream ID (2)
				// Flags (1)
				let _info = stsd.read_u32::<BigEndian>()?;

				// There is another descriptor embedded in the previous one
				let mut specific_config = [0; 4];
				stsd.read_exact(&mut specific_config)?;

				// [0x04, 0x80, 0x80, 0x80] marks the start of the descriptor configuration
				if specific_config == [0x04, 0x80, 0x80, 0x80] {
					// Skipping 10 bytes
					// Descriptor length (1)
					// MPEG4 Audio (1)
					// Stream type (1)
					// Buffer size (3)
					// Max bitrate (4)
					let mut info = [0; 10];
					stsd.read_exact(&mut info)?;

					let average_bitrate = stsd.read_u32::<BigEndian>()?;

					let overall_bitrate =
						u128::from(file_length * 8) / properties.duration.as_millis();

					if average_bitrate > 0 {
						properties.overall_bitrate = overall_bitrate as u32;
						properties.audio_bitrate = average_bitrate / 1000
					}
				}
			}
		}
	}

	Ok(())
}

fn alac_properties<R>(data: &mut R, properties: &mut Mp4Properties) -> Result<()>
where
	R: Read + Seek,
{
	// With ALAC, we can expect the length to be exactly 88 (80 here since we removed the size and identifier)
	if data.seek(SeekFrom::End(0))? != 80 {
		return Ok(());
	}

	// Unlike the mp4a atom, we cannot read the data that immediately follows it
	// For ALAC, we have to skip the first "alac" atom entirely, and read the one that
	// immediately follows it.
	//
	// We are skipping over 44 bytes total
	// stsd information/alac atom header (16, see `read_properties`)
	// First alac atom's content (28)
	data.seek(SeekFrom::Start(44))?;

	if let Ok(alac) = AtomInfo::read(data) {
		if alac.ident == AtomIdent::Fourcc(*b"alac") {
			properties.codec = Mp4Codec::ALAC;

			// Skipping 13 bytes
			// Version (4)
			// Samples per frame (4)
			// Compatible version (1)
			// Sample size (1)
			// Rice history mult (1)
			// Rice initial history (1)
			// Rice parameter limit (1)
			data.seek(SeekFrom::Current(13))?;

			properties.channels = data.read_u8()?;

			// Skipping 6 bytes
			// Max run (2)
			// Max frame size (4)
			data.seek(SeekFrom::Current(6))?;

			properties.audio_bitrate = data.read_u32::<BigEndian>()?;
			properties.sample_rate = data.read_u32::<BigEndian>()?;
		}
	}

	Ok(())
}
