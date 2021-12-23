use crate::error::{LoftyError, Result};
use crate::id3::v2::frame::{FrameFlags, FrameRef, FrameValueRef};
use crate::id3::v2::synch_u32;
use crate::id3::v2::Id3v2Version;

use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

pub(in crate::id3::v2) fn create_items<'a, W>(
	writer: &mut W,
	frames: &mut dyn Iterator<Item = FrameRef<'a>>,
) -> Result<()>
where
	W: Write,
{
	for frame in frames {
		let value = match frame.value {
			FrameValueRef::Comment(content) | FrameValueRef::UnSyncText(content) => {
				content.as_bytes()?
			},
			FrameValueRef::Text { value, encoding } => {
				let mut v = vec![encoding as u8];

				v.extend_from_slice(value.as_bytes());
				v
			},
			FrameValueRef::UserText(content) | FrameValueRef::UserURL(content) => {
				content.as_bytes()
			},
			FrameValueRef::URL(link) => link.as_bytes().to_vec(),
			FrameValueRef::Picture { encoding, picture } => {
				picture.as_apic_bytes(Id3v2Version::V4, encoding)?
			},
			FrameValueRef::Binary(binary) => binary.to_vec(),
		};

		write_frame(writer, frame.id, frame.flags, &value)?;
	}

	Ok(())
}

fn write_frame<W>(writer: &mut W, name: &str, flags: FrameFlags, value: &[u8]) -> Result<()>
where
	W: Write,
{
	if flags.encryption.0 {
		write_encrypted(writer, name, value, flags)?;
		return Ok(());
	}

	let len = value.len() as u32;
	let is_grouping_identity = flags.grouping_identity.0;

	write_frame_header(
		writer,
		name,
		if is_grouping_identity { len + 1 } else { len },
		flags,
	)?;

	if is_grouping_identity {
		writer.write_u8(flags.grouping_identity.1)?;
	}

	writer.write_all(value)?;

	Ok(())
}

fn write_encrypted<W>(writer: &mut W, name: &str, value: &[u8], flags: FrameFlags) -> Result<()>
where
	W: Write,
{
	let method_symbol = flags.encryption.1;
	let data_length_indicator = flags.data_length_indicator;

	if method_symbol > 0x80 {
		return Err(LoftyError::Id3v2(
			"Attempted to write an encrypted frame with an invalid method symbol (> 0x80)",
		));
	}

	if data_length_indicator.0 && data_length_indicator.1 > 0 {
		write_frame_header(writer, name, (value.len() + 1) as u32, flags)?;
		writer.write_u32::<BigEndian>(synch_u32(data_length_indicator.1)?)?;
		writer.write_u8(method_symbol)?;
		writer.write_all(value)?;

		return Ok(());
	}

	Err(LoftyError::Id3v2(
		"Attempted to write an encrypted frame without a data length indicator",
	))
}

fn write_frame_header<W>(writer: &mut W, name: &str, len: u32, flags: FrameFlags) -> Result<()>
where
	W: Write,
{
	writer.write_all(name.as_bytes())?;
	writer.write_u32::<BigEndian>(synch_u32(len)?)?;
	writer.write_u16::<BigEndian>(get_flags(flags))?;

	Ok(())
}

fn get_flags(tag_flags: FrameFlags) -> u16 {
	let mut flags = 0;

	if tag_flags == FrameFlags::default() {
		return flags;
	}

	if tag_flags.tag_alter_preservation {
		flags |= 0x4000
	}

	if tag_flags.file_alter_preservation {
		flags |= 0x2000
	}

	if tag_flags.read_only {
		flags |= 0x1000
	}

	if tag_flags.grouping_identity.0 {
		flags |= 0x0040
	}

	if tag_flags.compression {
		flags |= 0x0008
	}

	if tag_flags.encryption.0 {
		flags |= 0x0004
	}

	if tag_flags.unsynchronisation {
		flags |= 0x0002
	}

	if tag_flags.data_length_indicator.0 {
		flags |= 0x0001
	}

	flags
}
