use crate::components::logic::iff::aiff;
use crate::{
	Album, AnyTag, AudioTag, AudioTagEdit, AudioTagWrite, FileProperties, Result, TagType, ToAny,
	ToAnyTag,
};

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek};

use lofty_attr::{get_set_methods, LoftyTag};

#[derive(Default)]
struct AiffInnerTag {
	data: HashMap<String, String>,
}

#[derive(LoftyTag)]
/// Represents Aiff Text Chunks
pub struct AiffTag {
	inner: AiffInnerTag,
	properties: FileProperties,
	#[expected(TagType::AiffText)]
	_format: TagType,
}

impl AiffTag {
	#[allow(missing_docs, clippy::missing_errors_doc)]
	pub fn read_from<R>(reader: &mut R) -> Result<Self>
	where
		R: Read + Seek,
	{
		let data = aiff::read_from(reader)?;

		Ok(Self {
			inner: AiffInnerTag {
				data: data.metadata,
			},
			properties: data.properties,
			_format: TagType::AiffText,
		})
	}

	fn get_value(&self, key: &str) -> Option<&str> {
		self.inner.data.get_key_value(key).map(|(_, v)| v.as_str())
	}

	fn set_value<V>(&mut self, key: &str, val: V)
	where
		V: Into<String>,
	{
		self.inner.data.insert(key.into(), val.into());
	}

	fn remove_key(&mut self, key: &str) {
		self.inner.data.remove(key);
	}

	#[allow(missing_docs, clippy::missing_errors_doc)]
	pub fn remove_from(file: &mut File) -> Result<()> {
		aiff::write_to(file, &HashMap::<String, String>::new())?;
		Ok(())
	}
}

impl AudioTagEdit for AiffTag {
	get_set_methods!(title, "NAME");
	get_set_methods!(artist, "AUTH");
	get_set_methods!(copyright, "(c) ");

	fn tag_type(&self) -> TagType {
		TagType::AiffText
	}

	fn properties(&self) -> &FileProperties {
		&self.properties
	}
}

impl AudioTagWrite for AiffTag {
	fn write_to(&self, file: &mut File) -> Result<()> {
		aiff::write_to(file, &self.inner.data)
	}
}
