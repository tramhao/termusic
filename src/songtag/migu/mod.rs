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

use lazy_static::lazy_static;
use model::*;
use regex::Regex;
use reqwest::blocking::Client;
// use std::io::Write;
use anyhow::{anyhow, Result};
use std::time::Duration;

lazy_static! {
    static ref _CSRF: Regex = Regex::new(r"_csrf=(?P<csrf>[^(;|$)]+)").unwrap();
}

static URL_SEARCH: &str = "https://m.music.migu.cn/migu/remoting/scr_search_tag?";
static URL_LYRIC: &str = "https://music.migu.cn/v3/api/music/audioPlayer/getLyric?";
static URL_PIC: &str = "https://music.migu.cn/v3/api/music/audioPlayer/getSongPic?";

pub struct MiguApi {
    client: Client,
    #[allow(dead_code)]
    csrf: String,
}

impl MiguApi {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            // .cookies()
            .build()
            .expect("Initialize Web Client Failed!");

        Self {
            client,
            csrf: String::new(),
        }
    }

    pub fn search(
        &mut self,
        keywords: &str,
        types: u32,
        offset: u16,
        limit: u16,
    ) -> Result<String> {
        let result = self
            .client
            .get(URL_SEARCH)
            .header(
                "Referer",
                // format!(
                // "https://m.music.migu.cn/migu/l/?s=149&p=163&c=5111&j=l&keyword={}",
                // keywords.to_string()
                // ),
                "https://m.music.migu.cn",
            )
            .query(&[
                ("keyword", keywords.to_string()),
                ("pgc", offset.to_string()),
                ("rows", limit.to_string()),
                ("type", 2.to_string()),
            ])
            .send()
            .map_err(|_| anyhow!("None Error"))?
            .text()
            .map_err(|_| anyhow!("None Error"))?;

        // let mut file = std::fs::File::create("data.txt").expect("create failed");
        // file.write_all(result.as_bytes()).expect("write failed");

        match types {
            1 => to_song_info(result).and_then(|s| Ok(serde_json::to_string(&s)?)),
            _ => Err(anyhow!("None Error")),
        }
    }

    // search and download lyrics
    // music_id: 歌曲id
    pub fn song_lyric(&mut self, music_id: &str) -> Result<String> {
        let result = self
            .client
            .get(URL_LYRIC)
            .header("Referer", "https://m.music.migu.cn")
            .query(&[("copyrightId", music_id)])
            .send()
            .map_err(|_| anyhow!("None Error"))?
            .text()
            .map_err(|_| anyhow!("None Error"))?;

        to_lyric(result)
    }

    // download picture
    pub fn pic(&mut self, song_id: &str) -> Result<Vec<u8>> {
        let result = self
            .client
            .get(URL_PIC)
            .header("Referer", "https://m.music.migu.cn")
            .query(&[("songId", song_id)])
            .send()
            .map_err(|_| anyhow!("None Error"))?
            .text()
            .map_err(|_| anyhow!("None Error"))?;

        // let mut file = std::fs::File::create("data.txt").expect("create failed");
        // file.write_all(result.as_bytes()).expect("write failed");

        let mut url = String::from("https:");
        url.push_str(to_pic_url(result)?.as_str());

        let result = reqwest::blocking::get(url)
            .map_err(|_| anyhow!("None Error"))?
            .bytes()
            .map_err(|_| anyhow!("None Error"))?;
        let image = image::load_from_memory(&result).map_err(|_| anyhow!("None Error"))?;
        let mut encoded_image_bytes = Vec::new();
        // Unwrap: Writing to a Vec should always succeed;
        image
            .write_to(&mut encoded_image_bytes, image::ImageOutputFormat::Jpeg(90))
            // .unwrap();
            .map_err(|_| anyhow!("None Error"))?;

        Ok(encoded_image_bytes)
    }
}
