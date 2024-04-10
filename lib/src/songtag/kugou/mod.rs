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

use super::{encrypt::Crypto, SongTag};
use anyhow::{anyhow, Result};
use bytes::Buf;
use lofty::Picture;
use model::{to_lyric, to_lyric_id_accesskey, to_pic_url, to_song_info, to_song_url};
use reqwest::blocking::{Client, ClientBuilder};
use std::time::Duration;

const URL_SEARCH_KUGOU: &str = "http://mobilecdn.kugou.com/api/v3/search/song";
const URL_LYRIC_SEARCH_KUGOU: &str = "http://krcs.kugou.com/search";
const URL_LYRIC_DOWNLOAD_KUGOU: &str = "http://lyrics.kugou.com/download";
const URL_SONG_DOWNLOAD_KUGOU: &str = "http://www.kugou.com/yy/index.php?r=play/getdata";

#[derive(Debug, Clone, Copy)]
pub enum SearchRequestType {
    Song = 1,
}

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

    pub fn search(
        &self,
        keywords: &str,
        types: SearchRequestType,
        offset: u16,
        limit: u16,
    ) -> Result<Vec<SongTag>> {
        let q_1 = 1.to_string();
        let q_page = offset.to_string();
        let q_pagesize = limit.to_string();

        let query_vec = vec![
            ("format", "json"),
            ("page", "1"),
            ("showtype", &q_1),
            ("keyword", keywords),
            ("page", &q_page),
            ("pagesize", &q_pagesize),
            ("showtype", &q_1),
        ];
        let result = self
            .client
            .post(URL_SEARCH_KUGOU)
            .header("Referer", "https://m.music.migu.cn")
            .query(&query_vec)
            .send()?
            .text()?;

        // let mut file = std::fs::File::create("data.txt").expect("create failed");
        // file.write_all(result.as_bytes()).expect("write failed");

        match types {
            SearchRequestType::Song => {
                let song_info = to_song_info(&result).ok_or_else(|| anyhow!("Search Error"))?;
                Ok(song_info)
            }
        }
    }

    // search and download lyrics
    // music_id: 歌曲id
    pub fn song_lyric(&self, music_id: &str) -> Result<String> {
        let query_vec = vec![
            ("keyword", "%20-%20"),
            ("ver", "1"),
            ("hash", music_id),
            ("client", "mobi"),
            ("man", "yes"),
        ];

        let result = self
            .client
            .get(URL_LYRIC_SEARCH_KUGOU)
            .query(&query_vec)
            .send()?
            .text()?;

        let (accesskey, id) =
            to_lyric_id_accesskey(&result).ok_or_else(|| anyhow!("Search Error"))?;

        let query_vec = vec![
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
            .query(&query_vec)
            .send()?
            .text()?;

        to_lyric(&result).ok_or_else(|| anyhow!("Search Error"))
    }

    // 歌曲 URL
    // ids: 歌曲列表
    pub fn song_url(&self, id: &str, album_id: &str) -> Result<String> {
        let kg_mid = Crypto::alpha_lowercase_random_bytes(32);

        let query_vec = vec![("hash", id), ("album_id", album_id)];
        let result = self
            .client
            .get(URL_SONG_DOWNLOAD_KUGOU)
            .header("Cookie", format!("kg_mid={kg_mid}").as_str())
            .query(&query_vec)
            .send()?
            .text()?;

        // let mut file = std::fs::File::create("data.txt").expect("create failed");
        // file.write_all(result.as_bytes()).expect("write failed");

        to_song_url(&result).ok_or_else(|| anyhow!("Search Error"))
    }

    // download picture
    pub fn pic(&self, id: &str, album_id: &str) -> Result<Picture> {
        let kg_mid = Crypto::alpha_lowercase_random_bytes(32);
        let query_vec = vec![("hash", id), ("album_id", album_id)];
        let result = self
            .client
            .get(URL_SONG_DOWNLOAD_KUGOU)
            .header("Cookie", format!("kg_mid={kg_mid}").as_str())
            .query(&query_vec)
            .send()?
            .text()?;

        let url = to_pic_url(&result).ok_or_else(|| anyhow!("Search Error"))?;

        let result = self.client.get(url).send()?;

        // let mut bytes: Vec<u8> = Vec::new();
        // result.into_reader().read_to_end(&mut bytes)?;

        // Ok(bytes)
        // let mut bytes = Vec::new();
        // result.read_to_end(&mut bytes)?;

        let mut reader = result.bytes()?.reader();
        let picture = Picture::from_reader(&mut reader)?;
        Ok(picture)
    }
}
