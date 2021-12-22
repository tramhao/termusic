use super::item::{ItemKey, ItemValue, TagItem};
use super::picture::{Picture, PictureType};
use crate::error::{LoftyError, Result};
use crate::probe::Probe;

use std::fs::{File, OpenOptions};
use std::path::Path;

macro_rules! accessor_trait {
	($($name:ident),+) => {
		/// Provides accessors for common items
		pub trait Accessor {
			paste::paste! {
				$(
					#[doc = "Gets the " $name]
					fn $name(&self) -> Option<&str> { None }
					#[doc = "Sets the " $name]
					fn [<set_ $name>](&mut self, _value: String) {}
					#[doc = "Removes the " $name]
					fn [<remove_ $name>](&mut self) {}
				)+
			}
		}
	};
}

accessor_trait! {
	artist, title,
	album, album_artist,
	genre,lyrics
}

macro_rules! impl_accessor {
	($($item_key:ident => $name:tt),+) => {
		paste::paste! {
			impl Accessor for Tag {
				$(
					fn $name(&self) -> Option<&str> {
						if let Some(ItemValue::Text(txt)) = self.get_item_ref(&ItemKey::$item_key).map(TagItem::value) {
							return Some(&*txt)
						}

						None
					}

					fn [<set_ $name>](&mut self, value: String) {
						self.insert_item(TagItem::new(ItemKey::$item_key, ItemValue::Text(value)));
					}

					fn [<remove_ $name>](&mut self) {
						self.retain_items(|i| i.item_key != ItemKey::$item_key)
					}
				)+
			}
		}
	}
}

#[derive(Clone)]
/// Represents a parsed tag
///
/// This is a tag that is loosely bound to a specific [TagType].
/// It is used for conversions and as the return type for [`read_from`](crate::read_from).
///
/// Compared to other formats, this gives a much higher-level view of the
/// tag items. Rather than storing items according to their format-specific
/// keys, [`ItemKey`]s are used.
///
/// You can easily remap this to another [TagType] with [Tag::re_map].
///
/// Any conversion will, of course, be lossy to a varying degree.
///
/// ## Usage
///
/// Accessing common items
///
/// ```rust
/// # use lofty::{Tag, TagType, Accessor};
/// # let tag = Tag::new(TagType::Id3v2);
/// // There are multiple quick getter methods for common items
///
/// let title = tag.title();
/// let artist = tag.artist();
/// let album = tag.album();
/// let album_artist = tag.album_artist();
/// ```
///
/// Getting an item of a known type
///
/// ```rust
/// # use lofty::{Tag, TagType};
/// # let tag = Tag::new(TagType::Id3v2);
/// use lofty::ItemKey;
///
/// // If the type of an item is known, there are getter methods
/// // to prevent having to match against the value
///
/// tag.get_string(&ItemKey::TrackTitle);
/// tag.get_binary(&ItemKey::TrackTitle, false);
/// ```
///
/// Converting between formats
///
/// ```rust
/// use lofty::{Tag, TagType};
/// use lofty::id3::v2::Id3v2Tag;
///
/// // Converting between formats is as simple as an `into` call.
/// // However, such conversions can potentially be *very* lossy.
///
/// let tag = Tag::new(TagType::Id3v2);
/// let id3v2_tag: Id3v2Tag = tag.into();
/// ```
pub struct Tag {
	tag_type: TagType,
	pub(crate) pictures: Vec<Picture>,
	pub(crate) items: Vec<TagItem>,
}

impl IntoIterator for Tag {
	type Item = TagItem;
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.items.into_iter()
	}
}

impl_accessor!(
	TrackArtist => artist,
	TrackTitle => title,
	AlbumTitle => album,
	AlbumArtist => album_artist,
	Genre => genre,
	Lyrics => lyrics
);

impl Tag {
	/// Initialize a new tag with a certain [`TagType`]
	pub fn new(tag_type: TagType) -> Self {
		Self {
			tag_type,
			pictures: vec![],
			items: vec![],
		}
	}

	/// Change the [`TagType`], remapping all items
	pub fn re_map(&mut self, tag_type: TagType) {
		self.retain_items(|i| i.re_map(tag_type).is_some());
		self.tag_type = tag_type
	}

	/// Returns the [`TagType`]
	pub fn tag_type(&self) -> &TagType {
		&self.tag_type
	}

	/// Returns the number of [`TagItem`]s
	pub fn item_count(&self) -> u32 {
		self.items.len() as u32
	}

	/// Returns the number of [`Picture`]s
	pub fn picture_count(&self) -> u32 {
		self.pictures.len() as u32
	}

	/// Returns the stored [`TagItem`]s as a slice
	pub fn items(&self) -> &[TagItem] {
		&*self.items
	}

	/// Returns a reference to a [`TagItem`] matching an [`ItemKey`]
	pub fn get_item_ref(&self, item_key: &ItemKey) -> Option<&TagItem> {
		self.items.iter().find(|i| &i.item_key == item_key)
	}

	/// Get a string value from an [`ItemKey`]
	pub fn get_string(&self, item_key: &ItemKey) -> Option<&str> {
		if let Some(ItemValue::Text(ret)) = self.get_item_ref(item_key).map(TagItem::value) {
			return Some(ret);
		}

		None
	}

	/// Gets a byte slice from an [`ItemKey`]
	///
	/// Use `convert` to convert [`ItemValue::Text`] and [`ItemValue::Locator`] to byte slices
	pub fn get_binary(&self, item_key: &ItemKey, convert: bool) -> Option<&[u8]> {
		if let Some(item) = self.get_item_ref(item_key) {
			match item.value() {
				ItemValue::Text(text) if convert => return Some(text.as_bytes()),
				ItemValue::Locator(locator) => return Some(locator.as_bytes()),
				ItemValue::Binary(binary) => return Some(binary),
				_ => {}
			}
		}

		None
	}

	/// Insert a [`TagItem`], replacing any existing one of the same type
	///
	/// NOTE: This **will** verify an [`ItemKey`] mapping exists for the target [`TagType`]
	///
	/// This will return `true` if the item was inserted.
	pub fn insert_item(&mut self, item: TagItem) -> bool {
		if item.re_map(self.tag_type).is_some() {
			self.insert_item_unchecked(item);
			return true;
		}

		false
	}

	/// Insert a [`TagItem`], replacing any existing one of the same type
	///
	/// Notes:
	///
	/// * This **will not** verify an [`ItemKey`] mapping exists
	/// * This **will not** allow writing item keys that are out of spec (keys are verified before writing)
	///
	/// This is only necessary if dealing with [`ItemKey::Unknown`].
	pub fn insert_item_unchecked(&mut self, item: TagItem) {
		match self.items.iter_mut().find(|i| i.item_key == item.item_key) {
			None => self.items.push(item),
			Some(i) => *i = item,
		};
	}

	/// An alias for [`Tag::insert_item`] that doesn't require the user to create a [`TagItem`]
	pub fn insert_text(&mut self, item_key: ItemKey, text: String) -> bool {
		self.insert_item(TagItem::new(item_key, ItemValue::Text(text)))
	}

	/// Remove an item by its key
	///
	/// This will remove all items with this key.
	pub fn remove_item(&mut self, key: &ItemKey) {
		self.items.retain(|i| i.key() != key)
	}

	/// Retain tag items based on the predicate
	///
	/// See [`Vec::retain`](std::vec::Vec::retain)
	pub fn retain_items<F>(&mut self, f: F)
	where
		F: FnMut(&TagItem) -> bool,
	{
		self.items.retain(f)
	}

	/// Returns the stored [`Picture`]s as a slice
	pub fn pictures(&self) -> &[Picture] {
		&*self.pictures
	}

	/// Pushes a [`Picture`] to the tag
	pub fn push_picture(&mut self, picture: Picture) {
		self.pictures.push(picture)
	}

	/// Removes all [`Picture`]s of a [`PictureType`]
	pub fn remove_picture_type(&mut self, picture_type: PictureType) {
		self.pictures.retain(|p| p.pic_type != picture_type)
	}

	/// Save the `Tag` to a path
	///
	/// # Errors
	///
	/// * Path doesn't exist
	/// * Path is not writable
	/// * See [`Tag::save_to`]
	pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<()> {
		self.save_to(&mut OpenOptions::new().read(true).write(true).open(path)?)
	}

	/// Save the `Tag` to a [`File`](std::fs::File)
	///
	/// # Errors
	///
	/// * A [`FileType`](crate::FileType) couldn't be determined from the File
	/// * Attempting to write a tag to a format that does not support it. See [`FileType::supports_tag_type`](crate::FileType::supports_tag_type)
	pub fn save_to(&self, file: &mut File) -> Result<()> {
		let probe = Probe::new(file).guess_file_type()?;

		match probe.file_type() {
			Some(file_type) => {
				if file_type.supports_tag_type(self.tag_type()) {
					crate::tag_utils::write_tag(self, probe.into_inner(), file_type)
				} else {
					Err(LoftyError::UnsupportedTag)
				}
			}
			None => Err(LoftyError::UnknownFormat),
		}
	}

	/// Same as [`TagType::remove_from_path`]
	pub fn remove_from_path(&self, path: impl AsRef<Path>) -> bool {
		self.tag_type.remove_from_path(path)
	}

	/// Same as [`TagType::remove_from`]
	pub fn remove_from(&self, file: &mut File) -> bool {
		self.tag_type.remove_from(file)
	}
}

/// The tag's format
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TagType {
	/// This covers both APEv1 and APEv2 as it doesn't matter much
	Ape,
	/// Represents an ID3v1 tag
	Id3v1,
	/// This covers all ID3v2 versions since they all get upgraded to ID3v2.4
	Id3v2,
	/// Represents an MP4 ILST atom
	Mp4Ilst,
	/// Represents vorbis comments
	VorbisComments,
	/// Represents a RIFF INFO LIST
	RiffInfo,
	/// Represents AIFF text chunks
	AiffText,
}

impl TagType {
	/// Remove a tag from a [`Path`]
	///
	/// See [`TagType::remove_from`]
	pub fn remove_from_path(&self, path: impl AsRef<Path>) -> bool {
		if let Ok(mut file) = OpenOptions::new().read(true).write(true).open(path) {
			return self.remove_from(&mut file);
		}

		false
	}

	#[allow(clippy::shadow_unrelated)]
	/// Remove a tag from a [`File`]
	///
	/// This will return `false` if:
	///
	/// * It is unable to guess the file format
	/// * The format doesn't support the `TagType`
	/// * It is unable to write to the file
	pub fn remove_from(&self, file: &mut File) -> bool {
		if let Ok(probe) = Probe::new(file).guess_file_type() {
			if let Some(file_type) = probe.file_type() {
				if file_type.supports_tag_type(self) {
					let file = probe.into_inner();

					return crate::tag_utils::write_tag(&Tag::new(*self), file, file_type).is_ok();
				}
			}
		}

		false
	}
}
