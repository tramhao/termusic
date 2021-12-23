use crate::error::{LoftyError, Result};
use crate::id3::v2::frame::{EncodedTextFrame, FrameValue, LanguageFrame};
use crate::id3::v2::util::text_utils::{decode_text, TextEncoding};
use crate::id3::v2::Id3v2Version;
use crate::types::picture::Picture;

use std::io::Read;

use byteorder::ReadBytesExt;

pub(super) fn parse_content(
	content: &mut &[u8],
	id: &str,
	version: Id3v2Version,
) -> Result<FrameValue> {
	Ok(match id {
		// The ID was previously upgraded, but the content remains unchanged, so version is necessary
		"APIC" => {
			let (picture, encoding) = Picture::from_apic_bytes(content, version)?;

			FrameValue::Picture { encoding, picture }
		},
		"TXXX" => parse_user_defined(content, false)?,
		"WXXX" => parse_user_defined(content, true)?,
		"COMM" | "USLT" => parse_text_language(content, id)?,
		_ if id.starts_with('T') || id == "WFED" => parse_text(content)?,
		// Apple proprietary frames
		"WFED" | "GRP1" => parse_text(content)?,
		_ if id.starts_with('W') => parse_link(content)?,
		// SYLT, GEOB, and any unknown frames
		_ => FrameValue::Binary(content.to_vec()),
	})
}

// There are 2 possibilities for the frame's content: text or link.
fn parse_user_defined(content: &mut &[u8], link: bool) -> Result<FrameValue> {
	if content.len() < 2 {
		return Err(LoftyError::BadFrameLength);
	}

	let encoding = match TextEncoding::from_u8(content.read_u8()?) {
		None => return Err(LoftyError::TextDecode("Found invalid encoding")),
		Some(e) => e,
	};

	let description = decode_text(content, encoding, true)?.unwrap_or_else(String::new);

	Ok(if link {
		let content =
			decode_text(content, TextEncoding::Latin1, false)?.unwrap_or_else(String::new);

		FrameValue::UserURL(EncodedTextFrame {
			encoding,
			description,
			content,
		})
	} else {
		let content = decode_text(content, encoding, false)?.unwrap_or_else(String::new);

		FrameValue::UserText(EncodedTextFrame {
			encoding,
			description,
			content,
		})
	})
}

fn parse_text_language(content: &mut &[u8], id: &str) -> Result<FrameValue> {
	if content.len() < 5 {
		return Err(LoftyError::BadFrameLength);
	}

	let encoding = match TextEncoding::from_u8(content.read_u8()?) {
		None => return Err(LoftyError::TextDecode("Found invalid encoding")),
		Some(e) => e,
	};

	let mut lang = [0; 3];
	content.read_exact(&mut lang)?;

	let lang = std::str::from_utf8(&lang)
		.map_err(|_| LoftyError::TextDecode("Unable to decode language string"))?;

	let description = decode_text(content, encoding, true)?;
	let content = decode_text(content, encoding, false)?.unwrap_or_else(String::new);

	let information = LanguageFrame {
		encoding,
		language: lang.to_string(),
		description: description.unwrap_or_else(|| String::from("")),
		content,
	};

	let value = match id {
		"COMM" => FrameValue::Comment(information),
		"USLT" => FrameValue::UnSyncText(information),
		_ => unreachable!(),
	};

	Ok(value)
}

fn parse_text(content: &mut &[u8]) -> Result<FrameValue> {
	let encoding = match TextEncoding::from_u8(content.read_u8()?) {
		None => return Err(LoftyError::TextDecode("Found invalid encoding")),
		Some(e) => e,
	};

	let text = decode_text(content, encoding, true)?.unwrap_or_else(String::new);

	Ok(FrameValue::Text {
		encoding,
		value: text,
	})
}

fn parse_link(content: &mut &[u8]) -> Result<FrameValue> {
	let link = decode_text(content, TextEncoding::Latin1, true)?.unwrap_or_else(String::new);

	Ok(FrameValue::URL(link))
}
