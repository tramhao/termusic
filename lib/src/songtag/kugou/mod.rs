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
mod model;

use crate::{songtag::service::SongTagServiceError, utils::random_ascii};

use super::{ServiceProvider, SongTag, UrlTypes, service::SongTagService};
use anyhow::anyhow;
use bytes::Buf;
use lofty::picture::Picture;
use model::{to_lyric, to_lyric_id_accesskey, to_pic_url, to_song_info, to_song_url};
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

const URL_SEARCH_KUGOU: &str = "http://mobilecdn.kugou.com/api/v3/search/song";
const URL_LYRIC_SEARCH_KUGOU: &str = "http://krcs.kugou.com/search";
const URL_LYRIC_DOWNLOAD_KUGOU: &str = "http://lyrics.kugou.com/download";
const URL_SONG_DOWNLOAD_KUGOU: &str = "http://www.kugou.com/yy/index.php?r=play/getdata";

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
}

impl SongTagService for Api {
    type Error = anyhow::Error;

    fn display_name() -> &'static str
    where
        Self: Sized,
    {
        "kogou"
    }

    async fn search_recording(
        &self,
        keywords: &str,
        offset: u32,
        limit: u32,
    ) -> std::result::Result<Vec<SongTag>, super::service::SongTagServiceError<Self::Error>> {
        let offset_str = offset.to_string();
        let limit_str = limit.to_string();

        let query_params = vec![
            ("format", "json"),
            ("page", &offset_str),
            ("pagesize", &limit_str),
            // 1 is for type "Song"
            ("showtype", "1"),
            ("keyword", keywords),
        ];

        let result = self
            .client
            .post(URL_SEARCH_KUGOU)
            .header("Referer", "https://m.music.migu.cn")
            .query(&query_params)
            .send()
            .await
            .map_err(anyhow::Error::from)?
            .text()
            .await
            .map_err(anyhow::Error::from)?;

        to_song_info(&result).map_err(|err| {
            SongTagServiceError::Other(anyhow!(err).context("Parse result into SongTag Array"))
        })
    }

    async fn get_lyrics(
        &self,
        song: &SongTag,
    ) -> std::result::Result<String, super::service::SongTagServiceError<Self::Error>> {
        if song.service_provider() != ServiceProvider::Kugou {
            return Err(SongTagServiceError::IncorrectService(
                song.service_provider().to_string(),
                Self::display_name(),
            ));
        }

        let query_params = vec![
            ("keyword", "%20-%20"),
            ("ver", "1"),
            ("hash", &song.song_id),
            ("client", "mobi"),
            // what does this mean?
            ("man", "yes"),
        ];

        let result = self
            .client
            .get(URL_LYRIC_SEARCH_KUGOU)
            .query(&query_params)
            .send()
            .await
            .map_err(anyhow::Error::from)?
            .text()
            .await
            .map_err(anyhow::Error::from)?;

        let (accesskey, id) = to_lyric_id_accesskey(&result).map_err(|err| {
            SongTagServiceError::Other(anyhow!(err).context("Extract accesskey and id from result"))
        })?;

        let query_params = vec![
            ("charset", "utf8"),
            ("accesskey", &accesskey),
            ("id", &id),
            ("client", "mobi"),
            ("fmt", "lrc"),
            ("ver", "1"),
        ];

        let result = self
            .client
            .get(URL_LYRIC_DOWNLOAD_KUGOU)
            .query(&query_params)
            .send()
            .await
            .map_err(anyhow::Error::from)?
            .text()
            .await
            .map_err(anyhow::Error::from)?;

        to_lyric(&result).map_err(|err| {
            SongTagServiceError::Other(anyhow!(err).context("Extract Lyric text from result"))
        })
    }

    async fn get_picture(
        &self,
        song: &SongTag,
    ) -> std::result::Result<Picture, super::service::SongTagServiceError<Self::Error>> {
        if song.service_provider() != ServiceProvider::Kugou {
            return Err(SongTagServiceError::IncorrectService(
                song.service_provider().to_string(),
                Self::display_name(),
            ));
        }

        let Some(album_id) = song.album_id.as_ref() else {
            return Err(SongTagServiceError::Other(anyhow!(
                "Provided songtag does not have a album_id!"
            )));
        };

        let Some(pic_id) = song.pic_id.as_ref() else {
            return Err(SongTagServiceError::Other(anyhow!(
                "Provided songtag does not have a pic_id!"
            )));
        };

        let cookie_mid = random_ascii(32);

        let query_params = vec![("hash", &pic_id), ("album_id", &album_id)];

        let result = self
            .client
            .get(URL_SONG_DOWNLOAD_KUGOU)
            .header("Cookie", format!("kg_mid={cookie_mid}"))
            .query(&query_params)
            .send()
            .await
            .map_err(anyhow::Error::from)?
            .text()
            .await
            .map_err(anyhow::Error::from)?;

        let pic_url = to_pic_url(&result).map_err(|err| {
            SongTagServiceError::Other(anyhow!(err).context("Extract picture url from result"))
        })?;

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
        song: &SongTag,
    ) -> std::result::Result<String, SongTagServiceError<Self::Error>> {
        if song.service_provider() != ServiceProvider::Kugou {
            return Err(SongTagServiceError::IncorrectService(
                song.service_provider().to_string(),
                Self::display_name(),
            ));
        }

        if let Some(UrlTypes::Protected) = song.url {
            return Err(SongTagServiceError::Other(anyhow!("Song is protected!")));
        }

        let Some(album_id) = song.album_id.as_ref() else {
            return Err(SongTagServiceError::Other(anyhow!(
                "Provided songtag does not have a album_id!"
            )));
        };

        let cookie_mid = random_ascii(32);

        let query_params = vec![("hash", &song.song_id), ("album_id", album_id)];

        let result = self
            .client
            .get(URL_SONG_DOWNLOAD_KUGOU)
            .header("Cookie", format!("kg_mid={cookie_mid}"))
            .query(&query_params)
            .send()
            .await
            .map_err(anyhow::Error::from)?
            .text()
            .await
            .map_err(anyhow::Error::from)?;

        to_song_url(&result).map_err(|err| {
            SongTagServiceError::Other(
                anyhow!(err).context("Extract recording download url from result"),
            )
        })
    }
}
