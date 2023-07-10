use quick_xml::escape::unescape;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::error::Error;

#[derive(Clone)]
pub struct PlaylistItem {
    pub title: String,
    pub url: String,
}

pub fn decode(content: &str) -> Result<Vec<PlaylistItem>, Box<dyn Error>> {
    let mut list = vec![];
    let mut item = PlaylistItem {
        title: String::new(),
        url: String::new(),
    };

    let mut reader = Reader::from_str(content);
    reader.trim_text(true);
    let mut xml_stack = vec![];
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
                    // let key = reader.decode(a.key)?.to_lowercase();
                    let value = decoder.decode(&a.value)?;
                    // let value = reader.decode(&a.value)?;
                    if path == "asx/entry/ref" && key == "href" {
                        item.url = value.to_string();
                    }
                }

                xml_stack.pop();
            }
            Ok(Event::Start(ref e)) => {
                xml_stack.push(decoder.decode(e.name().as_ref())?.to_lowercase());
                // xml_stack.push(reader.decode(e.name())?.to_lowercase());

                let path = xml_stack.join("/");
                for a in e.attributes() {
                    let a = a?;
                    let key = decoder.decode(a.key.as_ref())?.to_lowercase();
                    // let key = reader.decode(a.key)?.to_lowercase();
                    let value = decoder.decode(&a.value)?;
                    // let value = reader.decode(&a.value)?;
                    if path == "asx/entry/ref" && key == "href" {
                        item.url = value.to_string();
                    }
                }
            }
            Ok(Event::End(_)) => {
                let path = xml_stack.join("/");
                if path == "asx/entry" {
                    list.push(item.clone());
                    item.title = String::new();
                    item.url = String::new();
                }
                xml_stack.pop();
            }
            Ok(Event::Text(e)) => {
                let path = xml_stack.join("/");

                //unescape(&decoder.decode(&e).unwrap())
                if path == "asx/entry/title" {
                    // item.title = e
                    //     .unescaped_and_decode(&reader)
                    //     .unwrap_or_else(|_| String::from(""))
                    //     .clone();
                    item.title = unescape(&decoder.decode(&e)?)?.to_string();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                println!("Error at position {}: {e:?}", reader.buffer_position());
                break;
            }
            _ => (), // There are several other `Event`s we do not consider here
        }
        buf.clear();
    }

    Ok(list)
}
