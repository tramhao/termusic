use crate::{set_artist, temp_file, verify_artist};
use lofty::{FileType, ItemKey, ItemValue, TagItem, TagType};
use std::io::{Seek, SeekFrom, Write};

#[test]
fn read() {
	// Here we have an MP3 file with an ID3v2, ID3v1, and an APEv2 tag
	let file = lofty::read_from_path("tests/files/assets/a.mp3", false).unwrap();

	assert_eq!(file.file_type(), &FileType::MP3);

	// Verify the ID3v2 tag first
	crate::verify_artist!(file, primary_tag, "Foo artist", 1);

	// Now verify ID3v1
	crate::verify_artist!(file, tag, TagType::Id3v1, "Bar artist", 1);

	// Finally, verify APEv2
	crate::verify_artist!(file, tag, TagType::Ape, "Baz artist", 1);
}

#[test]
fn write() {
	let mut file = temp_file!("tests/files/assets/a.mp3");

	let mut tagged_file = lofty::read_from(&mut file, false).unwrap();

	assert_eq!(tagged_file.file_type(), &FileType::MP3);

	// ID3v2
	crate::set_artist!(tagged_file, primary_tag_mut, "Foo artist", 1 => file, "Bar artist");

	// ID3v1
	crate::set_artist!(tagged_file, tag_mut, TagType::Id3v1, "Bar artist", 1 => file, "Baz artist");

	// APEv2
	crate::set_artist!(tagged_file, tag_mut, TagType::Ape, "Baz artist", 1 => file, "Qux artist");

	// Now reread the file
	file.seek(SeekFrom::Start(0)).unwrap();
	let mut tagged_file = lofty::read_from(&mut file, false).unwrap();

	crate::set_artist!(tagged_file, primary_tag_mut, "Bar artist", 1 => file, "Foo artist");

	crate::set_artist!(tagged_file, tag_mut, TagType::Id3v1, "Baz artist", 1 => file, "Bar artist");

	crate::set_artist!(tagged_file, tag_mut, TagType::Ape, "Qux artist", 1 => file, "Baz artist");
}

#[test]
fn remove_id3v2() {
	crate::remove_tag!("tests/files/assets/a.mp3", TagType::Id3v2);
}

#[test]
fn remove_id3v1() {
	crate::remove_tag!("tests/files/assets/a.mp3", TagType::Id3v1);
}

#[test]
fn remove_ape() {
	crate::remove_tag!("tests/files/assets/a.mp3", TagType::Ape);
}
