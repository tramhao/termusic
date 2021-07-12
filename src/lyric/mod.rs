mod kugou;
pub mod lrc;
mod netease;
use crate::song::Song;
use anyhow::{anyhow, Result};
use id3::frame::Lyrics;
use id3::frame::{Picture, PictureType};
use id3::{Tag, Version};
use netease::encrypt::Crypto;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::thread;
use unicode_truncate::{Alignment, UnicodeTruncateStr};
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
    pub pic_id: Option<String>,
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

        let mut netease_api = netease::NeteaseApi::new();
        let results = netease_api.search(search_str.clone(), 1, 0, 30)?;
        let mut results: Vec<SongTag> = serde_json::from_str(&results)?;

        // let service_provider = "kugou";
        let mut kugou_api = kugou::KugouApi::new();
        let results2 = kugou_api.search(search_str, 1, 0, 30)?; //self.get_lyric_options(service_provider)?;
        let results2: Vec<SongTag> = serde_json::from_str(&results2)?;

        results.extend(results2);

        Ok(results)
    }

    #[allow(dead_code)]
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
                        pic_id: Some(v.pic_id.clone()),
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
                        pic_id: Some(v.pic_id.clone()),
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
                let mut netease_api = netease::NeteaseApi::new();
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
                let mut netease_api = netease::NeteaseApi::new();
                let song_id = self
                    .song_id
                    .clone()
                    .ok_or_else(|| anyhow!("error downloading because no song id is found"))?;
                let song_id_u64 = song_id.parse::<u64>()?;
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
                let album = self.album.clone().unwrap_or_else(|| String::from("N/A"));
                let lyric = self.fetch_lyric()?;

                let filename = format!("{}-{}.%(ext)s", artists, title);
                let pic_id = self.pic_id.clone().unwrap();

                let args = vec![
                    Arg::new("--quiet"),
                    Arg::new_with_arg("--output", filename.as_ref()),
                    Arg::new("--extract-audio"),
                    Arg::new_with_arg("--audio-format", "mp3"),
                ];

                let ytd =
                    YoutubeDL::new(p_parent.as_ref(), args, result.get(0).unwrap().url.as_ref())
                        .unwrap();

                // let tx = self.sender.clone();
                thread::spawn(move || -> Result<()> {
                    // tx.send(super::TransferState::Running).unwrap();
                    // start download
                    let download = ytd.download();

                    // check what the result is and print out the path to the download or the error
                    match download.result_type() {
                        ResultType::SUCCESS => {
                            // println!("Your download: {}", download.output_dir().to_string_lossy())
                            // tx.send(super::TransferState::Completed).unwrap();
                            // println!("{}", download.output());
                            let mut tag_song = Tag::new();
                            tag_song.set_album(album);
                            tag_song.set_title(title.clone());
                            tag_song.set_artist(artists.clone());
                            let lyric_frame: Lyrics = Lyrics {
                                lang: String::from("chi"),
                                description: String::from("saved by termusic."),
                                text: lyric,
                            };
                            tag_song.add_lyrics(lyric_frame);

                            let id_encrypted = Crypto::encrypt_id(pic_id.clone());
                            // println!("{}", id_encrypted);
                            let mut url = String::from("https://p3.music.126.net/");
                            url.push_str(&id_encrypted);
                            url.push('/');
                            url.push_str(pic_id.as_str());
                            url.push_str(".jpg?param=300y300");

                            // let mut url = String::from(
                            //     "https://www.antarestec.com/music/?type=cover&id=",
                            // );
                            // url.push_str(&song_id);

                            let img_bytes = reqwest::blocking::get(url)?.bytes()?;

                            let image = image::load_from_memory(&img_bytes)?;
                            let mut encoded_image_bytes = Vec::new();
                            // Unwrap: Writing to a Vec should always succeed;
                            image
                                .write_to(
                                    &mut encoded_image_bytes,
                                    image::ImageOutputFormat::Jpeg(90),
                                )
                                .unwrap();
                            tag_song.add_picture(Picture {
                                mime_type: "image/jpeg".to_string(),
                                picture_type: PictureType::Other,
                                description: "some image".to_string(),
                                data: encoded_image_bytes,
                            });

                            let p_full = format!("{}/{}-{}.mp3", p_parent, artists, title);
                            if tag_song.write_to_path(p_full, Version::Id3v24).is_ok() {}
                        }
                        ResultType::IOERROR | ResultType::FAILURE => {
                            // println!("Couldn't start download: {}", download.output())
                            // tx.send(super::TransferState::ErrDownload).unwrap();
                            return Err(anyhow!("Error downloading, please retry!"));
                        }
                    };
                    Ok(())
                });
            }
            "kugou" => {}
            &_ => {}
        }
        Ok(())
    }
}

impl fmt::Display for SongTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut artists: String = String::from("");

        for a in self.artist.iter() {
            artists += a;
        }
        let artists_truncated = artists.unicode_pad(10, Alignment::Left, true);

        let title = self
            .title
            .clone()
            .unwrap_or_else(|| String::from("Unknown Title"));
        let title_truncated = title.unicode_pad(16, Alignment::Left, true);
        let album = self
            .album
            .clone()
            .unwrap_or_else(|| String::from("Unknown Album"));
        let album_truncated = album.unicode_pad(16, Alignment::Left, true);
        let service_provider = self
            .service_provider
            .clone()
            .unwrap_or_else(|| String::from("unknown source"));
        let service_provider_truncated = service_provider.unicode_pad(7, Alignment::Left, true);
        // let pic_id = self
        //     .pic_id
        //     .clone()
        //     .unwrap_or_else(|| String::from("No Pic url"));

        write!(
            f,
            "{} {} {} {}", // {}",
            artists_truncated,
            title_truncated,
            album_truncated,
            service_provider_truncated //, pic_id
        )
    }
}
