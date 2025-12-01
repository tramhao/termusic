//! Extract urls from M3U playlist files

// TODO: resolve relative paths

use super::PlaylistValue;

#[derive(Debug, Clone, PartialEq)]
pub struct M3UItem {
    pub url: PlaylistValue,
}

/// M3U(8) is a de-facto standard (meaning there is no formal standard), where each line that does not start with `#` is a entry, separated by newlines
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

        let mut p_value = match PlaylistValue::try_from_str(line) {
            Ok(v) => v,
            Err(err) => {
                warn!("Failed to parse url / path, ignoring! Error: {err:#?}");
                continue;
            }
        };
        if let Err(err) = p_value.file_url_to_path() {
            warn!("Failed to convert file:// url to path, ignoring! Error: {err:#?}");
            continue;
        }

        list.push(M3UItem { url: p_value });
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use reqwest::Url;

    #[test]
    fn should_parse() {
        let playlist = r"/some/absolute/unix/path.mp3
# this is a test comment, below is a empty line to be ignored

relative.mp3
https://somewhere.url/path";

        let results = decode(playlist);
        assert_eq!(results.len(), 3);
        assert_eq!(
            results[0].url,
            PlaylistValue::Path("/some/absolute/unix/path.mp3".into())
        );
        assert_eq!(results[1].url, PlaylistValue::Path("relative.mp3".into()));
        assert_eq!(
            results[2].url,
            PlaylistValue::Url(Url::parse("https://somewhere.url/path").unwrap())
        );
    }
}
