use super::properties::WavProperties;
#[cfg(feature = "riff_info_list")]
use super::tag::RiffInfoList;
use super::WavFile;
use crate::error::{LoftyError, Result};
#[cfg(feature = "id3v2")]
use crate::id3::v2::tag::Id3v2Tag;
use crate::iff::chunk::Chunks;

use std::io::{Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

pub(in crate::iff) fn verify_wav<T>(data: &mut T) -> Result<()>
where
	T: Read + Seek,
{
	let mut id = [0; 12];
	data.read_exact(&mut id)?;

	if &id[..4] != b"RIFF" {
		return Err(LoftyError::Wav("WAV file doesn't contain a RIFF chunk"));
	}

	if &id[8..] != b"WAVE" {
		return Err(LoftyError::Wav("Found RIFF file, format is not WAVE"));
	}

	Ok(())
}

pub(crate) fn read_from<R>(data: &mut R, read_properties: bool) -> Result<WavFile>
where
	R: Read + Seek,
{
	verify_wav(data)?;

	let mut stream_len = 0_u32;
	let mut total_samples = 0_u32;
	let mut fmt = Vec::new();

	#[cfg(feature = "riff_info_list")]
	let mut riff_info = RiffInfoList::default();
	#[cfg(feature = "id3v2")]
	let mut id3v2_tag: Option<Id3v2Tag> = None;

	let mut chunks = Chunks::<LittleEndian>::new();

	while chunks.next(data).is_ok() {
		match &chunks.fourcc {
			b"fmt " if read_properties => {
				if fmt.is_empty() {
					fmt = chunks.content(data)?;
				} else {
					data.seek(SeekFrom::Current(i64::from(chunks.size)))?;
				}
			}
			b"fact" if read_properties => {
				if total_samples == 0 {
					total_samples = data.read_u32::<LittleEndian>()?;
				} else {
					data.seek(SeekFrom::Current(4))?;
				}
			}
			b"data" if read_properties => {
				if stream_len == 0 {
					stream_len += chunks.size
				}

				data.seek(SeekFrom::Current(i64::from(chunks.size)))?;
			}
			b"LIST" => {
				let mut list_type = [0; 4];
				data.read_exact(&mut list_type)?;

				#[cfg(feature = "riff_info_list")]
				if &list_type == b"INFO" {
					let end = data.seek(SeekFrom::Current(0))? + u64::from(chunks.size - 4);
					super::tag::read::parse_riff_info(data, end, &mut riff_info)?;
				}

				#[cfg(not(feature = "riff_info_list"))]
				{
					data.seek(SeekFrom::Current(i64::from(chunks.size)))?;
				}
			}
			#[cfg(feature = "id3v2")]
			b"ID3 " | b"id3 " => id3v2_tag = Some(chunks.id3_chunk(data)?),
			#[cfg(not(feature = "id3v2"))]
			b"ID3 " | b"id3 " => chunks.id3_chunk(data)?,
			_ => {
				data.seek(SeekFrom::Current(i64::from(chunks.size)))?;
			}
		}

		chunks.correct_position(data)?;
	}

	let properties = if read_properties {
		if fmt.len() < 16 {
			return Err(LoftyError::Wav(
				"File does not contain a valid \"fmt \" chunk",
			));
		}

		if stream_len == 0 {
			return Err(LoftyError::Wav("File does not contain a \"data\" chunk"));
		}

		let file_length = data.seek(SeekFrom::Current(0))?;

		super::properties::read_properties(&mut &*fmt, total_samples, stream_len, file_length)?
	} else {
		WavProperties::default()
	};

	Ok(WavFile {
		properties,
		#[cfg(feature = "riff_info_list")]
		riff_info: (!riff_info.items.is_empty()).then(|| riff_info),
		#[cfg(feature = "id3v2")]
		id3v2_tag,
	})
}
