use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_json::Value;
// use std::io::Write;
use custom_error::custom_error;
use std::time::Duration;

pub struct InvidiousInstance {
    domain: String,
    client: Client,
}

pub struct YoutubeVideo {
    pub title: String,
    pub length_seconds: u64,
    pub video_id: String,
}

impl InvidiousInstance {
    pub fn new(domain: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            // .cookies()
            .build()
            .expect("Initialize Invidious Client Failed!");
        Self { domain, client }
    }

    // GetSearchQuery fetches query result from an Invidious instance.
    pub fn get_search_query(&mut self, query: &str, page: u32) -> Result<Vec<YoutubeVideo>> {
        // query = url.QueryEscape(query)

        let mut url: String = self.domain.clone();
        url.push_str("/api/v1/search?q=");
        url.push_str(query);
        url.push_str("&page=");
        url.push_str(&page.to_string());

        let result = self.client.get(&url).send()?;

        match result.status() {
            StatusCode::OK => InvidiousInstance::parse_youtube_options(result.text().unwrap()),
            _ => Err(anyhow!("Error during search")),
        }
    }

    // GetSuggestions returns video suggestions based on prefix strings. This is the
    // same result as youtube search autocomplete.
    pub fn get_suggestions(&mut self, prefix: &str) -> Result<Vec<YoutubeVideo>> {
        // query := url.QueryEscape(prefix)
        // targetUrl :=
        let mut url =
            "http://suggestqueries.google.com/complete/search?client=firefox&ds=yt&q=".to_string();
        url.push_str(prefix);

        let result = self.client.get(&url).send()?;
        match result.status() {
            StatusCode::OK => InvidiousInstance::parse_youtube_options(result.text().unwrap()),
            _ => Err(anyhow!("Error during search")),
        }

        // res := []json.RawMessage{}
        // err := getRequest(targetUrl, &res)
        // if err != nil {
        // 	return nil, tracerr.Wrap(err)
        // }

        // suggestions := []string{}
        // err = json.Unmarshal(res[1], &suggestions)
        // if err != nil {
        // 	return nil, tracerr.Wrap(err)
        // }

        // return suggestions, nil
    }

    // GetTrendingMusic fetch music trending based on region.
    // Region (ISO 3166 country code) can be provided in the argument.
    pub fn get_trending_music(&mut self, region: &str) -> Result<Vec<YoutubeVideo>> {
        // params := fmt.Sprintf("type=music&region=%s", region)
        // targetUrl := i.Domain + "/api/v1/trending?" + params
        let mut url: String = self.domain.clone();
        url.push_str("/api/v1/trending?");
        url.push_str("type=music&region=");
        url.push_str(region);

        let result = self.client.get(&url).send()?;

        match result.status() {
            StatusCode::OK => InvidiousInstance::parse_youtube_options(result.text().unwrap()),
            _ => Err(anyhow!("Error during search")),
        }
    }

    pub fn parse_youtube_options(data: String) -> Result<Vec<YoutubeVideo>> {
        let value = serde_json::from_str::<Value>(&data)?;
        let mut vec: Vec<YoutubeVideo> = Vec::new();
        // let mut file = std::fs::File::create("data.txt").expect("create failed");
        // file.write_all(data.as_bytes()).expect("write failed");
        let array = value.as_array().unwrap();
        for v in array.iter() {
            vec.push(YoutubeVideo {
                title: v
                    .get("title")
                    .ok_or(Errors::NoneError)?
                    .as_str()
                    .ok_or(Errors::NoneError)?
                    .to_owned(),
                video_id: v
                    .get("videoId")
                    .ok_or(Errors::NoneError)?
                    .as_str()
                    .ok_or(Errors::NoneError)?
                    .to_owned(),
                length_seconds: v
                    .get("lengthSeconds")
                    .ok_or(Errors::NoneError)?
                    .as_u64()
                    .ok_or(Errors::NoneError)? as u64,
            })
        }
        Ok(vec)
    }
}

custom_error! { pub Errors
    OpenSSLError{ source: openssl::error::ErrorStack } = "openSSL Error",
    RegexError{ source: regex::Error } = "regex Error",
    SerdeJsonError{ source: serde_json::error::Error } = "serde json Error",
    ParseError{ source: std::num::ParseIntError } = "parse Error",
    // AsyncIoError{ source: io::Error } = "async io Error",
    // IsahcError{ source: isahc::Error } = "isahc Error",
    NoneError = "None Error",
    // FromUtf8Error{source: std::string::FromUtf8Error} = "UTF8 Error",
}
