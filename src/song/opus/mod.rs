pub(crate) mod properties;
pub(super) mod write;

use super::find_last_page;
use super::ogg_reader_writer::VorbisComments;
use crate::error::Result;
use crate::logic::ogg::constants::{OPUSHEAD, OPUSTAGS};
use crate::types::file::{AudioFile, FileType, TaggedFile};
use crate::types::properties::FileProperties;
use crate::types::tag::TagType;
use properties::OpusProperties;

use std::io::{Read, Seek};

/// An OGG Opus file
pub struct OpusFile {
    // #[cfg(feature = "vorbis_comments")]
    /// The vorbis comments contained in the file
    ///
    /// NOTE: While a metadata packet is required, it isn't required to actually have any data.
    pub(crate) vorbis_comments: dyn VorbisComments,
    /// The file's audio properties
    pub(crate) properties: OpusProperties,
}

// impl From<OpusFile> for TaggedFile {
//     fn from(input: OpusFile) -> Self {
//         Self {
//             ty: FileType::Opus,
//             properties: FileProperties::from(input.properties),
//             tags: vec![input.vorbis_comments.into()],
//         }
//     }
// }

impl AudioFile for OpusFile {
    type Properties = OpusProperties;

    fn read_from<R>(reader: &mut R) -> Result<Self>
    where
        R: Read + Seek,
    {
        let file_information = super::read::read_from(reader, OPUSHEAD, OPUSTAGS)?;

        Ok(Self {
            properties: properties::read_properties(reader, &file_information.1)?,
            // #[cfg(feature = "vorbis_comments")]
            // Safe to unwrap, a metadata packet is mandatory in Opus
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

impl OpusFile {
    // #[cfg(feature = "vorbis_comments")]
    /// Returns a reference to the Vorbis comments tag
    pub fn vorbis_comments(&self) -> &VorbisComments {
        &self.vorbis_comments
    }

    // #[cfg(feature = "vorbis_comments")]
    /// Returns a mutable reference to the Vorbis comments tag
    pub fn vorbis_comments_mut(&mut self) -> &mut VorbisComments {
        &mut self.vorbis_comments
    }
}
