use anyhow::{Result, anyhow, bail};
use rand::seq::SliceRandom;
use serde_json::Value;
use reqwest::{Client, ClientBuilder, StatusCode};
use std::time::Duration;

const INVIDIOUS_INSTANCE_LIST: [&str; 5] = [
    "https://inv.nadeko.net",
    "https://invidious.nerdvpn.de",
    "https://yewtu.be",
    "https://y.com.sb",
    "https://yt.artemislena.eu",
];

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

impl Instance {
    pub async fn new(query: &str) -> Result<(Self, Vec<YoutubeVideo>)> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(5))
            .build()?;

        let mut instances: Vec<&str> = INVIDIOUS_INSTANCE_LIST.to_vec();
        instances.shuffle(&mut rand::rng());

        for v in instances {
            let url = format!("{v}/api/v1/search");
            let query_vec = vec![
                ("q", query),
                ("page", "1"),
                ("type", "video"),
                ("sort_by", "relevance"),
            ];

            if let Ok(result) = client.get(&url).query(&query_vec).send().await
                && result.status() == 200
                && let Ok(text) = result.text().await
                && let Some(vr) = Self::parse_youtube_options(&text)
            {
                let domain = Some(v.to_string());
                let instance = Self {
                    domain,
                    client,
                    query: Some(query.to_string()),
                };
                return Ok((instance, vr));
            }
        }

        bail!("All invidious servers are down. Try again later.")
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
                Err(e) => bail!("Error during search: {e}"),
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
                Err(e) => bail!("Error during search: {e}"),
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
}
