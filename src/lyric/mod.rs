pub mod lrc;

use crate::song::Song;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::fmt;
use std::path::Path;

pub struct SongTag {
    pub artist: Vec<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub lang_ext: Option<String>,
    pub service_provider: Option<String>,
    pub song_id: Option<String>,
    pub lyric_id: Option<String>,
}

// TagNetease is the tag get from netease
#[derive(Deserialize)]
#[allow(dead_code)]
struct TagNetease {
    album: String,
    artist: Vec<String>,
    id: i64,
    lyric_id: i64,
    name: String,
    pic_id: String,
    source: String,
    url_id: i64,
}

// TagKugou is the tag get from kugou
#[derive(Deserialize)]
#[allow(dead_code)]
struct TagKugou {
    album: String,
    artist: Vec<String>,
    id: String,
    lyric_id: String,
    name: String,
    pic_id: String,
    source: String,
    url_id: String,
}

// TagLyric is the lyric json get from both netease and kugou
#[derive(Deserialize)]
#[allow(dead_code)]
struct TagLyric {
    lyric: String,
    tlyric: String,
}

pub fn lyric_options(song: &Song) -> Result<Vec<SongTag>> {
    let service_provider = "netease";
    let mut results = get_lyric_options(song, service_provider)?;
    let service_provider = "kugou";
    let results2 = get_lyric_options(song, service_provider)?;

    results.extend(results2);

    Ok(results)
}

pub(super) fn get_lyric_options(song: &Song, service_provider: &str) -> Result<Vec<SongTag>> {
    let mut search_str: String = song.title.clone().unwrap();
    search_str += " ";
    search_str += song.artist.clone().as_ref().unwrap();
    if search_str.len() < 3 {
        let p: &Path = Path::new(&song.file);
        search_str = String::from(p.file_stem().unwrap().to_str().unwrap());
    }

    let url_search = "http://api.sunyj.xyz/?";
    let client = reqwest::blocking::Client::new();

    let resp = client
        .get(url_search)
        .query(&[("site", service_provider), ("search", search_str.as_ref())])
        .send()?;

    if resp.status() != 200 {
        return Err(anyhow!("Network error?"));
    }

    // println!("{:?}", resp);
    let mut result_tags: Vec<SongTag> = vec![];

    match service_provider {
        "kugou" => {
            let tag_kugou: Vec<TagKugou> = resp.json::<Vec<TagKugou>>()?;
            for v in tag_kugou.iter() {
                let song_tag: SongTag = SongTag {
                    artist: v.artist.clone(),
                    title: Some(v.name.clone()),
                    album: Some(v.album.clone()),
                    lang_ext: Some(String::from("chi")),
                    service_provider: Some(String::from("kugou")),
                    song_id: Some(v.id.clone()),
                    lyric_id: Some(v.lyric_id.clone()),
                };
                result_tags.push(song_tag);
            }
        }
        "netease" => {
            let tag_netease: Vec<TagNetease> = resp.json::<Vec<TagNetease>>()?;
            for v in tag_netease.iter() {
                let song_tag: SongTag = SongTag {
                    artist: v.artist.clone(),
                    title: Some(v.name.clone()),
                    album: Some(v.album.clone()),
                    lang_ext: Some(String::from("chi")),
                    service_provider: Some(String::from("kugou")),
                    song_id: Some(format!("{}", v.id)),
                    lyric_id: Some(format!("{}", v.lyric_id)),
                };
                result_tags.push(song_tag);
            }
        }
        &_ => {}
    }

    Ok(result_tags)
}
pub fn fetch_lyric(song_tag: &SongTag) -> Result<String> {
    let url_search = "http://api.sunyj.xyz/?";
    let client = reqwest::blocking::Client::new();

    let resp = client
        .get(url_search)
        .query(&[
            ("site", &song_tag.service_provider),
            ("lyric", &song_tag.lyric_id),
        ])
        .send()?;

    // println!("{:?}", resp);
    if resp.status() != 200 {
        return Err(anyhow!("Network error?"));
    }

    let tag_lyric = resp.json::<TagLyric>()?;
    Ok(tag_lyric.lyric)
}

impl fmt::Display for SongTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "{}-{}", self.file, self.file,)

        let mut artists: String = String::from("");

        for a in self.artist.iter() {
            artists += a;
        }

        let title = self.title.clone().unwrap_or(String::from("Unknown Title"));
        let album = self.album.clone().unwrap_or(String::from("Unknown Album"));

        write!(f, "{:.12}《{:.12}》{:.10}", artists, title, album,)
    }
}
