use crate::components::logic::iff::{aiff, riff};
use crate::components::logic::mpeg;
use crate::tag::Id3Format;
use crate::{
	Album, AnyTag, AudioTag, AudioTagEdit, AudioTagWrite, FileProperties, LoftyError, MimeType,
	Picture, Result, TagType, ToAny, ToAnyTag,
};

use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};

use filepath::FilePath;
pub use id3::Tag as Id3v2InnerTag;
use lofty_attr::LoftyTag;

#[derive(LoftyTag)]
/// Represents an ID3 tag
pub struct Id3v2Tag {
	inner: Id3v2InnerTag,
	properties: FileProperties,
	#[expected(TagType::Id3v2(Id3Format::Mp3))]
	_format: TagType,
}

impl Id3v2Tag {
	#[allow(missing_docs, clippy::missing_errors_doc)]
	pub fn read_from<R>(reader: &mut R, format: Id3Format) -> Result<Self>
	where
		R: Read + Seek,
	{
		let (properties, inner) = match format {
			Id3Format::Mp3 => {
				let data = mpeg::read::read_from(reader)?;

				let inner = match data.id3 {
					Some(id3) => Id3v2InnerTag::read_from(Cursor::new(id3)),
					None => Ok(Id3v2InnerTag::new()),
				};

				(data.properties, inner)
			},
			Id3Format::Riff => {
				let data = riff::read_from(reader)?;

				let inner = match data.id3 {
					Some(id3) => Id3v2InnerTag::read_from(Cursor::new(id3)),
					None => Ok(Id3v2InnerTag::new()),
				};

				(data.properties, inner)
			},
			Id3Format::Aiff => {
				let data = aiff::read_from(reader)?;

				let inner = match data.id3 {
					Some(id3) => Id3v2InnerTag::read_from(Cursor::new(id3)),
					None => Ok(Id3v2InnerTag::new()),
				};

				(data.properties, inner)
			},
		};

		Ok(Self {
			inner: inner.unwrap_or_else(|_| Id3v2InnerTag::new()),
			properties,
			_format: TagType::Id3v2(format),
		})
	}

	#[allow(missing_docs, clippy::missing_errors_doc)]
	pub fn remove_from(file: &mut File) -> Result<()> {
		Id3v2InnerTag::remove_from(file)?;
		Ok(())
	}
}

impl std::convert::TryFrom<&id3::frame::Picture> for Picture {
	type Error = LoftyError;

	fn try_from(inp: &id3::frame::Picture) -> Result<Self> {
		let id3::frame::Picture {
			mime_type,
			data,
			picture_type,
			description,
			..
		} = inp;
		let mime_type: MimeType = mime_type.as_str().try_into()?;
		let pic_type = *picture_type;
		let description = if description == &String::new() {
			None
		} else {
			Some(Cow::from(description.clone()))
		};

		Ok(Self {
			pic_type,
			mime_type,
			description,
			width: 0,
			height: 0,
			color_depth: 0,
			num_colors: 0,
			data: Cow::from(data.clone()),
		})
	}
}

impl TryFrom<Picture> for id3::frame::Picture {
	type Error = LoftyError;

	fn try_from(inp: Picture) -> Result<Self> {
		Ok(Self {
			mime_type: String::from(inp.mime_type),
			picture_type: inp.pic_type,
			description: inp
				.description
				.map_or_else(|| "".to_string(), |d| d.to_string()),
			data: Vec::from(inp.data),
		})
	}
}

impl AudioTagEdit for Id3v2Tag {
	fn title(&self) -> Option<&str> {
		self.inner.title()
	}
	fn set_title(&mut self, title: &str) {
		self.inner.set_title(title)
	}
	fn remove_title(&mut self) {
		self.inner.remove_title();
	}

	fn artist(&self) -> Option<&str> {
		self.inner.artist()
	}
	fn set_artist(&mut self, artist: &str) {
		self.inner.set_artist(artist)
	}
	fn remove_artist(&mut self) {
		self.inner.remove_artist()
	}

	fn date(&self) -> Option<String> {
		if let Some(released) = self.inner.get("TDRL") {
			return released
				.content()
				.text()
				.map(std::string::ToString::to_string);
		}

		if let Some(recorded) = self.inner.get("TRDC") {
			return recorded
				.content()
				.text()
				.map(std::string::ToString::to_string);
		}

		None
	}
	fn set_date(&mut self, date: &str) {
		if let Ok(t) = date.parse::<id3::Timestamp>() {
			self.inner.set_date_released(t)
		}
	}
	fn remove_date(&mut self) {
		self.inner.remove_date_released();
		self.inner.remove_date_recorded();
	}

	fn year(&self) -> Option<i32> {
		self.inner.year()
	}
	fn set_year(&mut self, year: i32) {
		self.inner.set_year(year)
	}
	fn remove_year(&mut self) {
		self.inner.remove_year()
	}

	fn copyright(&self) -> Option<&str> {
		if let Some(frame) = self.inner.get("TCOP") {
			return frame.content().text();
		}

		None
	}
	fn set_copyright(&mut self, copyright: &str) {
		self.inner.set_text("TCOP", copyright)
	}
	fn remove_copyright(&mut self) {
		self.inner.remove("TCOP")
	}

	fn genre(&self) -> Option<&str> {
		self.inner.genre()
	}
	fn set_genre(&mut self, genre: &str) {
		self.inner.set_genre(genre)
	}
	fn remove_genre(&mut self) {
		self.inner.remove_genre()
	}

	fn bpm(&self) -> Option<u16> {
		if let Some(frame) = self.inner.get("TBPM") {
			if let Some(text) = frame.content().text() {
				return text.parse::<u16>().ok();
			}
		}

		None
	}
	fn set_bpm(&mut self, bpm: u16) {
		self.inner.set_text("TBPM", bpm.to_string())
	}
	fn remove_bpm(&mut self) {
		self.inner.remove("TBPM")
	}

	fn lyricist(&self) -> Option<&str> {
		if let Some(frame) = self.inner.get("TEXT") {
			return frame.content().text();
		}

		None
	}
	fn set_lyricist(&mut self, lyricist: &str) {
		self.inner.set_text("TEXT", lyricist)
	}
	fn remove_lyricist(&mut self) {
		self.inner.remove("TEXT")
	}

	fn composer(&self) -> Option<&str> {
		if let Some(frame) = self.inner.get("TCOM") {
			return frame.content().text();
		}

		None
	}
	fn set_composer(&mut self, composer: &str) {
		self.inner.set_text("TCOM", composer)
	}
	fn remove_composer(&mut self) {
		self.inner.remove("TCOM")
	}

	fn encoder(&self) -> Option<&str> {
		if let Some(frame) = self.inner.get("TSSE") {
			return frame.content().text();
		}

		None
	}
	fn set_encoder(&mut self, encoder: &str) {
		self.inner.set_text("TSSE", encoder)
	}
	fn remove_encoder(&mut self) {
		self.inner.remove("TSSE")
	}

	fn album_title(&self) -> Option<&str> {
		self.inner.album()
	}
	fn set_album_title(&mut self, title: &str) {
		self.inner.set_album(title)
	}
	fn remove_album_title(&mut self) {
		self.inner.remove_album();
	}

	fn album_artist(&self) -> Option<&str> {
		self.inner.album_artist()
	}
	fn set_album_artist(&mut self, album_artist: &str) {
		self.inner.set_album_artist(album_artist)
	}
	fn remove_album_artist(&mut self) {
		self.inner.remove_album_artist()
	}

	fn front_cover(&self) -> Option<Picture> {
		self.inner
			.pictures()
			.find(|&pic| matches!(pic.picture_type, id3::frame::PictureType::CoverFront))
			.and_then(|pic| TryInto::<Picture>::try_into(pic).ok())
	}
	fn set_front_cover(&mut self, cover: Picture) {
		self.remove_front_cover();

		if let Ok(pic) = cover.try_into() {
			self.inner.add_picture(pic)
		}
	}
	fn remove_front_cover(&mut self) {
		self.inner
			.remove_picture_by_type(id3::frame::PictureType::CoverFront);
	}

	fn back_cover(&self) -> Option<Picture> {
		self.inner
			.pictures()
			.find(|&pic| matches!(pic.picture_type, id3::frame::PictureType::CoverBack))
			.and_then(|pic| TryInto::<Picture>::try_into(pic).ok())
	}
	fn set_back_cover(&mut self, cover: Picture) {
		self.remove_back_cover();

		if let Ok(pic) = cover.try_into() {
			self.inner.add_picture(pic)
		}
	}
	fn remove_back_cover(&mut self) {
		self.inner
			.remove_picture_by_type(id3::frame::PictureType::CoverBack);
	}

	fn pictures(&self) -> Option<Cow<'static, [Picture]>> {
		let mut pictures = self.inner.pictures().peekable();

		if pictures.peek().is_some() {
			let mut collection = Vec::new();

			for pic in pictures {
				match TryInto::<Picture>::try_into(pic) {
					Ok(p) => collection.push(p),
					Err(_) => return None,
				}
			}

			return Some(Cow::from(collection));
		}

		None
	}
	fn set_pictures(&mut self, pictures: Vec<Picture>) {
		self.remove_pictures();

		for p in pictures {
			if let Ok(pic) = TryInto::<id3::frame::Picture>::try_into(p) {
				self.inner.add_picture(pic)
			}
		}
	}
	fn remove_pictures(&mut self) {
		self.inner.remove_all_pictures()
	}

	fn track_number(&self) -> Option<u32> {
		self.inner.track()
	}
	fn set_track_number(&mut self, track: u32) {
		self.inner.set_track(track);
	}
	fn remove_track_number(&mut self) {
		self.inner.remove_track();
	}

	fn total_tracks(&self) -> Option<u32> {
		self.inner.total_tracks()
	}
	fn set_total_tracks(&mut self, total_track: u32) {
		self.inner.set_total_tracks(total_track as u32);
	}
	fn remove_total_tracks(&mut self) {
		self.inner.remove_total_tracks();
	}

	fn disc_number(&self) -> Option<u32> {
		self.inner.disc()
	}
	fn set_disc_number(&mut self, disc_number: u32) {
		self.inner.set_disc(disc_number as u32)
	}
	fn remove_disc_number(&mut self) {
		self.inner.remove_disc();
	}

	fn total_discs(&self) -> Option<u32> {
		self.inner.total_discs()
	}
	fn set_total_discs(&mut self, total_discs: u32) {
		self.inner.set_total_discs(total_discs)
	}
	fn remove_total_discs(&mut self) {
		self.inner.remove_total_discs();
	}

	fn tag_type(&self) -> TagType {
		self._format.clone()
	}

	fn get_key(&self, key: &str) -> Option<&str> {
		if let Some(frame) = self.inner.get(key) {
			return frame.content().text();
		}

		None
	}
	fn remove_key(&mut self, key: &str) {
		self.inner.remove(key)
	}

	fn properties(&self) -> &FileProperties {
		&self.properties
	}
}

impl AudioTagWrite for Id3v2Tag {
	fn write_to(&self, file: &mut File) -> Result<()> {
		let mut id = [0; 4];
		file.read_exact(&mut id)?;
		file.seek(SeekFrom::Start(0))?;

		match &id {
			b"RIFF" => self
				.inner
				.write_to_wav(file.path()?, id3::Version::Id3v24)?,
			b"FORM" => self
				.inner
				.write_to_aiff(file.path()?, id3::Version::Id3v24)?,
			_ => self.inner.write_to(file, id3::Version::Id3v24)?,
		}

		Ok(())
	}
	fn write_to_path(&self, path: &str) -> Result<()> {
		let id = &std::fs::read(&path)?[0..4];

		match id {
			b"RIFF" => self.inner.write_to_wav(path, id3::Version::Id3v24)?,
			b"FORM" => self.inner.write_to_aiff(path, id3::Version::Id3v24)?,
			_ => self.inner.write_to_path(path, id3::Version::Id3v24)?,
		}

		Ok(())
	}
}
