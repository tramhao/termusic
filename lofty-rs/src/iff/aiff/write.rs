use crate::error::{LoftyError, Result};
#[cfg(feature = "id3v2")]
use crate::id3::v2::tag::Id3v2TagRef;
#[cfg(feature = "aiff_text_chunks")]
use crate::iff::aiff::tag::AiffTextChunksRef;
#[allow(unused_imports)]
use crate::types::tag::{Tag, TagType};

use std::fs::File;

#[allow(unused_variables)]
pub(crate) fn write_to(data: &mut File, tag: &Tag) -> Result<()> {
	match tag.tag_type() {
		#[cfg(feature = "aiff_text_chunks")]
		TagType::AiffText => Into::<AiffTextChunksRef>::into(tag).write_to(data),
		#[cfg(feature = "id3v2")]
		TagType::Id3v2 => Into::<Id3v2TagRef>::into(tag).write_to(data),
		_ => Err(LoftyError::UnsupportedTag),
	}
}
