//! This is a very simple url extractor for different kinds of playlist formats: M3U, PLS, ASX, XSPF
//!
//! It is not optimized yet and does create a lot of strings on the way.

mod asx;
mod m3u;
mod pls;
mod xspf;

use std::{
    borrow::Cow,
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use reqwest::Url;

use crate::utils;

#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(clippy::module_name_repetitions)]
pub enum PlaylistValue {
    /// A Local path, specific to the current running system (unix / dos)
    Path(PathBuf),
    /// A URI / URL starting with a protocol
    Url(Url),
}

impl From<PathBuf> for PlaylistValue {
    fn from(value: PathBuf) -> Self {
        Self::Path(value)
    }
}

impl Display for PlaylistValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaylistValue::Path(v) => v.display().fmt(f),
            PlaylistValue::Url(v) => v.fmt(f),
        }
    }
}

impl PlaylistValue {
    /// If the current value is a [`PlaylistValue::Url`] and has the `file://` protocol, convert it to a path
    ///
    /// # Errors
    ///
    /// If the url's scheme is `file://` but converting to a pathbuf fails, see [`reqwest::Url::to_file_path`]
    pub fn file_url_to_path(&mut self) -> Result<()> {
        let Self::Url(url) = self else {
            // dont do anything if not a url
            return Ok(());
        };

        if url.scheme() == "file" {
            let as_path = url
                .to_file_path()
                .map_err(|()| anyhow!("Failed to convert URL to Path!"))
                .context(url.to_string())?;
            *self = Self::Path(as_path);
        }

        Ok(())
    }

    /// If the current value is a [`PlaylistValue::Path`] and not absolute, make it absolute via the provided `base`
    ///
    /// `base` is expected to be absolute!
    pub fn absoluteize(&mut self, base: &Path) {
        let Self::Path(path) = self else {
            return;
        };

        // do nothing if path is already absolute
        if path.is_absolute() {
            return;
        }

        // only need to change the path if the return is owned
        if let Cow::Owned(new_path) = utils::absolute_path_base(path, base) {
            *path = new_path;
        }
    }

    /// Try to parse the given string
    pub fn try_from_str(line: &str) -> Result<Self> {
        // maybe not the best check, but better than nothing
        if line.contains("://") {
            return Ok(Self::Url(Url::parse(line)?));
        }

        Ok(Self::Path(PathBuf::from_str(line)?))
    }
}

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
pub fn decode(content: &str) -> Result<Vec<PlaylistValue>> {
    let mut set: Vec<PlaylistValue> = vec![];
    let content_small = content.to_lowercase();

    if content_small.contains("<playlist") {
        let items = xspf::decode(content)?;
        set.reserve(items.len());
        for item in items {
            set.push(item.location);
        }
    } else if content_small.contains("<asx") {
        let items = asx::decode(content)?;
        set.reserve(items.len());
        for item in items {
            set.push(item.location);
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
        assert_eq!(
            items[0],
            PlaylistValue::Url(Url::parse("http://this.is.an.example").unwrap())
        );
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
        assert_eq!(items[0], PlaylistValue::Path("ref1".into()));
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
        assert_eq!(
            items[0],
            PlaylistValue::Url(Url::parse("http://this.is.an.example").unwrap())
        );
    }

    #[test]
    fn should_parse_m3u() {
        let playlist = "/some/absolute/unix/path.mp3";

        let results = decode(playlist).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0],
            PlaylistValue::Path("/some/absolute/unix/path.mp3".into())
        );
    }
}
