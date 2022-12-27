// Thanks to the author of shellcaster(https://github.com/jeff-hughes/shellcaster). Parts of following code is taken from it.

use anyhow::{anyhow, Result};
use opml::OPML;

#[derive(Debug, Clone)]
pub struct PodcastFeed {
    pub id: Option<i64>,
    pub url: String,
    pub title: Option<String>,
}

impl PodcastFeed {
    pub fn new(id: Option<i64>, url: String, title: Option<String>) -> Self {
        return Self {
            id: id,
            url: url,
            title: title,
        };
    }
}

pub fn podcast_import(xml: &str) -> Result<Vec<PodcastFeed>> {
    return match OPML::from_str(xml) {
        Err(err) => Err(anyhow!(err)),
        Ok(opml) => {
            let mut feeds = Vec::new();
            for pod in opml.body.outlines.into_iter() {
                if pod.xml_url.is_some() {
                    // match against title attribute first -- if this is
                    // not set or empty, then match against the text
                    // attribute; this must be set, but can be empty
                    let temp_title = pod.title.filter(|t| !t.is_empty());
                    let title = match temp_title {
                        Some(t) => Some(t),
                        None => {
                            if pod.text.is_empty() {
                                None
                            } else {
                                Some(pod.text)
                            }
                        }
                    };
                    feeds.push(PodcastFeed::new(None, pod.xml_url.unwrap(), title));
                }
            }
            Ok(feeds)
        }
    };
}
