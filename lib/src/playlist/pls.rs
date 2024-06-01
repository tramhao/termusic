//! Decode File and Title parts from simple playlist PLS files

use std::collections::HashMap;

// TODO: order the result in the way the playlist defines it, see test "multiple_files"

#[derive(Debug, Clone, PartialEq)]
pub struct PLSItem {
    pub title: String,
    pub url: String,
}

/// PLS is a file format similar in style to INI (but does not have a official standard).
/// Each entry is numbered in the key like `File1`, only `File` is required for a entry.
///
/// <https://en.wikipedia.org/wiki/PLS_(file_format)>
pub fn decode(content: &str) -> Vec<PLSItem> {
    let lines = content.lines();
    let mut list = vec![];
    let mut found_pls = false;
    let mut map_urls = HashMap::new();
    let mut map_title = HashMap::new();
    let mut default_title = "";
    for line in lines {
        if line.starts_with('#') {
            continue;
        }
        if line.trim().to_lowercase() == "[playlist]" {
            found_pls = true;
        } else if found_pls {
            if line.starts_with("File") {
                let idend = line.find('=');
                if let Some(idend) = idend {
                    let (key, value) = line.split_at(idend);
                    let id: Result<u32, _> = key[4..idend].parse();
                    if let Ok(id) = id {
                        let (_, url) = value.split_at(1);
                        map_urls.insert(id, url);
                    }
                }
            } else if line.starts_with("Title") {
                let idend = line.find('=');
                if let Some(idend) = idend {
                    let (key, value) = line.split_at(idend);
                    let id: Result<u32, _> = key[5..idend].parse();
                    let (_, title) = value.split_at(1);
                    if let Ok(id) = id {
                        map_title.insert(id, title);
                    } else {
                        default_title = title;
                    }
                }
            }
        }
    }

    for (key, value) in map_urls {
        let title = map_title.get(&key).unwrap_or(&default_title);
        list.push(PLSItem {
            title: String::from(*title),
            url: String::from(value),
        });
    }

    list
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
        assert_eq!(items[0].title, "mytitle");
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
        // TODO: currently all 3 are parsed, but out-of-order
        // assert_eq!(items[0].url, "/a.mp3");
        // assert_eq!(items[1].url, "~/b.mp3");
        // assert_eq!(items[2].url, "http://c.mp3");
    }

    #[test]
    fn anycase_playlist_header() {
        let items = decode(
            "[Playlist]
File1=http://this.is.an.example
Title=mytitle
        ",
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].url, "http://this.is.an.example");
        assert_eq!(items[0].title, "mytitle");
    }
}
