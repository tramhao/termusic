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
use anyhow::{anyhow, bail, Result};
use base64::{engine::general_purpose, Engine as _};
use serde_json::{from_str, json, Value};

/// Try to get the lyric lrc content from the given result
pub fn to_lyric(json: &str) -> Result<String> {
    let value = from_str::<Value>(json).map_err(anyhow::Error::from)?;

    if value.get("status").is_none() || !value.get("status").is_some_and(|v| v.eq(&200)) {
        let errcode = value
            .get("errcode")
            .and_then(Value::as_str)
            .unwrap_or("<none>");
        bail!(
            "Failed to get lyric text, \"status\" does not exist or is not 200 Errcode: {errcode}"
        );
    }

    let lyric = value
        .get("content")
        .and_then(Value::as_str)
        .ok_or(anyhow!("property \"content\" does not exist in result!"))?
        .to_owned();
    let lyric_decoded = general_purpose::STANDARD.decode(lyric)?;
    let lyric_str = String::from_utf8(lyric_decoded)?;

    Ok(lyric_str)
}

/// Try to get the `accesskey` and lyric `id` from the given json response
pub fn to_lyric_id_accesskey(json: &str) -> Result<(String, String)> {
    let value = from_str::<Value>(json).map_err(anyhow::Error::from)?;

    if value.get("errcode").is_none() || !value.get("errcode").is_some_and(|v| v.eq(&200)) {
        let errcode = value
            .get("errcode")
            .and_then(Value::as_str)
            .unwrap_or("<none>");
        bail!("Failed to get lyric id and accesskey, \"errcode\" does not exist or is not 200. Errcode: {errcode}");
    }

    let v = value
        .get("candidates")
        .and_then(|v| v.get(0))
        .ok_or(anyhow!(
            "property \"candidates.0\" does not exist in result!"
        ))?;
    let accesskey = v
        .get("accesskey")
        .and_then(Value::as_str)
        .ok_or(anyhow!(
            "property \"candidates.0.accesskey\" does not exist in result!"
        ))?
        .to_owned();
    let id = v
        .get("id")
        .and_then(Value::as_str)
        .ok_or(anyhow!(
            "property \"candidates.0.id\" does not exist in result!"
        ))?
        .to_owned();

    Ok((accesskey, id))
}

/// Try to get the play (download) url from the result
pub fn to_song_url(json: &str) -> Result<String> {
    let value = from_str::<Value>(json).map_err(anyhow::Error::from)?;

    if value.get("status").is_none() || !value.get("status").is_some_and(|v| v.eq(&200)) {
        let errcode = value
            .get("errcode")
            .and_then(Value::as_str)
            .unwrap_or("<none>");
        bail!("Failed to get download url, \"status\" does not exist or is not 200. Errcode: {errcode}");
    }

    let url = value
        .get("data")
        .and_then(|v| v.get("play_url"))
        .and_then(Value::as_str)
        .ok_or(anyhow!(
            "property \"data.play_url\" does not exist in result!"
        ))?
        .to_owned();

    Ok(url)
}

/// Try to get the picture url from the json response
pub fn to_pic_url(json: &str) -> Result<String> {
    let value = from_str::<Value>(json).map_err(anyhow::Error::from)?;

    if value.get("status").is_none() || !value.get("status").is_some_and(|v| v.eq(&1)) {
        bail!("Failed to get picture url, \"status\" does not exist or is not 200");
    }

    let url = value
        .get("data")
        .and_then(|v| v.get("img"))
        .and_then(Value::as_str)
        .ok_or(anyhow!("property \"data.img\" does not exist in result!"))?
        .to_owned();

    Ok(url)
}

/// Try to get individual [`SongTag`]s from the json response
pub fn to_song_info(json: &str) -> Result<Vec<SongTag>> {
    let value = from_str::<Value>(json).map_err(anyhow::Error::from)?;

    if value.get("status").is_none() || !value.get("status").is_some_and(|v| v.eq(&1)) {
        bail!("Failed to get picture url, \"status\" does not exist or is not 200");
    }

    let array = value
        .get("data")
        .and_then(Value::as_object)
        .and_then(|v| v.get("info"))
        .and_then(Value::as_array)
        .ok_or(anyhow!("property \"data.info\" does not exist in result!"))?;

    let mut vec: Vec<SongTag> = Vec::new();

    for elem in array {
        if let Some(parsed) = parse_song_info(elem) {
            vec.push(parsed);
        }
    }

    Ok(vec)
}

/// Try to parse a single [`SongTag`] from a given kugou value
fn parse_song_info(v: &Value) -> Option<SongTag> {
    let price = v.get("price").and_then(Value::as_u64);

    let urltype = price.map_or(UrlTypes::Protected, |price| {
        if price > 0 {
            UrlTypes::Protected
        } else {
            UrlTypes::AvailableRequiresFetching
        }
    });

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
        url: Some(urltype),
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
