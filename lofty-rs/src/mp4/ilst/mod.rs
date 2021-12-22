pub(super) mod atom;
pub(super) mod read;
pub(crate) mod write;

use super::AtomIdent;
use crate::error::Result;
use crate::types::item::{ItemKey, ItemValue, TagItem};
use crate::types::picture::{Picture, PictureType};
use crate::types::tag::{Accessor, Tag, TagType};
use atom::{Atom, AtomData, AtomDataRef, AtomIdentRef, AtomRef};

use std::convert::TryInto;
use std::fs::File;

const ARTIST: AtomIdent = AtomIdent::Fourcc(*b"\xa9ART");
const TITLE: AtomIdent = AtomIdent::Fourcc(*b"\xa9nam");
const ALBUM: AtomIdent = AtomIdent::Fourcc(*b"\xa9alb");
const ALBUM_ARTIST: AtomIdent = AtomIdent::Fourcc(*b"aART");
const GENRE: AtomIdent = AtomIdent::Fourcc(*b"\xa9gen");

macro_rules! impl_accessor {
	($($name:ident, $const:ident;)+) => {
		paste::paste! {
			impl Accessor for Ilst {
				$(
					fn $name(&self) -> Option<&str> {
						if let Some(atom) = self.atom(&$const) {
							if let AtomData::UTF8(val) | AtomData::UTF16(val) = atom.data() {
								return Some(val)
							}
						}

						None
					}

					fn [<set_ $name>](&mut self, value: String) {
						self.replace_atom(Atom {
							ident: $const,
							data: AtomData::UTF8(value),
						})
					}

					fn [<remove_ $name>](&mut self) {
						self.remove_atom(&$const)
					}
				)+
			}
		}
	}
}

#[derive(Default, PartialEq, Debug, Clone)]
/// An MP4 ilst atom
///
/// ## Supported file types
///
/// * [`FileType::MP4`](crate::FileType::MP4)
///
/// ## Pictures
///
/// Unlike other formats, ilst does not store a [`PictureType`]. All pictures will have
/// [PictureType::Other].
///
/// ## Conversions
///
/// ### To `Tag`
///
/// When converting to [`Tag`], only atoms with a value of [AtomData::UTF8] and [AtomData::UTF16], as
/// well as pictures, will be preserved.
///
/// Do note, all pictures will be [PictureType::Other](crate::PictureType::Other)
///
/// ### From `Tag`
///
/// When converting from [`Tag`], only items with a value of [`ItemValue::Text`](crate::ItemValue::Text), as
/// well as pictures, will be preserved
pub struct Ilst {
	pub(crate) atoms: Vec<Atom>,
}

impl_accessor!(
	artist,       ARTIST;
	title,        TITLE;
	album,        ALBUM;
	album_artist, ALBUM_ARTIST;
	genre,        GENRE;
);

impl Ilst {
	/// Get an item by its [`AtomIdent`]
	pub fn atom(&self, ident: &AtomIdent) -> Option<&Atom> {
		self.atoms.iter().find(|a| &a.ident == ident)
	}

	/// Inserts an [`Atom`]
	pub fn insert_atom(&mut self, atom: Atom) {
		self.atoms.push(atom);
	}

	/// Inserts an [`Atom`], replacing any atom with the same [`AtomIdent`]
	pub fn replace_atom(&mut self, atom: Atom) {
		self.remove_atom(&atom.ident);
		self.atoms.push(atom);
	}

	/// Remove an atom by its [`AtomIdent`]
	pub fn remove_atom(&mut self, ident: &AtomIdent) {
		self.atoms
			.iter()
			.position(|a| &a.ident == ident)
			.map(|p| self.atoms.remove(p));
	}

	/// Returns all pictures
	pub fn pictures(&self) -> impl Iterator<Item = &Picture> {
		const COVR: AtomIdent = AtomIdent::Fourcc(*b"covr");

		self.atoms.iter().filter_map(|a| match a {
			Atom {
				ident: COVR,
				data: AtomData::Picture(pic),
			} => Some(pic),
			_ => None,
		})
	}

	/// Inserts a picture
	pub fn insert_picture(&mut self, mut picture: Picture) {
		// This is just for correctness, it doesn't really matter.
		picture.pic_type = PictureType::Other;

		self.atoms.push(Atom {
			ident: AtomIdent::Fourcc(*b"covr"),
			data: AtomData::Picture(picture),
		})
	}

	/// Removes all pictures
	pub fn remove_pictures(&mut self) {
		self.atoms
			.retain(|a| !matches!(a.data(), AtomData::Picture(_)))
	}
}

impl Ilst {
	/// Writes the tag to a file
	///
	/// # Errors
	///
	/// * Attempting to write the tag to a format that does not support it
	pub fn write_to(&self, file: &mut File) -> Result<()> {
		Into::<IlstRef>::into(self).write_to(file)
	}
}

impl From<Ilst> for Tag {
	fn from(input: Ilst) -> Self {
		let mut tag = Self::new(TagType::Mp4Ilst);

		for atom in input.atoms {
			let value = match atom.data {
				AtomData::UTF8(text) | AtomData::UTF16(text) => ItemValue::Text(text),
				AtomData::Picture(pic) => {
					tag.pictures.push(pic);
					continue;
				}
				_ => continue,
			};

			let key = ItemKey::from_key(
				TagType::Mp4Ilst,
				&match atom.ident {
					AtomIdent::Fourcc(fourcc) => {
						fourcc.iter().map(|b| *b as char).collect::<String>()
					}
					AtomIdent::Freeform { mean, name } => {
						format!("----:{}:{}", mean, name)
					}
				},
			);

			tag.items.push(TagItem::new(key, value));
		}

		tag
	}
}

impl From<Tag> for Ilst {
	fn from(input: Tag) -> Self {
		let mut ilst = Self::default();

		for item in input.items {
			if let Some(ident) = item_key_to_ident(item.key()).map(Into::into) {
				let data = match item.item_value {
					ItemValue::Text(text) => AtomData::UTF8(text),
					_ => continue,
				};

				ilst.atoms.push(Atom { ident, data });
			}
		}

		for mut picture in input.pictures {
			// Just for correctness, since we can't actually
			// assign a picture type in this format
			picture.pic_type = PictureType::Other;

			ilst.atoms.push(Atom {
				ident: AtomIdent::Fourcc([b'c', b'o', b'v', b'r']),
				data: AtomData::Picture(picture),
			})
		}

		ilst
	}
}

pub(crate) struct IlstRef<'a> {
	atoms: Box<dyn Iterator<Item = AtomRef<'a>> + 'a>,
}

impl<'a> IlstRef<'a> {
	pub(crate) fn write_to(&mut self, file: &mut File) -> Result<()> {
		write::write_to(file, self)
	}
}

impl<'a> Into<IlstRef<'a>> for &'a Ilst {
	fn into(self) -> IlstRef<'a> {
		IlstRef {
			atoms: Box::new(self.atoms.iter().map(Into::into)),
		}
	}
}

impl<'a> Into<IlstRef<'a>> for &'a Tag {
	fn into(self) -> IlstRef<'a> {
		let iter =
			self.items
				.iter()
				.filter_map(|i| match (item_key_to_ident(i.key()), i.value()) {
					(Some(ident), ItemValue::Text(text)) => Some(AtomRef {
						ident,
						data: AtomDataRef::UTF8(text),
					}),
					_ => None,
				});

		IlstRef {
			atoms: Box::new(iter),
		}
	}
}

fn item_key_to_ident(key: &ItemKey) -> Option<AtomIdentRef> {
	key.map_key(TagType::Mp4Ilst, true).and_then(|ident| {
		if ident.starts_with("----") {
			let mut split = ident.split(':');

			split.next();

			let mean = split.next();
			let name = split.next();

			if let (Some(mean), Some(name)) = (mean, name) {
				Some(AtomIdentRef::Freeform { mean, name })
			} else {
				None
			}
		} else {
			let fourcc = ident.chars().map(|c| c as u8).collect::<Vec<_>>();

			if let Ok(fourcc) = TryInto::<[u8; 4]>::try_into(fourcc) {
				Some(AtomIdentRef::Fourcc(fourcc))
			} else {
				None
			}
		}
	})
}

#[cfg(test)]
mod tests {
	use crate::mp4::{Atom, AtomData, AtomIdent, Ilst};
	use crate::{Tag, TagType};

	use std::io::Read;

	#[test]
	fn parse_ilst() {
		let mut expected_tag = Ilst::default();

		// The track number is stored with a code 0,
		// meaning the there is no need to indicate the type,
		// which is `u64` in this case
		expected_tag.insert_atom(Atom::new(
			AtomIdent::Fourcc(*b"trkn"),
			AtomData::Unknown {
				code: 0,
				data: vec![0, 0, 0, 1, 0, 0, 0, 0],
			},
		));

		expected_tag.insert_atom(Atom::new(
			AtomIdent::Fourcc(*b"\xa9ART"),
			AtomData::UTF8(String::from("Bar artist")),
		));

		expected_tag.insert_atom(Atom::new(
			AtomIdent::Fourcc(*b"\xa9alb"),
			AtomData::UTF8(String::from("Baz album")),
		));

		expected_tag.insert_atom(Atom::new(
			AtomIdent::Fourcc(*b"\xa9cmt"),
			AtomData::UTF8(String::from("Qux comment")),
		));

		expected_tag.insert_atom(Atom::new(
			AtomIdent::Fourcc(*b"\xa9day"),
			AtomData::UTF8(String::from("1984")),
		));

		expected_tag.insert_atom(Atom::new(
			AtomIdent::Fourcc(*b"\xa9gen"),
			AtomData::UTF8(String::from("Classical")),
		));

		expected_tag.insert_atom(Atom::new(
			AtomIdent::Fourcc(*b"\xa9nam"),
			AtomData::UTF8(String::from("Foo title")),
		));

		let mut tag = Vec::new();
		std::fs::File::open("tests/tags/assets/test.ilst")
			.unwrap()
			.read_to_end(&mut tag)
			.unwrap();

		let parsed_tag = super::read::parse_ilst(&mut &tag[..], tag.len() as u64).unwrap();

		assert_eq!(expected_tag, parsed_tag);
	}

	#[test]
	fn ilst_to_tag() {
		let mut tag_bytes = Vec::new();
		std::fs::File::open("tests/tags/assets/test.ilst")
			.unwrap()
			.read_to_end(&mut tag_bytes)
			.unwrap();

		let ilst = super::read::parse_ilst(&mut &tag_bytes[..], tag_bytes.len() as u64).unwrap();

		let tag: Tag = ilst.into();

		crate::tag_utils::test_utils::verify_tag(&tag, false, true);
	}

	#[test]
	fn tag_to_ilst() {
		fn verify_atom(ilst: &Ilst, ident: [u8; 4], data: &str) {
			let atom = ilst.atom(&AtomIdent::Fourcc(ident)).unwrap();

			let data = AtomData::UTF8(String::from(data));

			assert_eq!(atom.data(), &data);
		}

		let tag = crate::tag_utils::test_utils::create_tag(TagType::Mp4Ilst);

		let ilst: Ilst = tag.into();

		verify_atom(&ilst, *b"\xa9nam", "Foo title");
		verify_atom(&ilst, *b"\xa9ART", "Bar artist");
		verify_atom(&ilst, *b"\xa9alb", "Baz album");
		verify_atom(&ilst, *b"\xa9cmt", "Qux comment");
		verify_atom(&ilst, *b"\xa9gen", "Classical");
	}
}
