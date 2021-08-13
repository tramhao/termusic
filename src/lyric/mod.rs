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
mod kugou;
pub mod lrc;
mod migu;
mod netease;
use crate::song::Song;
use crate::ui::activity::main::TransferState;
use anyhow::{anyhow, Result};
use id3::frame::Lyrics;
use id3::frame::{Picture, PictureType};
use id3::{Tag, Version};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::thread;
use unicode_truncate::{Alignment, UnicodeTruncateStr};
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
    pub fn lyric_options(&self) -> Result<Vec<SongTag>> {
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

        let mut migu_api = migu::MiguApi::new();
        if let Ok(r) = migu_api.search(search_str.as_str(), 1, 0, 30) {
            let results2: Vec<SongTag> = serde_json::from_str(&r)?;
            results.extend(results2);
        }

        let mut kugou_api = kugou::KugouApi::new();
        if let Ok(r) = kugou_api.search(search_str, 1, 0, 30) {
            let results2: Vec<SongTag> = serde_json::from_str(&r)?;
            results.extend(results2);
        }

        Ok(results)
    }
}

impl SongTag {
    pub fn fetch_lyric(&self) -> Result<String> {
        let mut lyric_string = String::new();

        match self.service_provider.as_ref().unwrap().as_str() {
            "kugou" => {
                let mut kugou_api = kugou::KugouApi::new();
                if let Some(lyric_id) = self.lyric_id.clone() {
                    let lyric = kugou_api.song_lyric(lyric_id)?;
                    lyric_string = lyric;
                }
            }

            "netease" => {
                let mut netease_api = netease::NeteaseApi::new();
                if let Some(lyric_id) = self.lyric_id.clone() {
                    let lyric = netease_api.song_lyric(lyric_id)?;
                    lyric_string = lyric;
                }
            }

            "migu" => {
                let mut migu_api = migu::MiguApi::new();
                if let Some(lyric_id) = self.lyric_id.clone() {
                    let lyric = migu_api.song_lyric(lyric_id)?;
                    lyric_string = lyric;
                }
            }

            &_ => {}
        }
        Ok(lyric_string)
    }

    pub fn download(&self, file: &str, tx_tageditor: Sender<TransferState>) -> Result<()> {
        let p: &Path = Path::new(file);
        let p_parent = PathBuf::from(p.parent().unwrap());
        let song_id = self
            .song_id
            .clone()
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
        let lyric = self.fetch_lyric()?;
        let album_id = self.album_id.clone().unwrap_or_else(|| String::from("N/A"));

        let filename = format!("{}-{}.%(ext)s", artist, title);
        let pic_id = self.pic_id.clone().unwrap();

        let args = vec![
            Arg::new("--quiet"),
            Arg::new_with_arg("--output", filename.as_ref()),
            Arg::new("--extract-audio"),
            Arg::new_with_arg("--audio-format", "mp3"),
        ];

        let p_full = format!("{}/{}-{}.mp3", p_parent.to_str().unwrap(), artist, title);
        if std::fs::remove_file(Path::new(p_full.as_str())).is_err() {}

        match self.service_provider.as_ref().unwrap().as_str() {
            "netease" => {
                let mp3_url = self.url.clone().unwrap_or_else(|| String::from("N/A"));
                if mp3_url.starts_with("Copyright") {
                    return Err(anyhow!("Copyright protection, please select another item."));
                }

                let song_id_u64 = song_id.parse::<u64>()?;

                let mut netease_api = netease::NeteaseApi::new();
                let result = netease_api.songs_url(&[song_id_u64])?;
                if result.is_empty() {
                    return Err(anyhow!("Copyright protected, please try another item!"));
                }

                let ytd =
                    YoutubeDL::new(&p_parent, args, result.get(0).unwrap().url.as_ref()).unwrap();

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
                            tag_song.set_title(title.clone());
                            tag_song.set_artist(artist.clone());
                            tag_song.add_lyrics(Lyrics {
                                lang: String::from("chi"),
                                description: String::from("saved by termusic."),
                                text: lyric,
                            });
                            let encoded_image_bytes = match netease_api.pic(pic_id.as_str()) {
                                Ok(image) => image,
                                Err(_) => {
                                    tx.send(TransferState::ErrEmbedData)?;
                                    return Ok(());
                                }
                            };

                            tag_song.add_picture(Picture {
                                mime_type: "image/jpeg".to_string(),
                                picture_type: PictureType::Other,
                                description: "some image".to_string(),
                                data: encoded_image_bytes,
                            });

                            if tag_song.write_to_path(p_full, Version::Id3v24).is_ok() {
                                tx.send(TransferState::Completed)?;
                            } else {
                                tx.send(TransferState::ErrEmbedData)?;
                            }
                        }
                        ResultType::IOERROR | ResultType::FAILURE => {
                            tx.send(TransferState::ErrDownload)?;
                            return Err(anyhow!("Error downloading, please retry!"));
                        }
                    };
                    Ok(())
                });
            }
            "migu" => {
                let mp3_url = self.url.clone().unwrap_or_else(|| String::from("N/A"));
                if !mp3_url.starts_with("http") {
                    return Err(anyhow!("Copyright protection, please select another item."));
                }
                let ytd = YoutubeDL::new(&p_parent, args, &mp3_url).unwrap();

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
                            tag_song.set_title(title.clone());
                            tag_song.set_artist(artist.clone());
                            tag_song.add_lyrics(Lyrics {
                                lang: String::from("chi"),
                                description: String::from("saved by termusic."),
                                text: lyric,
                            });

                            let mut migu_api = migu::MiguApi::new();
                            let encoded_image_bytes = match migu_api.pic(song_id.as_str()) {
                                Ok(image) => image,
                                Err(_) => {
                                    tx.send(TransferState::ErrEmbedData)?;
                                    return Ok(());
                                }
                            };

                            tag_song.add_picture(Picture {
                                mime_type: "image/jpeg".to_string(),
                                picture_type: PictureType::Other,
                                description: "some image".to_string(),
                                data: encoded_image_bytes,
                            });

                            let p_full =
                                format!("{}/{}-{}.mp3", p_parent.to_str().unwrap(), artist, title);
                            if tag_song.write_to_path(p_full, Version::Id3v24).is_ok() {
                                tx.send(TransferState::Completed)?;
                            } else {
                                tx.send(TransferState::ErrEmbedData)?;
                            }
                        }
                        ResultType::IOERROR | ResultType::FAILURE => {
                            tx.send(TransferState::ErrDownload)?;
                            return Err(anyhow!("Error downloading, please retry!"));
                        }
                    };
                    Ok(())
                });
            }
            "kugou" => {
                let mp3_url = self.url.clone().unwrap_or_else(|| String::from("N/A"));
                if mp3_url.starts_with("Copyright") {
                    return Err(anyhow!("Copyright protection, please select another item."));
                }

                let mut kugou_api = kugou::KugouApi::new();
                let url = kugou_api.song_url(song_id.clone(), album_id.clone())?;
                if url.is_empty() {
                    return Err(anyhow!("url fetch failed, please try another item."));
                }
                let ytd = YoutubeDL::new(&p_parent, args, &url).unwrap();

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
                            tag_song.set_title(title.clone());
                            tag_song.set_artist(artist.clone());
                            tag_song.add_lyrics(Lyrics {
                                lang: String::from("chi"),
                                description: String::from("saved by termusic."),
                                text: lyric,
                            });

                            let encoded_image_bytes =
                                match kugou_api.pic(song_id.clone(), album_id.clone()) {
                                    Ok(image) => image,
                                    Err(_) => {
                                        tx.send(TransferState::ErrEmbedData)?;
                                        return Ok(());
                                    }
                                };

                            tag_song.add_picture(Picture {
                                mime_type: "image/jpeg".to_string(),
                                picture_type: PictureType::Other,
                                description: "some image".to_string(),
                                data: encoded_image_bytes,
                            });

                            let p_full =
                                format!("{}/{}-{}.mp3", p_parent.to_str().unwrap(), artist, title);
                            if tag_song.write_to_path(p_full, Version::Id3v24).is_ok() {
                                tx.send(TransferState::Completed)?;
                            } else {
                                tx.send(TransferState::ErrEmbedData)?;
                            }
                        }
                        ResultType::IOERROR | ResultType::FAILURE => {
                            tx.send(TransferState::ErrDownload)?;
                            return Err(anyhow!("Error downloading, please retry!"));
                        }
                    };
                    Ok(())
                });
            }
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
        // let album_id = self
        //     .album_id
        //     .clone()
        //     .unwrap_or_else(|| String::from("No AlbumId"));
        let mut url = self.url.clone().unwrap_or_else(|| String::from("No url"));
        if url.starts_with("http") {
            url = "Downloadable".to_string();
        }

        write!(
            f,
            "{} {} {} {} {}",
            artists_truncated, title_truncated, album_truncated, service_provider_truncated, url,
        )
    }
}
