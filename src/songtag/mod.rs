mod kugou;
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
pub mod lrc;
mod migu;
mod netease;
use crate::song::Song;
use crate::ui::activity::main::TransferState;
use anyhow::{anyhow, bail, Result};
use id3::frame::Lyrics;
use id3::frame::{Picture, PictureType};
use id3::{Tag, Version};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use ytd_rs::{Arg, ResultType, YoutubeDL};

#[derive(Deserialize, Serialize)]
pub struct SongTag {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub lang_ext: Option<String>,
    pub service_provider: Option<String>,
    pub song_id: Option<String>,
    pub lyric_id: Option<String>,
    pub url: Option<String>,
    pub pic_id: Option<String>,
    pub album_id: Option<String>,
}

impl Song {
    // Search function of 3 servers. Run parallel to get results faster.
    pub fn lyric_options(&self) -> Result<Vec<SongTag>> {
        let mut search_str = String::new();
        if let Some(title) = &self.title {
            search_str = title.to_string();
        }
        search_str += " ";

        if let Some(artist) = &self.artist {
            search_str += artist;
        }

        if search_str.len() < 3 {
            if let Some(file) = &self.file {
                let p: &Path = Path::new(file.as_str());
                if let Some(stem) = p.file_stem() {
                    search_str = stem.to_string_lossy().to_string();
                }
            }
        }

        let mut results: Vec<SongTag> = Vec::new();
        let (tx, rx): (Sender<Vec<SongTag>>, Receiver<Vec<SongTag>>) = mpsc::channel();

        let tx1 = tx.clone();
        let search_str_netease = search_str.clone();
        let handle_netease = thread::spawn(move || -> Result<()> {
            let mut netease_api = netease::NeteaseApi::new();
            if let Ok(results) = netease_api.search(&search_str_netease, 1, 0, 30) {
                let result2: Vec<SongTag> = serde_json::from_str(&results)?;
                if tx1.send(result2).is_ok() {}
            }
            Ok(())
        });

        let tx2 = tx.clone();
        let search_str_migu = search_str.clone();
        let handle_migu = thread::spawn(move || -> Result<()> {
            let mut migu_api = migu::MiguApi::new();
            if let Ok(results) = migu_api.search(&search_str_migu, 1, 0, 30) {
                let results2: Vec<SongTag> = serde_json::from_str(&results)?;
                if tx2.send(results2).is_ok() {}
            }
            Ok(())
        });

        let mut kugou_api = kugou::KugouApi::new();
        let handle_kugou = thread::spawn(move || -> Result<()> {
            if let Ok(r) = kugou_api.search(&search_str, 1, 0, 30) {
                let results2: Vec<SongTag> = serde_json::from_str(&r)?;
                if tx.send(results2).is_ok() {}
            }
            Ok(())
        });

        if handle_netease.join().is_ok() {
            if let Ok(result2) = rx.try_recv() {
                results.extend(result2);
            }
        }

        if handle_migu.join().is_ok() {
            if let Ok(result2) = rx.try_recv() {
                results.extend(result2);
            }
        }

        if handle_kugou.join().is_ok() {
            if let Ok(result2) = rx.try_recv() {
                results.extend(result2);
            }
        }

        Ok(results)
    }
}

impl SongTag {
    // get lyric by lyric_id
    pub fn fetch_lyric(&self) -> Result<String> {
        let mut lyric_string = String::new();

        if let Some(service_provider) = &self.service_provider {
            match service_provider.as_str() {
                "kugou" => {
                    let mut kugou_api = kugou::KugouApi::new();
                    if let Some(lyric_id) = &self.lyric_id {
                        lyric_string = kugou_api.song_lyric(lyric_id)?;
                    }
                }
                "netease" => {
                    let mut netease_api = netease::NeteaseApi::new();
                    if let Some(lyric_id) = &self.lyric_id {
                        lyric_string = netease_api.song_lyric(lyric_id)?;
                    }
                }
                "migu" => {
                    let mut migu_api = migu::MiguApi::new();
                    if let Some(lyric_id) = &self.lyric_id {
                        lyric_string = migu_api.song_lyric(lyric_id)?;
                    }
                }
                &_ => {}
            }
        }

        Ok(lyric_string)
    }

    // get photo by pic_id(kugou/netease) or song_id(migu)
    pub fn fetch_photo(&self) -> Result<Picture> {
        let mut encoded_image_bytes: Vec<u8> = Vec::new();

        if let Some(service_provider) = &self.service_provider {
            match service_provider.as_str() {
                "kugou" => {
                    let mut kugou_api = kugou::KugouApi::new();
                    if let Some(p) = self.pic_id.to_owned() {
                        if let Some(album_id) = self.album_id.to_owned() {
                            encoded_image_bytes = kugou_api.pic(p, album_id)?;
                        }
                    }
                }

                "netease" => {
                    let mut netease_api = netease::NeteaseApi::new();
                    if let Some(p) = &self.pic_id {
                        encoded_image_bytes = netease_api.pic(p.as_str())?;
                    }
                }

                "migu" => {
                    let mut migu_api = migu::MiguApi::new();
                    if let Some(p) = &self.song_id {
                        encoded_image_bytes = migu_api.pic(p.as_str())?;
                    }
                }

                &_ => {}
            }
        }
        Ok(Picture {
            mime_type: "image/jpeg".to_string(),
            picture_type: PictureType::Other,
            description: "some image".to_string(),
            data: encoded_image_bytes,
        })
    }

    pub fn download(&self, file: &str, tx_tageditor: Sender<TransferState>) -> Result<()> {
        let p: &Path = Path::new(file);
        let p_parent = PathBuf::from(p.parent().unwrap_or_else(|| Path::new("/tmp")));
        let song_id = self
            .song_id
            .as_ref()
            .ok_or_else(|| anyhow!("error downloading because no song id is found"))?;
        let artist = self
            .artist
            .clone()
            .unwrap_or_else(|| String::from("Unknown Artist"));
        let title = self
            .title
            .clone()
            .unwrap_or_else(|| String::from("Unknown Title"));

        let album = self.album.clone().unwrap_or_else(|| String::from("N/A"));
        let lyric = self.fetch_lyric();
        let photo = self.fetch_photo();
        let album_id = self.album_id.clone().unwrap_or_else(|| String::from("N/A"));

        let filename = format!("{}-{}.%(ext)s", artist, title);

        let args = vec![
            Arg::new("--quiet"),
            Arg::new_with_arg("--output", filename.as_ref()),
            Arg::new("--extract-audio"),
            Arg::new_with_arg("--audio-format", "mp3"),
        ];

        let p_full = format!(
            "{}/{}-{}.mp3",
            p_parent.to_str().unwrap_or("/tmp"),
            artist,
            title
        );
        if std::fs::remove_file(Path::new(p_full.as_str())).is_err() {}

        let mp3_url = self.url.clone().unwrap_or_else(|| String::from("N/A"));
        if mp3_url.starts_with("Copyright") {
            bail!("Copyright protected, please select another item.");
        }
        let mut url = mp3_url;

        if let Some(service_provider) = &self.service_provider {
            match service_provider.as_str() {
                "netease" => {
                    let song_id_u64 = song_id.parse::<u64>()?;

                    let mut netease_api = netease::NeteaseApi::new();
                    let result = netease_api.songs_url(&[song_id_u64])?;
                    if result.is_empty() {
                        bail!("no url list found");
                    }

                    let r = result.get(0).ok_or_else(|| anyhow!("no url list found"))?;
                    url = r.url.clone();
                }
                "migu" => {}
                "kugou" => {
                    let mut kugou_api = kugou::KugouApi::new();
                    url = kugou_api.song_url(song_id.to_string(), album_id)?;
                }
                &_ => {}
            }
        }

        if url.is_empty() {
            bail!("url fetch failed, please try another item.");
        }

        let ytd = YoutubeDL::new(&p_parent, args, &url)?;

        let tx = tx_tageditor;
        thread::spawn(move || -> Result<()> {
            tx.send(TransferState::Running)?;
            // start download
            let download = ytd.download();

            // check what the result is and print out the path to the download or the error
            match download.result_type() {
                ResultType::SUCCESS => {
                    let mut tag_song = Tag::new();
                    tag_song.set_album(album);
                    tag_song.set_title(title);
                    tag_song.set_artist(artist);
                    if let Ok(l) = lyric {
                        tag_song.add_lyrics(Lyrics {
                            lang: String::from("chi"),
                            description: String::from("saved by termusic."),
                            text: l,
                        });
                    }

                    if let Ok(p) = photo {
                        tag_song.add_picture(p);
                    }

                    let file = p_full.clone();
                    if tag_song.write_to_path(p_full, Version::Id3v24).is_ok() {
                        tx.send(TransferState::Completed(Some(file)))?;
                    } else {
                        tx.send(TransferState::ErrEmbedData)?;
                    }
                }
                ResultType::IOERROR | ResultType::FAILURE => {
                    tx.send(TransferState::ErrDownload)?;
                    bail!("Error downloading, please retry!");
                }
            };
            Ok(())
        });
        Ok(())
    }
}
