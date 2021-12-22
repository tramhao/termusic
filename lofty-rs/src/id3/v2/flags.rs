#[cfg(feature = "id3v2_restrictions")]
use super::restrictions::TagRestrictions;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
/// Flags that apply to the entire tag
pub struct Id3v2TagFlags {
	/// Whether or not all frames are unsynchronised. See [`FrameFlags::unsynchronisation`](crate::id3::v2::FrameFlags::unsynchronisation)
	pub unsynchronisation: bool,
	/// Indicates if the tag is in an experimental stage
	pub experimental: bool,
	/// Indicates that the tag includes a footer
	pub footer: bool,
	/// Whether or not to include a CRC-32 in the extended header
	///
	/// This is calculated if the tag is written
	pub crc: bool,
	#[cfg(feature = "id3v2_restrictions")]
	/// Restrictions on the tag, written in the extended header
	///
	/// In addition to being setting this flag, all restrictions must be provided. See [`TagRestrictions`]
	pub restrictions: (bool, TagRestrictions),
}
