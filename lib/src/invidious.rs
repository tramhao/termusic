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
// left for debug
// use std::io::Write;
use reqwest::{Client, ClientBuilder, StatusCode};
use std::time::Duration;

const INVIDIOUS_INSTANCE_LIST: [&str; 7] = [
    "https://vid.puffyan.us",
    "https://inv.riverside.rocks",
    "https://invidious.osi.kr",
    "https://youtube.076.ne.jp",
    "https://y.com.sb",
    "https://yt.artemislena.eu",
    "https://invidious.tiekoetter.com",
    // Below lines are left for testing
    // "https://www.google.com",
    // "https://www.google.com",
    // "https://www.google.com",
    // "https://www.google.com",
    // "https://www.google.com",
    // "https://www.google.com",
    // "https://www.google.com",
];

const INVIDIOUS_DOMAINS: &str = "https://api.invidious.io/instances.json?sort_by=type,users";

#[derive(Clone, Debug)]
pub struct Instance {
    pub domain: Option<String>,
    client: Client,
    query: Option<String>,
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        self.domain == other.domain
    }
}

impl Eq for Instance {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YoutubeVideo {
    pub title: String,
    pub length_seconds: u64,
    pub video_id: String,
}

impl Default for Instance {
    fn default() -> Self {
        let client = Client::new();
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
    pub async fn new(query: &str) -> Result<(Self, Vec<YoutubeVideo>)> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(10))
            .build()?;

        let mut domain = String::new();
        let mut domains = vec![];

        // prefor fetch invidious instance from website, but will provide 7 backups
        if let Ok(domain_list) = Self::get_invidious_instance_list(&client).await {
            domains = domain_list;
        } else {
            for item in &INVIDIOUS_INSTANCE_LIST {
                domains.push((*item).to_string());
            }
        }

        domains.shuffle(&mut rand::thread_rng());

        let mut video_result: Vec<YoutubeVideo> = Vec::new();
        for v in domains {
            let url = format!("{v}/api/v1/search");

            let query_vec = vec![
                ("q", query),
                ("page", "1"),
                ("type", "video"),
                ("sort_by", "relevance"),
            ];
            if let Ok(result) = client.get(&url).query(&query_vec).send().await {
                if result.status() == 200 {
                    if let Ok(text) = result.text().await {
                        if let Some(vr) = Self::parse_youtube_options(&text) {
                            video_result = vr;
                            domain = v;
                            break;
                        }
                    }
                }
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
    pub async fn get_search_query(&self, page: u32) -> Result<Vec<YoutubeVideo>> {
        if self.domain.is_none() {
            bail!("No server available");
        }
        let url = format!(
            "{}/api/v1/search",
            self.domain
                .as_ref()
                .ok_or(anyhow!("error in domain name"))?
        );

        let Some(query) = &self.query else {
            bail!("No query string found")
        };

        let result = self
            .client
            .get(url)
            .query(&[("q", query), ("page", &page.to_string())])
            .send()
            .await?;

        match result.status() {
            StatusCode::OK => match result.text().await {
                Ok(text) => Self::parse_youtube_options(&text).ok_or_else(|| anyhow!("None Error")),
                Err(e) => bail!("Error during search: {}", e),
            },
            _ => bail!("Error during search"),
        }
    }

    // GetSuggestions returns video suggestions based on prefix strings. This is the
    // same result as youtube search autocomplete.
    pub async fn get_suggestions(&self, prefix: &str) -> Result<Vec<YoutubeVideo>> {
        let url = format!(
            "http://suggestqueries.google.com/complete/search?client=firefox&ds=yt&q={prefix}"
        );
        let result = self.client.get(url).send().await?;
        match result.status() {
            StatusCode::OK => match result.text().await {
                Ok(text) => Self::parse_youtube_options(&text).ok_or_else(|| anyhow!("None Error")),
                Err(e) => bail!("Error during search: {}", e),
            },
            _ => bail!("Error during search"),
        }
    }

    // GetTrendingMusic fetch music trending based on region.
    // Region (ISO 3166 country code) can be provided in the argument.
    pub async fn get_trending_music(&self, region: &str) -> Result<Vec<YoutubeVideo>> {
        if self.domain.is_none() {
            bail!("No server available");
        }
        let url = format!(
            "{}/api/v1/trending?type=music&region={region}",
            self.domain
                .as_ref()
                .ok_or(anyhow!("error in domain names"))?
        );

        let result = self.client.get(url).send().await?;

        match result.status() {
            StatusCode::OK => match result.text().await {
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
                for v in array {
                    if let Some((title, video_id, length_seconds)) = Self::parse_youtube_item(v) {
                        vec.push(YoutubeVideo {
                            title,
                            length_seconds,
                            video_id,
                        });
                    }
                }
                return Some(vec);
            }
        }
        None
    }

    fn parse_youtube_item(value: &Value) -> Option<(String, String, u64)> {
        let title = value.get("title")?.as_str()?.to_owned();
        let video_id = value.get("videoId")?.as_str()?.to_owned();
        let length_seconds = value.get("lengthSeconds")?.as_u64()?;
        Some((title, video_id, length_seconds))
    }

    async fn get_invidious_instance_list(client: &Client) -> Result<Vec<String>> {
        let result = client.get(INVIDIOUS_DOMAINS).send().await?.text().await?;
        // Left here for debug
        // let mut file = std::fs::File::create("data.txt").expect("create failed");
        // file.write_all(result.as_bytes()).expect("write failed");
        if let Some(vec) = Self::parse_invidious_instance_list(&result) {
            return Ok(vec);
        }
        bail!("no instance list fetched")
    }

    fn parse_invidious_instance_list(data: &str) -> Option<Vec<String>> {
        if let Ok(value) = serde_json::from_str::<Value>(data) {
            let mut vec = Vec::new();
            if let Some(array) = value.as_array() {
                for inner_value in array {
                    if let Some((uri, health)) = Self::parse_instance(inner_value) {
                        if health > 95.0 {
                            vec.push(uri);
                        }
                    }
                }
            }
            if !vec.is_empty() {
                return Some(vec);
            }
        }
        None
    }

    fn parse_instance(value: &Value) -> Option<(String, f64)> {
        let obj = value.get(1)?.as_object()?;
        if obj.get("api")?.as_bool()? {
            let uri = obj.get("uri")?.as_str()?.to_owned();
            let health = obj
                .get("monitor")?
                .as_object()?
                .get("30dRatio")?
                .get("ratio")?
                .as_str()?
                .to_owned()
                .parse::<f64>()
                .ok();
            health.map(|health| (uri, health))
        } else {
            None
        }
    }
}
