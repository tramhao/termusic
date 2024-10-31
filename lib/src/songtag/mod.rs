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
pub mod encrypt;
mod kugou;
pub mod lrc;
mod migu;
mod netease;

use crate::types::{DLMsg, Msg, SearchLyricState};
use crate::utils::get_parent_folder;
use anyhow::{anyhow, bail, Result};
use lofty::config::WriteOptions;
use lofty::id3::v2::{Frame, Id3v2Tag, UnsynchronizedTextFrame};
use lofty::picture::Picture;
use lofty::prelude::{Accessor, TagExt};
use lofty::TextEncoding;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::thread::{self, sleep};
use std::time::Duration;
use ytd_rs::{Arg, YoutubeDL};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct SongTag {
    artist: Option<String>,
    title: Option<String>,
    album: Option<String>,
    lang_ext: Option<String>,
    service_provider: Option<ServiceProvider>,
    song_id: Option<String>,
    lyric_id: Option<String>,
    url: Option<String>,
    pic_id: Option<String>,
    album_id: Option<String>,
    // genre: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy)]
pub enum ServiceProvider {
    Netease,
    Kugou,
    Migu,
}

impl std::fmt::Display for ServiceProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let service_provider = match self {
            Self::Netease => "Netease",
            Self::Kugou => "Kugou",
            Self::Migu => "Migu",
        };
        write!(f, "{service_provider}")
    }
}

// Search function of 3 servers. Run in parallel to get results faster.
pub async fn search(search_str: &str, tx_tageditor: Sender<SearchLyricState>) {
    let mut results: Vec<SongTag> = Vec::new();

    let handle_netease = async {
        let mut netease_api = netease::Api::new();
        netease_api
            .search(search_str, netease::SearchRequestType::Single, 0, 30)
            .await
    };

    let handle_migu = async {
        let migu_api = migu::Api::new();
        migu_api
            .search(search_str, migu::SearchRequestType::Song, 0, 30)
            .await
    };

    let handle_kugou = async {
        let kugou_api = kugou::Api::new();
        kugou_api
            .search(search_str, kugou::SearchRequestType::Song, 0, 30)
            .await
    };

    let (netease_res, migu_res, kugou_res) =
        futures::join!(handle_netease, handle_migu, handle_kugou);

    match netease_res {
        Ok(vec) => results.extend(vec),
        Err(err) => error!("Netease Error: {:#}", err),
    }

    match migu_res {
        Ok(vec) => results.extend(vec),
        Err(err) => error!("Migu Error: {:#}", err),
    }

    match kugou_res {
        Ok(vec) => results.extend(vec),
        Err(err) => error!("Kogou Error: {:#}", err),
    }

    tx_tageditor.send(SearchLyricState::Finish(results)).ok();
}

impl SongTag {
    pub fn artist(&self) -> Option<&str> {
        self.artist.as_deref()
        // match self.artist.as_ref() {
        //     Some(artist) => Some(artist),
        //     None => None,
        // }
    }

    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
        // match self.album.as_ref() {
        //     Some(album) => Some(album),
        //     None => None,
        // }
    }
    /// Optionally return the title of the song
    /// If `None` it wasn't able to read the tags
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
        // match self.title.as_ref() {
        //     Some(title) => Some(title),
        //     None => None,
        // }
    }

    pub fn lang_ext(&self) -> Option<&str> {
        self.lang_ext.as_deref()
        // match self.lang_ext.as_ref() {
        //     Some(lang_ext) => Some(lang_ext),
        //     None => None,
        // }
    }

    pub const fn service_provider(&self) -> Option<&ServiceProvider> {
        self.service_provider.as_ref()
        // match self.service_provider.as_ref() {
        //     Some(service_provider) => Some(service_provider),
        //     None => None,
        // }
    }

    pub fn url(&self) -> Option<String> {
        self.url.as_ref().map(std::string::ToString::to_string)
    }
    // get lyric by lyric_id
    pub async fn fetch_lyric(&self) -> Result<String> {
        let mut lyric_string = String::new();

        match self.service_provider {
            Some(ServiceProvider::Kugou) => {
                let kugou_api = kugou::Api::new();
                if let Some(lyric_id) = &self.lyric_id {
                    lyric_string = kugou_api.song_lyric(lyric_id).await?;
                }
            }
            Some(ServiceProvider::Netease) => {
                let mut netease_api = netease::Api::new();
                if let Some(lyric_id) = &self.lyric_id {
                    lyric_string = netease_api.song_lyric(lyric_id).await?;
                }
            }
            Some(ServiceProvider::Migu) => {
                let migu_api = migu::Api::new();
                if let Some(lyric_id) = &self.lyric_id {
                    lyric_string = migu_api.song_lyric(lyric_id).await?;
                }
            }
            None => {}
        }

        Ok(lyric_string)
    }

    // get photo by pic_id(kugou/netease) or song_id(migu)
    pub async fn fetch_photo(&self) -> Result<Picture> {
        // let mut encoded_image_bytes: Vec<u8> = Vec::new();

        match self.service_provider {
            Some(ServiceProvider::Kugou) => {
                let kugou_api = kugou::Api::new();
                if let Some(p) = &self.pic_id {
                    if let Some(album_id) = &self.album_id {
                        Ok(kugou_api.pic(p, album_id).await?)
                    } else {
                        bail!("album_id is missing for kugou")
                    }
                } else {
                    bail!("pic_id is missing for kugou")
                }
            }
            Some(ServiceProvider::Netease) => {
                let mut netease_api = netease::Api::new();
                if let Some(p) = &self.pic_id {
                    Ok(netease_api.pic(p).await?)
                } else {
                    bail!("pic_id is missing for netease")
                }
            }
            Some(ServiceProvider::Migu) => {
                let migu_api = migu::Api::new();
                if let Some(p) = &self.song_id {
                    Ok(migu_api.pic(p).await?)
                } else {
                    bail!("song_id is missing for migu")
                }
            }
            None => {
                bail!("no servie provider given");
            }
        }

        // if encoded_image_bytes.is_empty() {
        //     bail!("failed to fetch image");
        // }

        // Ok(Picture::new_unchecked(
        //     PictureType::Other,
        //     MimeType::Jpeg,
        //     Some(String::from("Image")),
        //     encoded_image_bytes,
        // ))
    }

    #[allow(clippy::too_many_lines)]
    pub async fn download(&self, file: &str, tx_tageditor: &Sender<Msg>) -> Result<()> {
        let p_parent = get_parent_folder(Path::new(file));
        let song_id = self
            .song_id
            .as_ref()
            .ok_or_else(|| anyhow!("failed to download because no song id was found"))?;
        let artist = self
            .artist
            .clone()
            .unwrap_or_else(|| "Unknown Artist".to_string());
        let title = self
            .title
            .clone()
            .unwrap_or_else(|| "Unknown Title".to_string());

        let album = self.album.clone().unwrap_or_else(|| String::from("N/A"));
        let lyric = self.fetch_lyric().await;
        let photo = self.fetch_photo().await;
        let album_id = self.album_id.clone().unwrap_or_else(|| String::from("N/A"));

        let filename = format!("{artist}-{title}.%(ext)s");

        let args = vec![
            Arg::new("--quiet"),
            Arg::new_with_arg("--output", filename.as_ref()),
            Arg::new("--extract-audio"),
            Arg::new_with_arg("--audio-format", "mp3"),
        ];

        let p_full = p_parent.join(format!("{artist}-{title}.mp3"));
        match std::fs::remove_file(&p_full) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => (),
            v => v?,
        }

        let mp3_url = self.url.clone().unwrap_or_else(|| String::from("N/A"));
        if mp3_url.starts_with("Copyright") {
            bail!("The item is protected by copyright, please, select another one.");
        }
        let mut url = mp3_url;

        if let Some(s) = &self.service_provider {
            match s {
                ServiceProvider::Netease => {
                    let mut netease_api = netease::Api::new();
                    url = netease_api.song_url(song_id).await?;
                }
                ServiceProvider::Migu => {}
                ServiceProvider::Kugou => {
                    let kugou_api = kugou::Api::new();
                    url = kugou_api.song_url(song_id, &album_id).await?;
                }
            }
        }

        if url.is_empty() {
            bail!("failed to fetch url, please, try another item.");
        }

        let ytd = YoutubeDL::new(&PathBuf::from(p_parent), args, &url)?;

        let tx = tx_tageditor.clone();
        thread::spawn(move || -> Result<()> {
            tx.send(Msg::Download(DLMsg::DownloadRunning(
                url.clone(),
                title.clone(),
            )))
            .ok();
            // start download
            let download = ytd.download();

            // check what the result is and print out the path to the download or the error
            match download {
                Ok(_result) => {
                    tx.send(Msg::Download(DLMsg::DownloadSuccess(url.clone())))
                        .ok();
                    let mut tag = Id3v2Tag::default();

                    tag.set_title(title.clone());
                    tag.set_artist(artist);
                    tag.set_album(album);

                    // safe to unwrap these frames, since the ID is valid
                    if let Ok(l) = lyric {
                        let frame = Frame::UnsynchronizedText(UnsynchronizedTextFrame::new(
                            TextEncoding::UTF8,
                            *b"eng",
                            String::from("saved by termusic"),
                            l,
                        ));
                        tag.insert(frame);
                    }

                    if let Ok(picture) = photo {
                        tag.insert_picture(picture);
                    }

                    if tag.save_to_path(&p_full, WriteOptions::new()).is_ok() {
                        sleep(Duration::from_secs(10));
                        tx.send(Msg::Download(DLMsg::DownloadCompleted(
                            url.clone(),
                            Some(p_full.to_string_lossy().to_string()),
                        )))
                        .ok();
                    } else {
                        tx.send(Msg::Download(DLMsg::DownloadErrEmbedData(
                            url.clone(),
                            title,
                        )))
                        .ok();
                        sleep(Duration::from_secs(10));
                        tx.send(Msg::Download(DLMsg::DownloadCompleted(url.clone(), None)))
                            .ok();
                    }
                }
                Err(e) => {
                    tx.send(Msg::Download(DLMsg::DownloadErrDownload(
                        url.clone(),
                        title.clone(),
                        e.to_string(),
                    )))
                    .ok();
                    sleep(Duration::from_secs(10));
                    tx.send(Msg::Download(DLMsg::DownloadCompleted(url.clone(), None)))
                        .ok();
                }
            };
            Ok(())
        });
        Ok(())
    }
}
