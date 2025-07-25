use crate::songtag::UrlTypes;

use super::super::{ServiceProvider, SongTag};
use serde_json::{Value, from_str, json};

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
    let success = get_code_prop(value, "success")?;

    if !success.eq(&true) {
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
        .get("musics")
        .and_then(Value::as_array)
        .ok_or(MiguParseError::MissingProperty("musics"))?;

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
        .get("cover")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let artist = v
        .get("singerName")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let title = v
        .get("songName")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let album_id = v
        .get("albumId")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let url = v
        .get("mp3")
        .and_then(Value::as_str)
        .map_or(UrlTypes::Protected, |v| {
            UrlTypes::FreeDownloadable(v.to_owned())
        });

    let lyric_id = v
        .get("copyrightId")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    // a songid is always required
    let song_id = v.get("id").and_then(Value::as_str).map(ToOwned::to_owned)?;

    Some(SongTag {
        song_id,
        title,
        artist,
        album: Some(
            v.get("albumName")
                .unwrap_or(&json!("Unknown Album"))
                .as_str()
                .unwrap_or("")
                .to_owned(),
        ),
        pic_id,
        lang_ext: Some("migu".to_string()),
        service_provider: ServiceProvider::Migu,
        lyric_id,
        url: Some(url),
        album_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_parse_songinfo() {
        const ARTIST: &str = "Some Artist";

        let sample_data = r#"{
            "musics": [
              {
                "songName": "Track A",
                "isHdCrbt": null,
                "albumName": "Some Album 1",
                "has24Bitqq": null,
                "hasMv": null,
                "artist": "Some Artist",
                "hasHQqq": "1",
                "albumId": "0000000000",
                "title": "Track A",
                "singerName": "Some Artist",
                "cover": "https://mcontent.migu.cn/newlv2/new/album/20230810/0000000000/someRandomCode.jpg",
                "mp3": "https://freetyst.nf.migu.cn/SomeLongPercentFilename.mp3?Key=AAAAAAAAAAAAAAAA&Tim=1111111111111&channelid=01&msisdn=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                "hasSQqq": null,
                "has3Dqq": null,
                "singerId": "0000000001",
                "mvCopyrightId": null,
                "copyrightId": "0000000AAAA",
                "unuseFlag": null,
                "auditionsFlag": null,
                "auditionsLength": null,
                "mvId": "",
                "id": "0000000002",
                "lyrics": "https://tyqk.migu.cn/files/lyric/2018-04-20/CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC.lrc"
              },
              {
                "songName": "Track B",
                "isHdCrbt": null,
                "albumName": "Some Album 2",
                "has24Bitqq": null,
                "hasMv": null,
                "artist": "Some Artist",
                "hasHQqq": "1",
                "albumId": "1111111111",
                "title": "Track B",
                "singerName": "Some Artist",
                "cover": "https://tyqk.migu.cn/files/resize/album/2023-12-19/someOtherRandomCode.jpg?200x200",
                "mp3": "https://freetyst.nf.migu.cn/SomeOtherLongPercentFilename.mp3?Key=AAAAAAAAAAAAAAAA&Tim=1111111111111&channelid=01&msisdn=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                "hasSQqq": null,
                "has3Dqq": null,
                "singerId": "0000000001",
                "mvCopyrightId": null,
                "copyrightId": "1111111BBBB",
                "unuseFlag": null,
                "auditionsFlag": null,
                "auditionsLength": null,
                "mvId": "",
                "id": "1111111112",
                "lyrics": "https://tyqk.migu.cn/files/lyric/2018-12-22/DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD.lrc"
              }
            ],
            "pgt": 100,
            "keyword": "Some Artist Track A",
            "pageNo": "0",
            "success": true
        }"#;

        let res = to_song_info(sample_data).unwrap();

        assert_eq!(res.len(), 2);

        assert_eq!(
            res[0],
            SongTag {
                artist: Some(ARTIST.to_owned()),
                title: Some("Track A".to_owned()),
                album: Some("Some Album 1".to_owned()),
                lang_ext: Some("migu".to_string()),
                service_provider: ServiceProvider::Migu,
                song_id: "0000000002".to_owned(),
                lyric_id: Some("0000000AAAA".to_owned()),
                url: Some(UrlTypes::FreeDownloadable("https://freetyst.nf.migu.cn/SomeLongPercentFilename.mp3?Key=AAAAAAAAAAAAAAAA&Tim=1111111111111&channelid=01&msisdn=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned())),
                pic_id: Some("https://mcontent.migu.cn/newlv2/new/album/20230810/0000000000/someRandomCode.jpg".to_owned()),
                album_id: Some("0000000000".to_owned())
            }
        );

        assert_eq!(
            res[1],
            SongTag {
                artist: Some(ARTIST.to_owned()),
                title: Some("Track B".to_owned()),
                album: Some("Some Album 2".to_owned()),
                lang_ext: Some("migu".to_string()),
                service_provider: ServiceProvider::Migu,
                song_id: "1111111112".to_owned(),
                lyric_id: Some("1111111BBBB".to_owned()),
                url: Some(UrlTypes::FreeDownloadable("https://freetyst.nf.migu.cn/SomeOtherLongPercentFilename.mp3?Key=AAAAAAAAAAAAAAAA&Tim=1111111111111&channelid=01&msisdn=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned())),
                pic_id: Some("https://tyqk.migu.cn/files/resize/album/2023-12-19/someOtherRandomCode.jpg?200x200".to_owned()),
                album_id: Some("1111111111".to_owned())
            }
        );
    }
}
