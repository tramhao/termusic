use crate::components::logic::iff::riff;
use crate::{
	Album, AnyTag, AudioTag, AudioTagEdit, AudioTagWrite, FileProperties, Result, TagType, ToAny,
	ToAnyTag,
};

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek};

use lofty_attr::{get_set_methods, LoftyTag};

#[derive(Default)]
struct RiffInnerTag {
	data: HashMap<String, String>,
}

#[derive(LoftyTag)]
/// Represents a RIFF INFO LIST
pub struct RiffTag {
	inner: RiffInnerTag,
	properties: FileProperties,
	#[expected(TagType::RiffInfo)]
	_format: TagType,
}

impl RiffTag {
	#[allow(missing_docs, clippy::missing_errors_doc)]
	pub fn read_from<R>(reader: &mut R) -> Result<Self>
	where
		R: Read + Seek,
	{
		let data = riff::read_from(reader)?;

		Ok(Self {
			inner: RiffInnerTag {
				data: data.metadata,
			},
			properties: data.properties,
			_format: TagType::RiffInfo,
		})
	}

	#[allow(missing_docs, clippy::missing_errors_doc)]
	pub fn remove_from(file: &mut File) -> Result<()> {
		riff::write_to(file, HashMap::new())?;
		Ok(())
	}
}

impl RiffTag {
	fn get_value(&self, key: &str) -> Option<&str> {
		self.inner.data.get_key_value(key).map(|(_, v)| v.as_str())
	}

	fn set_value<V>(&mut self, key: &str, val: V)
	where
		V: Into<String>,
	{
		self.inner.data.insert(key.to_string(), val.into());
	}

	fn remove_key(&mut self, key: &str) {
		self.inner.data.remove(key);
	}
}

impl AudioTagEdit for RiffTag {
	get_set_methods!(title, "INAM");
	get_set_methods!(artist, "IART");
	get_set_methods!(copyright, "ICOP");
	get_set_methods!(genre, "IGNR");
	get_set_methods!(album_title, "IPRD");
	get_set_methods!(encoder, "ISFT");

	fn date(&self) -> Option<String> {
		self.get_value("ICRD").map(std::string::ToString::to_string)
	}
	fn set_date(&mut self, date: &str) {
		self.set_value("ICRD", date)
	}
	fn remove_date(&mut self) {
		self.remove_key("ICRD")
	}

	fn track_number(&self) -> Option<u32> {
		if let Some(Ok(track_num)) = self
			.get_value("ITRK")
			.or_else(|| self.get_value("IPRT"))
			.or_else(|| self.get_value("TRAC"))
			.map(str::parse::<u32>)
		{
			return Some(track_num);
		}

		None
	}
	fn set_track_number(&mut self, track_number: u32) {
		self.set_value("ITRK", track_number.to_string())
	}
	fn remove_track_number(&mut self) {
		self.remove_key("ITRK")
	}

	fn total_tracks(&self) -> Option<u32> {
		if let Some(Ok(total_tracks)) = self.get_value("IFRM").map(str::parse::<u32>) {
			return Some(total_tracks);
		}

		None
	}
	fn set_total_tracks(&mut self, total_track: u32) {
		self.set_value("IFRM", total_track.to_string())
	}
	fn remove_total_tracks(&mut self) {
		self.remove_key("IFRM")
	}

	fn disc_number(&self) -> Option<u32> {
		if let Some(Ok(disc_number)) = self.get_value("DISC").map(str::parse::<u32>) {
			return Some(disc_number);
		}

		None
	}
	fn set_disc_number(&mut self, disc_number: u32) {
		self.set_value("DISC", disc_number.to_string())
	}
	fn remove_disc_number(&mut self) {
		self.remove_key("DISC")
	}

	fn total_discs(&self) -> Option<u32> {
		self.disc_number()
	}
	fn set_total_discs(&mut self, total_discs: u32) {
		self.set_disc_number(total_discs)
	}
	fn remove_total_discs(&mut self) {
		self.remove_disc_number()
	}

	fn tag_type(&self) -> TagType {
		TagType::RiffInfo
	}

	fn get_key(&self, key: &str) -> Option<&str> {
		self.get_value(key)
	}
	fn remove_key(&mut self, key: &str) {
		self.remove_key(key)
	}

	fn properties(&self) -> &FileProperties {
		&self.properties
	}
}

impl AudioTagWrite for RiffTag {
	fn write_to(&self, file: &mut File) -> Result<()> {
		riff::write_to(file, self.inner.data.clone())
	}
}
