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

use anyhow::{anyhow, Result};
use bytes::Buf;
use lofty::Picture;
use model::{to_lyric, to_pic_url, to_song_info};
// use std::io::Write;
use reqwest::blocking::{Client, ClientBuilder};
use std::time::Duration;

use super::SongTag;

static URL_SEARCH_MIGU: &str = "https://m.music.migu.cn/migu/remoting/scr_search_tag";
static URL_LYRIC_MIGU: &str = "https://music.migu.cn/v3/api/music/audioPlayer/getLyric";
static URL_PIC_MIGU: &str = "https://music.migu.cn/v3/api/music/audioPlayer/getSongPic";

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
            .expect("client build error");

        Self { client }
    }

    pub fn search(
        &self,
        keywords: &str,
        types: SearchRequestType,
        offset: u16,
        limit: u16,
    ) -> Result<Vec<SongTag>> {
        let q_pgc = offset.to_string();
        let q_rows = limit.to_string();
        let q_type = 2.to_string();
        let query_vec = vec![
            ("keyword", keywords),
            ("pgc", &q_pgc),
            ("rows", &q_rows),
            ("type", &q_type),
        ];
        let result = self
            .client
            .post(URL_SEARCH_MIGU)
            .header("Referer", "https://m.music.migu.cn")
            .query(&query_vec)
            .send()?
            .text()?;

        // let mut file = std::fs::File::create("data.txt").expect("create failed");
        // file.write_all(result.as_bytes()).expect("write failed");

        match types {
            SearchRequestType::Song => {
                let songtag_vec = to_song_info(&result).ok_or_else(|| anyhow!("Search Error"))?;
                Ok(songtag_vec)
            }
        }
    }

    // search and download lyrics
    // music_id: 歌曲id
    pub fn song_lyric(&self, music_id: &str) -> Result<String> {
        let result = self
            .client
            .get(URL_LYRIC_MIGU)
            .header("Referer", "https://m.music.migu.cn")
            .query(&[("copyrightId", music_id)])
            .send()?
            .text()?;

        to_lyric(&result).ok_or_else(|| anyhow!("None Error"))
    }

    // download picture
    pub fn pic(&self, song_id: &str) -> Result<Picture> {
        let result = self
            .client
            .get(URL_PIC_MIGU)
            .header("Referer", "https://m.music.migu.cn")
            .query(&[("songId", song_id)])
            .send()?
            .text()?;

        let pic_url = to_pic_url(&result).ok_or_else(|| anyhow!("Pic url error"))?;
        let url = format!("https:{pic_url}");

        let result = self.client.get(url).send()?;

        let mut reader = result.bytes()?.reader();
        let picture = Picture::from_reader(&mut reader)?;
        Ok(picture)
        // let mut bytes: Vec<u8> = Vec::new();
        // result.into_reader().read_to_end(&mut bytes)?;

        // Ok(bytes)
    }
}
