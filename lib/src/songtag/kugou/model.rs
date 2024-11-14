use crate::songtag::UrlTypes;

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
use super::super::{ServiceProvider, SongTag};
use base64::{engine::general_purpose, Engine as _};
use serde_json::{from_str, json, Value};

pub fn to_lyric(json: &str) -> Option<String> {
    if let Ok(value) = from_str::<Value>(json) {
        if value.get("status")?.eq(&200) {
            let lyric = value.get("content")?.as_str()?.to_owned();
            if let Ok(lyric_decoded) = general_purpose::STANDARD.decode(lyric) {
                if let Ok(s) = String::from_utf8(lyric_decoded) {
                    return Some(s);
                }
            }
        }
    }
    None
}

pub fn to_lyric_id_accesskey(json: &str) -> Option<(String, String)> {
    if let Ok(value) = from_str::<Value>(json) {
        if value.get("errcode")?.eq(&200) {
            let v = value.get("candidates")?.get(0)?;
            let accesskey = v
                .get("accesskey")
                .unwrap_or(&json!("Unknown Access Key"))
                .as_str()
                .unwrap_or("Unknown Access Key")
                .to_owned();
            let id = v.get("id")?.as_str()?.to_owned();

            return Some((accesskey, id));
        }
    }
    None
}

pub fn to_song_url(json: &str) -> Option<String> {
    if let Ok(value) = from_str::<Value>(json) {
        if value.get("status")?.eq(&1) {
            let url = value
                .get("data")?
                .get("play_url")
                .unwrap_or(&json!(""))
                .as_str()
                .unwrap_or("")
                .to_owned();
            return Some(url);
        }
    }
    None
}

pub fn to_pic_url(json: &str) -> Option<String> {
    if let Ok(value) = from_str::<Value>(json) {
        if value.get("status")?.eq(&1) {
            let url = value
                .get("data")?
                .get("img")
                .unwrap_or(&json!(""))
                .as_str()
                .unwrap_or("")
                .to_owned();
            return Some(url);
        }
    }
    None
}

// parse: 解析方式
pub fn to_song_info(json: &str) -> Option<Vec<SongTag>> {
    if let Ok(value) = from_str::<Value>(json) {
        if value.get("status")?.eq(&1) {
            let mut vec: Vec<SongTag> = Vec::new();
            let array = value.get("data")?.as_object()?.get("info")?.as_array()?;
            for v in array {
                if let Some(item) = parse_song_info(v) {
                    vec.push(item);
                }
            }
            return Some(vec);
        }
    }
    None
}

fn parse_song_info(v: &Value) -> Option<SongTag> {
    let price = v
        .get("price")
        .unwrap_or(&json!("Unknown Price"))
        .as_u64()
        .unwrap_or(0);
    let url = if price == 0 {
        UrlTypes::AvailableRequiresFetching
    } else {
        UrlTypes::Protected
    };

    Some(SongTag {
        song_id: v.get("hash")?.as_str()?.to_owned(),
        title: Some(v.get("songname")?.as_str()?.to_owned()),
        artist: Some(
            v.get("singername")
                .unwrap_or(&json!("Unknown Artist"))
                .as_str()
                .unwrap_or("Unknown Artist")
                .to_owned(),
        ),
        album: Some(
            v.get("album_name")
                .unwrap_or(&json!("Unknown Album"))
                .as_str()
                .unwrap_or("")
                .to_owned(),
        ),
        pic_id: Some(v.get("hash")?.as_str()?.to_owned()),
        lang_ext: Some("kugou".to_string()),
        service_provider: ServiceProvider::Kugou,
        lyric_id: Some(v.get("hash")?.as_str()?.to_owned()),
        url: Some(url),
        album_id: Some(v.get("album_id")?.as_str()?.to_owned()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_parse_songinfo_empty() {
        let sample_data = r#"{
            "status": 1,
            "errcode": 0,
            "data": {
              "timestamp": 1111111111,
              "tab": "",
              "forcecorrection": 0,
              "correctiontype": 0,
              "total": 0,
              "istag": 0,
              "allowerr": 0,
              "info": [],
              "aggregation": [],
              "correctiontip": "",
              "istagresult": 0
            },
            "error": ""
          }"#;

        let res = to_song_info(sample_data).unwrap();

        assert_eq!(res.len(), 0);
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn should_parse_songinfo() {
        let sample_data = r#"{
            "status": 1,
            "errcode": 0,
            "data": {
                "timestamp": 1111111111,
                "tab": "",
                "forcecorrection": 0,
                "correctiontype": 0,
                "total": 1,
                "istag": 0,
                "allowerr": 0,
                "info": [
                    {
                        "hash": "11111111111111111111111111111111",
                        "sqfilesize": 11111,
                        "sourceid": 0,
                        "pay_type_sq": 3,
                        "bitrate": 128,
                        "ownercount": 743,
                        "pkg_price_sq": 1,
                        "songname": "test song",
                        "album_name": "test album",
                        "songname_original": "original songname?",
                        "Accompany": 0,
                        "sqhash": "22222222222222222222222222222222",
                        "fail_process": 4,
                        "pay_type": 3,
                        "rp_type": "audio",
                        "album_id": "88888888",
                        "othername_original": "",
                        "mvhash": "",
                        "extname": "mp3",
                        "group": [],
                        "price_320": 200,
                        "320hash": "33333333333333333333333333333333",
                        "topic": "",
                        "othername": "",
                        "isnew": 0,
                        "fold_type": 0,
                        "old_cpy": 0,
                        "srctype": 1,
                        "singername": "some singer",
                        "album_audio_id": 999999999,
                        "duration": 60,
                        "320filesize": 11111,
                        "pkg_price_320": 1,
                        "audio_id": 555555555,
                        "feetype": 0,
                        "price": 200,
                        "filename": "some filename",
                        "source": "",
                        "price_sq": 200,
                        "fail_process_320": 4,
                        "trans_param": {
                            "cpy_level": 1,
                            "cpy_grade": 5,
                            "qualitymap": {
                                "attr0": 116
                            },
                            "union_cover": "http:\/\/imge.kugou.com\/stdmusic\/{size}\/20220101\/00000000000000000000.jpg",
                            "classmap": {
                                "attr0": 777777777
                            },
                            "language": "日语",
                            "pay_block_tpl": 1,
                            "cpy_attr0": 8192,
                            "ipmap": {
                                "attr0": 131313131313
                            },
                            "cid": 110101010,
                            "musicpack_advance": 0,
                            "display": 0,
                            "display_rate": 0
                        },
                        "pkg_price": 1,
                        "pay_type_320": 3,
                        "topic_url": "",
                        "m4afilesize": 0,
                        "rp_publish": 1,
                        "privilege": 8,
                        "filesize": 111111,
                        "isoriginal": 1,
                        "320privilege": 10,
                        "sqprivilege": 10,
                        "fail_process_sq": 4
                    }
                ],
                "aggregation": [],
                "correctiontip": "",
                "istagresult": 0
            },
            "error": ""
        }
        "#;

        let res = to_song_info(sample_data).unwrap();

        assert_eq!(res.len(), 1);

        assert_eq!(
            res[0],
            SongTag {
                artist: Some("some singer".to_owned()),
                title: Some("test song".to_owned()),
                album: Some("test album".to_owned()),
                lang_ext: Some("kugou".to_string()),
                service_provider: ServiceProvider::Kugou,
                song_id: "11111111111111111111111111111111".to_owned(),
                lyric_id: Some("11111111111111111111111111111111".to_owned()),
                url: Some(UrlTypes::Protected),
                pic_id: Some("11111111111111111111111111111111".to_owned()),
                album_id: Some("88888888".to_owned())
            }
        );
    }
}
