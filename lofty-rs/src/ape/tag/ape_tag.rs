use crate::ape::tag::item::{ApeItem, ApeItemRef};
use crate::error::Result;
use crate::types::item::{ItemKey, ItemValue, TagItem};
use crate::types::tag::{Accessor, Tag, TagType};

use std::convert::TryInto;
use std::fs::File;

macro_rules! impl_accessor {
	($($name:ident, $($key:literal)|+;)+) => {
		paste::paste! {
			impl Accessor for ApeTag {
				$(
					fn $name(&self) -> Option<&str> {
						$(
							if let Some(i) = self.get_key($key) {
								if let ItemValue::Text(val) = i.value() {
									return Some(val)
								}
							}
						)+

						None
					}

					fn [<set_ $name>](&mut self, value: String) {
						self.insert(ApeItem {
							read_only: false,
							key: String::from(crate::types::item::first_key!($($key)|*)),
							value: ItemValue::Text(value)
						})
					}

					fn [<remove_ $name>](&mut self) {
						$(
							self.remove_key($key);
						)+
					}
				)+
			}
		}
	}
}

#[derive(Default, Debug, PartialEq, Clone)]
/// An `APE` tag
///
/// ## Supported file types
///
/// * [`FileType::APE`](crate::FileType::APE)
/// * [`FileType::MP3`](crate::FileType::MP3)
///
/// ## Item storage
///
/// `APE` isn't a very strict format. An [`ApeItem`] only restricted by its name, meaning it can use
/// a normal [`ItemValue`](crate::ItemValue) unlike other formats.
///
/// Pictures are stored as [`ItemValue::Binary`](crate::ItemValue::Binary), and can be converted with
/// [`Picture::from_ape_bytes`](crate::Picture::from_ape_bytes). For the appropriate item keys, see
/// [APE_PICTURE_TYPES](crate::ape::APE_PICTURE_TYPES).
///
/// ## Conversions
///
/// ### From `Tag`
///
/// When converting pictures, any of type [`PictureType::Undefined`](crate::PictureType::Undefined) will be discarded.
/// For items, see [ApeItem::new].
pub struct ApeTag {
	/// Whether or not to mark the tag as read only
	pub read_only: bool,
	pub(super) items: Vec<ApeItem>,
}

impl_accessor!(
	artist,       "Artist";
	title,        "Title";
	album,        "Album";
	album_artist, "Album Artist" | "ALBUMARTST";
	genre,        "GENRE";
);

impl ApeTag {
	/// Get an [`ApeItem`] by key
	///
	/// NOTE: While `APE` items are supposed to be case-sensitive,
	/// this rule is rarely followed, so this will ignore case when searching.
	pub fn get_key(&self, key: &str) -> Option<&ApeItem> {
		self.items
			.iter()
			.find(|i| i.key().eq_ignore_ascii_case(key))
	}

	/// Insert an [`ApeItem`]
	///
	/// This will remove any item with the same key prior to insertion
	pub fn insert(&mut self, value: ApeItem) {
		self.remove_key(value.key());
		self.items.push(value);
	}

	/// Remove an [`ApeItem`] by key
	///
	/// NOTE: Like [`ApeTag::get_key`], this is not case-sensitive
	pub fn remove_key(&mut self, key: &str) {
		self.items
			.iter()
			.position(|i| i.key().eq_ignore_ascii_case(key))
			.map(|p| self.items.remove(p));
	}

	/// Returns all of the tag's items
	pub fn items(&self) -> &[ApeItem] {
		&self.items
	}
}

impl ApeTag {
	/// Write an `APE` tag to a file
	///
	/// # Errors
	///
	/// * Attempting to write the tag to a format that does not support it
	/// * An existing tag has an invalid size
	pub fn write_to(&self, file: &mut File) -> Result<()> {
		Into::<ApeTagRef>::into(self).write_to(file)
	}
}

impl From<ApeTag> for Tag {
	fn from(input: ApeTag) -> Self {
		fn split_pair(
			content: &str,
			tag: &mut Tag,
			current_key: ItemKey,
			total_key: ItemKey,
		) -> Option<()> {
			let mut split = content.splitn(2, '/');
			let current = split.next()?.to_string();
			tag.insert_item_unchecked(TagItem::new(current_key, ItemValue::Text(current)));

			if let Some(total) = split.next() {
				tag.insert_item_unchecked(TagItem::new(
					total_key,
					ItemValue::Text(total.to_string()),
				))
			}

			Some(())
		}

		let mut tag = Tag::new(TagType::Ape);

		for item in input.items {
			let item_key = ItemKey::from_key(TagType::Ape, item.key());

			// The text pairs need some special treatment
			match (item_key, item.value()) {
				(ItemKey::TrackNumber | ItemKey::TrackTotal, ItemValue::Text(val))
					if split_pair(val, &mut tag, ItemKey::TrackNumber, ItemKey::TrackTotal)
						.is_some() =>
				{
					continue
				},
				(ItemKey::DiscNumber | ItemKey::DiscTotal, ItemValue::Text(val))
					if split_pair(val, &mut tag, ItemKey::DiscNumber, ItemKey::DiscTotal)
						.is_some() =>
				{
					continue
				},
				(k, _) => tag.insert_item_unchecked(TagItem::new(k, item.value)),
			}
		}

		tag
	}
}

impl From<Tag> for ApeTag {
	fn from(input: Tag) -> Self {
		let mut ape_tag = Self::default();

		for item in input.items {
			if let Ok(ape_item) = item.try_into() {
				ape_tag.insert(ape_item)
			}
		}

		for pic in input.pictures {
			if let Some(key) = pic.pic_type.as_ape_key() {
				if let Ok(item) =
					ApeItem::new(key.to_string(), ItemValue::Binary(pic.as_ape_bytes()))
				{
					ape_tag.insert(item)
				}
			}
		}

		ape_tag
	}
}

pub(crate) struct ApeTagRef<'a> {
	pub(crate) read_only: bool,
	pub(super) items: Box<dyn Iterator<Item = ApeItemRef<'a>> + 'a>,
}

impl<'a> ApeTagRef<'a> {
	pub(crate) fn write_to(&mut self, file: &mut File) -> Result<()> {
		super::write::write_to(file, self)
	}
}

impl<'a> Into<ApeTagRef<'a>> for &'a Tag {
	fn into(self) -> ApeTagRef<'a> {
		ApeTagRef {
			read_only: false,
			items: Box::new(self.items.iter().filter_map(|i| {
				i.key().map_key(TagType::Ape, true).map(|key| ApeItemRef {
					read_only: false,
					key,
					value: (&i.item_value).into(),
				})
			})),
		}
	}
}

impl<'a> Into<ApeTagRef<'a>> for &'a ApeTag {
	fn into(self) -> ApeTagRef<'a> {
		ApeTagRef {
			read_only: self.read_only,
			items: Box::new(self.items.iter().map(Into::into)),
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::ape::{ApeItem, ApeTag};
	use crate::{ItemValue, Tag, TagType};

	use crate::ape::tag::read_ape_header;
	use std::io::{Cursor, Read};

	#[test]
	#[allow(clippy::similar_names)]
	fn parse_ape() {
		let mut expected_tag = ApeTag::default();

		let title_item = ApeItem::new(
			String::from("TITLE"),
			ItemValue::Text(String::from("Foo title")),
		)
		.unwrap();

		let artist_item = ApeItem::new(
			String::from("ARTIST"),
			ItemValue::Text(String::from("Bar artist")),
		)
		.unwrap();

		let album_item = ApeItem::new(
			String::from("ALBUM"),
			ItemValue::Text(String::from("Baz album")),
		)
		.unwrap();

		let comment_item = ApeItem::new(
			String::from("COMMENT"),
			ItemValue::Text(String::from("Qux comment")),
		)
		.unwrap();

		let year_item =
			ApeItem::new(String::from("YEAR"), ItemValue::Text(String::from("1984"))).unwrap();

		let track_number_item =
			ApeItem::new(String::from("TRACK"), ItemValue::Text(String::from("1"))).unwrap();

		let genre_item = ApeItem::new(
			String::from("GENRE"),
			ItemValue::Text(String::from("Classical")),
		)
		.unwrap();

		expected_tag.insert(title_item);
		expected_tag.insert(artist_item);
		expected_tag.insert(album_item);
		expected_tag.insert(comment_item);
		expected_tag.insert(year_item);
		expected_tag.insert(track_number_item);
		expected_tag.insert(genre_item);

		let mut tag = Vec::new();
		std::fs::File::open("tests/tags/assets/test.apev2")
			.unwrap()
			.read_to_end(&mut tag)
			.unwrap();

		let mut reader = Cursor::new(tag);

		let header = read_ape_header(&mut reader, false).unwrap();
		let parsed_tag = crate::ape::tag::read::read_ape_tag(&mut reader, header).unwrap();

		assert_eq!(expected_tag.items().len(), parsed_tag.items().len());

		for item in expected_tag.items() {
			assert!(parsed_tag.items().contains(item))
		}
	}

	#[test]
	#[allow(clippy::similar_names)]
	fn ape_to_tag() {
		let mut tag_bytes = Vec::new();
		std::fs::File::open("tests/tags/assets/test.apev2")
			.unwrap()
			.read_to_end(&mut tag_bytes)
			.unwrap();

		let mut reader = Cursor::new(tag_bytes);

		let header = read_ape_header(&mut reader, false).unwrap();
		let ape = crate::ape::tag::read::read_ape_tag(&mut reader, header).unwrap();

		let tag: Tag = ape.into();

		crate::tag_utils::test_utils::verify_tag(&tag, true, true);
	}

	#[test]
	fn tag_to_ape() {
		fn verify_key(tag: &ApeTag, key: &str, expected_val: &str) {
			assert_eq!(
				tag.get_key(key).map(ApeItem::value),
				Some(&ItemValue::Text(String::from(expected_val)))
			);
		}

		let tag = crate::tag_utils::test_utils::create_tag(TagType::Ape);

		let ape_tag: ApeTag = tag.into();

		verify_key(&ape_tag, "Title", "Foo title");
		verify_key(&ape_tag, "Artist", "Bar artist");
		verify_key(&ape_tag, "Album", "Baz album");
		verify_key(&ape_tag, "Comment", "Qux comment");
		verify_key(&ape_tag, "Track", "1");
		verify_key(&ape_tag, "Genre", "Classical");
	}
}
