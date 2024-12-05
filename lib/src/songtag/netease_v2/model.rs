use super::super::{ServiceProvider, SongTag};
use crate::songtag::UrlTypes;
use anyhow::{anyhow, bail, Result};
use serde_json::{from_str, Value};

/// Try to get the lyric lrc content from the given result
pub fn to_lyric(json: &str) -> Result<String> {
    let value = from_str::<Value>(json).map_err(anyhow::Error::from)?;

    if value.get("code").is_none() || !value.get("code").map_or(false, |v| v.eq(&200)) {
        let code = value
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or("<none>");
        bail!("Failed to get lyric text, \"code\" does not exist or is not 200 code: {code}");
    }

    let lyric = value
        .get("lrc")
        .and_then(Value::as_str)
        .ok_or(anyhow!("property \"lrc\" does not exist in result!"))?
        .to_owned();

    Ok(lyric)
}

/// Try to get the play (download) url from the result
pub fn to_song_url(json: &str) -> Result<String> {
    let value = from_str::<Value>(json).map_err(anyhow::Error::from)?;

    if value.get("code").is_none() || !value.get("code").map_or(false, |v| v.eq(&200)) {
        let errcode = value
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or("<none>");
        bail!("Failed to get download url, \"code\" does not exist or is not 200 code: {errcode}");
    }

    let first_url = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or(anyhow!("property \"data\" does not exist in result!"))?
        .iter()
        .flat_map(parse_song_url)
        // only get one for now
        .next()
        .ok_or(anyhow!("no urls in \"data\"!"))?;

    Ok(first_url.to_owned())
}

/// Get all the urls and their metadata from the response
fn parse_song_url(value: &Value) -> Option<&str> {
    let url = value.get("url").and_then(Value::as_str)?;

    // let id = value.get("id").and_then(Value::as_str)?;
    // let rate = value.get("br").and_then(Value::as_str)?;

    Some(url)
}

/// Try to get individual [`SongTag`]s from the json response
pub fn to_song_info(json: &str) -> Result<Vec<SongTag>> {
    let value = from_str::<Value>(json).map_err(anyhow::Error::from)?;

    if value.get("code").is_none() || !value.get("code").map_or(false, |v| v.eq(&200)) {
        let code = value
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or("<none>");
        bail!("Failed to get lyric text, \"code\" does not exist or is not 200 code: {code}");
    }

    let array = value
        .get("result")
        .and_then(Value::as_object)
        .and_then(|v| v.get("songs"))
        .and_then(Value::as_array)
        .ok_or(anyhow!(
            "property \"result.songs\" does not exist in result!"
        ))?;

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
    let fee = v.get("fee").and_then(Value::as_u64);

    let urltype = fee.map_or(UrlTypes::Protected, |price| {
        if price > 0 {
            UrlTypes::Protected
        } else {
            UrlTypes::AvailableRequiresFetching
        }
    });

    let song_id = v.get("id").and_then(Value::as_u64).map(|v| v.to_string())?;

    let artist = v
        .get("artists")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("name"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let title = v.get("name").and_then(Value::as_str).map(ToOwned::to_owned);

    let album = v
        .get("album")
        .and_then(|v| v.get("name"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let lyric_id = v.get("id").and_then(Value::as_u64).map(|v| v.to_string());

    let album_id = v
        .get("album")
        .and_then(|v| v.get("picId"))
        .and_then(Value::as_u64)
        .map(|v| v.to_string());

    // i dont know why, but pic_id uses the same value as album
    let pic_id = album_id.clone();
    // v.get("album")
    // .and_then(|v| v.get("picId"))
    // .and_then(Value::as_u64).map(|v| v.to_string());

    Some(SongTag {
        song_id,
        title,
        artist,
        album,
        pic_id,
        lang_ext: Some("netease".to_string()),
        service_provider: ServiceProvider::Netease,
        lyric_id,
        url: Some(urltype),
        album_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn should_parse_songinfo() {
        const ARTIST: &str = "Some Artist";

        let sample_data = r#"{
            "result": {
              "songs": [
                {
                  "id": 1000000000,
                  "name": "Track A",
                  "artists": [
                    {
                      "id": 3333333,
                      "name": "Some Artist",
                      "picUrl": null,
                      "alias": [],
                      "albumSize": 0,
                      "picId": 0,
                      "fansGroup": null,
                      "img1v1Url": "https://p3.music.126.net/AAAAAAAAAAAAAAAAAAAAAAAA/0000000000000000.jpg",
                      "img1v1": 0,
                      "trans": null
                    }
                  ],
                  "album": {
                    "id": 100000001,
                    "name": "Some Album 1",
                    "artist": {
                      "id": 0,
                      "name": "",
                      "picUrl": null,
                      "alias": [],
                      "albumSize": 0,
                      "picId": 0,
                      "fansGroup": null,
                      "img1v1Url": "https://p3.music.126.net/AAAAAAAAAAAAAAAAAAAAAAAA/0000000000000000.jpg",
                      "img1v1": 0,
                      "trans": null
                    },
                    "publishTime": 1111111111111,
                    "size": 6,
                    "copyrightId": 1111,
                    "status": 1,
                    "picId": 444444444444444444,
                    "mark": 0
                  },
                  "duration": 89595,
                  "copyrightId": 1111,
                  "status": 0,
                  "alias": [],
                  "rtype": 0,
                  "ftype": 0,
                  "mvid": 0,
                  "fee": 1,
                  "rUrl": null,
                  "mark": 66666666666
                },
                {
                  "id": 1111111111,
                  "name": "Track B",
                  "artists": [
                    {
                      "id": 3333333,
                      "name": "Some Artist",
                      "picUrl": null,
                      "alias": [],
                      "albumSize": 0,
                      "picId": 0,
                      "fansGroup": null,
                      "img1v1Url": "https://p4.music.126.net/AAAAAAAAAAAAAAAAAAAAAAAA/0000000000000000.jpg",
                      "img1v1": 0,
                      "trans": null
                    }
                  ],
                  "album": {
                    "id": 11111112,
                    "name": "Some Album 2",
                    "artist": {
                      "id": 0,
                      "name": "",
                      "picUrl": null,
                      "alias": [],
                      "albumSize": 0,
                      "picId": 0,
                      "fansGroup": null,
                      "img1v1Url": "https://p3.music.126.net/AAAAAAAAAAAAAAAAAAAAAAAA/0000000000000000.jpg",
                      "img1v1": 0,
                      "trans": null
                    },
                    "publishTime": 2222222222222,
                    "size": 4,
                    "copyrightId": 1111,
                    "status": 1,
                    "picId": 555555555555555555,
                    "mark": 0
                  },
                  "duration": 158143,
                  "copyrightId": 1111,
                  "status": 0,
                  "alias": [],
                  "rtype": 0,
                  "ftype": 0,
                  "mvid": 8888888,
                  "fee": 1,
                  "rUrl": null,
                  "mark": 77777777777
                }
              ],
              "hasMore": false,
              "songCount": 20
            },
            "code": 200
          }"#;

        let res = to_song_info(sample_data).unwrap();

        assert_eq!(res.len(), 2);

        assert_eq!(
            res[0],
            SongTag {
                artist: Some(ARTIST.to_owned()),
                title: Some("Track A".to_owned()),
                album: Some("Some Album 1".to_owned()),
                lang_ext: Some("netease".to_string()),
                service_provider: ServiceProvider::Netease,
                song_id: "1000000000".to_owned(),
                lyric_id: Some("1000000000".to_owned()),
                url: Some(UrlTypes::Protected),
                pic_id: Some("444444444444444444".to_owned()),
                album_id: Some("444444444444444444".to_owned())
            }
        );

        assert_eq!(
            res[1],
            SongTag {
                artist: Some(ARTIST.to_owned()),
                title: Some("Track B".to_owned()),
                album: Some("Some Album 2".to_owned()),
                lang_ext: Some("netease".to_string()),
                service_provider: ServiceProvider::Netease,
                song_id: "1111111111".to_owned(),
                lyric_id: Some("1111111111".to_owned()),
                url: Some(UrlTypes::Protected),
                pic_id: Some("555555555555555555".to_owned()),
                album_id: Some("555555555555555555".to_owned())
            }
        );
    }
}
