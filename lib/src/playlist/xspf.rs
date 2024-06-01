use quick_xml::escape::unescape;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct XSPFItem {
    pub title: String,
    pub url: String,
    pub identifier: String,
}

/// XSPF or "XML Shareable Playlist Format", based on XML (as the name implies).
///
/// <https://www.xspf.org/spec>
pub fn decode(content: &str) -> Result<Vec<XSPFItem>, Box<dyn Error>> {
    let mut list = vec![];
    let mut item = XSPFItem {
        title: String::new(),
        url: String::new(),
        identifier: String::new(),
    };

    let mut reader = Reader::from_str(content);
    reader.trim_text(true);
    let mut xml_stack = Vec::with_capacity(4);
    let mut buf = Vec::new();
    let decoder = reader.decoder();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) => {
                xml_stack.push(decoder.decode(e.name().as_ref())?.to_lowercase());

                let path = xml_stack.join("/");
                for a in e.attributes() {
                    let a = a?;
                    let key = decoder.decode(a.key.as_ref())?.to_lowercase();
                    let value = decoder.decode(&a.value)?;
                    if path == "asx/entry/ref" && key == "href" {
                        item.url = value.to_string();
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
                    if path == "asx/entry/ref" && key == "href" {
                        item.url = value.to_string();
                    }
                }
            }
            Ok(Event::End(_)) => {
                let path = xml_stack.join("/");
                if path == "playlist/tracklist/track" {
                    list.push(item.clone());
                    item.title = String::new();
                    item.url = String::new();
                    item.identifier = String::new();
                }
                xml_stack.pop();
            }
            Ok(Event::Text(e)) => {
                let path = xml_stack.join("/");
                if path == "playlist/tracklist/track/title" {
                    // item.title = e.unescape_and_decode(&reader)?.clone();
                    item.title = unescape(&decoder.decode(&e)?)?.to_string();
                }
                if path == "playlist/tracklist/track/location" {
                    // item.url = e.unescape_and_decode(&reader)?.clone();
                    item.url = unescape(&decoder.decode(&e)?)?.to_string();
                }
                if path == "playlist/tracklist/track/identifier" {
                    // item.identifier = e.unescape_and_decode(&reader)?.clone();
                    item.identifier = unescape(&decoder.decode(&e)?)?.to_string();
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
        assert_eq!(items[0].title, "Title");
        assert_eq!(items[0].identifier, "Identifier");
        assert_eq!(items[1].url, "http://this.is.an.example2");
        assert_eq!(items[1].title, "Title2");
        assert_eq!(items[1].identifier, "Identifier2");
    }
}
