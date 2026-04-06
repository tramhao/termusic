use crate::songtag::UrlTypes;

use super::super::{ServiceProvider, SongTag};
use serde_json::{Value, from_str};

#[derive(Debug, thiserror::Error)]
pub enum MiguParseError {
    #[error(
        "Expected field \"{field}\" to have value \"{expected}\", got \"{got}\", error message: \"{errmsg:#?}\""
    )]
    UnexpectedStatus {
        field: &'static str,
        got: String,
        errmsg: Option<String>,
        expected: &'static str,
    },

    #[error("Expected property \"{0}\" to exist")]
    MissingProperty(&'static str),

    #[error(transparent)]
    ParseError(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, MiguParseError>;

/// Get the given `field`, otherwise return as [`MissingProperty`](MiguParseError::MissingProperty) Error
fn get_code_prop<'a>(value: &'a Value, field: &'static str) -> Result<&'a Value> {
    let Some(value) = value.get(field) else {
        return Err(MiguParseError::MissingProperty(field));
    };

    Ok(value)
}

/// Check the `msg` property for if there was a error
fn check_msg(value: &Value) -> Result<()> {
    let msg = get_code_prop(value, "msg")?;

    // english for the chinese characters: "success"
    if !msg.eq(&"成功") {
        let message = msg.to_string();

        return Err(MiguParseError::UnexpectedStatus {
            field: "msg",
            got: message,
            errmsg: None,
            expected: "成功",
        });
    }

    Ok(())
}

/// Check the `success` property for if there was a error
fn check_success(value: &Value) -> Result<()> {
    let success = get_code_prop(value, "code")?;

    if !success.eq("000000") {
        let message = success.to_string();

        return Err(MiguParseError::UnexpectedStatus {
            field: "success",
            got: message,
            errmsg: None,
            expected: "true",
        });
    }

    Ok(())
}

#[allow(unused)]
/// Try to get the lyric lrc content from the given result
pub fn to_lyric(json: &str) -> Result<String> {
    let value = from_str::<Value>(json)?;

    check_msg(&value)?;

    let lyric = value
        .get("lyric")
        .and_then(Value::as_str)
        .ok_or(MiguParseError::MissingProperty("lyric"))?
        .to_owned();

    Ok(lyric)
}

#[allow(unused)]
/// Try to get the picture url from the json response
pub fn to_pic_url(json: &str) -> Result<String> {
    let value = from_str::<Value>(json)?;

    check_msg(&value)?;

    let pic_url = value
        .get("largePic")
        .and_then(Value::as_str)
        .ok_or(MiguParseError::MissingProperty("largePic"))?
        .to_owned();

    Ok(pic_url)
}

/// Try to get individual [`SongTag`]s from the json response
pub fn to_song_info(json: &str) -> Result<Vec<SongTag>> {
    let value = from_str::<Value>(json)?;

    check_success(&value)?;

    let array = value
        .get("songResultData")
        .and_then(Value::as_object)
        .and_then(|v| v.get("result"))
        .and_then(Value::as_array)
        .ok_or(MiguParseError::MissingProperty("songResultData.result"))?;

    let mut vec: Vec<SongTag> = Vec::new();

    for elem in array {
        if let Some(parsed) = parse_song_info(elem) {
            vec.push(parsed);
        }
    }

    Ok(vec)
}

/// Try to parse a single [`SongTag`] from a given migu value
fn parse_song_info(v: &Value) -> Option<SongTag> {
    // not using "Value::to_string()" as that produces a escaped string

    let pic_id = v
        .get("imgItems")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("img"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let artist = v
        .get("singers")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("name"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let title = v.get("name").and_then(Value::as_str).map(ToOwned::to_owned);

    let album_id = v
        .get("albums")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("id"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let album = v
        .get("albums")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("name"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let url = v
        .get("mp3")
        .and_then(Value::as_str)
        .map_or(UrlTypes::Protected, |v| {
            UrlTypes::FreeDownloadable(v.to_owned())
        });

    let lyric_id = v
        .get("lyricUrl")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    // a songid is always required
    let song_id = v.get("id").and_then(Value::as_str).map(ToOwned::to_owned)?;

    Some(SongTag {
        song_id,
        title,
        artist,
        album,
        pic_id,
        lang_ext: Some("migu".to_string()),
        service_provider: ServiceProvider::Migu,
        lyric_id,
        url: Some(url),
        album_id,
    })
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use pretty_assertions::assert_eq;

//     #[test]
//     fn should_parse_songinfo() {
//         const ARTIST: &str = "Some Artist";

//         let sample_data = r#"{
//   "bestShowResultToneData": {},
//   "code": "000000",
//   "info": "成功",
//   "songResultData": {
//     "correct": [],
//     "isFromCache": "0",
//     "result": [
//       {
//         "albums": [
//           {
//             "id": "1137587980",
//             "name": "Some Album 1",
//             "type": "1"
//           }
//         ],
//         "chargeAuditions": "1",
//         "contentId": "600919000003404514",
//         "copyright": "1",
//         "copyrightId": "6005663CHX2",
//         "dalbumId": "",
//         "digitalColumnId": "",
//         "highlightStr": [
//           "track a",
//           "track"
//         ],
//         "id": "0000000002",
//         "imgItems": [
//           {
//             "img": "https://d.musicapp.migu.cn/data/oss/resource/00/2z/hl/c01052cd884249a28f9e456b727eb93f.webp",
//             "imgSizeType": "01"
//           },
//           {
//             "img": "https://d.musicapp.migu.cn/data/oss/resource/00/2z/hl/1f08cf7f26ac468dbeaa2a1787cd7f59.webp",
//             "imgSizeType": "02"
//           },
//           {
//             "img": "https://d.musicapp.migu.cn/data/oss/resource/00/2z/hl/be4d28ca66b9481bb25bc9c41c59ca0d.webp",
//             "imgSizeType": "03"
//           }
//         ],
//         "invalidateDate": "2099-12-31",
//         "isInDAlbum": "0",
//         "isInSalesPeriod": "0",
//         "isInSideDalbum": "0",
//         "lyricUrl": "0000000AAAA",
//         "mrcurl": "",
//         "name": "Track A",
//         "newRateFormats": [
//           {
//             "fileType": "mp3",
//             "format": "020007",
//             "formatType": "PQ",
//             "price": "200",
//             "resourceType": "2",
//             "showTag": [
//               "vip"
//             ],
//             "size": "6381194"
//           },
//           {
//             "fileType": "mp3",
//             "format": "020010",
//             "formatType": "HQ",
//             "price": "200",
//             "resourceType": "2",
//             "showTag": [
//               "vip"
//             ],
//             "size": "15952669"
//           }
//         ],
//         "rateFormats": [
//           {
//             "fileType": "mp3",
//             "format": "000019",
//             "formatType": "LQ",
//             "price": "200",
//             "resourceType": "3",
//             "size": "3190784"
//           },
//           {
//             "fileType": "mp3",
//             "format": "020007",
//             "formatType": "PQ",
//             "price": "200",
//             "resourceType": "2",
//             "showTag": [
//               "vip"
//             ],
//             "size": "6381194"
//           },
//           {
//             "fileType": "mp3",
//             "format": "020010",
//             "formatType": "HQ",
//             "price": "200",
//             "resourceType": "2",
//             "showTag": [
//               "vip"
//             ],
//             "size": "15952669"
//           }
//         ],
//         "relatedSongs": [
//           {
//             "copyrightId": "6005661VFV2",
//             "productId": "600913000003477829",
//             "resourceType": "0",
//             "resourceTypeName": "彩铃"
//           },
//           {
//             "copyrightId": "6005663CHX2",
//             "productId": "600919000003404515",
//             "resourceType": "3",
//             "resourceTypeName": "随身听"
//           }
//         ],
//         "resourceType": "2",
//         "scopeOfcopyright": "01",
//         "singers": [
//           {
//             "id": "22592",
//             "name": "Charles Mingus"
//           }
//         ],
//         "songAliasName": "",
//         "songDescs": "",
//         "songType": "",
//         "tags": [
//           "纯音乐",
//           "欢快",
//           "贝司",
//           "录音室版",
//           "酒吧",
//           "爵士"
//         ],
//         "televisionNames": [
//           ""
//         ],
//         "toneControl": "110000",
//         "tones": [
//           {
//             "copyrightId": "6005661VFV2",
//             "expireDate": "2099-12-31",
//             "id": "600913000003477829",
//             "price": "200"
//           }
//         ],
//         "translateName": "",
//         "trcUrl": "",
//         "vipType": "1"
//       },
//       {
//         "albums": [
//           {
//             "id": "1111111111",
//             "name": "Some Album 2",
//             "type": "1"
//           }
//         ],
//         "chargeAuditions": "0",
//         "contentId": "600929000001419906",
//         "copyright": "1",
//         "copyrightId": "6005861LRSB",
//         "dalbumId": "",
//         "digitalColumnId": "",
//         "highlightStr": [
//           "track a",
//           "track"
//         ],
//         "id": "1111111112",
//         "imgItems": [
//           {
//             "img": "https://d.musicapp.migu.cn/data/oss/resource/00/46/7g/f46c38e28d414c4c9efe598bdbf3b042.webp",
//             "imgSizeType": "01"
//           },
//           {
//             "img": "https://d.musicapp.migu.cn/data/oss/resource/00/46/7g/b7fde6f599ee42deabacfcc31503ba58.webp",
//             "imgSizeType": "02"
//           },
//           {
//             "img": "https://d.musicapp.migu.cn/data/oss/resource/00/46/7g/b34f144db79342e48444f574fe3abb82.webp",
//             "imgSizeType": "03"
//           }
//         ],
//         "invalidateDate": "2026-10-29",
//         "isInDAlbum": "0",
//         "isInSalesPeriod": "0",
//         "isInSideDalbum": "0",
//         "lyricUrl": "https://d.musicapp.migu.cn/data/oss/resource/00/49/1k/425c7bbf0f84495cad986220f82f608b",
//         "mrcurl": "",
//         "name": "Track B",
//         "newRateFormats": [
//           {
//             "fileType": "mp3",
//             "format": "020007",
//             "formatType": "PQ",
//             "price": "200",
//             "resourceType": "2",
//             "size": "2975244"
//           },
//           {
//             "fileType": "mp3",
//             "format": "020010",
//             "formatType": "HQ",
//             "price": "200",
//             "resourceType": "2",
//             "size": "7437794"
//           }
//         ],
//         "rateFormats": [
//           {
//             "fileType": "mp3",
//             "format": "000019",
//             "formatType": "LQ",
//             "price": "200",
//             "resourceType": "3",
//             "size": "1487808"
//           },
//           {
//             "fileType": "mp3",
//             "format": "020007",
//             "formatType": "PQ",
//             "price": "200",
//             "resourceType": "2",
//             "size": "2975244"
//           },
//           {
//             "fileType": "mp3",
//             "format": "020010",
//             "formatType": "HQ",
//             "price": "200",
//             "resourceType": "2",
//             "size": "7437794"
//           }
//         ],
//         "relatedSongs": [
//           {
//             "copyrightId": "6005861LRSB",
//             "productId": "600929000001419905",
//             "resourceType": "1",
//             "resourceTypeName": "振铃"
//           },
//           {
//             "copyrightId": "6005861LRSB",
//             "productId": "600929000001419904",
//             "resourceType": "0",
//             "resourceTypeName": "彩铃"
//           },
//           {
//             "copyrightId": "6005861LRSB",
//             "productId": "600929000001419907",
//             "resourceType": "3",
//             "resourceTypeName": "随身听"
//           }
//         ],
//         "resourceType": "2",
//         "scopeOfcopyright": "01",
//         "singers": [
//           {
//             "id": "943302",
//             "name": "Kid Cudi"
//           },
//           {
//             "id": "1108414723",
//             "name": "Trackademicks"
//           },
//           {
//             "id": "1140891033",
//             "name": "Kanye West & Common"
//           }
//         ],
//         "songDescs": "",
//         "songType": "",
//         "tags": [
//           "纯音乐",
//           "嘻哈",
//           "原创单曲"
//         ],
//         "televisionNames": [
//           ""
//         ],
//         "toneControl": "110000",
//         "tones": [
//           {
//             "copyrightId": "6005861LRSB",
//             "expireDate": "2026-10-29",
//             "id": "600929000001419904",
//             "price": "200"
//           }
//         ],
//         "trcUrl": "",
//         "vipType": ""
//       },
// }"#;
//         let res = to_song_info(sample_data).unwrap();

//         assert_eq!(res.len(), 2);

//         assert_eq!(
//             res[0],
//             SongTag {
//                 artist: Some(ARTIST.to_owned()),
//                 title: Some("Track A".to_owned()),
//                 album: Some("Some Album 1".to_owned()),
//                 lang_ext: Some("migu".to_string()),
//                 service_provider: ServiceProvider::Migu,
//                 song_id: "0000000002".to_owned(),
//                 lyric_id: Some("0000000AAAA".to_owned()),
//                 url: Some(UrlTypes::Protected),
//                 pic_id: Some("https://d.musicapp.migu.cn/data/oss/resource/00/2z/hl/c01052cd884249a28f9e456b727eb93f.webp".to_owned()),
//                 album_id: Some("1137587980".to_owned())
//             }
//         );

//         assert_eq!(
//             res[1],
//             SongTag {
//                 artist: Some(ARTIST.to_owned()),
//                 title: Some("Track B".to_owned()),
//                 album: Some("Some Album 2".to_owned()),
//                 lang_ext: Some("migu".to_string()),
//                 service_provider: ServiceProvider::Migu,
//                 song_id: "1111111112".to_owned(),
//                 lyric_id: Some("https://d.musicapp.migu.cn/data/oss/resource/00/49/1k/425c7bbf0f84495cad986220f82f608b".to_owned()),
//                 url: Some(UrlTypes::Protected),
//                 pic_id: Some("https://d.musicapp.migu.cn/data/oss/resource/00/46/7g/f46c38e28d414c4c9efe598bdbf3b042.webp".to_owned()),
//                 album_id: Some("1111111111".to_owned())
//             }
//         );
//     }
// }
