#[macro_export]
macro_rules! temp_file {
	($path:tt) => {{
		let mut file = tempfile::tempfile().unwrap();
		file.write_all(&std::fs::read($path).unwrap()).unwrap();

		file.seek(std::io::SeekFrom::Start(0)).unwrap();

		file
	}};
}

#[macro_export]
macro_rules! verify_artist {
	($file:ident, $method:ident, $expected_value:literal, $item_count:expr) => {{
		println!("VERIFY: Expecting `{}` to have {} items, with an artist of \"{}\"", stringify!($method), $item_count, $expected_value);

		verify_artist!($file, $method(), $expected_value, $item_count)
	}};
	($file:ident, $method:ident, $arg:path, $expected_value:literal, $item_count:expr) => {{
		println!("VERIFY: Expecting `{}` to have {} items, with an artist of \"{}\"", stringify!($arg), $item_count, $expected_value);

		verify_artist!($file, $method($arg), $expected_value, $item_count)
	}};
	($file:ident, $method:ident($($arg:path)?), $expected_value:literal, $item_count:expr) => {{
		assert!($file.$method($(&$arg)?).is_some());

		let tag = $file.$method($(&$arg)?).unwrap();

		assert_eq!(tag.item_count(), $item_count);

		assert_eq!(
			tag.get_item_ref(&ItemKey::TrackArtist),
			Some(&TagItem::new(
				ItemKey::TrackArtist,
				ItemValue::Text(String::from($expected_value))
			))
		);

		tag
	}};
}

#[macro_export]
macro_rules! set_artist {
	($tagged_file:ident, $method:ident, $expected_value:literal, $item_count:expr => $file_write:ident, $new_value:literal) => {
		let tag = verify_artist!($tagged_file, $method, $expected_value, $item_count);
		println!(
			"WRITE: Writing artist \"{}\" to {}\n",
			$new_value,
			stringify!($method)
		);
		set_artist!($file_write, $new_value, tag)
	};
	($tagged_file:ident, $method:ident, $arg:path, $expected_value:literal, $item_count:expr => $file_write:ident, $new_value:literal) => {
		let tag = verify_artist!($tagged_file, $method, $arg, $expected_value, $item_count);
		println!(
			"WRITE: Writing artist \"{}\" to {}\n",
			$new_value,
			stringify!($arg)
		);
		set_artist!($file_write, $new_value, tag)
	};
	($file_write:ident, $new_value:literal, $tag:ident) => {
		$tag.insert_item_unchecked(TagItem::new(
			ItemKey::TrackArtist,
			ItemValue::Text(String::from($new_value)),
		));

		$file_write.seek(std::io::SeekFrom::Start(0)).unwrap();

		assert!($tag.save_to(&mut $file_write).is_ok());
	};
}

#[macro_export]
macro_rules! remove_tag {
	($path:tt, $tag_type:path) => {
		let mut file = temp_file!($path);

		let tagged_file = lofty::read_from(&mut file, false).unwrap();
		assert!(tagged_file.tag(&$tag_type).is_some());

		file.seek(std::io::SeekFrom::Start(0)).unwrap();

		assert!($tag_type.remove_from(&mut file));

		file.seek(std::io::SeekFrom::Start(0)).unwrap();

		let tagged_file = lofty::read_from(&mut file, false).unwrap();
		assert!(tagged_file.tag(&$tag_type).is_none());
	};
}
