use anyhow::Result;
use quick_xml::escape::unescape;
use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Debug, Clone, PartialEq)]
pub struct XSPFItem {
    /// According to the spec, a `track` MAY contain exactly one `title`
    pub title: Option<String>,
    /// According to the spec, a `track` MAY contain exactly one `location`, though we require one, otherwise the track is ignored
    pub url: String,
    /// According to the spec, a `track` MAY contain zero or more `identifier` (only last will be used here though)
    pub identifier: Option<String>,
}

/// A temporary storage to build a [`XSPFItem`] while still being in a element and not having all values
#[derive(Debug, Clone, PartialEq, Default)]
struct PrivateItem {
    pub title: Option<String>,
    pub location: Option<String>,
    pub identifier: Option<String>,
}

impl PrivateItem {
    /// Try to transform the current item into a [`XSPFItem`], on fail reset to default values for next loop
    fn try_into_xspf_item_and_reset(&mut self) -> Option<XSPFItem> {
        if let Some(location) = self.location.take() {
            return Some(XSPFItem {
                title: self.title.take(),
                url: location,
                identifier: self.identifier.take(),
            });
        }

        self.reset();

        None
    }

    /// Reset a self reference to be all the default value for the next loop
    #[inline]
    fn reset(&mut self) {
        *self = Self::default();
    }
}

/// XSPF or "XML Shareable Playlist Format", based on XML (as the name implies).
///
/// <https://www.xspf.org/spec>
pub fn decode(content: &str) -> Result<Vec<XSPFItem>> {
    let mut list: Vec<XSPFItem> = vec![];
    let mut current_item = PrivateItem::default();

    let mut reader = Reader::from_str(content);
    reader.trim_text(true);
    let mut xml_stack = Vec::with_capacity(4);
    let mut buf = Vec::new();
    let decoder = reader.decoder();
    loop {
        match reader.read_event_into(&mut buf) {
            // Ok(Event::Empty(ref e)) => {}
            Ok(Event::Start(ref e)) => {
                xml_stack.push(decoder.decode(e.name().as_ref())?.to_lowercase());
            }
            Ok(Event::End(_)) => {
                let path = xml_stack.join("/");
                if path == "playlist/tracklist/track" {
                    if let Some(transformed) = current_item.try_into_xspf_item_and_reset() {
                        list.push(transformed);
                    } else {
                        warn!("Element could not be transformed into a valid item, ignoring!");
                    }
                }
                xml_stack.pop();
            }
            Ok(Event::Text(e)) => {
                let path = xml_stack.join("/");
                if path == "playlist/tracklist/track/title" {
                    current_item
                        .title
                        .replace(unescape(&decoder.decode(&e)?)?.to_string());
                }
                if path == "playlist/tracklist/track/location" {
                    current_item
                        .location
                        .replace(unescape(&decoder.decode(&e)?)?.to_string());
                }
                if path == "playlist/tracklist/track/identifier" {
                    current_item
                        .identifier
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
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn xspf() {
        let s = r#"<?xml version="1.0" encoding="UTF-8"?>
<playlist version="1" xmlns="http://xspf.org/ns/0/">
    <trackList>
    <track>
        <title>Title</title>
        <identifier>Identifier</identifier>
        <location>http://this.is.an.example</location>
    </track>
    <track>
        <title>Title2</title>
        <identifier>Identifier2</identifier>
        <location>http://this.is.an.example2</location>
    </track>
    </trackList>
</playlist>"#;
        let items = decode(s);
        assert!(items.is_ok());
        let items = items.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].url, "http://this.is.an.example");
        assert_eq!(items[0].title, Some("Title".to_string()));
        assert_eq!(items[0].identifier, Some("Identifier".to_string()));
        assert_eq!(items[1].url, "http://this.is.an.example2");
        assert_eq!(items[1].title, Some("Title2".to_string()));
        assert_eq!(items[1].identifier, Some("Identifier2".to_string()));
    }

    #[test]
    fn should_ignore_tracks_without_location() {
        let s = r#"<?xml version="1.0" encoding="UTF-8"?>
<playlist version="1" xmlns="http://xspf.org/ns/0/">
    <trackList>
    <track>
        <title>Title</title>
        <identifier>Identifier</identifier>
        <location>http://this.is.an.example</location>
    </track>
    <track>
        <title>Title2</title>
        <identifier>Identifier2</identifier>
    </track>
    <track>
        <title>Title3</title>
        <identifier>Identifier2</identifier>
        <location>http://this.is.an.example2</location>
    </track>
    </trackList>
</playlist>"#;
        let items = decode(s).unwrap();
        assert_eq!(items.len(), 2);
    }
}
