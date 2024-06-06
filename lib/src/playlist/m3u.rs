//! Extract urls from M3U playlist files

// TODO: resolve relative paths

#[derive(Debug, Clone, PartialEq)]
pub struct M3UItem {
    pub url: String,
}

/// M3U(8) is a de-facto standart (meaning there is no formal standard), where each line that does not start with `#` is a entry, separated by newlines
///
/// <https://en.wikipedia.org/wiki/M3U#File_format>
pub fn decode(content: &str) -> Vec<M3UItem> {
    let lines = content.lines();
    let mut list = vec![];
    for line in lines {
        if line.is_empty() {
            continue;
        }

        if line.starts_with('#') {
            continue;
        }

        list.push(M3UItem {
            url: String::from(line),
        });
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_parse() {
        let playlist = r#"/some/absolute/unix/path.mp3
# this is a test comment, below is a empty line to be ignored

relative.mp3
https://somewhere.url/path"#;

        let results = decode(&playlist);
        assert_eq!(results.len(), 3);
        assert_eq!(&results[0].url, "/some/absolute/unix/path.mp3");
        assert_eq!(&results[1].url, "relative.mp3");
        assert_eq!(&results[2].url, "https://somewhere.url/path");
    }
}
