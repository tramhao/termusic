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

use anyhow::{anyhow, Result};
use model::*;
// use std::io::Write;
use std::io::Read;
use std::time::Duration;
use ureq::{Agent, AgentBuilder};

static URL_SEARCH: &str = "https://m.music.migu.cn/migu/remoting/scr_search_tag?";
static URL_LYRIC: &str = "https://music.migu.cn/v3/api/music/audioPlayer/getLyric?";
static URL_PIC: &str = "https://music.migu.cn/v3/api/music/audioPlayer/getSongPic?";

pub struct MiguApi {
    client: Agent,
    // client: Client,
}

impl MiguApi {
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
        let mut url = URL_SEARCH.to_string();
        url.push_str("keyword=");
        url.push_str(keywords);
        url.push_str("&pgc=");
        url.push_str(&offset.to_string());
        url.push_str("&rows=");
        url.push_str(&limit.to_string());
        url.push_str("&type=");
        url.push_str(&2.to_string());

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
            _ => Err(anyhow!("None Error")),
        }
    }

    // search and download lyrics
    // music_id: 歌曲id
    pub fn song_lyric(&mut self, music_id: &str) -> Result<String> {
        let mut url = URL_LYRIC.to_string();
        url.push_str("copyrightId=");
        url.push_str(music_id);

        // println!("{}", url);

        let result = self
            .client
            .get(&url)
            .set("Referer", "https://m.music.migu.cn")
            .call()?
            .into_string()?;

        to_lyric(result)
    }

    // download picture
    pub fn pic(&mut self, song_id: &str) -> Result<Vec<u8>> {
        let mut url = URL_PIC.to_string();
        url.push_str("songId=");
        url.push_str(song_id);

        let result = self
            .client
            .get(&url)
            .set("Referer", "https://m.music.migu.cn")
            .call()?
            .into_string()?;

        let mut url = String::from("https:");
        url.push_str(to_pic_url(result)?.as_str());

        let result = self.client.get(&url).call()?;

        // let image =
        //     image::load_from_memory(result.as_bytes()).map_err(|_| anyhow!("None Error"))?;
        // let mut encoded_image_bytes = Vec::new();

        // image
        //     .write_to(&mut encoded_image_bytes, image::ImageOutputFormat::Jpeg(90))
        //     .map_err(|_| anyhow!("None Error"))?;
        // Ok(encoded_image_bytes)
        let mut bytes: Vec<u8> = Vec::new();
        result
            .into_reader()
            // .take(10_000_000)
            .read_to_end(&mut bytes)?;

        Ok(bytes)
    }
}
