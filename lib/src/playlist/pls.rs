//! Decode File and Title parts from simple playlist PLS files

use std::collections::{hash_map::Entry, HashMap};

#[derive(Debug, Clone, PartialEq)]
pub struct PLSItem {
    pub title: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct PrivateItem {
    pub title: Option<String>,
    pub url: Option<String>,
}

/// PLS is a file format similar in style to INI (but does not have a official standard).
/// Each entry is numbered in the key like `File1`, only `File` is required for a entry.
///
/// <https://en.wikipedia.org/wiki/PLS_(file_format)>
pub fn decode(content: &str) -> Vec<PLSItem> {
    let mut lines = content.lines();
    let mut list: HashMap<u16, PrivateItem> = HashMap::with_capacity(1);

    if let Found::No = skip_until_playlist(&mut lines) {
        error!("Requested PLS playlist, but did not find PLS header!");

        return Vec::new();
    }

    // optional "NumberOfEntries" footer checking
    let mut number_of_entries: Option<u16> = None;

    for line in lines {
        if is_comment(line) || line.is_empty() {
            continue;
        }

        // because it matches INI style, there could be another header, we dont want to parse that though
        if line.starts_with('[') {
            info!("Found another header in PLS playlist!");
            break;
        }

        if let Some(remainder) = line.strip_prefix("File") {
            let Some((num, url)) = parse_id(remainder, line) else {
                continue;
            };

            match list.entry(num) {
                Entry::Occupied(mut o) => {
                    let val = o.get_mut();
                    if val.url.is_some() {
                        warn!("Entry {} already had a URL set, overwriting!", num);
                    }
                    val.url.replace(url.to_string());
                }
                Entry::Vacant(v) => {
                    v.insert(PrivateItem {
                        url: Some(url.to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        if let Some(remainder) = line.strip_prefix("Title") {
            let Some((num, title)) = parse_id(remainder, line) else {
                continue;
            };

            match list.entry(num) {
                Entry::Occupied(mut o) => {
                    let val = o.get_mut();
                    if val.title.is_some() {
                        warn!("Entry {} already had a Title set, overwriting!", num);
                    }
                    val.title.replace(title.to_string());
                }
                Entry::Vacant(v) => {
                    v.insert(PrivateItem {
                        title: Some(title.to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        if let Some(remainder) = line.strip_prefix("NumberOfEntries") {
            let Some((_, remainder)) = remainder.split_once('=') else {
                warn!("Malformed line: {:#?}", line);
                continue;
            };

            let Ok(num) = remainder.parse::<u16>() else {
                warn!("Couldnt parse NumberOfEntries number! line: {:#?}", line);
                continue;
            };

            number_of_entries.replace(num);
        }
    }

    // convert to the returned struct format, while also warning about bad entries and preserving number
    let mut list: Vec<(u16, PLSItem)> = list
        .into_iter()
        .filter(|v| {
            if v.1.url.is_none() {
                warn!("PLS Entry {} has no url, excluding!", v.0);
            }

            v.1.url.is_some()
        })
        .map(|v| {
            (
                v.0,
                PLSItem {
                    title: v.1.title,
                    // Safe unwrap, because of the filter
                    url: v.1.url.unwrap(),
                },
            )
        })
        .collect();

    if let Some(number_of_entries) = number_of_entries {
        if number_of_entries as usize != list.len() {
            warn!(
                "NumberOfEntries mismatch! List: {}, NumberOfEntries: {}",
                list.len(),
                number_of_entries
            );
        }
    }

    // sort by the numbers (like `File1`) to actually be in playlist order
    list.sort_by(|a, b| a.0.cmp(&b.0));

    // convert into array without the preserved number as it is now sorted
    list.into_iter().map(|v| v.1).collect()
}

/// Parse a Entry id from the start of the value until the first `=`.
///
/// Returns the parsed number and the remainder after the first `=`.
fn parse_id<'a>(val: &'a str, line: &str) -> Option<(u16, &'a str)> {
    if let Some((id, remainder)) = val.split_once('=') {
        let Ok(num) = id.parse::<u16>() else {
            error!("Couldnt parse PLS entry id for line {:#?}", line);
            return None;
        };

        return Some((num, remainder));
    }

    None
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Found {
    Yes,
    No,
}

/// Skip the given iterator until a `[playlist]` header, or return [`Found::No`]
fn skip_until_playlist<'a, 'b>(iter: &'a mut impl Iterator<Item = &'b str>) -> Found {
    loop {
        // if there is no line, we have not found the start
        let Some(line) = iter.next() else {
            return Found::No;
        };

        // skip all comments or empty lines, as they *could* be before anything
        if line.is_empty() || is_comment(line) {
            continue;
        }

        // return when finding the correct header
        if line.trim().to_ascii_lowercase() == "[playlist]" {
            return Found::Yes;
        }
    }
}

/// Check if the given value starts with a PLS / INI comment
#[inline]
fn is_comment(val: &str) -> bool {
    val.starts_with('#') || val.starts_with(';')
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn one_file() {
        let items = decode(
            "[playlist]
File1=http://this.is.an.example
Title1=mytitle
        ",
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].url, "http://this.is.an.example");
        assert_eq!(items[0].title, Some("mytitle".to_string()));
    }

    #[test]
    fn multiple_files() {
        let items = decode(
            "[playlist]
File1=/a.mp3
File2=~/b.mp3
File3=http://c.mp3",
        );
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].url, "/a.mp3");
        assert_eq!(items[1].url, "~/b.mp3");
        assert_eq!(items[2].url, "http://c.mp3");
    }

    #[test]
    fn anycase_playlist_header() {
        let items = decode(
            "[Playlist]
File1=http://this.is.an.example
Title1=mytitle
        ",
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].url, "http://this.is.an.example");
        assert_eq!(items[0].title, Some("mytitle".to_string()));
    }
}
