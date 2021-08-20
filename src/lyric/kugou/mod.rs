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

pub(crate) type NCMResult<T> = Result<T, Errors>;
use super::netease::encrypt::Crypto;
use lazy_static::lazy_static;
use model::*;
use regex::Regex;
use reqwest::blocking::Client;
use std::time::Duration;

lazy_static! {
    static ref _CSRF: Regex = Regex::new(r"_csrf=(?P<csrf>[^(;|$)]+)").unwrap();
}

static BASE_URL_SEARCH: &str =
    "http://mobilecdn.kugou.com/api/v3/search/song?format=json&showtype=1";
static BASE_URL_LYRIC_SEARCH: &str = "http://krcs.kugou.com/search";
static BASE_URL_LYRIC_DOWNLOAD: &str = "http://lyrics.kugou.com/download";
static URL_SONG_DOWNLOAD: &str = "http://www.kugou.com/yy/index.php?r=play/getdata";

pub struct KugouApi {
    client: Client,
    #[allow(dead_code)]
    csrf: String,
}

impl KugouApi {
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

    #[allow(unused)]
    pub fn search(
        &mut self,
        keywords: &str,
        types: u32,
        offset: u16,
        limit: u16,
    ) -> NCMResult<String> {
        let result = self
            .client
            .get(BASE_URL_SEARCH)
            .query(&[
                ("keyword", keywords),
                ("page", &offset.to_string()),
                ("pagesize", &limit.to_string()),
                ("showtype", &1.to_string()),
            ])
            .send()
            .map_err(|_| Errors::None)?
            .text()
            .map_err(|_| Errors::None)?;

        // let mut file = std::fs::File::create("data.txt").expect("create failed");
        // file.write_all(result.as_bytes()).expect("write failed");

        match types {
            1 => to_song_info(result, Parse::SEARCH).and_then(|s| Ok(serde_json::to_string(&s)?)),
            _ => Err(Errors::None),
        }
    }

    // search and download lyrics
    // music_id: 歌曲id
    pub fn song_lyric(&mut self, music_id: &str) -> NCMResult<String> {
        let result = self
            .client
            .get(BASE_URL_LYRIC_SEARCH)
            .query(&[
                ("keyword", "%20-%20".to_string()),
                ("ver", 1.to_string()),
                ("hash", music_id.to_string()),
                ("client", "mobi".to_string()),
                ("man", "yes".to_string()),
            ])
            .send()
            .map_err(|_| Errors::None)?
            .text()
            .map_err(|_| Errors::None)?;

        let (accesskey, id) = to_lyric_id_accesskey(result)?;

        let result = self
            .client
            .get(BASE_URL_LYRIC_DOWNLOAD)
            .query(&[
                ("charset", "utf8".to_string()),
                ("accesskey", accesskey),
                ("id", id),
                ("client", "mobi".to_string()),
                ("fmt", "lrc".to_string()),
                ("ver", 1.to_string()),
            ])
            .send()
            .map_err(|_| Errors::None)?
            .text()
            .map_err(|_| Errors::None)?;

        to_lyric(result)
    }

    // 歌曲 URL
    // ids: 歌曲列表
    #[allow(unused)]
    pub fn song_url(&mut self, id: String, album_id: String) -> NCMResult<String> {
        let kg_mid = Crypto::alpha_lowercase_random_bytes(32);

        let result = self
            .client
            .get(URL_SONG_DOWNLOAD)
            .header("Cookie", format!("kg_mid={}", kg_mid))
            // .header("Cookie", "kg_mid=6e9349f2dc9a66bfcfbb08ec2bf882b1")
            .query(&[("hash", id), ("album_id", album_id)])
            .send()
            .map_err(|_| Errors::None)?
            .text()
            .map_err(|_| Errors::None)?;

        // let mut file = std::fs::File::create("data.txt").expect("create failed");
        // file.write_all(result.as_bytes()).expect("write failed");

        to_song_url(result)
    }

    // download picture
    pub fn pic(&mut self, id: String, album_id: String) -> NCMResult<Vec<u8>> {
        let kg_mid = Crypto::alpha_lowercase_random_bytes(32);
        let result = self
            .client
            .get(URL_SONG_DOWNLOAD)
            .header("Cookie", format!("kg_mid={}", kg_mid))
            // .header("Cookie", "kg_mid=6e9349f2dc9a66bfcfbb08ec2bf882b1")
            .query(&[("hash", id), ("album_id", album_id)])
            .send()
            .map_err(|_| Errors::None)?
            .text()
            .map_err(|_| Errors::None)?;

        let url = to_pic_url(result)?;

        let result = reqwest::blocking::get(url)
            .map_err(|_| Errors::None)?
            .bytes()
            .map_err(|_| Errors::None)?;
        let image = image::load_from_memory(&result).map_err(|_| Errors::None)?;
        let mut encoded_image_bytes = Vec::new();
        // Unwrap: Writing to a Vec should always succeed;
        image
            .write_to(&mut encoded_image_bytes, image::ImageOutputFormat::Jpeg(90))
            .map_err(|_| Errors::None)?;

        Ok(encoded_image_bytes)
    }
}
