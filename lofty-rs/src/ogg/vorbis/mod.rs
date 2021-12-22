pub(super) mod properties;
#[cfg(feature = "vorbis_comments")]
pub(in crate::ogg) mod write;

use super::find_last_page;
#[cfg(feature = "vorbis_comments")]
use super::tag::VorbisComments;
use crate::error::Result;
use crate::ogg::constants::{VORBIS_COMMENT_HEAD, VORBIS_IDENT_HEAD};
use crate::types::file::{AudioFile, FileType, TaggedFile};
use crate::types::properties::FileProperties;
use crate::types::tag::TagType;
use properties::VorbisProperties;

use std::io::{Read, Seek};

/// An OGG Vorbis file
pub struct VorbisFile {
	#[cfg(feature = "vorbis_comments")]
	/// The vorbis comments contained in the file
	///
	/// NOTE: While a metadata packet is required, it isn't required to actually have any data.
	pub(crate) vorbis_comments: VorbisComments,
	/// The file's audio properties
	pub(crate) properties: VorbisProperties,
}

impl From<VorbisFile> for TaggedFile {
	fn from(input: VorbisFile) -> Self {
		Self {
			ty: FileType::Vorbis,
			properties: FileProperties::from(input.properties),
			#[cfg(feature = "vorbis_comments")]
			tags: vec![input.vorbis_comments.into()],
			#[cfg(not(feature = "vorbis_comments"))]
			tags: Vec::new(),
		}
	}
}

impl AudioFile for VorbisFile {
	type Properties = VorbisProperties;

	fn read_from<R>(reader: &mut R, read_properties: bool) -> Result<Self>
	where
		R: Read + Seek,
	{
		let file_information =
			super::read::read_from(reader, VORBIS_IDENT_HEAD, VORBIS_COMMENT_HEAD)?;

		Ok(Self {
			properties: if read_properties { properties::read_properties(reader, &file_information.1)? } else { VorbisProperties::default() },
			#[cfg(feature = "vorbis_comments")]
			// Safe to unwrap, a metadata packet is mandatory in OGG Vorbis
			vorbis_comments: file_information.0.unwrap(),
		})
	}

	fn properties(&self) -> &Self::Properties {
		&self.properties
	}

	fn contains_tag(&self) -> bool {
		true
	}

	fn contains_tag_type(&self, tag_type: &TagType) -> bool {
		tag_type == &TagType::VorbisComments
	}
}

impl VorbisFile {
	#[cfg(feature = "vorbis_comments")]
	/// Returns a reference to the Vorbis comments tag
	pub fn vorbis_comments(&self) -> &VorbisComments {
		&self.vorbis_comments
	}

	#[cfg(feature = "vorbis_comments")]
	/// Returns a mutable reference to the Vorbis comments tag
	pub fn vorbis_comments_mut(&mut self) -> &mut VorbisComments {
		&mut self.vorbis_comments
	}
}
