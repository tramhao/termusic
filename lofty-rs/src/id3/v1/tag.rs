use crate::error::Result;
use crate::id3::v1::constants::GENRES;
use crate::types::item::{ItemKey, ItemValue, TagItem};
use crate::types::tag::{Accessor, Tag, TagType};

use std::fs::File;

macro_rules! impl_accessor {
	($($name:ident,)+) => {
		paste::paste! {
			$(
				fn $name(&self) -> Option<&str> {
					self.$name.as_deref()
				}

				fn [<set_ $name>](&mut self, value: String) {
					self.$name = Some(value)
				}

				fn [<remove_ $name>](&mut self) {
					self.$name = None
				}
			)+
		}
	}
}

#[derive(Default, Debug, PartialEq, Clone)]
/// An ID3v1 tag
///
/// ID3v1 is a severely limited format, with each field
/// being incredibly small in size. All fields have been
/// commented with their maximum sizes and any other additional
/// restrictions.
///
/// Attempting to write a field greater than the maximum size
/// will **not** error, it will just be shrunk.
///
/// ## Conversions
///
/// ### From `Tag`
///
/// Two checks are performed when converting a genre:
///
/// * [`GENRES`] contains the string
/// * The [`ItemValue`](crate::ItemValue) can be parsed into a `u8`
pub struct Id3v1Tag {
	/// Track title, 30 bytes max
	pub title: Option<String>,
	/// Track artist, 30 bytes max
	pub artist: Option<String>,
	/// Album title, 30 bytes max
	pub album: Option<String>,
	/// Release year, 4 bytes max
	pub year: Option<String>,
	/// A short comment
	///
	/// The number of bytes differs between versions, but not much.
	/// A V1 tag may have been read, which limits this field to 30 bytes.
	/// A V1.1 tag, however, only has 28 bytes available.
	///
	/// Lofty will *always* write a V1.1 tag.
	pub comment: Option<String>,
	/// The track number, 1 byte max
	///
	/// Issues:
	///
	/// * The track number **cannot** be 0. Many readers, including Lofty,
	/// look for a zeroed byte at the end of the comment to differentiate
	/// between V1 and V1.1.
	/// * A V1 tag may have been read, which does *not* have a track number.
	pub track_number: Option<u8>,
	/// The track's genre, 1 byte max
	///
	/// ID3v1 has a predefined set of genres, see [`GENRES`](crate::id3::v1::GENRES).
	/// This byte should be an index to a genre.
	pub genre: Option<u8>,
}

impl Accessor for Id3v1Tag {
	impl_accessor!(title, artist, album,);

	fn genre(&self) -> Option<&str> {
		if let Some(g) = self.genre {
			let g = g as usize;

			if g < GENRES.len() {
				return Some(GENRES[g]);
			}
		}

		None
	}

	fn remove_genre(&mut self) {
		self.genre = None
	}
}

impl Id3v1Tag {
	/// Returns `true` if the tag contains no data
	pub fn is_empty(&self) -> bool {
		self.title.is_none()
			&& self.artist.is_none()
			&& self.album.is_none()
			&& self.year.is_none()
			&& self.comment.is_none()
			&& self.track_number.is_none()
			&& self.genre.is_none()
	}

	/// Write the tag to a file
	///
	/// # Errors
	///
	/// * Attempting to write the tag to a format that does not support it
	pub fn write_to(&self, file: &mut File) -> Result<()> {
		Into::<Id3v1TagRef>::into(self).write_to(file)
	}
}

impl From<Id3v1Tag> for Tag {
	fn from(input: Id3v1Tag) -> Self {
		let mut tag = Self::new(TagType::Id3v1);

		input.title.map(|t| tag.insert_text(ItemKey::TrackTitle, t));
		input
			.artist
			.map(|a| tag.insert_text(ItemKey::TrackArtist, a));
		input.album.map(|a| tag.insert_text(ItemKey::AlbumTitle, a));
		input.year.map(|y| tag.insert_text(ItemKey::Year, y));
		input.comment.map(|c| tag.insert_text(ItemKey::Comment, c));

		if let Some(t) = input.track_number {
			tag.insert_item_unchecked(TagItem::new(
				ItemKey::TrackNumber,
				ItemValue::Text(t.to_string()),
			))
		}

		if let Some(genre_index) = input.genre {
			if let Some(genre) = GENRES.get(genre_index as usize) {
				tag.insert_text(ItemKey::Genre, (*genre).to_string());
			}
		}

		tag
	}
}

impl From<Tag> for Id3v1Tag {
	fn from(input: Tag) -> Self {
		Self {
			title: input.get_string(&ItemKey::TrackTitle).map(str::to_owned),
			artist: input.get_string(&ItemKey::TrackArtist).map(str::to_owned),
			album: input.get_string(&ItemKey::AlbumTitle).map(str::to_owned),
			year: input.get_string(&ItemKey::Year).map(str::to_owned),
			comment: input.get_string(&ItemKey::Comment).map(str::to_owned),
			track_number: input
				.get_string(&ItemKey::TrackNumber)
				.map(|g| g.parse::<u8>().ok())
				.and_then(|g| g),
			genre: input
				.get_string(&ItemKey::Genre)
				.map(|g| {
					GENRES
						.iter()
						.position(|v| v == &g)
						.map_or_else(|| g.parse::<u8>().ok(), |p| Some(p as u8))
				})
				.and_then(|g| g),
		}
	}
}

pub(crate) struct Id3v1TagRef<'a> {
	pub title: Option<&'a str>,
	pub artist: Option<&'a str>,
	pub album: Option<&'a str>,
	pub year: Option<&'a str>,
	pub comment: Option<&'a str>,
	pub track_number: Option<u8>,
	pub genre: Option<u8>,
}

impl<'a> Into<Id3v1TagRef<'a>> for &'a Id3v1Tag {
	fn into(self) -> Id3v1TagRef<'a> {
		Id3v1TagRef {
			title: self.title.as_deref(),
			artist: self.artist.as_deref(),
			album: self.album.as_deref(),
			year: self.year.as_deref(),
			comment: self.comment.as_deref(),
			track_number: self.track_number,
			genre: self.genre,
		}
	}
}

impl<'a> Into<Id3v1TagRef<'a>> for &'a Tag {
	fn into(self) -> Id3v1TagRef<'a> {
		Id3v1TagRef {
			title: self.get_string(&ItemKey::TrackTitle),
			artist: self.get_string(&ItemKey::TrackArtist),
			album: self.get_string(&ItemKey::AlbumTitle),
			year: self.get_string(&ItemKey::Year),
			comment: self.get_string(&ItemKey::Comment),
			track_number: self
				.get_string(&ItemKey::TrackNumber)
				.map(|g| g.parse::<u8>().ok())
				.and_then(|g| g),
			genre: self
				.get_string(&ItemKey::Genre)
				.map(|g| {
					GENRES
						.iter()
						.position(|v| v == &g)
						.map_or_else(|| g.parse::<u8>().ok(), |p| Some(p as u8))
				})
				.and_then(|g| g),
		}
	}
}

impl<'a> Id3v1TagRef<'a> {
	pub(crate) fn write_to(&self, file: &mut File) -> Result<()> {
		super::write::write_id3v1(file, self)
	}

	pub(super) fn is_empty(&self) -> bool {
		self.title.is_none()
			&& self.artist.is_none()
			&& self.album.is_none()
			&& self.year.is_none()
			&& self.comment.is_none()
			&& self.track_number.is_none()
			&& self.genre.is_none()
	}
}

#[cfg(test)]
mod tests {
	use crate::id3::v1::Id3v1Tag;
	use crate::{Tag, TagType};

	use std::io::Read;

	#[test]
	fn parse_id3v1() {
		let expected_tag = Id3v1Tag {
			title: Some(String::from("Foo title")),
			artist: Some(String::from("Bar artist")),
			album: Some(String::from("Baz album")),
			year: Some(String::from("1984")),
			comment: Some(String::from("Qux comment")),
			track_number: Some(1),
			genre: Some(32),
		};

		let mut tag = [0; 128];
		std::fs::File::open("tests/tags/assets/test.id3v1")
			.unwrap()
			.read_exact(&mut tag)
			.unwrap();

		let parsed_tag = crate::id3::v1::read::parse_id3v1(tag);

		assert_eq!(expected_tag, parsed_tag);
	}

	#[test]
	fn id3v1_to_tag() {
		let mut tag_bytes = [0; 128];
		std::fs::File::open("tests/tags/assets/test.id3v1")
			.unwrap()
			.read_exact(&mut tag_bytes)
			.unwrap();

		let id3v1 = crate::id3::v1::read::parse_id3v1(tag_bytes);

		let tag: Tag = id3v1.into();

		crate::tag_utils::test_utils::verify_tag(&tag, true, true);
	}

	#[test]
	fn tag_to_id3v1() {
		let tag = crate::tag_utils::test_utils::create_tag(TagType::Id3v1);

		let id3v1_tag: Id3v1Tag = tag.into();

		assert_eq!(id3v1_tag.title.as_deref(), Some("Foo title"));
		assert_eq!(id3v1_tag.artist.as_deref(), Some("Bar artist"));
		assert_eq!(id3v1_tag.album.as_deref(), Some("Baz album"));
		assert_eq!(id3v1_tag.comment.as_deref(), Some("Qux comment"));
		assert_eq!(id3v1_tag.track_number, Some(1));
		assert_eq!(id3v1_tag.genre, Some(32));
	}
}
