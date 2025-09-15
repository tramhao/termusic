use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Result, anyhow, bail};
use lofty::TextEncoding;
use lofty::config::WriteOptions;
use lofty::id3::v2::{Frame, Id3v2Tag, UnsynchronizedTextFrame};
use lofty::picture::Picture;
use lofty::prelude::{Accessor, TagExt};
use service::SongTagService;
use ytd_rs::{Arg, YoutubeDL};

use crate::common::const_unknown::{UNKNOWN_ARTIST, UNKNOWN_TITLE};
use crate::utils::get_parent_folder;

mod kugou;
pub mod lrc;
mod migu;
mod netease_v2;
mod service;

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

/// All events that can happen in [`search`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SongtagSearchResult {
    Finish(Vec<SongTag>),
}

// Search function of 3 servers. Run in parallel to get results faster.
pub async fn search(search_str: &str, tx_done: impl Fn(SongtagSearchResult) + Send + 'static) {
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

    tx_done(SongtagSearchResult::Finish(results));
}

pub type TrackDLMsgURL = Arc<str>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TrackDLMsg {
    /// Indicates a Start of a download.
    ///
    /// `(Url, Title)`
    Start(TrackDLMsgURL, String),
    /// Indicates the Download was a Success, though termusic post-processing is not done yet.
    ///
    /// `(Url)`
    Success(TrackDLMsgURL),
    /// Indicates the Download thread finished in both Success or Error.
    ///
    /// `(Url, Filename)`
    Completed(TrackDLMsgURL, Option<String>),
    /// Indicates that the Download has Errored and has been aborted.
    ///
    /// `(Url, Title, ErrorAsString)`
    Err(TrackDLMsgURL, String, String),
    /// Indicates that the Download was a Success, but termusic post-processing failed.
    /// Like re-saving tags after editing.
    ///
    /// `(Url, Title)`
    ErrEmbedData(TrackDLMsgURL, String),
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

    /// Try to download the currently selected item in the tag editor list.
    ///
    /// Async for fetching lyrics, coverart and url. The actual track download is blocking and in another thread.
    pub async fn download(
        &self,
        file: &Path,
        tx: impl Fn(TrackDLMsg) + Send + 'static,
    ) -> Result<()> {
        if self.url().is_some_and(|v| *v == UrlTypes::Protected) {
            bail!("The item is protected by copyright, please select another one.");
        }

        let artist = self
            .artist
            .clone()
            .unwrap_or_else(|| UNKNOWN_ARTIST.to_string());
        let title = self
            .title
            .clone()
            .unwrap_or_else(|| UNKNOWN_TITLE.to_string());

        let album = self.album.clone().unwrap_or_else(|| String::from("N/A"));
        let lyric = self
            .fetch_lyric()
            .await
            .map_err(|err| {
                warn!("Fetching Lyric failed: {err:#?}");
            })
            .ok()
            .flatten();
        let photo = self
            .fetch_photo()
            .await
            .map_err(|err| {
                warn!("Fetching Photo failed: {err:#?}");
            })
            .ok();

        let p_parent = get_parent_folder(file);
        let out_path = p_parent.join(format!("{artist}-{title}.mp3"));
        match std::fs::remove_file(&out_path) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => (),
            v => v?,
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

        let filename = format!("{artist}-{title}.%(ext)s");

        let mut tag = Id3v2Tag::default();

        tag.set_title(title.clone());
        tag.set_artist(artist);
        tag.set_album(album);

        if let Some(l) = lyric {
            let frame = Frame::UnsynchronizedText(UnsynchronizedTextFrame::new(
                TextEncoding::UTF8,
                *b"eng",
                String::from("saved by termusic"),
                l,
            ));
            tag.insert(frame);
        }

        if let Some(picture) = photo {
            tag.insert_picture(picture);
        }

        let args = vec![
            Arg::new("--quiet"),
            Arg::new_with_arg("--output", filename.as_ref()),
            Arg::new("--extract-audio"),
            Arg::new_with_arg("--audio-format", "mp3"),
        ];

        // avoid full string clones when sending via a channel
        let url = Arc::from(url.into_boxed_str());

        let ytd = YoutubeDL::new(&PathBuf::from(p_parent), args, &url)?;

        let jh = tokio::task::spawn_blocking(move || {
            Self::download_ytd(url, &out_path, title, &tag, &ytd, tx);
        });
        drop(jh);

        Ok(())
    }

    /// Run the download and emit Err / Success.
    ///
    /// This function is blocking until the download finishes.
    fn download_ytd(
        url: Arc<str>,
        out_path: &Path,
        title: String,
        tag: &Id3v2Tag,
        ytd: &YoutubeDL,
        tx: impl Fn(TrackDLMsg),
    ) {
        tx(TrackDLMsg::Start(url.clone(), title.clone()));

        // do the actual download
        if let Err(err) = ytd.download() {
            tx(TrackDLMsg::Err(url.clone(), title.clone(), err.to_string()));

            return;
        }

        tx(TrackDLMsg::Success(url.clone()));

        if tag.save_to_path(out_path, WriteOptions::new()).is_ok() {
            tx(TrackDLMsg::Completed(
                url,
                Some(out_path.to_string_lossy().to_string()),
            ));
        } else {
            tx(TrackDLMsg::ErrEmbedData(url.clone(), title));
            tx(TrackDLMsg::Completed(url, None));
        }
    }
}
