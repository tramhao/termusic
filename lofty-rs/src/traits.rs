#[allow(clippy::wildcard_imports)]
use crate::components::tags::*;
use crate::{Album, AnyTag, FileProperties, Picture, Result, TagType};

use std::borrow::Cow;
use std::fs::{File, OpenOptions};

use lofty_attr::{i32_accessor, str_accessor, u16_accessor, u32_accessor};

/// Combination of [`AudioTagEdit`], [`AudioTagWrite`], and [`ToAnyTag`]
pub trait AudioTag: AudioTagEdit + AudioTagWrite + ToAnyTag {}

/// Implementors of this trait are able to read and write audio metadata.
///
/// Constructor methods e.g. `from_file` should be implemented separately.
pub trait AudioTagEdit {
	str_accessor!(title);
	str_accessor!(artist);

	/// Splits the artist string into a `Vec`
	fn artists(&self, delimiter: &str) -> Option<Vec<&str>> {
		self.artist().map(|a| a.split(delimiter).collect())
	}

	i32_accessor!(year);

	/// Returns the date
	fn date(&self) -> Option<String> {
		self.year().map(|y| y.to_string())
	}
	/// Sets the date
	fn set_date(&mut self, _date: &str) {}
	/// Removes the date
	fn remove_date(&mut self) {}

	str_accessor!(copyright);
	str_accessor!(genre);
	str_accessor!(lyrics);

	u16_accessor!(bpm);

	str_accessor!(lyricist);
	str_accessor!(composer);

	str_accessor!(encoder);

	/// Returns the track's [`Album`]
	fn album(&self) -> Album<'_> {
		Album {
			title: self.album_title(),
			artist: self.album_artist(),
			covers: self.album_covers(),
		}
	}

	str_accessor!(album_title);
	str_accessor!(album_artist);

	/// Splits the artist string into a `Vec`
	fn album_artists(&self, delimiter: &str) -> Option<Vec<&str>> {
		self.album_artist().map(|a| a.split(delimiter).collect())
	}

	/// Returns the front and back album covers
	fn album_covers(&self) -> (Option<Picture>, Option<Picture>) {
		(self.front_cover(), self.back_cover())
	}
	/// Removes both album covers
	fn remove_album_covers(&mut self) {
		self.remove_front_cover();
		self.remove_back_cover();
	}

	/// Returns the front cover
	fn front_cover(&self) -> Option<Picture> {
		None
	}
	/// Sets the front cover
	fn set_front_cover(&mut self, _cover: Picture) {}
	/// Removes the front cover
	fn remove_front_cover(&mut self) {}

	/// Returns the front cover
	fn back_cover(&self) -> Option<Picture> {
		None
	}
	/// Sets the front cover
	fn set_back_cover(&mut self, _cover: Picture) {}
	/// Removes the front cover
	fn remove_back_cover(&mut self) {}

	/// Returns all pictures stored in the track, or `None` if empty
	fn pictures(&self) -> Option<Cow<'static, [Picture]>> {
		None
	}
	/// Replace all pictures
	fn set_pictures(&mut self, _pictures: Vec<Picture>) {}
	/// Remove all pictures
	fn remove_pictures(&mut self) {}

	/// Returns the track number and total tracks
	fn track(&self) -> (Option<u32>, Option<u32>) {
		(self.track_number(), self.total_tracks())
	}

	u32_accessor!(track_number);
	u32_accessor!(total_tracks);

	/// Returns the disc number and total discs
	fn disc(&self) -> (Option<u32>, Option<u32>) {
		(self.disc_number(), self.total_discs())
	}

	u32_accessor!(disc_number);
	u32_accessor!(total_discs);

	/// Returns the TagType
	fn tag_type(&self) -> TagType;

	/// Gets a value by its key
	///
	/// NOTE: keys are format-specific, it is recommended to use this in
	/// combination with [`tag_type`][AudioTagEdit::tag_type] if formats are unknown
	fn get_key(&self, _key: &str) -> Option<&str> {
		None
	}
	/// Remove's a key/value pair
	///
	/// See [`get_key`][AudioTagEdit::get_key]'s note
	fn remove_key(&mut self, _key: &str) {}

	/// Returns the [`FileProperties`][crate::FileProperties]
	fn properties(&self) -> &FileProperties;
}

/// Functions for writing to a file
pub trait AudioTagWrite {
	/// Write tag to a [`File`][std::fs::File]
	///
	/// # Errors
	///
	/// Will return `Err` if unable to write to the `File`
	fn write_to(&self, file: &mut File) -> Result<()>;
	/// Write tag to a path
	///
	/// # Errors
	///
	/// Will return `Err` if `path` doesn't exist
	fn write_to_path(&self, path: &str) -> Result<()> {
		let mut file = OpenOptions::new().read(true).write(true).open(path)?;
		self.write_to(&mut file)?;

		Ok(())
	}
}

/// Conversions between tag types
pub trait ToAnyTag: ToAny {
	/// Converts the tag to [`AnyTag`]
	fn to_anytag(&self) -> AnyTag<'_>;

	/// Convert the tag type, which can be lossy.
	fn to_dyn_tag(&self, tag_type: TagType) -> Box<dyn AudioTag> {
		// TODO: write a macro or something that implement this method for every tag type so that if the
		// TODO: target type is the same, just return self
		match tag_type {
			#[cfg(feature = "format-ape")]
			TagType::Ape => Box::new(ApeTag::from(self.to_anytag())),
			#[cfg(feature = "format-id3")]
			TagType::Id3v2(_) => Box::new(Id3v2Tag::from(self.to_anytag())),
			#[cfg(feature = "format-mp4")]
			TagType::Mp4 => Box::new(Mp4Tag::from(self.to_anytag())),
			#[cfg(any(
				feature = "format-vorbis",
				feature = "format-flac",
				feature = "format-opus"
			))]
			TagType::Ogg(_) => Box::new(OggTag::from(self.to_anytag())),
			#[cfg(feature = "format-riff")]
			TagType::RiffInfo => Box::new(RiffTag::from(self.to_anytag())),
			#[cfg(feature = "format-aiff")]
			TagType::AiffText => Box::new(AiffTag::from(self.to_anytag())),
		}
	}
}

/// Tag conversion to `Any`
pub trait ToAny {
	/// Convert tag to `Any`
	fn to_any(&self) -> &dyn std::any::Any;
	/// Mutably convert tag to `Any`
	#[allow(clippy::wrong_self_convention)]
	fn to_any_mut(&mut self) -> &mut dyn std::any::Any;
}
