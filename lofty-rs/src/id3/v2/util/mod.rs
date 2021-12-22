pub(crate) mod text_utils;
#[cfg(feature = "id3v2")]
pub(crate) mod upgrade;

#[cfg(feature = "id3v2")]
use crate::error::{LoftyError, Result};

#[cfg(feature = "id3v2")]
pub(in crate::id3::v2) fn unsynch_content(content: &[u8]) -> Result<Vec<u8>> {
	let mut unsynch_content = Vec::new();

	let mut discard = false;

	let mut i = 0;
	let mut next = 0;
	let content_len = content.len();

	// Check for (0xFF, 0x00, 0x00), replace with (0xFF, 0x00)
	while i < content_len && next < content_len {
		// Verify the next byte is less than 0xE0 (0b111xxxxx)
		// Then remove the next byte if it is a zero
		if discard {
			if content[next] >= 0xE0 {
				return Err(LoftyError::Id3v2(
					"Encountered an invalid unsynchronisation",
				));
			}

			if content[next] == 0 {
				discard = false;
				next += 1;

				continue;
			}
		}

		discard = false;

		unsynch_content.push(content[next]);

		if content[next] == 0xFF {
			discard = true
		}

		i += 1;
		next += 1;
	}

	Ok(unsynch_content)
}

#[cfg(test)]
mod tests {
	#[test]
	fn unsynchronisation() {
		let valid_unsynch = vec![0xFF, 0x00, 0x00, 0xFF, 0x12, 0xB0, 0x05, 0xFF, 0x00, 0x00];

		assert_eq!(
			super::unsynch_content(valid_unsynch.as_slice()).unwrap(),
			vec![0xFF, 0x00, 0xFF, 0x12, 0xB0, 0x05, 0xFF, 0x00]
		);

		let invalid_unsynch = vec![
			0xFF, 0xE0, 0x00, 0xFF, 0x12, 0xB0, 0x05, 0xFF, 0x00, 0x50, 0x01,
		];

		assert!(super::unsynch_content(invalid_unsynch.as_slice()).is_err());
	}
}
