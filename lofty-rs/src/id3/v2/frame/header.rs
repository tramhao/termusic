use super::FrameFlags;
use crate::error::{LoftyError, Result};
use crate::id3::v2::util::upgrade::{upgrade_v2, upgrade_v3};
use crate::id3::v2::FrameID;

use std::io::Read;

pub(crate) fn parse_v2_header<R>(reader: &mut R) -> Result<Option<(FrameID, u32, FrameFlags)>>
where
	R: Read,
{
	let mut frame_header = [0; 6];
	match reader.read_exact(&mut frame_header) {
		Ok(_) => {}
		Err(_) => return Ok(None),
	}

	// Assume we just started reading padding
	if frame_header[0] == 0 {
		return Ok(None);
	}

	let id_str = std::str::from_utf8(&frame_header[..3]).map_err(|_| LoftyError::BadFrameID)?;
	let id = upgrade_v2(id_str).unwrap_or(id_str);

	let frame_id = create_frame_id(id)?;

	let size = u32::from_be_bytes([0, frame_header[3], frame_header[4], frame_header[5]]);

	// V2 doesn't store flags
	Ok(Some((frame_id, size, FrameFlags::default())))
}

pub(crate) fn parse_header<R>(
	reader: &mut R,
	synchsafe: bool,
) -> Result<Option<(FrameID, u32, FrameFlags)>>
where
	R: Read,
{
	let mut frame_header = [0; 10];
	match reader.read_exact(&mut frame_header) {
		Ok(_) => {}
		Err(_) => return Ok(None),
	}

	// Assume we just started reading padding
	if frame_header[0] == 0 {
		return Ok(None);
	}

	let id_str = std::str::from_utf8(&frame_header[..4]).map_err(|_| LoftyError::BadFrameID)?;

	let (id, size) = if synchsafe {
		let size = crate::id3::v2::unsynch_u32(u32::from_be_bytes([
			frame_header[4],
			frame_header[5],
			frame_header[6],
			frame_header[7],
		]));

		(id_str, size)
	} else {
		let mapped = upgrade_v3(id_str).unwrap_or(id_str);

		let size = u32::from_be_bytes([
			frame_header[4],
			frame_header[5],
			frame_header[6],
			frame_header[7],
		]);

		(mapped, size)
	};

	let frame_id = create_frame_id(id)?;

	let flags = u16::from_be_bytes([frame_header[8], frame_header[9]]);
	let flags = parse_flags(flags, synchsafe);

	Ok(Some((frame_id, size, flags)))
}

fn create_frame_id(id: &str) -> Result<FrameID> {
	for c in id.chars() {
		if !('A'..='Z').contains(&c) && !('0'..='9').contains(&c) {
			return Err(LoftyError::Id3v2("Encountered a bad frame ID"));
		}
	}

	Ok(match id.len() {
		3 => FrameID::Outdated(id.to_string()),
		4 => FrameID::Valid(id.to_string()),
		_ => unreachable!(),
	})
}

pub(crate) fn parse_flags(flags: u16, v4: bool) -> FrameFlags {
	FrameFlags {
		tag_alter_preservation: if v4 {
			flags & 0x4000 == 0x4000
		} else {
			flags & 0x8000 == 0x8000
		},
		file_alter_preservation: if v4 {
			flags & 0x2000 == 0x2000
		} else {
			flags & 0x4000 == 0x4000
		},
		read_only: if v4 {
			flags & 0x1000 == 0x1000
		} else {
			flags & 0x2000 == 0x2000
		},
		grouping_identity: (
			if v4 {
				flags & 0x0040 == 0x0040
			} else {
				flags & 0x0020 == 0x0020
			},
			0,
		),
		compression: if v4 {
			flags & 0x0008 == 0x0008
		} else {
			flags & 0x0080 == 0x0080
		},
		encryption: if v4 {
			(flags & 0x0004 == 0x0004, 0)
		} else {
			(flags & 0x0040 == 0x0040, 0)
		},
		unsynchronisation: if v4 { flags & 0x0002 == 0x0002 } else { false },
		data_length_indicator: if v4 {
			(flags & 0x0001 == 0x0001, 0)
		} else {
			(false, 0)
		},
	}
}
