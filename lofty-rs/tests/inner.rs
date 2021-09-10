#![cfg(feature = "default")]

use lofty::{AudioTagEdit, Id3Format, OggFormat, OggTag, Tag, TagType, ToAnyTag};

use std::io::Cursor;

#[test]
fn test_inner() {
	// Create a new flac OggTag
	let mut flac_data = Cursor::new(std::fs::read("tests/assets/a.flac").unwrap());
	let mut flac_tag = OggTag::read_from(&mut flac_data, &OggFormat::Flac).unwrap();

	// Set the title of the flac tag
	flac_tag.set_title("Foo title");

	// Turn the VorbisTag into an Id3v2Tag
	let id3tag = flac_tag.to_dyn_tag(TagType::Id3v2(Id3Format::Mp3));

	// Write the Id3v2Tag to `a.mp3`
	id3tag
		.write_to_path("tests/assets/a.mp3")
		.expect("Fail to write!");

	// Read from `a.mp3`
	let id3tag_reload = Tag::new()
		.read_from_path("tests/assets/a.mp3")
		.expect("Fail to read!");

	// Confirm title still matches
	assert_eq!(id3tag_reload.title(), Some("Foo title"));

	// Convert Id3v2Tag to id3::Tag
	let mut id3tag_inner: id3::Tag = id3tag_reload.into();

	// Create timestamp and change date_recorded
	let timestamp = id3::Timestamp {
		year: 2013,
		month: Some(2_u8),
		day: Some(5_u8),
		hour: Some(6_u8),
		minute: None,
		second: None,
	};

	id3tag_inner.set_date_recorded(timestamp);

	// Write id3::Tag to `a.mp3`
	id3tag_inner
		.write_to_path("tests/assets/a.mp3", id3::Version::Id3v24)
		.expect("Fail to write!");

	// Read from `a.mp3`
	let id3tag_reload = id3::Tag::read_from_path("tests/assets/a.mp3").expect("Fail to read!");

	// Confirm timestamp still matches
	assert_eq!(id3tag_reload.date_recorded(), Some(timestamp));
}
