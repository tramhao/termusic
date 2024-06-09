//! This is a very simple url extractor for different kinds of playlist formats: M3U, PLS, ASX, XSPF
//!
//! It is not optimized yet and does create a lot of strings on the way.

mod asx;
mod m3u;
mod pls;
mod xspf;

use anyhow::Result;

/// Decode playlist content string. It checks for M3U, PLS, XSPF and ASX content in the string.
///
/// Returns the parsed entries from the playlist, in playlist order.
///
/// NOTE: currently there is a mix of url and other things in this list
///
/// # Example
///
/// ```rust
/// let list = playlist_decoder::decode(r##"<?xml version="1.0" encoding="UTF-8"?>
///    <playlist version="1" xmlns="http://xspf.org/ns/0/">
///      <trackList>
///        <track>
///          <title>Nobody Move, Nobody Get Hurt</title>
///          <creator>We Are Scientists</creator>
///          <location>file:///mp3s/titel_1.mp3</location>
///        </track>
///        <track>
///          <title>See The World</title>
///          <creator>The Kooks</creator>
///          <location>http://www.example.org/musik/world.ogg</location>
///        </track>
///      </trackList>
///    </playlist>"##).unwrap();
/// assert!(list.len() == 2, "Did not find 2 urls in example");
/// for item in list {
///     println!("{:?}", item);
/// }
/// ```
pub fn decode(content: &str) -> Result<Vec<String>> {
    let mut set: Vec<String> = vec![];
    let content_small = content.to_lowercase();

    if content_small.contains("<playlist") {
        let items = xspf::decode(content)?;
        set.reserve(items.len());
        for item in items {
            if !item.url.is_empty() {
                set.push(item.url);
            }
        }
    } else if content_small.contains("<asx") {
        let items = asx::decode(content)?;
        set.reserve(items.len());
        for item in items {
            set.push(item.url);
        }
    } else if content_small.contains("[playlist]") {
        let items = pls::decode(content);
        set.reserve(items.len());
        for item in items {
            set.push(item.url);
        }
    } else {
        let items = m3u::decode(content);
        set.reserve(items.len());
        for item in items {
            set.push(item.url);
        }
    }

    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_parse_xspf() {
        let s = r#"<?xml version="1.0" encoding="UTF-8"?>
        <playlist version="1" xmlns="http://xspf.org/ns/0/">
            <trackList>
            <track>
                <title>Title</title>
                <identifier>Identifier</identifier>
                <location>http://this.is.an.example</location>
            </track>
            </trackList>
        </playlist>"#;
        let items = decode(s).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], "http://this.is.an.example");
    }

    #[test]
    fn should_parse_asx() {
        let s = r#"<asx version="3.0">
  <title>Test-Liste</title>
  <entry>
    <title>title1</title>
    <ref href="ref1"/>
  </entry>
</asx>"#;
        let items = decode(s).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], "ref1");
    }

    #[test]
    fn should_parse_pls() {
        let items = decode(
            "[playlist]
File1=http://this.is.an.example
Title1=mytitle
        ",
        )
        .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], "http://this.is.an.example");
    }

    #[test]
    fn should_parse_m3u() {
        let playlist = "/some/absolute/unix/path.mp3";

        let results = decode(&playlist).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "/some/absolute/unix/path.mp3");
    }
}
