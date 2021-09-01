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
use super::NCMResult;
// , NCM_CACHE};
// use async_std::io;
use super::super::{SongTag, SongtagProvider};
use custom_error::custom_error;
use serde_json::{json, Value};
// use std::path::PathBuf;

pub fn to_lyric(json: String) -> NCMResult<String> {
    let value = serde_json::from_str::<Value>(&json)?;
    if value.get("status").ok_or(Errors::None)?.eq(&200) {
        let lyric = value
            .get("content")
            .ok_or(Errors::None)?
            .as_str()
            .ok_or(Errors::None)?
            .to_owned();
        return String::from_utf8(base64::decode(lyric).map_err(|_| Errors::None)?)
            .map_err(|_| Errors::None);
    }
    Err(Errors::None)
}

pub fn to_lyric_id_accesskey(json: String) -> NCMResult<(String, String)> {
    let value = serde_json::from_str::<Value>(&json)?;
    if value.get("errcode").ok_or(Errors::None)?.eq(&200) {
        let v = value
            .get("candidates")
            .ok_or(Errors::None)?
            .get(0)
            .ok_or(Errors::None)?;
        let accesskey = v
            .get("accesskey")
            .unwrap_or(&json!("未知"))
            .as_str()
            .unwrap_or("未知")
            .to_owned();
        let id = v
            .get("id")
            .ok_or(Errors::None)?
            .as_str()
            .ok_or(Errors::None)?
            .to_owned();

        return Ok((accesskey, id));
    }
    Err(Errors::None)
}

pub fn to_song_url(json: String) -> NCMResult<String> {
    let value = serde_json::from_str::<Value>(&json)?;
    if value.get("status").ok_or(Errors::None)?.eq(&1) {
        let url = value
            .get("data")
            .ok_or(Errors::None)?
            .get("play_url")
            .unwrap_or(&json!(""))
            .as_str()
            .unwrap_or("")
            .to_owned();
        return Ok(url);
    }
    Err(Errors::None)
}

pub fn to_pic_url(json: String) -> NCMResult<String> {
    let value = serde_json::from_str::<Value>(&json)?;
    if value.get("status").ok_or(Errors::None)?.eq(&1) {
        let url = value
            .get("data")
            .ok_or(Errors::None)?
            .get("img")
            .unwrap_or(&json!(""))
            .as_str()
            .unwrap_or("")
            .to_owned();
        return Ok(url);
    }
    Err(Errors::None)
}

// parse: 解析方式
pub fn to_song_info(json: String) -> NCMResult<Vec<SongTag>> {
    let value = serde_json::from_str::<Value>(&json)?;
    if value.get("status").ok_or(Errors::None)?.eq(&1) {
        let mut vec: Vec<SongTag> = Vec::new();
        let array = value
            .get("data")
            .ok_or(Errors::None)?
            .as_object()
            .ok_or(Errors::None)?
            .get("info")
            .ok_or(Errors::None)?
            .as_array()
            .ok_or(Errors::None)?;
        for v in array.iter() {
            let price = v
                .get("price")
                .unwrap_or(&json!("未知"))
                .as_u64()
                .unwrap_or(0);
            let url: String;
            if price == 0 {
                url = "Downloadable".to_string();
            } else {
                url = "Copyright Protected".to_string();
            }

            vec.push(SongTag {
                song_id: Some(
                    v.get("hash")
                        .ok_or(Errors::None)?
                        .as_str()
                        .ok_or(Errors::None)?
                        .to_owned(),
                ),
                title: Some(
                    v.get("songname")
                        .ok_or(Errors::None)?
                        .as_str()
                        .ok_or(Errors::None)?
                        .to_owned(),
                ),
                artist: Some(
                    v.get("singername")
                        .unwrap_or(&json!("未知"))
                        .as_str()
                        .unwrap_or("未知")
                        .to_owned(),
                ),
                album: Some(
                    v.get("album_name")
                        .unwrap_or(&json!("未知"))
                        .as_str()
                        .unwrap_or("")
                        .to_owned(),
                ),
                pic_id: Some(
                    v.get("hash")
                        .ok_or(Errors::None)?
                        .as_str()
                        .ok_or(Errors::None)?
                        .to_owned(),
                ),
                lang_ext: Some("chi".to_string()),
                service_provider: Some(SongtagProvider::Kugou),
                lyric_id: Some(
                    v.get("hash")
                        .ok_or(Errors::None)?
                        .as_str()
                        .ok_or(Errors::None)?
                        .to_owned(),
                ),
                url: Some(url),
                album_id: Some(
                    v.get("album_id")
                        .ok_or(Errors::None)?
                        .as_str()
                        .ok_or(Errors::None)?
                        .to_owned(),
                ),
            });
        }
        return Ok(vec);
    }
    Err(Errors::None)
}

custom_error! { pub Errors
    // OpenSSL{ source: openssl::error::ErrorStack } = "openSSL Error",
    // Regex{ source: regex::Error } = "regex Error",
    SerdeJson{ source: serde_json::error::Error } = "serde json Error",
    // Parse{ source: std::num::ParseIntError } = "parse Error",
    // AsyncIoError{ source: io::Error } = "async io Error",
    // IsahcError{ source: isahc::Error } = "isahc Error",
    None = "None Error",
    // FromUtf8Error{source: std::string::FromUtf8Error} = "UTF8 Error",
}
