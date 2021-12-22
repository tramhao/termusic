use crate::{set_artist, temp_file, verify_artist};
use lofty::{FileType, ItemKey, ItemValue, TagItem, TagType};
use std::io::{Seek, SeekFrom, Write};

#[test]
fn read() {
	// Here we have a WAV file with both an ID3v2 chunk and a RIFF INFO chunk
	let file = lofty::read_from_path("tests/files/assets/a.wav", false).unwrap();

	assert_eq!(file.file_type(), &FileType::WAV);

	// Verify the ID3v2 tag first
	crate::verify_artist!(file, primary_tag, "Foo artist", 1);

	// Now verify the RIFF INFO chunk
	crate::verify_artist!(file, tag, TagType::RiffInfo, "Bar artist", 1);
}

#[test]
fn write() {
	let mut file = temp_file!("tests/files/assets/a.wav");

	let mut tagged_file = lofty::read_from(&mut file, false).unwrap();

	assert_eq!(tagged_file.file_type(), &FileType::WAV);

	// ID3v2
	crate::set_artist!(tagged_file, primary_tag_mut, "Foo artist", 1 => file, "Bar artist");

	// RIFF INFO
	crate::set_artist!(tagged_file, tag_mut, TagType::RiffInfo, "Bar artist", 1 => file, "Baz artist");

	// Now reread the file
	file.seek(SeekFrom::Start(0)).unwrap();
	let mut tagged_file = lofty::read_from(&mut file, false).unwrap();

	crate::set_artist!(tagged_file, primary_tag_mut, "Bar artist", 1 => file, "Foo artist");

	crate::set_artist!(tagged_file, tag_mut, TagType::RiffInfo, "Baz artist", 1 => file, "Bar artist");
}

#[test]
fn remove_id3v2() {
	crate::remove_tag!("tests/files/assets/a.wav", TagType::Id3v2);
}

#[test]
fn remove_riff_info() {
	crate::remove_tag!("tests/files/assets/a.wav", TagType::RiffInfo);
}
