mod block;
mod properties;
mod read;
#[cfg(feature = "vorbis_comments")]
pub(crate) mod write;

#[cfg(feature = "vorbis_comments")]
use super::tag::VorbisComments;
use crate::error::Result;
use crate::tag_utils::tag_methods;
use crate::types::file::{AudioFile, FileType, TaggedFile};
use crate::types::properties::FileProperties;
use crate::types::tag::TagType;

use std::io::{Read, Seek};

/// A FLAC file
pub struct FlacFile {
	#[cfg(feature = "vorbis_comments")]
	/// The vorbis comments contained in the file
	///
	/// NOTE: This field being `Some` does not mean the file has vorbis comments, as Picture blocks exist.
	pub(crate) vorbis_comments: Option<VorbisComments>,
	/// The file's audio properties
	pub(crate) properties: FileProperties,
}

impl From<FlacFile> for TaggedFile {
	fn from(input: FlacFile) -> Self {
		Self {
			ty: FileType::FLAC,
			properties: input.properties,
			#[cfg(feature = "vorbis_comments")]
			tags: input
				.vorbis_comments
				.map_or_else(Vec::new, |t| vec![t.into()]),
			#[cfg(not(feature = "vorbis_comments"))]
			tags: Vec::new(),
		}
	}
}

impl AudioFile for FlacFile {
	type Properties = FileProperties;

	fn read_from<R>(reader: &mut R, read_properties: bool) -> Result<Self>
	where
		R: Read + Seek,
	{
		read::read_from(reader, read_properties)
	}

	fn properties(&self) -> &Self::Properties {
		&self.properties
	}

	fn contains_tag(&self) -> bool {
		#[cfg(feature = "vorbis_comments")]
		return self.vorbis_comments.is_some();

		#[cfg(not(feature = "vorbis_comments"))]
		return false;
	}

	#[allow(unused_variables)]
	fn contains_tag_type(&self, tag_type: &TagType) -> bool {
		#[cfg(feature = "vorbis_comments")]
		return tag_type == &TagType::VorbisComments && self.vorbis_comments.is_some();

		#[cfg(not(feature = "vorbis_comments"))]
		return false;
	}
}

impl FlacFile {
	tag_methods! {
		#[cfg(feature = "vorbis_comments")];
		Vorbis_Comments, vorbis_comments, VorbisComments
	}
}
