#![cfg(feature = "default")]

use lofty::{Id3Format, OggFormat, Tag, TagType};

macro_rules! convert_tag {
	($tag: ident) => {
		assert_eq!($tag.title(), Some("Title Updated"));
		assert_eq!($tag.artist(), Some("Artist Updated"));
		assert_eq!($tag.track_number(), Some(5));
	};
}

#[test]
fn test_conversions() {
	let mut tag = Tag::new().read_from_path("tests/assets/a.mp3").unwrap();

	tag.set_title("Title Updated");
	tag.set_artist("Artist Updated");
	tag.set_track_number(5);
	convert_tag!(tag);

	let tag = tag.to_dyn_tag(TagType::Ape);
	convert_tag!(tag);

	let tag = tag.to_dyn_tag(TagType::Mp4);
	convert_tag!(tag);

	let tag = tag.to_dyn_tag(TagType::RiffInfo);
	convert_tag!(tag);

	let tag = tag.to_dyn_tag(TagType::Ogg(OggFormat::Vorbis));
	convert_tag!(tag);

	let tag = tag.to_dyn_tag(TagType::Id3v2(Id3Format::Aiff));
	convert_tag!(tag);
}
