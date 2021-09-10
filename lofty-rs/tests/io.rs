#![cfg(feature = "default")]

use lofty::{MimeType, Picture, PictureType, Tag};

use std::borrow::Cow;

macro_rules! full_test {
	($function:ident, $file:expr) => {
		#[test]
		#[allow(clippy::shadow_same)]
		fn $function() {
			println!("-- Adding tags --");
			add_tags!($file);
			println!("-- Verifying tags --");
			verify_write!($file);
			println!("-- Emptying tag --");
			empty_tag!($file);
			println!("-- Removing tag --");
			remove_tag!($file);
		}
	};
}

macro_rules! add_tags {
	($file:expr) => {
		println!("Reading file");
		let mut tag = Tag::default().read_from_path_signature($file).unwrap();

		let file = stringify!($file);

		println!("Setting title");
		tag.set_title("foo title");

		println!("Setting artist");
		tag.set_artist("foo artist");

		println!("Setting year");
		tag.set_year(2020);

		println!("Setting date");
		tag.set_date("20200415");

		println!("Setting copyright");
		tag.set_copyright("1988");

		println!("Setting genre");
		tag.set_genre("Country");

		if file != stringify!("tests/assets/a.mp3")
			&& file != stringify!("tests/assets/a.aiff")
			&& file != stringify!("tests/assets/a-id3.wav")
		{
			println!("Setting Lyrics");
			tag.set_lyrics("foo bar baz");
		}

		println!("Setting BPM");
		tag.set_bpm(50);

		println!("Setting lyricist");
		tag.set_lyricist("Serial-ATA");

		println!("Setting composer");
		tag.set_composer("Serial-ATA1");

		println!("Setting encoder");
		tag.set_encoder("Lofty");

		println!("Setting album title");
		tag.set_album_title("foo album title");

		println!("Setting album artists");
		tag.set_album_artist("foo album artist");

		let mut picture_data = vec![0x89, b'P', b'N', b'G'];
		let mut filler_data = vec![0; 50000];

		picture_data.append(&mut filler_data);

		let covers = (
			Picture {
				pic_type: PictureType::CoverFront,
				mime_type: MimeType::Png,
				description: Some(Cow::from("test")),
				width: 0,
				height: 0,
				color_depth: 0,
				num_colors: 0,
				data: Cow::from(picture_data.clone()),
			},
			Picture {
				pic_type: PictureType::CoverBack,
				mime_type: MimeType::Png,
				description: Some(Cow::from("test")),
				width: 0,
				height: 0,
				color_depth: 0,
				num_colors: 0,
				data: Cow::from(picture_data.clone()),
			},
		);

		// Skip this since RIFF INFO doesn't store images, and MP4 doesn't specify what pictures are
		if file != stringify!("tests/assets/a.wav") && file != stringify!("tests/assets/a.m4a") {
			println!("Setting front cover");
			tag.set_front_cover(covers.0.clone());
			assert_eq!(tag.front_cover(), Some(covers.0));

			println!("Setting back cover");
			tag.set_back_cover(covers.1.clone());
			assert_eq!(tag.back_cover(), Some(covers.1));
		}

		// All MP4 Pictures are PictureType::Other
		if file == stringify!("tests/assets/a.m4a") {
			let cover = Picture {
				pic_type: PictureType::Other,
				mime_type: MimeType::Png,
				description: None,
				width: 0,
				height: 0,
				color_depth: 0,
				num_colors: 0,
				data: Cow::from(picture_data),
			};

			println!("Setting cover");
			tag.set_front_cover(cover.clone());
			assert_eq!(tag.front_cover(), Some(cover));
		}

		println!("Writing");
		tag.write_to_path($file).unwrap();
	};
}

macro_rules! verify_write {
	($file:expr) => {
		println!("Reading file");
		let tag = Tag::default().read_from_path_signature($file).unwrap();

		let file_name = stringify!($file);

		println!("Verifying title");
		assert_eq!(tag.title(), Some("foo title"));

		println!("Verifying artist");
		assert_eq!(tag.artist(), Some("foo artist"));

		// Skip this since RIFF INFO doesn't support year
		if file_name != stringify!("tests/assets/a.wav") {
			println!("Verifying year");
			assert_eq!(tag.year(), Some(2020));
		}

		if file_name != stringify!("tests/assets/a.m4a") {
			println!("Verifying date");
			assert_eq!(tag.date(), Some("20200415".to_string()));

			if file_name != stringify!("tests/assets/a.wav") {
				println!("Verifying lyricist");
				assert_eq!(tag.lyricist(), Some("Serial-ATA"));

				println!("Verifying composer");
				assert_eq!(tag.composer(), Some("Serial-ATA1"));
			}
		}

		println!("Verifying copyright");
		assert_eq!(tag.copyright(), Some("1988"));

		println!("Verifying genre");
		assert_eq!(tag.genre(), Some("Country"));

		println!("Verifying encoder");
		assert_eq!(tag.encoder(), Some("Lofty"));

		println!("Verifying album title");
		assert_eq!(tag.album_title(), Some("foo album title"));

		let mut picture_data = vec![0x89, b'P', b'N', b'G'];
		let mut filler_data = vec![0; 50000];

		picture_data.append(&mut filler_data);

		// Skip this since RIFF INFO doesn't store images
		if file_name != stringify!("tests/assets/a.wav") {
			let covers = if file_name == stringify!("tests/assets/a.m4a") {
				(
					Picture {
						pic_type: PictureType::Other,
						mime_type: MimeType::Png,
						description: None,
						width: 0,
						height: 0,
						color_depth: 0,
						num_colors: 0,
						data: Cow::from(picture_data.clone()),
					},
					Picture {
						pic_type: PictureType::Other,
						mime_type: MimeType::Png,
						description: None,
						width: 0,
						height: 0,
						color_depth: 0,
						num_colors: 0,
						data: Cow::from(picture_data),
					},
				)
			} else {
				(
					Picture {
						pic_type: PictureType::CoverFront,
						mime_type: MimeType::Png,
						description: Some(Cow::from("test")),
						width: 0,
						height: 0,
						color_depth: 0,
						num_colors: 0,
						data: Cow::from(picture_data.clone()),
					},
					Picture {
						pic_type: PictureType::CoverBack,
						mime_type: MimeType::Png,
						description: Some(Cow::from("test")),
						width: 0,
						height: 0,
						color_depth: 0,
						num_colors: 0,
						data: Cow::from(picture_data),
					},
				)
			};

			if file_name != stringify!("tests/assets/a.mp3")
				&& file_name != stringify!("tests/assets/a.aiff")
				&& file_name != stringify!("tests/assets/a-id3.wav")
			{
				println!("Verifying lyrics");
				assert_eq!(tag.lyrics(), Some("foo bar baz"));
			}

			println!("Verifying BPM");
			assert_eq!(tag.bpm(), Some(50));

			println!("Verifying album artist");
			assert_eq!(tag.album_artist(), Some("foo album artist"));

			println!("Verifying album covers");

			println!("Verifying front cover");
			assert_eq!(tag.front_cover(), Some(covers.0));

			println!("Verifying back cover");
			assert_eq!(tag.back_cover(), Some(covers.1));
		}
	};
}

macro_rules! empty_tag {
	($file:expr) => {
		println!("Reading file");
		let mut tag = Tag::default().read_from_path_signature($file).unwrap();

		println!("Removing title");
		tag.remove_title();
		assert!(tag.title().is_none());
		tag.remove_title(); // should not panic

		println!("Removing artist");
		tag.remove_artist();
		assert!(tag.artist().is_none());
		tag.remove_artist();

		println!("Removing year");
		tag.remove_year();
		assert!(tag.year().is_none());
		tag.remove_year();

		println!("Removing date");
		tag.remove_date();
		assert!(tag.date().is_none());
		tag.remove_date();

		println!("Removing copyright");
		tag.remove_copyright();
		assert!(tag.copyright().is_none());
		tag.remove_copyright();

		println!("Removing genre");
		tag.remove_genre();
		assert!(tag.genre().is_none());
		tag.remove_genre();

		println!("Removing lyricist");
		tag.remove_lyricist();
		assert!(tag.lyricist().is_none());
		tag.remove_lyricist();

		println!("Removing composer");
		tag.remove_composer();
		assert!(tag.composer().is_none());
		tag.remove_composer();

		println!("Removing lyrics");
		tag.remove_lyrics();
		assert!(tag.lyrics().is_none());
		tag.remove_lyrics();

		println!("Removing BPM");
		tag.remove_bpm();
		assert!(tag.bpm().is_none());
		tag.remove_bpm();

		println!("Removing encoder");
		tag.remove_encoder();
		assert!(tag.encoder().is_none());
		tag.remove_encoder();

		println!("Removing album title");
		tag.remove_album_title();
		assert!(tag.album_title().is_none());
		tag.remove_album_title();

		println!("Removing album artists");
		tag.remove_album_artist();
		assert!(tag.album_artist().is_none());
		tag.remove_album_artist();

		println!("Removing album covers");
		tag.remove_album_covers();
		assert_eq!(tag.album_covers(), (None, None));
		tag.remove_album_covers();

		println!("Writing");
		tag.write_to_path($file).unwrap();
	};
}

macro_rules! remove_tag {
	($file:expr) => {
		Tag::new().remove_from_path($file).unwrap();
	};
}

// APEv2
full_test!(test_ape, "tests/assets/a.ape");

// ID3v2
full_test!(test_mp3, "tests/assets/a.mp3");
full_test!(test_aiff, "tests/assets/a.aiff");
full_test!(test_wav_id3, "tests/assets/a-id3.wav");

// RIFF INFO
full_test!(test_wav_riff_info, "tests/assets/a.wav");

// Vorbis comments
full_test!(test_flac, "tests/assets/a.flac");
full_test!(test_m4a, "tests/assets/a.m4a");
full_test!(test_ogg, "tests/assets/a.ogg");
full_test!(test_opus, "tests/assets/a.opus");

// AIFF text chunks only provide 2 values
#[test]
fn test_aiff_text() {
	let file = "tests/assets/a_text.aiff";
	println!("-- Adding tags --");

	println!("Reading file");
	let mut tag = Tag::default().read_from_path_signature(file).unwrap();

	println!("Setting title");
	tag.set_title("foo title");
	println!("Setting artist");
	tag.set_artist("foo artist");
	println!("Setting copyright");
	tag.set_copyright("1988");

	println!("Writing");
	tag.write_to_path(file).unwrap();

	println!("-- Verifying tags --");

	println!("Reading file");
	let mut tag = Tag::default().read_from_path_signature(file).unwrap();

	println!("Verifying title");
	assert_eq!(tag.title(), Some("foo title"));
	println!("Verifying artist");
	assert_eq!(tag.artist(), Some("foo artist"));
	println!("Verifying copyright");
	assert_eq!(tag.copyright(), Some("1988"));

	println!("-- Removing tags --");

	println!("Removing title");
	tag.remove_title();

	// Keep artist around so there's something to read
	// println!("Removing artist");
	// tag.remove_artist();

	println!("Removing copyright");
	tag.remove_copyright();

	println!("Writing");
	tag.write_to_path(file).unwrap()
}
