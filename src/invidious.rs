/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use anyhow::{anyhow, bail, Result};
use rand::seq::SliceRandom;
use serde_json::Value;
use std::time::Duration;
use ureq::{Agent, AgentBuilder};

const INVIDIOUS_INSTANCE_LIST: [&str; 7] = [
    "https://vid.puffyan.us",
    "https://ytprivate.com",
    "https://invidio.xamh.de",
    "https://youtube.076.ne.jp",
    "https://y.com.cm",
    "https://invidious.hub.ne.kr",
    "https://invidious.namazso.eu",
];

pub struct Instance {
    pub domain: Option<String>,
    client: Agent,
    query: Option<String>,
}

pub struct YoutubeVideo {
    pub title: String,
    pub length_seconds: u64,
    pub video_id: String,
}

impl Default for Instance {
    fn default() -> Self {
        let client = Agent::new();
        let domain = Some(String::new());
        let query = Some(String::new());

        Self {
            domain,
            client,
            query,
        }
    }
}

#[allow(unused)]
impl Instance {
    pub fn new(query: &str) -> Result<(Self, Vec<YoutubeVideo>)> {
        let client = AgentBuilder::new().timeout(Duration::from_secs(10)).build();

        let mut domain = String::new();
        let mut domains = INVIDIOUS_INSTANCE_LIST;
        let mut video_result: Vec<YoutubeVideo> = Vec::new();
        domains.shuffle(&mut rand::thread_rng());
        for v in domains {
            let url = format!("{}/api/v1/search", v);

            let result = client
                .get(&url)
                .query("q", query)
                .query("page", "1")
                .call()?;
            if result.status() == 200 {
                let text = result.into_string()?;
                let vr =
                    Self::parse_youtube_options(&text).ok_or_else(|| anyhow!("parse error"))?;
                video_result = vr;
                domain = v.to_string();
                break;
            }
        }
        if domain.len() < 2 {
            bail!("All 7 invidious servers are down? Please check your network connection first.");
        }

        let domain = Some(domain);
        Ok((
            Self {
                domain,
                client,
                query: Some(query.to_string()),
            },
            video_result,
        ))
    }

    // GetSearchQuery fetches query result from an Invidious instance.
    pub fn get_search_query(&self, page: u32) -> Result<Vec<YoutubeVideo>> {
        if self.domain.is_none() {
            bail!("No server available");
        }
        let url = format!("{}/api/v1/search", self.domain.as_ref().unwrap());

        let query = match &self.query {
            Some(q) => q,
            None => bail!("No query string found"),
        };

        let result = self
            .client
            .get(&url)
            .query("q", query)
            .query("page", &page.to_string())
            .call()?;

        match result.status() {
            200 => match result.into_string() {
                Ok(text) => Self::parse_youtube_options(&text).ok_or_else(|| anyhow!("None Error")),
                Err(e) => bail!("Error during search: {}", e),
            },
            _ => bail!("Error during search"),
        }
    }

    // GetSuggestions returns video suggestions based on prefix strings. This is the
    // same result as youtube search autocomplete.
    pub fn get_suggestions(&self, prefix: &str) -> Result<Vec<YoutubeVideo>> {
        let url = format!(
            "http://suggestqueries.google.com/complete/search?client=firefox&ds=yt&q={}",
            prefix
        );
        let result = self.client.get(&url).call()?;
        match result.status() {
            200 => match result.into_string() {
                Ok(text) => Self::parse_youtube_options(&text).ok_or_else(|| anyhow!("None Error")),
                Err(e) => bail!("Error during search: {}", e),
            },
            _ => bail!("Error during search"),
        }
    }

    // GetTrendingMusic fetch music trending based on region.
    // Region (ISO 3166 country code) can be provided in the argument.
    pub fn get_trending_music(&self, region: &str) -> Result<Vec<YoutubeVideo>> {
        if self.domain.is_none() {
            bail!("No server available");
        }
        let url = format!(
            "{}/api/v1/trending?type=music&region={}",
            self.domain.as_ref().unwrap(),
            region
        );

        let result = self.client.get(&url).call()?;

        match result.status() {
            200 => match result.into_string() {
                Ok(text) => Self::parse_youtube_options(&text).ok_or_else(|| anyhow!("None Error")),
                _ => bail!("Error during search"),
            },
            _ => bail!("Error during search"),
        }
    }

    fn parse_youtube_options(data: &str) -> Option<Vec<YoutubeVideo>> {
        if let Ok(value) = serde_json::from_str::<Value>(data) {
            let mut vec: Vec<YoutubeVideo> = Vec::new();
            // below two lines are left for debug purpose
            // let mut file = std::fs::File::create("data.txt").expect("create failed");
            // file.write_all(data.as_bytes()).expect("write failed");
            if let Some(array) = value.as_array() {
                for v in array.iter() {
                    vec.push(YoutubeVideo {
                        title: v.get("title")?.as_str()?.to_owned(),
                        video_id: v.get("videoId")?.as_str()?.to_owned(),
                        length_seconds: v.get("lengthSeconds")?.as_u64()?,
                    });
                }
                return Some(vec);
            }
        }
        None
    }
}
