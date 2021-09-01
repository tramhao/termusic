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
pub mod model;

use super::netease::encrypt::Crypto;
use anyhow::{bail, Result};
use model::*;
// use std::io::Write;
use std::io::Read;
use std::time::Duration;
use ureq::{Agent, AgentBuilder};

static BASE_URL_SEARCH: &str =
    "http://mobilecdn.kugou.com/api/v3/search/song?format=json&showtype=1";
static BASE_URL_LYRIC_SEARCH: &str = "http://krcs.kugou.com/search?";
static BASE_URL_LYRIC_DOWNLOAD: &str = "http://lyrics.kugou.com/download?";
static URL_SONG_DOWNLOAD: &str = "http://www.kugou.com/yy/index.php?r=play/getdata";

pub struct KugouApi {
    client: Agent,
}

impl KugouApi {
    pub fn new() -> Self {
        let client = AgentBuilder::new().timeout(Duration::from_secs(10)).build();

        Self { client }
    }

    pub fn search(
        &mut self,
        keywords: &str,
        types: u32,
        offset: u16,
        limit: u16,
    ) -> Result<String> {
        let mut url = BASE_URL_SEARCH.to_string();
        url.push_str("&keyword=");
        url.push_str(keywords);
        url.push_str("&pgge=");
        url.push_str(&offset.to_string());
        url.push_str("&pagesize=");
        url.push_str(&limit.to_string());
        url.push_str("&showtype=");
        url.push_str(&1.to_string());

        let result = self
            .client
            .post(&url)
            .set("Referer", "https://m.music.migu.cn")
            .call()?
            .into_string()?;

        // let mut file = std::fs::File::create("data.txt").expect("create failed");
        // file.write_all(result.as_bytes()).expect("write failed");

        match types {
            1 => to_song_info(result).and_then(|s| Ok(serde_json::to_string(&s)?)),
            _ => bail!("None Error"),
        }
    }

    // search and download lyrics
    // music_id: 歌曲id
    pub fn song_lyric(&mut self, music_id: &str) -> Result<String> {
        let mut url = BASE_URL_LYRIC_SEARCH.to_string();
        url.push_str("keyword=%20-%20&ver=1&hash=");
        url.push_str(music_id);
        url.push_str("&client=mobi&man=yes");

        let result = self
            .client
            .get(&url)
            // .set("Referer", "https://m.music.migu.cn")
            .call()?
            .into_string()?;

        let (accesskey, id) = to_lyric_id_accesskey(result)?;

        let mut url = BASE_URL_LYRIC_DOWNLOAD.to_string();
        url.push_str("charset=utf8&accesskey=");
        url.push_str(&accesskey);
        url.push_str("&id=");
        url.push_str(&id);
        url.push_str("&client=mobi&fmt=lrc&ver=1");

        let result = self.client.get(&url).call()?.into_string()?;

        to_lyric(result)
    }

    // 歌曲 URL
    // ids: 歌曲列表
    pub fn song_url(&mut self, id: String, album_id: String) -> Result<String> {
        let kg_mid = Crypto::alpha_lowercase_random_bytes(32);
        let mut url = URL_SONG_DOWNLOAD.to_string();
        url.push_str("hash=");
        url.push_str(&id);
        url.push_str("&album_id=");
        url.push_str(&album_id);

        let result = self
            .client
            .get(&url)
            .set("Cookie", format!("kg_mid={}", kg_mid).as_str())
            .call()?
            .into_string()?;

        // let mut file = std::fs::File::create("data.txt").expect("create failed");
        // file.write_all(result.as_bytes()).expect("write failed");

        to_song_url(result)
    }

    // download picture
    pub fn pic(&mut self, id: &str, album_id: &str) -> Result<Vec<u8>> {
        let kg_mid = Crypto::alpha_lowercase_random_bytes(32);
        let mut url = URL_SONG_DOWNLOAD.to_string();
        url.push_str("&hash=");
        url.push_str(id);
        url.push_str("&album_id=");
        url.push_str(album_id);

        let result = self
            .client
            .get(&url)
            .set("Cookie", format!("kg_mid={}", kg_mid).as_str())
            .call()?
            .into_string()?;

        let url = to_pic_url(result)?;

        let result = self.client.get(&url).call()?;
        // assert!(result.has("Content-Length"));
        // let len = result
        //     .header("Content-Length")
        //     .and_then(|s| s.parse::<usize>().ok())
        //     .unwrap();

        // let mut bytes: Vec<u8> = Vec::with_capacity(len);
        let mut bytes: Vec<u8> = Vec::new();
        result
            .into_reader()
            // .take(10_000_000)
            .read_to_end(&mut bytes)?;

        Ok(bytes)
    }
}
