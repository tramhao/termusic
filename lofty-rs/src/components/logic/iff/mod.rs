use crate::FileProperties;

use std::collections::HashMap;

#[cfg(any(feature = "format-aiff", feature = "format-id3"))]
pub(crate) mod aiff;
#[cfg(any(feature = "format-riff", feature = "format-id3"))]
pub(crate) mod riff;

pub(crate) struct IffData {
	pub properties: FileProperties,
	pub metadata: HashMap<String, String>,
	pub id3: Option<Vec<u8>>,
}
