/*
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
mod netease_v2;
mod service;

use crate::library_db::const_unknown::{UNKNOWN_ARTIST, UNKNOWN_TITLE};
use crate::types::{DLMsg, Msg, SongTagRecordingResult, TEMsg};
use crate::utils::get_parent_folder;
use anyhow::{anyhow, bail, Result};
use lofty::config::WriteOptions;
use lofty::id3::v2::{Frame, Id3v2Tag, UnsynchronizedTextFrame};
use lofty::picture::Picture;
use lofty::prelude::{Accessor, TagExt};
use lofty::TextEncoding;
use service::SongTagService;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::{self, sleep};
use std::time::Duration;
use ytd_rs::{Arg, YoutubeDL};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SongTag {
    service_provider: ServiceProvider,
    song_id: String,
    artist: Option<String>,
    title: Option<String>,
    album: Option<String>,
    lang_ext: Option<String>,
    lyric_id: Option<String>,
    url: Option<UrlTypes>,
    pic_id: Option<String>,
    album_id: Option<String>,
    // genre: Option<String>,
}

/// Indicate in which way the song can be downloaded, if at all.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UrlTypes {
    /// Download is protected by DRM or a fee, something which we dont do here
    Protected,
    /// Download is freely available, but requires extra fetching (`Api::song_url()`)
    AvailableRequiresFetching,
    /// Url is freely available to be downloaded
    FreeDownloadable(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
pub async fn search(search_str: &str, tx_done: Sender<Msg>) {
    let mut results: Vec<SongTag> = Vec::new();

    let handle_netease = async {
        let neteasev2_api = netease_v2::Api::new();
        neteasev2_api.search_recording(search_str, 0, 30).await
    };

    let handle_migu = async {
        let migu_api = migu::Api::new();
        migu_api.search_recording(search_str, 0, 30).await
    };

    let handle_kugou = async {
        let kugou_api = kugou::Api::new();
        kugou_api.search_recording(search_str, 0, 30).await
    };

    let (netease_res, migu_res, kugou_res) =
        futures_util::join!(handle_netease, handle_migu, handle_kugou);

    match netease_res {
        Ok(vec) => results.extend(vec),
        Err(err) => error!("Netease Error: {err:#}"),
    }

    match migu_res {
        Ok(vec) => results.extend(vec),
        Err(err) => error!("Migu Error: {err:#}"),
    }

    match kugou_res {
        Ok(vec) => results.extend(vec),
        Err(err) => error!("Kogou Error: {err:#}"),
    }

    let _ = tx_done.send(Msg::TagEditor(TEMsg::TESearchLyricResult(
        SongTagRecordingResult::Finish(results),
    )));
}

impl SongTag {
    #[must_use]
    pub fn artist(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    #[must_use]
    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
    }

    /// Optionally return the title of the song
    /// If `None` it wasn't able to read the tags
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    #[must_use]
    pub fn lang_ext(&self) -> Option<&str> {
        self.lang_ext.as_deref()
    }

    #[must_use]
    pub const fn service_provider(&self) -> ServiceProvider {
        self.service_provider
    }

    #[must_use]
    pub const fn url(&self) -> Option<&UrlTypes> {
        self.url.as_ref()
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.song_id
    }

    // get lyric by lyric_id
    pub async fn fetch_lyric(&self) -> Result<Option<String>> {
        let lyric_string = match self.service_provider {
            ServiceProvider::Kugou => {
                let kugou_api = kugou::Api::new();
                kugou_api.get_lyrics(self).await.map_err(|v| anyhow!(v))?
            }
            ServiceProvider::Netease => {
                let neteasev2_api = netease_v2::Api::new();
                neteasev2_api
                    .get_lyrics(self)
                    .await
                    .map_err(|v| anyhow!(v))?
            }
            ServiceProvider::Migu => {
                let migu_api = migu::Api::new();
                migu_api.get_lyrics(self).await.map_err(|v| anyhow!(v))?
            }
        };

        Ok(Some(lyric_string))
    }

    /// Fetch a picture for the current song
    /// For kugou & netease `pic_id()` or for migu `song_id` is used
    pub async fn fetch_photo(&self) -> Result<Picture> {
        match self.service_provider {
            ServiceProvider::Kugou => {
                let kugou_api = kugou::Api::new();
                Ok(kugou_api.get_picture(self).await.map_err(|v| anyhow!(v))?)
            }
            ServiceProvider::Netease => {
                let neteasev2_api = netease_v2::Api::new();
                Ok(neteasev2_api
                    .get_picture(self)
                    .await
                    .map_err(|v| anyhow!(v))?)
            }
            ServiceProvider::Migu => {
                let migu_api = migu::Api::new();
                Ok(migu_api.get_picture(self).await.map_err(|v| anyhow!(v))?)
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn download(&self, file: &Path, tx_tageditor: &Sender<Msg>) -> Result<()> {
        let p_parent = get_parent_folder(file);
        let artist = self
            .artist
            .clone()
            .unwrap_or_else(|| UNKNOWN_ARTIST.to_string());
        let title = self
            .title
            .clone()
            .unwrap_or_else(|| UNKNOWN_TITLE.to_string());

        let album = self.album.clone().unwrap_or_else(|| String::from("N/A"));
        let lyric = self.fetch_lyric().await;
        let photo = self.fetch_photo().await;

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

        if self.url().is_some_and(|v| *v == UrlTypes::Protected) {
            bail!("The item is protected by copyright, please, select another one.");
        }

        let mut url = if let Some(UrlTypes::FreeDownloadable(url)) = &self.url {
            url.clone()
        } else {
            String::new()
        };

        match self.service_provider {
            ServiceProvider::Netease => {
                let neteasev2_api = netease_v2::Api::new();
                url = neteasev2_api
                    .download_recording(self)
                    .await
                    .map_err(|v| anyhow!(v))?;
            }
            ServiceProvider::Migu => {}
            ServiceProvider::Kugou => {
                let kugou_api = kugou::Api::new();
                url = kugou_api
                    .download_recording(self)
                    .await
                    .map_err(|v| anyhow!(v))?;
            }
        }

        if url.is_empty() {
            bail!("failed to fetch url, please, try another item.");
        }

        // avoid full string clones when sending via a channel
        let url = Arc::from(url.into_boxed_str());

        let ytd = YoutubeDL::new(&PathBuf::from(p_parent), args, &url)?;

        let tx = tx_tageditor.clone();
        thread::spawn(move || -> Result<()> {
            tx.send(Msg::Download(DLMsg::DownloadRunning(
                url.clone(),
                title.clone(),
            )))
            .ok();

            // start download
            // check what the result is and print out the path to the download or the error
            let _download_result = match ytd.download() {
                Ok(res) => res,
                Err(err) => {
                    tx.send(Msg::Download(DLMsg::DownloadErrDownload(
                        url.clone(),
                        title.clone(),
                        err.to_string(),
                    )))
                    .ok();
                    sleep(Duration::from_secs(1));
                    tx.send(Msg::Download(DLMsg::DownloadCompleted(url.clone(), None)))
                        .ok();

                    return Ok(());
                }
            };

            tx.send(Msg::Download(DLMsg::DownloadSuccess(url.clone())))
                .ok();
            let mut tag = Id3v2Tag::default();

            tag.set_title(title.clone());
            tag.set_artist(artist);
            tag.set_album(album);

            if let Ok(Some(l)) = lyric {
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
                sleep(Duration::from_secs(1));
                tx.send(Msg::Download(DLMsg::DownloadCompleted(
                    url,
                    Some(p_full.to_string_lossy().to_string()),
                )))
                .ok();
            } else {
                tx.send(Msg::Download(DLMsg::DownloadErrEmbedData(
                    url.clone(),
                    title,
                )))
                .ok();
                sleep(Duration::from_secs(1));
                tx.send(Msg::Download(DLMsg::DownloadCompleted(url, None)))
                    .ok();
            }

            Ok(())
        });
        Ok(())
    }
}
