mod content;
mod header;
pub(super) mod read;

use super::util::text_utils::TextEncoding;
use crate::error::{LoftyError, Result};
use crate::id3::v2::util::text_utils::encode_text;
use crate::id3::v2::util::upgrade::{upgrade_v2, upgrade_v3};
use crate::types::item::{ItemKey, ItemValue, TagItem};
use crate::types::picture::Picture;
use crate::types::tag::TagType;

use std::convert::{TryFrom, TryInto};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Eq)]
/// Represents an `ID3v2` frame
///
/// ## Outdated Frames
///
/// ### ID3v2.2
///
/// `ID3v2.2` frame IDs are 3 characters. When reading these tags, [`upgrade_v2`] is used, which has a list of all of the common IDs
/// that have a mapping to `ID3v2.4`. Any ID that fails to be converted will be stored as [`FrameID::Outdated`], and it must be manually
/// upgraded before it can be written. **Lofty** will not write `ID3v2.2` tags.
///
/// ### ID3v2.3
///
/// `ID3v2.3`, unlike `ID3v2.2`, stores frame IDs in 4 characters like `ID3v2.4`. There are some IDs that need upgrading (See [`upgrade_v3`]),
/// but anything that fails to be upgraded **will not** be stored as [`FrameID::Outdated`], as it is likely not an issue to write.
pub struct Frame {
	pub(super) id: FrameID,
	pub(super) value: FrameValue,
	pub(super) flags: FrameFlags,
}

impl PartialEq for Frame {
	fn eq(&self, other: &Self) -> bool {
		match self.value {
			FrameValue::Text { .. } => self.id == other.id,
			_ => self.id == other.id && self.value == other.value,
		}
	}
}

impl Hash for Frame {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self.value {
			FrameValue::Text { .. } => self.id.hash(state),
			_ => {
				self.id.hash(state);
				self.content().hash(state);
			}
		}
	}
}

impl Frame {
	/// Create a new frame
	///
	/// NOTE: This will accept both `ID3v2.2` and `ID3v2.3/4` frame IDs
	///
	/// # Errors
	///
	/// * `id` is less than 3 or greater than 4 bytes
	/// * `id` contains non-ascii characters
	pub fn new(id: &str, value: FrameValue, flags: FrameFlags) -> Result<Self> {
		let id = match id.len() {
			// An ID with a length of 4 could be either V3 or V4.
			4 => match upgrade_v3(id) {
				None => FrameID::Valid(id.to_string()),
				Some(id) => FrameID::Valid(id.to_string()),
			},
			3 => match upgrade_v2(id) {
				None => FrameID::Outdated(id.to_string()),
				Some(upgraded) => FrameID::Valid(upgraded.to_string()),
			},
			_ => {
				return Err(LoftyError::Id3v2(
					"Frame ID has a bad length (!= 3 || != 4)",
				))
			}
		};

		match id {
			FrameID::Valid(id) | FrameID::Outdated(id) if !id.is_ascii() => {
				return Err(LoftyError::Id3v2("Frame ID contains non-ascii characters"))
			}
			_ => {}
		}

		Ok(Self { id, value, flags })
	}

	/// Extract the string from the [`FrameID`]
	pub fn id_str(&self) -> &str {
		self.id.as_str()
	}

	/// Returns the frame's content
	pub fn content(&self) -> &FrameValue {
		&self.value
	}

	/// Returns a reference to the [`FrameFlags`]
	pub fn flags(&self) -> &FrameFlags {
		&self.flags
	}

	/// Set the item's flags
	pub fn set_flags(&mut self, flags: FrameFlags) {
		self.flags = flags
	}
}

#[derive(Clone, Debug, Eq)]
/// Information about an `ID3v2` frame that requires a language
///
/// See [`EncodedTextFrame`]
pub struct LanguageFrame {
	/// The encoding of the description and comment text
	pub encoding: TextEncoding,
	/// ISO-639-2 language code (3 bytes)
	pub language: String,
	/// Unique content description
	pub description: String,
	/// The actual frame content
	pub content: String,
}

impl PartialEq for LanguageFrame {
	fn eq(&self, other: &Self) -> bool {
		self.description == other.description
	}
}

impl Hash for LanguageFrame {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.description.hash(state);
	}
}

impl LanguageFrame {
	/// Convert a [`LanguageFrame`] to a byte vec
	///
	/// NOTE: This does not include a frame header
	///
	/// # Errors
	///
	/// * `language` is not exactly 3 bytes
	/// * `language` contains non-ascii characters
	pub fn as_bytes(&self) -> Result<Vec<u8>> {
		let mut bytes = vec![self.encoding as u8];

		if self.language.len() != 3 || !self.language.is_ascii() {
			return Err(LoftyError::Id3v2(
				"Invalid frame language found (expected 3 ascii characters)",
			));
		}

		bytes.extend(self.language.as_bytes().iter());
		bytes.extend(encode_text(&*self.description, self.encoding, true).iter());
		bytes.extend(encode_text(&*self.content, self.encoding, false));

		Ok(bytes)
	}
}

#[derive(Clone, Debug, Eq)]
/// An `ID3v2` text frame
///
/// This is used in the frames `TXXX` and `WXXX`, where the frames
/// are told apart by descriptions, rather than their [`FrameID`]s.
/// This means for each `EncodedTextFrame` in the tag, the description
/// must be unique.
pub struct EncodedTextFrame {
	/// The encoding of the description and comment text
	pub encoding: TextEncoding,
	/// Unique content description
	pub description: String,
	/// The actual frame content
	pub content: String,
}

impl PartialEq for EncodedTextFrame {
	fn eq(&self, other: &Self) -> bool {
		self.description == other.description
	}
}

impl Hash for EncodedTextFrame {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.description.hash(state);
	}
}

impl EncodedTextFrame {
	/// Convert an [`EncodedTextFrame`] to a byte vec
	pub fn as_bytes(&self) -> Vec<u8> {
		let mut bytes = vec![self.encoding as u8];

		bytes.extend(encode_text(&*self.description, self.encoding, true).iter());
		bytes.extend(encode_text(&*self.content, self.encoding, false));

		bytes
	}
}

#[derive(PartialEq, Clone, Debug, Eq, Hash)]
/// An `ID3v2` frame ID
pub enum FrameID {
	/// A valid `ID3v2.3/4` frame
	Valid(String),
	/// When an `ID3v2.2` key couldn't be upgraded
	///
	/// This **will not** be written. It is up to the user to upgrade and store the key as [`Id3v2Frame::Valid`](Self::Valid).
	///
	/// The entire frame is stored as [`ItemValue::Binary`](crate::ItemValue::Binary).
	Outdated(String),
}

impl FrameID {
	/// Extracts the string from the ID
	pub fn as_str(&self) -> &str {
		match self {
			FrameID::Valid(v) | FrameID::Outdated(v) => v.as_str(),
		}
	}
}

impl TryFrom<ItemKey> for FrameID {
	type Error = LoftyError;

	fn try_from(value: ItemKey) -> std::prelude::rust_2015::Result<Self, Self::Error> {
		match value {
			ItemKey::Unknown(unknown) if unknown.len() == 4 && unknown.is_ascii() => {
				Ok(Self::Valid(unknown.to_ascii_uppercase()))
			}
			k => k.map_key(TagType::Id3v2, false).map_or(
				Err(LoftyError::Id3v2(
					"ItemKey does not meet the requirements to be a FrameID",
				)),
				|id| Ok(Self::Valid(id.to_string())),
			),
		}
	}
}

#[derive(PartialEq, Clone, Debug, Eq, Hash)]
/// The value of an `ID3v2` frame
pub enum FrameValue {
	/// Represents a "COMM" frame
	///
	/// Due to the amount of information needed, it is contained in a separate struct, [`LanguageFrame`]
	Comment(LanguageFrame),
	/// Represents a "USLT" frame
	///
	/// Due to the amount of information needed, it is contained in a separate struct, [`LanguageFrame`]
	UnSyncText(LanguageFrame),
	/// Represents a "T..." (excluding TXXX) frame
	///
	/// NOTE: Text frame descriptions **must** be unique
	Text {
		/// The encoding of the text
		encoding: TextEncoding,
		/// The text itself
		value: String,
	},
	/// Represents a "TXXX" frame
	///
	/// Due to the amount of information needed, it is contained in a separate struct, [`EncodedTextFrame`]
	UserText(EncodedTextFrame),
	/// Represents a "W..." (excluding WXXX) frame
	///
	/// NOTE: URL frame descriptions **must** be unique
	///
	/// No encoding needs to be provided as all URLs are [`TextEncoding::Latin1`]
	URL(String),
	/// Represents a "WXXX" frame
	///
	/// Due to the amount of information needed, it is contained in a separate struct, [`EncodedTextFrame`]
	UserURL(EncodedTextFrame),
	/// Represents an "APIC" or "PIC" frame
	Picture {
		/// The encoding of the description
		encoding: TextEncoding,
		/// The picture itself
		picture: Picture,
	},
	/// Binary data
	///
	/// NOTES:
	///
	/// * This is used for "GEOB" and "SYLT" frames, see
	/// [`GeneralEncapsulatedObject::parse`](crate::id3::v2::GeneralEncapsulatedObject::parse) and [`SynchronizedText::parse`](crate::id3::v2::SynchronizedText::parse) respectively
	/// * This is used for **all** frames with an ID of [`FrameID::Outdated`]
	/// * This is used for unknown frames
	Binary(Vec<u8>),
}

impl TryFrom<TagItem> for Frame {
	type Error = LoftyError;

	fn try_from(value: TagItem) -> std::prelude::rust_2015::Result<Self, Self::Error> {
		let id: FrameID = value.item_key.try_into()?;

		// We make the VERY bold assumption the language is English
		let value = match (&id, value.item_value) {
			(FrameID::Valid(ref s), ItemValue::Text(text)) if s == "COMM" => {
				FrameValue::Comment(LanguageFrame {
					encoding: TextEncoding::UTF8,
					language: String::from("eng"),
					description: String::new(),
					content: text,
				})
			}
			(FrameID::Valid(ref s), ItemValue::Text(text)) if s == "USLT" => {
				FrameValue::UnSyncText(LanguageFrame {
					encoding: TextEncoding::UTF8,
					language: String::from("eng"),
					description: String::new(),
					content: text,
				})
			}
			(FrameID::Valid(ref s), ItemValue::Locator(text) | ItemValue::Text(text))
				if s == "WXXX" =>
			{
				FrameValue::UserURL(EncodedTextFrame {
					encoding: TextEncoding::UTF8,
					description: String::new(),
					content: text,
				})
			}
			(FrameID::Valid(ref s), ItemValue::Text(text)) if s == "TXXX" => {
				FrameValue::UserText(EncodedTextFrame {
					encoding: TextEncoding::UTF8,
					description: String::new(),
					content: text,
				})
			}
			(_, value) => value.into(),
		};

		Ok(Self {
			id,
			value,
			flags: FrameFlags::default(),
		})
	}
}

impl From<ItemValue> for FrameValue {
	fn from(input: ItemValue) -> Self {
		match input {
			ItemValue::Text(text) => FrameValue::Text {
				encoding: TextEncoding::UTF8,
				value: text,
			},
			ItemValue::Locator(locator) => FrameValue::URL(locator),
			ItemValue::Binary(binary) => FrameValue::Binary(binary),
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[allow(clippy::struct_excessive_bools)]
/// Various flags to describe the content of an item
pub struct FrameFlags {
	/// Preserve frame on tag edit
	pub tag_alter_preservation: bool,
	/// Preserve frame on file edit
	pub file_alter_preservation: bool,
	/// Item cannot be written to
	pub read_only: bool,
	/// Frame belongs in a group
	///
	/// In addition to setting this flag, a group identifier byte must be added.
	/// All frames with the same group identifier byte belong to the same group.
	pub grouping_identity: (bool, u8),
	/// Frame is zlib compressed
	///
	/// It is **required** `data_length_indicator` be set if this is set.
	pub compression: bool,
	/// Frame is encrypted
	///
	/// NOTE: Since the encryption method is unknown, lofty cannot do anything with these frames
	///
	/// In addition to setting this flag, an encryption method symbol must be added.
	/// The method symbol **must** be > 0x80.
	pub encryption: (bool, u8),
	/// Frame is unsynchronised
	///
	/// In short, this makes all "0xFF 0x00" combinations into "0xFF 0x00 0x00" to avoid confusion
	/// with the MPEG frame header, which is often identified by its "frame sync" (11 set bits).
	/// It is preferred an ID3v2 tag is either *completely* unsynchronised or not unsynchronised at all.
	///
	/// NOTE: For the sake of simplicity, this will have no effect when writing. There isn't much reason
	/// to write unsynchronized data. Unsynchronized data **will** always be read, however.
	pub unsynchronisation: bool,
	/// Frame has a data length indicator
	///
	/// The data length indicator is the size of the frame if the flags were all zeroed out.
	/// This is usually used in combination with `compression` and `encryption` (depending on encryption method).
	///
	/// If using encryption, the final size must be added. It will be ignored if using compression.
	pub data_length_indicator: (bool, u32),
}

pub(crate) struct FrameRef<'a> {
	pub id: &'a str,
	pub value: FrameValueRef<'a>,
	pub flags: FrameFlags,
}

impl<'a> Frame {
	pub(crate) fn as_opt_ref(&'a self) -> Option<FrameRef<'a>> {
		if let FrameID::Valid(id) = &self.id {
			Some(FrameRef {
				id,
				value: (&self.value).into(),
				flags: self.flags,
			})
		} else {
			None
		}
	}
}

impl<'a> TryFrom<&'a TagItem> for FrameRef<'a> {
	type Error = LoftyError;

	fn try_from(value: &'a TagItem) -> std::prelude::rust_2015::Result<Self, Self::Error> {
		let id = match value.key() {
			ItemKey::Unknown(unknown)
				if unknown.len() == 4
					&& unknown.is_ascii()
					&& unknown.chars().all(|c| c.is_ascii_uppercase()) =>
			{
				Ok(unknown.as_str())
			}
			k => k.map_key(TagType::Id3v2, false).ok_or(LoftyError::Id3v2(
				"ItemKey does not meet the requirements to be a FrameID",
			)),
		}?;

		Ok(FrameRef {
			id,
			value: Into::<FrameValueRef<'a>>::into(value.value()),
			flags: FrameFlags::default(),
		})
	}
}

pub(crate) enum FrameValueRef<'a> {
	Comment(&'a LanguageFrame),
	UnSyncText(&'a LanguageFrame),
	Text {
		encoding: TextEncoding,
		value: &'a str,
	},
	UserText(&'a EncodedTextFrame),
	URL(&'a str),
	UserURL(&'a EncodedTextFrame),
	Picture {
		encoding: TextEncoding,
		picture: &'a Picture,
	},
	Binary(&'a [u8]),
}

impl<'a> Into<FrameValueRef<'a>> for &'a FrameValue {
	fn into(self) -> FrameValueRef<'a> {
		match self {
			FrameValue::Comment(lf) => FrameValueRef::Comment(lf),
			FrameValue::UnSyncText(lf) => FrameValueRef::UnSyncText(lf),
			FrameValue::Text { encoding, value } => FrameValueRef::Text {
				encoding: *encoding,
				value: value.as_str(),
			},
			FrameValue::UserText(etf) => FrameValueRef::UserText(etf),
			FrameValue::URL(url) => FrameValueRef::URL(url.as_str()),
			FrameValue::UserURL(etf) => FrameValueRef::UserURL(etf),
			FrameValue::Picture { encoding, picture } => FrameValueRef::Picture {
				encoding: *encoding,
				picture,
			},
			FrameValue::Binary(bin) => FrameValueRef::Binary(bin.as_slice()),
		}
	}
}

impl<'a> Into<FrameValueRef<'a>> for &'a ItemValue {
	fn into(self) -> FrameValueRef<'a> {
		match self {
			ItemValue::Text(text) => FrameValueRef::Text {
				encoding: TextEncoding::UTF8,
				value: text.as_str(),
			},
			ItemValue::Locator(locator) => FrameValueRef::URL(locator.as_str()),
			ItemValue::Binary(binary) => FrameValueRef::Binary(binary.as_slice()),
		}
	}
}
