use crate::mp4::AtomIdent;
use crate::types::picture::Picture;

#[derive(Debug, PartialEq, Clone)]
/// Represents an `MP4` atom
pub struct Atom {
	pub(crate) ident: AtomIdent,
	pub(crate) data: AtomData,
}

impl Atom {
	/// Create a new [`Atom`]
	pub fn new(ident: AtomIdent, data: AtomData) -> Self {
		Self { ident, data }
	}

	/// Returns the atom's [`AtomIdent`]
	pub fn ident(&self) -> &AtomIdent {
		&self.ident
	}

	/// Returns the atom's [`AtomData`]
	pub fn data(&self) -> &AtomData {
		&self.data
	}
}

#[derive(Debug, PartialEq, Clone)]
/// The data of an atom
///
/// NOTES:
///
/// * This only covers the most common data types.
/// See the list of [well-known data types](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html#//apple_ref/doc/uid/TP40000939-CH1-SW34)
/// for codes.
/// * There are only two variants for integers, which
/// will come from codes `21` and `22`. All other integer
/// types will be stored as [`AtomData::Unknown`], refer
/// to the link above for codes.
pub enum AtomData {
	/// A UTF-8 encoded string
	UTF8(String),
	/// A UTf-16 encoded string
	UTF16(String),
	/// A JPEG, PNG, GIF *(Deprecated)*, or BMP image
	///
	/// The type is read from the picture itself
	Picture(Picture),
	/// A big endian signed integer (1-4 bytes)
	SignedInteger(i32),
	/// A big endian unsigned integer (1-4 bytes)
	UnsignedInteger(u32),
	/// Unknown data
	///
	/// Due to the number of possible types, there are many
	/// **specified** types that are going to fall into this
	/// variant.
	Unknown {
		/// The code, or type of the item
		code: u32,
		/// The binary data of the atom
		data: Vec<u8>,
	},
}

pub(crate) struct AtomRef<'a> {
	pub(crate) ident: AtomIdentRef<'a>,
	pub(crate) data: AtomDataRef<'a>,
}

impl<'a> Into<AtomRef<'a>> for &'a Atom {
	fn into(self) -> AtomRef<'a> {
		AtomRef {
			ident: (&self.ident).into(),
			data: (&self.data).into(),
		}
	}
}

pub(crate) enum AtomIdentRef<'a> {
	Fourcc([u8; 4]),
	Freeform { mean: &'a str, name: &'a str },
}

impl<'a> Into<AtomIdentRef<'a>> for &'a AtomIdent {
	fn into(self) -> AtomIdentRef<'a> {
		match self {
			AtomIdent::Fourcc(fourcc) => AtomIdentRef::Fourcc(*fourcc),
			AtomIdent::Freeform { mean, name } => AtomIdentRef::Freeform { mean, name },
		}
	}
}

impl<'a> From<AtomIdentRef<'a>> for AtomIdent {
	fn from(input: AtomIdentRef<'a>) -> Self {
		match input {
			AtomIdentRef::Fourcc(fourcc) => AtomIdent::Fourcc(fourcc),
			AtomIdentRef::Freeform { mean, name } => AtomIdent::Freeform {
				mean: mean.to_string(),
				name: name.to_string(),
			},
		}
	}
}

pub(crate) enum AtomDataRef<'a> {
	UTF8(&'a str),
	UTF16(&'a str),
	Picture(&'a Picture),
	SignedInteger(i32),
	UnsignedInteger(u32),
	Unknown { code: u32, data: &'a [u8] },
}

impl<'a> Into<AtomDataRef<'a>> for &'a AtomData {
	fn into(self) -> AtomDataRef<'a> {
		match self {
			AtomData::UTF8(utf8) => AtomDataRef::UTF8(utf8),
			AtomData::UTF16(utf16) => AtomDataRef::UTF16(utf16),
			AtomData::Picture(pic) => AtomDataRef::Picture(pic),
			AtomData::SignedInteger(int) => AtomDataRef::SignedInteger(*int),
			AtomData::UnsignedInteger(uint) => AtomDataRef::UnsignedInteger(*uint),
			AtomData::Unknown { code, data } => AtomDataRef::Unknown { code: *code, data },
		}
	}
}
