use lofty::{FileProperties, Tag};

use std::time::Duration;

const OPUS_PROPERTIES: FileProperties =
	FileProperties::new(Duration::from_millis(1428), Some(120), Some(48000), Some(2));

const VORBIS_PROPERTIES: FileProperties =
	FileProperties::new(Duration::from_millis(1450), Some(112), Some(48000), Some(2));

const FLAC_PROPERTIES: FileProperties = FileProperties::new(
	Duration::from_millis(1428),
	Some(35084),
	Some(48000),
	Some(2),
);

const AIFF_PROPERTIES: FileProperties = FileProperties::new(
	Duration::from_millis(1428),
	Some(1536),
	Some(48000),
	Some(2),
);

const RIFF_PROPERTIES: FileProperties = FileProperties::new(
	Duration::from_millis(1428),
	Some(1536),
	Some(48000),
	Some(2),
);

const MP3_PROPERTIES: FileProperties =
	FileProperties::new(Duration::from_millis(1464), Some(63), Some(48000), Some(2));

const MP4_PROPERTIES: FileProperties =
	FileProperties::new(Duration::from_millis(1450), Some(129), Some(48000), Some(2));

const APE_PROPERTIES: FileProperties =
	FileProperties::new(Duration::from_millis(1428), Some(360), Some(48000), Some(2));

macro_rules! properties_test {
	($function:ident, $path:expr, $expected:ident) => {
		#[test]
		fn $function() {
			let tag = Tag::new().read_from_path_signature($path).unwrap();
			let read_properties = tag.properties();

			assert_eq!(read_properties.duration(), $expected.duration());
			assert_eq!(read_properties.sample_rate(), $expected.sample_rate());
			assert_eq!(read_properties.channels(), $expected.channels());
		}
	};
}

properties_test!(test_aiff_id3, "tests/assets/a.aiff", AIFF_PROPERTIES);
properties_test!(test_aiff_text, "tests/assets/a_text.aiff", AIFF_PROPERTIES);

properties_test!(test_opus, "tests/assets/a.opus", OPUS_PROPERTIES);
properties_test!(test_vorbis, "tests/assets/a.ogg", VORBIS_PROPERTIES);
properties_test!(test_flac, "tests/assets/a.flac", FLAC_PROPERTIES);

properties_test!(test_wav_id3, "tests/assets/a-id3.wav", RIFF_PROPERTIES);
properties_test!(test_wav_info, "tests/assets/a.wav", RIFF_PROPERTIES);

properties_test!(test_mp3, "tests/assets/a.mp3", MP3_PROPERTIES);

properties_test!(test_mp4, "tests/assets/a.m4a", MP4_PROPERTIES);

properties_test!(test_ape, "tests/assets/a.ape", APE_PROPERTIES);
