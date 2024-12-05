use std::{collections::HashMap, time::Duration};

use anyhow::anyhow;
use bytes::Buf as _;
use lofty::picture::Picture;
use model::{to_lyric, to_song_info, to_song_url};
use reqwest::{Client, ClientBuilder, RequestBuilder};

use crate::songtag::ServiceProvider;

use super::{
    netease::encrypt::Crypto,
    service::{SongTagService, SongTagServiceError},
};

mod model;

const URL_SEARCH_NETEASE: &str = "https://music.163.com/weapi/search/get";
const URL_LYRIC_NETEASE: &str = "https://music.163.com/weapi/song/lyric";
const URL_DOWNLOAD_NETEASE: &str = "https://music.163.com/weapi/song/enhance/player/url/v1";
const URL_PICTURE_SERVICE: &str = "https://p3.music.126.net/";

const REFERER: &str = "https://music.163.com";
const HOST: &str = "music.163.com";

pub struct Api {
    client: Client,
}

impl Api {
    pub fn new() -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build reqwest client.");

        Self { client }
    }

    fn common_request(builder: RequestBuilder, params: &[(&str, &str)]) -> RequestBuilder {
        let body = {
            // using hashmap as the API expects a object, a Vec would create a array
            let params: HashMap<&str, &str> = HashMap::from_iter(params.iter().map(|v| (v.0, v.1)));
            // there does not seem to be any CSRF-Token need to be set from the responses
            // params.insert("csrf_token", "");

            let text = serde_json::to_string(&params).unwrap();

            Crypto::weapi(&text).unwrap()
        };

        builder
            .header("Cookie", "os=pc; appver=2.7.1.198277")
            .header("Accept", "*/*")
            .header("Accept-Encoding", "gzip,deflate")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Connection", "keep-alive")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Host", HOST)
            .header("Referer", REFERER)
            .body(body)
    }
}

impl SongTagService for Api {
    type Error = anyhow::Error;

    fn display_name() -> &'static str
    where
        Self: Sized,
    {
        "netease"
    }

    async fn search_recording(
        &self,
        keywords: &str,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<super::SongTag>, super::service::SongTagServiceError<Self::Error>> {
        let offset = offset.to_string();
        let limit = limit.to_string();

        let query_params = &[
            ("s", keywords),
            // types: single (1), singer (100), album (10), playlist (1000), user (1002) *(type)*
            ("type", "1"),
            ("offset", &offset),
            ("limit", &limit),
        ];

        let response = Self::common_request(self.client.post(URL_SEARCH_NETEASE), query_params)
            .send()
            .await
            .map_err(anyhow::Error::from)?;

        let result = response.text().await.map_err(anyhow::Error::from)?;

        to_song_info(&result).map_err(|err| {
            SongTagServiceError::Other(err.context("convert result to songtag array"))
        })
    }

    async fn get_lyrics(
        &self,
        song: &super::SongTag,
    ) -> Result<String, super::service::SongTagServiceError<Self::Error>> {
        if song.service_provider() != ServiceProvider::Netease {
            return Err(SongTagServiceError::IncorrectService(
                song.service_provider().to_string(),
                Self::display_name(),
            ));
        }

        let Some(lyric_id) = &song.lyric_id else {
            return Err(SongTagServiceError::Other(anyhow!(
                "Provided songtag does not have a lyric_id!"
            )));
        };

        let query_params = &[
            ("id", lyric_id.as_str()),
            // what do they mean?
            ("lv", "-1"),
            ("tv", "-1"),
        ];

        let response = Self::common_request(self.client.post(URL_LYRIC_NETEASE), query_params)
            .send()
            .await
            .map_err(anyhow::Error::from)?;

        let result = response.text().await.map_err(anyhow::Error::from)?;

        to_lyric(&result)
            .map_err(|err| SongTagServiceError::Other(err.context("get lyric text from response")))
    }

    async fn get_picture(
        &self,
        song: &super::SongTag,
    ) -> Result<lofty::picture::Picture, super::service::SongTagServiceError<Self::Error>> {
        if song.service_provider() != ServiceProvider::Netease {
            return Err(SongTagServiceError::IncorrectService(
                song.service_provider().to_string(),
                Self::display_name(),
            ));
        }

        let Some(pic_id) = &song.pic_id else {
            return Err(SongTagServiceError::Other(anyhow!(
                "Provided songtag does not have a pic_id!"
            )));
        };

        let id_encrypted = Crypto::encrypt_id(&pic_id);
        let pic_url = format!("{URL_PICTURE_SERVICE}{id_encrypted}/{pic_id}.jpg?param=300y300");

        let result = self
            .client
            .get(pic_url)
            .send()
            .await
            .map_err(anyhow::Error::from)?;

        let mut reader = result.bytes().await.map_err(anyhow::Error::from)?.reader();
        let picture = Picture::from_reader(&mut reader).map_err(anyhow::Error::from)?;

        Ok(picture)
    }

    async fn download_recording(
        &self,
        song: &super::SongTag,
    ) -> Result<String, super::service::SongTagServiceError<Self::Error>> {
        if song.service_provider() != ServiceProvider::Netease {
            return Err(SongTagServiceError::IncorrectService(
                song.service_provider().to_string(),
                Self::display_name(),
            ));
        }

        // netease wants the "ids" as a string array, but we only have 1
        let ids = serde_json::to_string(&[&song.song_id]).unwrap();

        let query_params = &[
            ("ids", ids.as_str()),
            // what does this control?
            ("level", "standard"),
            ("encodeType", "aac"),
        ];

        let response = Self::common_request(self.client.post(URL_DOWNLOAD_NETEASE), query_params)
            .send()
            .await
            .map_err(anyhow::Error::from)?;

        let result = response.text().await.map_err(anyhow::Error::from)?;

        to_song_url(&result).map_err(|err| {
            SongTagServiceError::Other(err.context("get download url from response"))
        })
    }
}
