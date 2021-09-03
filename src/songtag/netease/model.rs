//
// model.rs
// Copyright (C) 2019 gmg137 <gmg137@live.com>
// Distributed under terms of the GPLv3 license.
//
use super::super::{SongTag, SongtagProvider};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[allow(unused)]
pub fn to_lyric(json: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<Value>(json) {
        if value.get("code")?.eq(&200) {
            let mut vec: Vec<String> = Vec::new();
            let lyric = value.get("lrc")?.get("lyric")?.as_str()?.to_owned();
            return Some(lyric);
        }
    }
    None
}

// 歌手信息
#[derive(Debug, Deserialize, Serialize)]
pub struct SingerInfo {
    // 歌手 id
    pub id: u64,
    // 歌手姓名
    pub name: String,
    // 歌手照片
    pub pic_url: String,
}

#[allow(unused)]
pub fn to_singer_info(json: &str) -> Option<Vec<SingerInfo>> {
    if let Ok(value) = serde_json::from_str::<Value>(json) {
        if value.get("code")?.eq(&200) {
            let mut vec: Vec<SingerInfo> = Vec::new();
            let array = value.get("result")?.get("artists")?.as_array()?;
            for v in array.iter() {
                vec.push(SingerInfo {
                    id: v.get("id")?.as_u64()?,
                    name: v.get("name")?.as_str()?.to_owned(),
                    pic_url: v
                        .get("picUrl")
                        .unwrap_or(&json!(""))
                        .as_str()
                        .unwrap_or("")
                        .to_owned(),
                });
            }
            return Some(vec);
        }
    }
    None
}

// 歌曲 URL
#[derive(Debug, Deserialize, Serialize)]
pub struct SongUrl {
    // 歌曲 id
    pub id: u64,
    // 歌曲 URL
    pub url: String,
    // 码率
    pub rate: u32,
}

#[allow(unused)]
pub fn to_song_url(json: &str) -> Option<Vec<SongUrl>> {
    if let Ok(value) = serde_json::from_str::<Value>(json) {
        if value.get("code")?.eq(&200) {
            let mut vec: Vec<SongUrl> = Vec::new();
            let array = value.get("data")?.as_array()?;
            for v in array.iter() {
                let url = v
                    .get("url")
                    .unwrap_or(&json!(""))
                    .as_str()
                    .unwrap_or("")
                    .to_owned();
                if !url.is_empty() {
                    vec.push(SongUrl {
                        id: v.get("id")?.as_u64()?,
                        url,
                        rate: v.get("br")?.as_u64()? as u32,
                    });
                }
            }
            return Some(vec);
        }
    }
    None
}

// 歌曲信息
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SongInfo {
    // 歌曲 id
    pub id: u64,
    // 歌名
    pub name: String,
    // 歌手
    pub singer: String,
    // 专辑
    pub album: String,
    // 封面图
    pub pic_url: String,
    // 歌曲时长
    pub duration: String,
    // 歌曲链接
    pub song_url: String,
}

impl SongInfo {
    // pub fn get_song_cache_path(&self) -> PathBuf {
    //     PathBuf::from(
    //         format!(
    //             "{}{}-{}.m4a",
    //             NCM_CACHE.to_string_lossy(),
    //             self.name,
    //             self.id
    //         )
    //         .as_str(),
    //     )
    // }
}

// parse: 解析方式
#[allow(unused)]
pub fn to_song_info(json: &str, parse: Parse) -> Option<Vec<SongTag>> {
    if let Ok(value) = serde_json::from_str::<Value>(json) {
        if value.get("code")?.eq(&200) {
            let mut vec: Vec<SongInfo> = Vec::new();
            let list = json!([]);
            match parse {
                Parse::USL => {
                    let mut array = value.get("songs").unwrap_or(&list).as_array()?;
                    if array.is_empty() {
                        array = value.get("playlist")?.get("tracks")?.as_array()?;
                    }
                    for v in array.iter() {
                        let duration = v.get("dt")?.as_u64()?;
                        vec.push(SongInfo {
                            id: v.get("id")?.as_u64()?,
                            name: v.get("name")?.as_str()?.to_owned(),
                            singer: v
                                .get("ar")?
                                .get(0)?
                                .get("name")
                                .unwrap_or(&json!("未知"))
                                .as_str()
                                .unwrap_or("未知")
                                .to_owned(),
                            album: v
                                .get("al")?
                                .get("name")
                                .unwrap_or(&json!("未知"))
                                .as_str()
                                .unwrap_or("未知")
                                .to_owned(),
                            pic_url: v
                                .get("al")?
                                .get("picUrl")
                                .unwrap_or(&json!(""))
                                .as_str()
                                .unwrap_or("")
                                .to_owned(),
                            duration: format!(
                                "{:0>2}:{:0>2}",
                                duration / 1000 / 60,
                                duration / 1000 % 60
                            ),
                            song_url: String::new(),
                        });
                    }
                }
                Parse::UCD => {
                    let array = value.get("data")?.as_array()?;
                    for v in array.iter() {
                        let duration = v.get("simpleSong")?.get("dt")?.as_u64()? as u32;
                        vec.push(SongInfo {
                            id: v.get("songId")?.as_u64()?,
                            name: v.get("songName")?.as_str()?.to_owned(),
                            singer: v
                                .get("artist")
                                .unwrap_or(&json!("未知"))
                                .as_str()
                                .unwrap_or("未知")
                                .to_owned(),
                            album: v
                                .get("album")
                                .unwrap_or(&json!("未知"))
                                .as_str()
                                .unwrap_or("未知")
                                .to_owned(),
                            pic_url: String::new(),
                            duration: format!(
                                "{:0>2}:{:0>2}",
                                duration / 1000 / 60,
                                duration / 1000 % 60
                            ),
                            song_url: String::new(),
                        });
                    }
                }
                Parse::RMD => {
                    let array = value.get("data")?.as_array()?;
                    for v in array.iter() {
                        let duration = v.get("duration")?.as_u64()? as u32;
                        vec.push(SongInfo {
                            id: v.get("id")?.as_u64()?,
                            name: v.get("name")?.as_str()?.to_owned(),
                            singer: v
                                .get("artists")?
                                .get(0)?
                                .get("name")
                                .unwrap_or(&json!("未知"))
                                .as_str()
                                .unwrap_or("未知")
                                .to_owned(),
                            album: v
                                .get("album")?
                                .get("name")
                                .unwrap_or(&json!("未知"))
                                .as_str()
                                .unwrap_or("未知")
                                .to_owned(),
                            pic_url: v
                                .get("album")?
                                .get("picUrl")
                                .unwrap_or(&json!(""))
                                .as_str()
                                .unwrap_or("")
                                .to_owned(),
                            duration: format!(
                                "{:0>2}:{:0>2}",
                                duration / 1000 / 60,
                                duration / 1000 % 60
                            ),
                            song_url: String::new(),
                        });
                    }
                }
                Parse::RMDS => {
                    let array = value
                        .get("data")?
                        .as_object()?
                        .get("dailySongs")?
                        .as_array()?;
                    for v in array.iter() {
                        let duration = v.get("duration")?.as_u64()? as u32;
                        vec.push(SongInfo {
                            id: v.get("id")?.as_u64()?,
                            name: v.get("name")?.as_str()?.to_owned(),
                            singer: v
                                .get("artists")?
                                .get(0)?
                                .get("name")
                                .unwrap_or(&json!("未知"))
                                .as_str()
                                .unwrap_or("未知")
                                .to_owned(),
                            album: v
                                .get("album")?
                                .get("name")
                                .unwrap_or(&json!("未知"))
                                .as_str()
                                .unwrap_or("未知")
                                .to_owned(),
                            pic_url: v
                                .get("album")?
                                .get("picUrl")
                                .unwrap_or(&json!(""))
                                .as_str()
                                .unwrap_or("")
                                .to_owned(),
                            duration: format!(
                                "{:0>2}:{:0>2}",
                                duration / 1000 / 60,
                                duration / 1000 % 60
                            ),
                            song_url: String::new(),
                        });
                    }
                }
                Parse::SEARCH => {
                    let array = value.get("result")?.as_object()?.get("songs")?.as_array()?;
                    for v in array.iter() {
                        let duration = v.get("duration")?.as_u64()? as u32;
                        let pic_id = v
                            .get("album")?
                            .get("picId")
                            .unwrap_or(&json!("Unknown"))
                            .as_u64()?;
                        let fee = v.get("fee")?.as_u64()?;
                        let mut url = String::from("Copyright Protected.");
                        if fee == 0 {
                            url = "Downloadable".to_string();
                        }
                        vec.push(SongInfo {
                            id: v.get("id")?.as_u64()?,
                            name: v.get("name")?.as_str()?.to_owned(),
                            singer: v
                                .get("artists")?
                                .get(0)?
                                .get("name")
                                .unwrap_or(&json!("未知"))
                                .as_str()
                                .unwrap_or("未知")
                                .to_owned(),
                            album: v
                                .get("album")?
                                .get("name")
                                .unwrap_or(&json!("未知"))
                                .as_str()
                                .unwrap_or("未知")
                                .to_owned(),
                            pic_url: pic_id.to_string(),
                            duration: format!(
                                "{:0>2}:{:0>2}",
                                duration / 1000 / 60,
                                duration / 1000 % 60
                            ),
                            song_url: url,
                        });
                    }
                }
                Parse::ALBUM => {
                    let array = value.get("songs")?.as_array()?;
                    for v in array.iter() {
                        let duration = v.get("dt")?.as_u64()? as u32;
                        vec.push(SongInfo {
                            id: v.get("id")?.as_u64()?,
                            name: v.get("name")?.as_str()?.to_owned(),
                            singer: v
                                .get("ar")?
                                .get(0)?
                                .get("name")
                                .unwrap_or(&json!("未知"))
                                .as_str()
                                .unwrap_or("未知")
                                .to_owned(),
                            album: value
                                .get("album")?
                                .get("name")
                                .unwrap_or(&json!("未知"))
                                .as_str()
                                .unwrap_or("未知")
                                .to_owned(),
                            pic_url: value
                                .get("album")?
                                .get("picUrl")
                                .unwrap_or(&json!(""))
                                .as_str()
                                .unwrap_or("")
                                .to_owned(),
                            duration: format!(
                                "{:0>2}:{:0>2}",
                                duration / 1000 / 60,
                                duration / 1000 % 60
                            ),
                            song_url: String::new(),
                        });
                    }
                }
                _ => {}
            }
            let mut song_tags: Vec<SongTag> = Vec::new();
            for v in vec.iter() {
                let song_tag = SongTag {
                    artist: Some(v.singer.to_owned()),
                    title: Some(v.name.to_owned()),
                    album: Some(v.album.to_owned()),
                    lang_ext: Some(String::from("zh_CN")),
                    lyric_id: Some(v.id.to_string()),
                    song_id: Some(v.id.to_string()),
                    service_provider: Some(SongtagProvider::Netease),
                    url: Some(v.song_url.to_owned()),
                    pic_id: Some(v.pic_url.to_owned()),
                    album_id: Some(v.pic_url.to_owned()),
                };
                song_tags.push(song_tag);
            }
            return Some(song_tags);
        }
    }
    None
}

// 歌单信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SongList {
    // 歌单 id
    pub id: u64,
    // 歌单名
    pub name: String,
    // 歌单封面
    pub cover_img_url: String,
}

// parse: 解析方式
#[allow(unused)]
pub fn to_song_list(json: &str, parse: Parse) -> Option<Vec<SongList>> {
    if let Ok(value) = serde_json::from_str::<Value>(json) {
        if value.get("code")?.eq(&200) {
            let mut vec: Vec<SongList> = Vec::new();
            match parse {
                Parse::USL => {
                    let array = value.get("playlist")?.as_array()?;
                    for v in array.iter() {
                        vec.push(SongList {
                            id: v.get("id")?.as_u64()?,
                            name: v.get("name")?.as_str()?.to_owned(),
                            cover_img_url: v.get("coverImgUrl")?.as_str()?.to_owned(),
                        });
                    }
                }
                Parse::RMD => {
                    let array = value.get("recommend")?.as_array()?;
                    for v in array.iter() {
                        vec.push(SongList {
                            id: v.get("id")?.as_u64()?,
                            name: v.get("name")?.as_str()?.to_owned(),
                            cover_img_url: v
                                .get("picUrl")
                                .unwrap_or(&json!(""))
                                .as_str()
                                .unwrap_or("")
                                .to_owned(),
                        });
                    }
                }
                Parse::ALBUM => {
                    let array = value.get("albums")?.as_array()?;
                    for v in array.iter() {
                        vec.push(SongList {
                            id: v.get("id")?.as_u64()?,
                            name: v.get("name")?.as_str()?.to_owned(),
                            cover_img_url: v
                                .get("picUrl")
                                .unwrap_or(&json!(""))
                                .as_str()
                                .unwrap_or("")
                                .to_owned(),
                        });
                    }
                }
                Parse::TOP => {
                    let array = value.get("playlists")?.as_array()?;
                    for v in array.iter() {
                        vec.push(SongList {
                            id: v.get("id")?.as_u64()?,
                            name: v.get("name")?.as_str()?.to_owned(),
                            cover_img_url: v.get("coverImgUrl")?.as_str()?.to_owned(),
                        });
                    }
                }
                _ => {}
            }
            return Some(vec);
        }
    }
    None
}

// 消息
#[derive(Debug, Deserialize, Serialize)]
pub struct Msg {
    pub code: i32,
    pub msg: String,
}

#[allow(unused)]
pub fn to_msg(json: &str) -> Option<Msg> {
    if let Ok(value) = serde_json::from_str::<Value>(json) {
        let code = value.get("code")?.as_i64()? as i32;
        if code.eq(&200) {
            return Some(Msg {
                code: 200,
                msg: "".to_owned(),
            });
        }
        let msg = value.get("msg")?.as_str()?.to_owned();
        return Some(Msg { code, msg });
    }
    None
}

// 登陆信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginInfo {
    // 登陆状态码
    pub code: i32,
    // 用户 id
    pub uid: u64,
    // 用户昵称
    pub nickname: String,
    // 用户头像
    pub avatar_url: String,
    // 状态消息
    pub msg: String,
}

#[allow(unused)]
pub fn to_login_info(json: &str) -> Option<LoginInfo> {
    if let Ok(value) = serde_json::from_str::<Value>(json) {
        let code = value.get("code")?.as_i64()? as i32;
        if code.eq(&200) {
            let profile = value.get("profile")?.as_object()?;
            return Some(LoginInfo {
                code,
                uid: profile.get("userId")?.as_u64()?,
                nickname: profile.get("nickname")?.as_str()?.to_owned(),
                avatar_url: profile.get("avatarUrl")?.as_str()?.to_owned(),
                msg: "".to_owned(),
            });
        }
        let msg = value.get("msg")?.as_str()?.to_owned();
        return Some(LoginInfo {
            code,
            uid: 0,
            nickname: "".to_owned(),
            avatar_url: "".to_owned(),
            msg,
        });
    }
    None
}

// 请求方式
#[allow(unused, clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum Method {
    POST,
    GET,
}

// 解析方式
// USL: 用户
// UCD: 云盘
// RMD: 推荐
// RMDS: 推荐歌曲
// SEARCH: 搜索
// SD: 单曲详情
// ALBUM: 专辑
// TOP: 热门
#[allow(unused, clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub enum Parse {
    USL,
    UCD,
    RMD,
    RMDS,
    SEARCH,
    SD,
    ALBUM,
    TOP,
}
