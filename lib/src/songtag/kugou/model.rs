use std::string::FromUtf8Error;

use crate::songtag::UrlTypes;

use super::super::{ServiceProvider, SongTag};
use base64::{Engine as _, engine::general_purpose};
use serde_json::{Value, from_str, json};

#[derive(Debug, thiserror::Error)]
pub enum KugouParseError {
    #[error(
        "Expected field \"{field}\" to have code \"{expected}\", got \"{got}\", errcode: \"{errcode:#?}\""
    )]
    UnexpectedStatus {
        field: &'static str,
        got: String,
        errcode: Option<String>,
        expected: usize,
    },

    #[error("Expected property \"{0}\" to exist")]
    MissingProperty(&'static str),

    #[error(transparent)]
    ParseError(#[from] serde_json::Error),

    #[error(transparent)]
    Base64DecodeError(#[from] base64::DecodeError),

    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
}

type Result<T> = std::result::Result<T, KugouParseError>;

/// Check the `status` field against `expected`, if `status` does not exist or does not match, return [`Err`]
fn check_status(value: &Value, expected: usize) -> Result<()> {
    const FIELD: &str = "status";
    let Some(status) = value.get(FIELD) else {
        return Err(KugouParseError::MissingProperty(FIELD));
    };
    if !status.eq(&expected) {
        let errcode = value.get("errcode").map(Value::to_string);

        return Err(KugouParseError::UnexpectedStatus {
            field: FIELD,
            got: status.to_string(),
            errcode,
            expected,
        });
    }

    Ok(())
}

/// Try to get the lyric lrc content from the given result
pub fn to_lyric(json: &str) -> Result<String> {
    let value = from_str::<Value>(json)?;

    check_status(&value, 200)?;

    let lyric = value
        .get("content")
        .and_then(Value::as_str)
        .ok_or(KugouParseError::MissingProperty("content"))?
        .to_owned();
    let lyric_decoded = general_purpose::STANDARD.decode(lyric)?;
    let lyric_str = String::from_utf8(lyric_decoded)?;

    Ok(lyric_str)
}

/// Try to get the `accesskey` and lyric `id` from the given json response
pub fn to_lyric_id_accesskey(json: &str) -> Result<(String, String)> {
    let value = from_str::<Value>(json)?;

    // TODO: confirm that there is only "errcode" and not "status" like the others
    {
        let Some(errcode) = value.get("errcode") else {
            return Err(KugouParseError::MissingProperty("errcode"));
        };
        if !errcode.eq(&200) {
            return Err(KugouParseError::UnexpectedStatus {
                field: "errocode",
                got: errcode.to_string(),
                errcode: None,
                expected: 200,
            });
        }
    }

    let v = value
        .get("candidates")
        .and_then(|v| v.get(0))
        .ok_or(KugouParseError::MissingProperty("candidates.0"))?;
    let accesskey = v
        .get("accesskey")
        .and_then(Value::as_str)
        .ok_or(KugouParseError::MissingProperty("candidates.0.accesskey"))?
        .to_owned();
    let id = v
        .get("id")
        .and_then(Value::as_str)
        .ok_or(KugouParseError::MissingProperty("candidates.0.id"))?
        .to_owned();

    Ok((accesskey, id))
}

/// Try to get the play (download) url from the result
pub fn to_song_url(json: &str) -> Result<String> {
    let value = from_str::<Value>(json)?;

    check_status(&value, 200)?;

    let url = value
        .get("data")
        .and_then(|v| v.get("play_url"))
        .and_then(Value::as_str)
        .ok_or(KugouParseError::MissingProperty("data.play_url"))?
        .to_owned();

    Ok(url)
}

/// Try to get the picture url from the json response
pub fn to_pic_url(json: &str) -> Result<String> {
    let value = from_str::<Value>(json)?;

    check_status(&value, 1)?;

    let url = value
        .get("data")
        .and_then(|v| v.get("img"))
        .and_then(Value::as_str)
        .ok_or(KugouParseError::MissingProperty("data.img"))?
        .to_owned();

    Ok(url)
}

/// Try to get individual [`SongTag`]s from the json response
pub fn to_song_info(json: &str) -> Result<Vec<SongTag>> {
    let value = from_str::<Value>(json)?;

    check_status(&value, 1)?;

    let array = value
        .get("data")
        .and_then(Value::as_object)
        .and_then(|v| v.get("info"))
        .and_then(Value::as_array)
        .ok_or(KugouParseError::MissingProperty("data.info"))?;

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
