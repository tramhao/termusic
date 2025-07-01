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

use anyhow::anyhow;
use bytes::Buf;
use lofty::picture::Picture;
use model::{to_lyric, to_pic_url, to_song_info};
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

use super::{
    ServiceProvider, SongTag, UrlTypes,
    service::{SongTagService, SongTagServiceError, SongTagServiceErrorWhere},
};

const URL_SEARCH_MIGU: &str = "https://m.music.migu.cn/migu/remoting/scr_search_tag";
const URL_LYRIC_MIGU: &str = "https://music.migu.cn/v3/api/music/audioPlayer/getLyric";
const URL_PIC_MIGU: &str = "https://music.migu.cn/v3/api/music/audioPlayer/getSongPic";
const REFERER: &str = "https://m.music.migu.cn";

pub struct Api {
    client: Client,
}

impl Api {
    pub fn new() -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("client build error");

        Self { client }
    }
}

impl SongTagService for Api {
    type Error = anyhow::Error;

    fn display_name() -> &'static str
    where
        Self: Sized,
    {
        "migu"
    }

    async fn search_recording(
        &self,
        keywords: &str,
        offset: u32,
        limit: u32,
    ) -> std::result::Result<Vec<SongTag>, super::service::SongTagServiceError<Self::Error>> {
        let offset = offset.to_string();
        let limit = limit.to_string();

        let query_params = vec![
            ("keyword", keywords),
            ("pgc", &offset),
            ("rows", &limit),
            // i assume "2" stands for type "Song"
            ("type", "2"),
        ];

        let result = self
            .client
            .post(URL_SEARCH_MIGU)
            .header("Referer", REFERER)
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
        if song.service_provider() != ServiceProvider::Migu {
            return Err(SongTagServiceError::IncorrectService(
                song.service_provider().to_string(),
                Self::display_name(),
            ));
        }

        let Some(lyric_id) = song.lyric_id.as_ref() else {
            return Err(SongTagServiceError::Other(anyhow!(
                "Provided songtag does not have a lyric_id!"
            )));
        };

        let query_params = &[("copyrightId", &lyric_id)];

        let result = self
            .client
            .get(URL_LYRIC_MIGU)
            .header("Referer", REFERER)
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
        if song.service_provider() != ServiceProvider::Migu {
            return Err(SongTagServiceError::IncorrectService(
                song.service_provider().to_string(),
                Self::display_name(),
            ));
        }

        let query_params = &[("songId", &song.song_id)];

        let result = self
            .client
            .get(URL_PIC_MIGU)
            .header("Referer", REFERER)
            .query(query_params)
            .send()
            .await
            .map_err(anyhow::Error::from)?
            .text()
            .await
            .map_err(anyhow::Error::from)?;

        let pic_url = to_pic_url(&result).map_err(|err| {
            SongTagServiceError::Other(anyhow!(err).context("Extract picture url from result"))
        })?;
        let url = format!("https:{pic_url}");

        let result = self
            .client
            .get(url)
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
    ) -> std::result::Result<String, super::service::SongTagServiceError<Self::Error>> {
        // this function is to get the url for downloading, which in migu does not require extra fetching
        // so if its available, use it, otherwise report "NotSupported"
        if let Some(UrlTypes::FreeDownloadable(url)) = song.url.as_ref() {
            return Ok(url.clone());
        }

        Err(SongTagServiceError::NotSupported(
            SongTagServiceErrorWhere::DownloadRecording,
            Self::display_name(),
        ))
    }
}
