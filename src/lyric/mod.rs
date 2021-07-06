pub mod lrc;
mod netease;
use crate::song::Song;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::thread;
use ytd_rs::{Arg, ResultType, YoutubeDL};

#[derive(Deserialize, Serialize)]
pub struct SongTag {
    pub artist: Vec<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub lang_ext: Option<String>,
    pub service_provider: Option<String>,
    pub song_id: Option<String>,
    pub lyric_id: Option<String>,
    pub url: Option<String>,
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

impl Song {
    pub fn lyric_options(&self) -> Result<Vec<SongTag>> {
        // let service_provider = "netease";
        // let mut results = self.get_lyric_options(service_provider)?;
        let mut search_str: String = self.title.clone().unwrap();
        search_str += " ";
        search_str += self.artist.clone().as_ref().unwrap();
        if search_str.len() < 3 {
            if let Some(file) = self.file.as_ref() {
                let p: &Path = Path::new(file.as_str());
                search_str = String::from(p.file_stem().unwrap().to_str().unwrap());
            }
        }

        let mut netease_api = netease::MusicApi::new();
        let results = netease_api.search(search_str, 1, 0, 30)?;
        let mut results: Vec<SongTag> = serde_json::from_str(&results)?;

        let service_provider = "kugou";
        let results2 = self.get_lyric_options(service_provider)?;

        results.extend(results2);

        Ok(results)
    }

    pub(super) fn get_lyric_options(&self, service_provider: &str) -> Result<Vec<SongTag>> {
        let mut search_str: String = self.title.clone().unwrap();
        search_str += " ";
        search_str += self.artist.clone().as_ref().unwrap();
        if search_str.len() < 3 {
            if let Some(file) = self.file.as_ref() {
                let p: &Path = Path::new(file.as_str());
                search_str = String::from(p.file_stem().unwrap().to_str().unwrap());
            }
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
                        url: Some(v.url_id.clone()),
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
                        url: Some(v.url_id.to_string()),
                    };
                    result_tags.push(song_tag);
                }
            }
            &_ => {}
        }

        Ok(result_tags)
    }
}

impl SongTag {
    pub fn fetch_lyric(&self) -> Result<String> {
        let mut tag_lyric: TagLyric = TagLyric {
            lyric: String::new(),
            tlyric: String::new(),
        };

        match self.service_provider.as_ref().unwrap().as_str() {
            "kugou" => {
                let url_search = "http://api.sunyj.xyz/?";
                let client = reqwest::blocking::Client::new();

                let resp = client
                    .get(url_search)
                    .query(&[
                        ("site", self.service_provider.as_ref()),
                        ("lyric", self.lyric_id.as_ref()),
                    ])
                    .send()?;

                // println!("{:?}", resp);
                if resp.status() != 200 {
                    return Err(anyhow!("Network error?"));
                }

                tag_lyric = resp.json::<TagLyric>()?;
            }
            "netease" => {
                let mut netease_api = netease::MusicApi::new();
                if let Some(lyric_id) = self.lyric_id.clone() {
                    let lyric = netease_api.song_lyric(lyric_id)?;
                    tag_lyric.lyric = lyric;
                }
            }
            &_ => {}
        }
        Ok(tag_lyric.lyric)
    }

    pub fn download(&self, file: &str) -> Result<()> {
        let p: &Path = Path::new(file);
        let p_parent = String::from(p.parent().unwrap().to_string_lossy());

        match self.service_provider.as_ref().unwrap().as_str() {
            "netease" => {
                let mut netease_api = netease::MusicApi::new();
                if let Some(song_id) = self.song_id.clone() {
                    if let Ok(song_id_u64) = song_id.parse::<u64>() {
                        let result = netease_api.songs_url(&[song_id_u64])?;
                        if result.is_empty() {
                            return Ok(());
                        }

                        let mut artists: String = String::from("");

                        for a in self.artist.iter() {
                            artists += a;
                        }

                        let title = self
                            .title
                            .clone()
                            .unwrap_or_else(|| String::from("Unknown Title"));

                        let filename = format!("{}-{}.mp3", artists, title);

                        let args = vec![
                            // Arg::new("--quiet"),
                            Arg::new("--extract-audio"),
                            Arg::new_with_arg("--audio-format", "mp3"),
                            // Arg::new("--add-metadata"),
                            // Arg::new("--embed-thumbnail"),
                            // Arg::new_with_arg("--metadata-from-title", "%(artist) - %(title)s"),
                            // Arg::new("--write-sub"),
                            // Arg::new("--all-subs"),
                            // Arg::new_with_arg("--convert-subs", "lrc"),
                            // Arg::new_with_arg("--output", "%(title).90s.%(ext)s"),
                            // Arg::new_with_arg("--output", &filename),
                        ];

                        if let Ok(ytd) =
                            YoutubeDL::new(p_parent.as_ref(), args, result[0].url.as_ref())
                        {
                            // let tx = self.sender.clone();
                            thread::spawn(move || {
                                // tx.send(super::TransferState::Running).unwrap();
                                // start download
                                let download = ytd.download();

                                // check what the result is and print out the path to the download or the error
                                match download.result_type() {
                                    ResultType::SUCCESS => {
                                        // println!("Your download: {}", download.output_dir().to_string_lossy())
                                        // tx.send(super::TransferState::Completed).unwrap();
                                    }
                                    ResultType::IOERROR | ResultType::FAILURE => {
                                        // println!("Couldn't start download: {}", download.output())
                                        // tx.send(super::TransferState::ErrDownload).unwrap();
                                    }
                                };
                            });
                        }
                    }
                }
            }
            "kugou" => {}
            &_ => {}
        }
        // if let Some(url) = self.url.clone() {
        //     if let Ok(ytd) = YoutubeDL::new(p_parent.as_ref(), args, url.as_ref()) {
        //         // let tx = self.sender.clone();
        //         thread::spawn(move || {
        //             // tx.send(super::TransferState::Running).unwrap();
        //             // start download
        //             let download = ytd.download();

        //             // check what the result is and print out the path to the download or the error
        //             match download.result_type() {
        //                 ResultType::SUCCESS => {
        //                     // println!("Your download: {}", download.output_dir().to_string_lossy())
        //                     // tx.send(super::TransferState::Completed).unwrap();
        //                 }
        //                 ResultType::IOERROR | ResultType::FAILURE => {
        //                     // println!("Couldn't start download: {}", download.output())
        //                     // tx.send(super::TransferState::ErrDownload).unwrap();
        //                 }
        //             };
        //         });
        //     }
        // }
        Ok(())
    }
}

impl fmt::Display for SongTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "{}-{}", self.file, self.file,)

        let mut artists: String = String::from("");

        for a in self.artist.iter() {
            artists += a;
        }

        let title = self
            .title
            .clone()
            .unwrap_or_else(|| String::from("Unknown Title"));
        let album = self
            .album
            .clone()
            .unwrap_or_else(|| String::from("Unknown Album"));
        let service_provider = self
            .service_provider
            .clone()
            .unwrap_or_else(|| String::from("unknown source"));
        let url = self.url.clone().unwrap_or_else(|| String::from("No url"));

        write!(
            f,
            "{:.12}《{:.12}》{:.10} {:.7} {:.10}",
            artists, title, album, service_provider, url
        )
    }
}
