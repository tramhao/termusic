//! MP3 specific items
mod constants;
pub(crate) mod header;
mod properties;
mod read;
pub(crate) mod write;

pub use header::{ChannelMode, Layer, MpegVersion};
pub use properties::Mp3Properties;

#[cfg(feature = "ape")]
use crate::ape::tag::ape_tag::ApeTag;
use crate::error::Result;
#[cfg(feature = "id3v1")]
use crate::id3::v1::tag::Id3v1Tag;
#[cfg(feature = "id3v2")]
use crate::id3::v2::tag::Id3v2Tag;
use crate::tag_utils::tag_methods;
use crate::types::file::{AudioFile, FileType, TaggedFile};
use crate::types::properties::FileProperties;
use crate::types::tag::{Tag, TagType};

use std::io::{Read, Seek};

/// An MP3 file
pub struct Mp3File {
	#[cfg(feature = "id3v2")]
	/// An ID3v2 tag
	pub(crate) id3v2_tag: Option<Id3v2Tag>,
	#[cfg(feature = "id3v1")]
	/// An ID3v1 tag
	pub(crate) id3v1_tag: Option<Id3v1Tag>,
	#[cfg(feature = "ape")]
	/// An APEv1/v2 tag
	pub(crate) ape_tag: Option<ApeTag>,
	/// The file's audio properties
	pub(crate) properties: Mp3Properties,
}

impl From<Mp3File> for TaggedFile {
	#[allow(clippy::vec_init_then_push, unused_mut)]
	fn from(input: Mp3File) -> Self {
		let mut tags = Vec::<Option<Tag>>::with_capacity(3);

		#[cfg(feature = "id3v2")]
		tags.push(input.id3v2_tag.map(Into::into));
		#[cfg(feature = "id3v1")]
		tags.push(input.id3v1_tag.map(Into::into));
		#[cfg(feature = "ape")]
		tags.push(input.ape_tag.map(Into::into));

		Self {
			ty: FileType::MP3,
			properties: FileProperties::from(input.properties),
			tags: tags.into_iter().flatten().collect(),
		}
	}
}

impl AudioFile for Mp3File {
	type Properties = Mp3Properties;

	fn read_from<R>(reader: &mut R, read_properties: bool) -> Result<Self>
	where
		R: Read + Seek,
	{
		read::read_from(reader, read_properties)
	}

	fn properties(&self) -> &Self::Properties {
		&self.properties
	}

	#[allow(unreachable_code)]
	fn contains_tag(&self) -> bool {
		#[cfg(feature = "id3v2")]
		return self.id3v2_tag.is_some();
		#[cfg(feature = "id3v1")]
		return self.id3v1_tag.is_some();
		#[cfg(feature = "ape")]
		return self.ape_tag.is_some();

		false
	}

	fn contains_tag_type(&self, tag_type: &TagType) -> bool {
		match tag_type {
			#[cfg(feature = "ape")]
			TagType::Ape => self.ape_tag.is_some(),
			#[cfg(feature = "id3v2")]
			TagType::Id3v2 => self.id3v2_tag.is_some(),
			#[cfg(feature = "id3v1")]
			TagType::Id3v1 => self.id3v1_tag.is_some(),
			_ => false,
		}
	}
}

impl Mp3File {
	tag_methods! {
		#[cfg(feature = "id3v2")];
		ID3v2, id3v2_tag, Id3v2Tag;
		#[cfg(feature = "id3v1")];
		ID3v1, id3v1_tag, Id3v1Tag;
		#[cfg(feature = "ape")];
		APE, ape_tag, ApeTag
	}
}
