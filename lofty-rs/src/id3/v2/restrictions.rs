#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
/// Restrictions on the tag size
pub enum TagSizeRestrictions {
	/// No more than 128 frames and 1 MB total tag size
	S_128F_1M,
	/// No more than 64 frames and 128 KB total tag size
	S_64F_128K,
	/// No more than 32 frames and 40 KB total tag size
	S_32F_40K,
	/// No more than 32 frames and 4 KB total tag size
	S_32F_4K,
}

impl Default for TagSizeRestrictions {
	fn default() -> Self {
		Self::S_128F_1M
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
/// Restrictions on text field sizes
pub enum TextSizeRestrictions {
	/// No size restrictions
	None,
	/// No longer than 1024 characters
	C_1024,
	/// No longer than 128 characters
	C_128,
	/// No longer than 30 characters
	C_30,
}

impl Default for TextSizeRestrictions {
	fn default() -> Self {
		Self::None
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
/// Restrictions on all image sizes
pub enum ImageSizeRestrictions {
	/// No size restrictions
	None,
	/// All images are 256x256 or smaller
	P_256,
	/// All images are 64x64 or smaller
	P_64,
	/// All images are **exactly** 64x64
	P_64_64,
}

impl Default for ImageSizeRestrictions {
	fn default() -> Self {
		Self::None
	}
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
/// Restrictions on the content of an ID3v2 tag
pub struct TagRestrictions {
	/// Restriction on the size of the tag. See [`TagSizeRestrictions`]
	pub size: TagSizeRestrictions,
	/// Text encoding restrictions
	///
	/// `false` - No restrictions
	/// `true` - Strings are only encoded with [`TextEncoding::Latin1`](crate::id3::v2::TextEncoding::Latin1) or [`TextEncoding::UTF8`](crate::id3::v2::TextEncoding::UTF8)
	pub text_encoding: bool,
	/// Restrictions on all text field sizes. See [`TextSizeRestrictions`]
	pub text_fields_size: TextSizeRestrictions,
	/// Image encoding restrictions
	///
	/// `false` - No restrictions
	/// `true` - Images can only be `PNG` or `JPEG`
	pub image_encoding: bool,
	/// Restrictions on all image sizes. See [`ImageSizeRestrictions`]
	pub image_size: ImageSizeRestrictions,
}

impl TagRestrictions {
	/// Read a [`TagRestrictions`] from a byte
	///
	/// NOTE: See <https://id3.org/id3v2.4.0-structure> section 3.2, item d
	pub fn from_byte(byte: u8) -> Self {
		let mut restrictions = TagRestrictions::default();

		let restriction_flags = byte;

		// xx000000
		match (
			restriction_flags & 0x80 == 0x80,
			restriction_flags & 0x40 == 0x40,
		) {
			(false, false) => {}, // default
			(false, true) => restrictions.size = TagSizeRestrictions::S_64F_128K,
			(true, false) => restrictions.size = TagSizeRestrictions::S_32F_40K,
			(true, true) => restrictions.size = TagSizeRestrictions::S_32F_4K,
		}

		// 00x00000
		if restriction_flags & 0x20 == 0x20 {
			restrictions.text_encoding = true
		}

		// 000xx000
		match (
			restriction_flags & 0x10 == 0x10,
			restriction_flags & 0x08 == 0x08,
		) {
			(false, false) => {}, // default
			(false, true) => restrictions.text_fields_size = TextSizeRestrictions::C_1024,
			(true, false) => restrictions.text_fields_size = TextSizeRestrictions::C_128,
			(true, true) => restrictions.text_fields_size = TextSizeRestrictions::C_30,
		}

		// 00000x00
		if restriction_flags & 0x04 == 0x04 {
			restrictions.image_encoding = true
		}

		// 000000xx
		match (
			restriction_flags & 0x02 == 0x02,
			restriction_flags & 0x01 == 0x01,
		) {
			(false, false) => {}, // default
			(false, true) => restrictions.image_size = ImageSizeRestrictions::P_256,
			(true, false) => restrictions.image_size = ImageSizeRestrictions::P_64,
			(true, true) => restrictions.image_size = ImageSizeRestrictions::P_64_64,
		}

		restrictions
	}

	#[allow(clippy::trivially_copy_pass_by_ref)]
	/// Convert a [`TagRestrictions`] into a `u8`
	pub fn as_bytes(&self) -> u8 {
		let mut byte = 0;

		match self.size {
			TagSizeRestrictions::S_128F_1M => {},
			TagSizeRestrictions::S_64F_128K => byte |= 0x40,
			TagSizeRestrictions::S_32F_40K => byte |= 0x80,
			TagSizeRestrictions::S_32F_4K => {
				byte |= 0x80;
				byte |= 0x40;
			},
		}

		if self.text_encoding {
			byte |= 0x20
		}

		match self.text_fields_size {
			TextSizeRestrictions::None => {},
			TextSizeRestrictions::C_1024 => byte |= 0x08,
			TextSizeRestrictions::C_128 => byte |= 0x10,
			TextSizeRestrictions::C_30 => {
				byte |= 0x10;
				byte |= 0x08;
			},
		}

		if self.image_encoding {
			byte |= 0x04
		}

		match self.image_size {
			ImageSizeRestrictions::None => {},
			ImageSizeRestrictions::P_256 => byte |= 0x01,
			ImageSizeRestrictions::P_64 => byte |= 0x02,
			ImageSizeRestrictions::P_64_64 => {
				byte |= 0x02;
				byte |= 0x01;
			},
		}

		byte
	}
}
