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
use serde_json::{json, Value};

pub fn to_lyric(json: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<Value>(json) {
        if value.get("msg")?.eq("\u{6210}\u{529f}") {
            // if value.get("msg")?.eq("成功") {
            let lyric = value.get("lyric")?.as_str()?.to_owned();
            return Some(lyric);
        }
    }
    None
}

pub fn to_pic_url(json: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<Value>(json) {
        if value.get("msg")?.eq("\u{6210}\u{529f}") {
            // if value.get("msg")?.eq("成功") {
            let pic_url = value.get("largePic")?.as_str()?.to_owned();
            return Some(pic_url);
        }
    }
    None
}

pub fn to_song_info(json: &str) -> Option<Vec<SongTag>> {
    if let Ok(value) = serde_json::from_str::<Value>(json) {
        if value.get("success")?.eq(&true) {
            let mut vec: Vec<SongTag> = Vec::new();
            let list = json!([]);
            let array = value.get("musics").unwrap_or(&list).as_array()?;
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
    let pic_id = v
        .get("cover")
        .unwrap_or(&json!("N/A"))
        .as_str()
        .unwrap_or("")
        .to_owned();
    let artist = v
        .get("singerName")
        .unwrap_or(&json!("Unknown Singer"))
        .as_str()
        .unwrap_or("Unknown Singer")
        .to_owned();
    let title = v.get("songName")?.as_str()?.to_owned();

    let album_id = v.get("albumId")?.as_str()?.to_owned();

    let url = v
        .get("mp3")
        .unwrap_or(&json!("N/A"))
        .as_str()
        .unwrap_or("Copyright protected")
        .to_owned();

    Some(SongTag {
        song_id: Some(v.get("id")?.as_str()?.to_owned()),
        title: Some(title),
        artist: Some(artist),
        album: Some(
            v.get("albumName")
                .unwrap_or(&json!("Unknown Album"))
                .as_str()
                .unwrap_or("")
                .to_owned(),
        ),
        pic_id: Some(pic_id),
        lang_ext: Some("migu".to_string()),
        service_provider: Some(ServiceProvider::Migu),
        lyric_id: Some(v.get("copyrightId")?.as_str()?.to_owned()),
        url: Some(url),
        album_id: Some(album_id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
                service_provider: Some(ServiceProvider::Migu),
                song_id: Some("0000000002".to_owned()),
                lyric_id: Some("0000000AAAA".to_owned()),
                url: Some("https://freetyst.nf.migu.cn/SomeLongPercentFilename.mp3?Key=AAAAAAAAAAAAAAAA&Tim=1111111111111&channelid=01&msisdn=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned()),
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
                service_provider: Some(ServiceProvider::Migu),
                song_id: Some("1111111112".to_owned()),
                lyric_id: Some("1111111BBBB".to_owned()),
                url: Some("https://freetyst.nf.migu.cn/SomeOtherLongPercentFilename.mp3?Key=AAAAAAAAAAAAAAAA&Tim=1111111111111&channelid=01&msisdn=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned()),
                pic_id: Some("https://tyqk.migu.cn/files/resize/album/2023-12-19/someOtherRandomCode.jpg?200x200".to_owned()),
                album_id: Some("1111111111".to_owned())
            }
        );
    }
}
