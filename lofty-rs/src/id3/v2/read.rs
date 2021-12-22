use super::frame::Frame;
use super::tag::Id3v2Tag;
use super::Id3v2Header;
use crate::error::Result;

use std::io::Read;

#[allow(clippy::similar_names)]
pub(crate) fn parse_id3v2<R>(bytes: &mut R, header: Id3v2Header) -> Result<Id3v2Tag>
where
	R: Read,
{
	let mut tag_bytes = vec![0; header.size as usize];
	bytes.read_exact(&mut tag_bytes)?;

	let mut tag = Id3v2Tag::default();
	tag.original_version = header.version;
	tag.set_flags(header.flags);

	let reader = &mut &*tag_bytes;

	loop {
		match Frame::read(reader, header.version)? {
			None => break,
			Some(f) => drop(tag.insert(f)),
		}
	}

	Ok(tag)
}
