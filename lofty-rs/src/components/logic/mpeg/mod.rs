mod constants;
pub(crate) mod header;
pub(crate) mod read;

use crate::FileProperties;

#[allow(dead_code)]
pub(crate) struct MpegData {
	pub id3: Option<Vec<u8>>,
	pub ape: Option<Vec<u8>>, // TODO
	pub properties: FileProperties,
}
