use anyhow::Result;
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::Event;

use super::PlaylistValue;

#[derive(Debug, Clone, PartialEq)]
pub struct ASXItem {
    /// According to the spec, a `entry` SHOULD contain exactly one `title` (all after the first are ignored)
    pub title: String,
    /// According to the spec, a `entry` SHOULD contain exactly one `ref` (all after the first are ignored for our purposes)
    pub location: PlaylistValue,
}

/// A temporary storage to build a [`ASXItem`] while still being in a element and not having all values
#[derive(Debug, Clone, PartialEq, Default)]
struct PrivateItem {
    pub title: Option<String>,
    pub ref_href: Option<PlaylistValue>,
}

impl PrivateItem {
    /// Try to transform the current item into a [`ASXItem`], on fail reset to default values for next loop
    fn try_into_xspf_item_and_reset(&mut self) -> Option<ASXItem> {
        if self.check_required_values() {
            return Some(ASXItem {
                // SAFETY: unwrap is safe here because it is checked by `check_required_values` to be `Some`
                title: self.title.take().unwrap(),
                location: self.ref_href.take().unwrap(),
            });
        }

        self.reset();

        None
    }

    /// Check that all required values are [`Some`] and issue warnings, returns `true` if that is the case, `false` otherwise
    fn check_required_values(&self) -> bool {
        if self.ref_href.is_none() {
            error!("ASX Entry had no \"ref href\", ignoring!");
            return false;
        } else if self.title.is_none() {
            error!("ASX Entry had no \"title\", ignoring!");
            return false;
        }

        true
    }

    /// Reset a self reference to be all the default value for the next loop
    #[inline]
    fn reset(&mut self) {
        *self = Self::default();
    }
}

/// ASX or "Advanced Stream Redirector" is a standard made by microsoft for windows media player, XML based.
///
/// <https://en.wikipedia.org/wiki/Advanced_Stream_Redirector>
/// <https://learn.microsoft.com/en-us/windows/win32/wmp/asx-element>
pub fn decode(content: &str) -> Result<Vec<ASXItem>> {
    let mut list: Vec<ASXItem> = vec![];
    let mut item = PrivateItem::default();

    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut xml_stack = Vec::with_capacity(3);
    let mut buf = Vec::new();
    let decoder = reader.decoder();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) => {
                xml_stack.push(decoder.decode(e.name().as_ref())?.to_lowercase());
                // xml_stack.push(reader.decoder().decode(e.name())?.to_lowercase());

                let path = xml_stack.join("/");
                for a in e.attributes() {
                    let a = a?;
                    let key = decoder.decode(a.key.as_ref())?.to_lowercase();
                    let value = decoder.decode(&a.value)?;
                    if path == "asx/entry/ref" && key == "href" && item.ref_href.is_none() {
                        let mut p_value = PlaylistValue::try_from_str(&value)?;
                        p_value.file_url_to_path()?;
                        item.ref_href.replace(p_value);
                    }
                }

                xml_stack.pop();
            }
            Ok(Event::Start(ref e)) => {
                xml_stack.push(decoder.decode(e.name().as_ref())?.to_lowercase());

                let path = xml_stack.join("/");
                for a in e.attributes() {
                    let a = a?;
                    let key = decoder.decode(a.key.as_ref())?.to_lowercase();
                    let value = decoder.decode(&a.value)?;
                    if path == "asx/entry/ref" && key == "href" && item.ref_href.is_none() {
                        let mut p_value = PlaylistValue::try_from_str(&value)?;
                        p_value.file_url_to_path()?;
                        item.ref_href.replace(p_value);
                    }
                }
            }
            Ok(Event::End(_)) => {
                let path = xml_stack.join("/");
                if path == "asx/entry" {
                    if let Some(transformed) = item.try_into_xspf_item_and_reset() {
                        list.push(transformed);
                    }
                    // no else case, as log errors have already been issued
                }
                xml_stack.pop();
            }
            Ok(Event::Text(e)) => {
                let path = xml_stack.join("/");

                if path == "asx/entry/title" && item.title.is_none() {
                    item.title
                        .replace(unescape(&decoder.decode(&e)?)?.to_string());
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                error!("Error at position {}: {e:?}", reader.buffer_position());
                break;
            }
            _ => (), // There are several other `Event`s we do not consider here
        }
        buf.clear();
    }

    Ok(list)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn asx() {
        let s = r#"<asx version="3.0">
  <title>Test-Liste</title>
  <entry>
    <title>title1</title>
    <ref href="ref1"/>
  </entry>
  <entry>
    <title>title2</title>
    <ref href="ref2"/>
  </entry>
</asx>"#;
        let items = decode(s);
        assert!(items.is_ok());
        let items = items.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0].location,
            PlaylistValue::Path(PathBuf::from("ref1"))
        );
        assert_eq!(items[0].title, "title1");
        assert_eq!(
            items[1].location,
            PlaylistValue::Path(PathBuf::from("ref2"))
        );
        assert_eq!(items[1].title, "title2");
    }

    #[test]
    fn should_refuse_missing_elements() {
        let s = r#"<asx version="3.0">
  <title>Test-Liste</title>
  <entry>
    <title>title1</title>
    <!--Missing ref-->
  </entry>
  <entry>
    <!--Missing title-->
    <ref href="ref2"/>
  </entry>
</asx>"#;
        let items = decode(s).unwrap();
        assert_eq!(items.len(), 0);
    }
}
