mod constants;
mod properties;
pub(crate) mod read;
mod tag;
pub(crate) mod write;

use crate::FileProperties;

use std::collections::HashMap;

use unicase::UniCase;

#[allow(dead_code)]
pub(crate) struct ApeData {
	pub id3v1: Option<[u8; 128]>, // TODO
	pub id3v2: Option<Vec<u8>>,
	pub ape: Option<HashMap<UniCase<String>, ItemType>>,
	pub properties: FileProperties,
}

#[derive(Clone)]
pub(crate) enum ItemType {
	// The bool indicates if the value is read only
	String(String, bool),
	Locator(String, bool), // TODO: figure out some way to expose
	Binary(Vec<u8>, bool),
}
