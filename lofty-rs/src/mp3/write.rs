#[cfg(feature = "ape")]
use crate::ape::tag::ape_tag::ApeTagRef;
use crate::error::{LoftyError, Result};
#[cfg(feature = "id3v1")]
use crate::id3::v1::tag::Id3v1TagRef;
#[cfg(feature = "id3v2")]
use crate::id3::v2::tag::Id3v2TagRef;
#[allow(unused_imports)]
use crate::types::tag::{Tag, TagType};

use std::fs::File;

#[allow(unused_variables)]
pub(crate) fn write_to(data: &mut File, tag: &Tag) -> Result<()> {
	match tag.tag_type() {
		#[cfg(feature = "ape")]
		TagType::Ape => Into::<ApeTagRef>::into(tag).write_to(data),
		#[cfg(feature = "id3v1")]
		TagType::Id3v1 => Into::<Id3v1TagRef>::into(tag).write_to(data),
		#[cfg(feature = "id3v2")]
		TagType::Id3v2 => Into::<Id3v2TagRef>::into(tag).write_to(data),
		_ => Err(LoftyError::UnsupportedTag),
	}
}
