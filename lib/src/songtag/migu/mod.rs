mod model;

use anyhow::anyhow;
use image::ImageFormat;
use lofty::picture::{MimeType, Picture, PictureType};
use model::to_song_info;
use reqwest::{Client, ClientBuilder};
use std::io::Cursor;
use std::time::Duration;

use super::{
    ServiceProvider, SongTag, UrlTypes,
    service::{SongTagService, SongTagServiceError, SongTagServiceErrorWhere},
};

const URL_SEARCH_MIGU: &str = "https://pd.musicapp.migu.cn/MIGUM2.0/v1.0/content/search_all.do?&ua=Android_migu&version=5.0.1";
const REFERER: &str = "https://music.migu.cn";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";
pub struct Api {
    client: Client,
}

impl Api {
    pub fn new() -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("client build error");

        Self { client }
    }
}

impl SongTagService for Api {
    type Error = anyhow::Error;

    fn display_name() -> &'static str
    where
        Self: Sized,
    {
        "migu"
    }

    async fn search_recording(
        &self,
        keywords: &str,
        offset: u32,
        limit: u32,
    ) -> std::result::Result<Vec<SongTag>, super::service::SongTagServiceError<Self::Error>> {
        // let offset = offset.to_string();
        // let limit = limit.to_string();

        let url = format!(
            "{URL_SEARCH_MIGU}&text={keywords}&pageNo={offset}&pageSize={limit}&searchSwitch="
        );

        let result = self
            .client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .header("Referer", REFERER)
            .send()
            .await
            .map_err(anyhow::Error::from)?
            .text()
            .await
            .map_err(anyhow::Error::from)?;

        to_song_info(&result).map_err(|err| {
            SongTagServiceError::Other(anyhow!(err).context("Parse result into SongTag Array"))
        })
    }

    async fn get_lyrics(
        &self,
        song: &SongTag,
    ) -> std::result::Result<String, super::service::SongTagServiceError<Self::Error>> {
        if song.service_provider() != ServiceProvider::Migu {
            return Err(SongTagServiceError::IncorrectService(
                song.service_provider().to_string(),
                Self::display_name(),
            ));
        }

        if let Some(lyric_id) = song.lyric_id.as_ref() {
            let result = self
                .client
                .get(lyric_id)
                .header("Referer", REFERER)
                .send()
                .await
                .map_err(anyhow::Error::from)?
                .text()
                .await
                .map_err(anyhow::Error::from)?;

            Ok(result)
        } else {
            Err(SongTagServiceError::Other(anyhow!(
                "Provided songtag does not have a lyric_id!"
            )))
        }
    }

    async fn get_picture(
        &self,
        song: &SongTag,
    ) -> std::result::Result<Picture, super::service::SongTagServiceError<Self::Error>> {
        if song.service_provider() != ServiceProvider::Migu {
            return Err(SongTagServiceError::IncorrectService(
                song.service_provider().to_string(),
                Self::display_name(),
            ));
        }

        if let Some(url) = song.pic_id.as_ref() {
            let result = self
                .client
                .get(url)
                .send()
                .await
                .map_err(anyhow::Error::from)?;

            let bytes = result.bytes().await.map_err(anyhow::Error::from)?;

            let format = image::guess_format(&bytes).map_err(anyhow::Error::from)?;

            let picture = if format == ImageFormat::WebP {
                let img = image::load_from_memory(&bytes).map_err(anyhow::Error::from)?;
                let mut jpg_buf = Vec::new();
                img.write_to(&mut Cursor::new(&mut jpg_buf), ImageFormat::Jpeg)
                    .map_err(anyhow::Error::from)?;

                Picture::unchecked(jpg_buf)
                    .pic_type(PictureType::CoverFront)
                    .mime_type(MimeType::Jpeg)
                    .build()
            } else {
                Picture::from_reader(&mut Cursor::new(bytes)).map_err(anyhow::Error::from)?
            };

            Ok(picture)
        } else {
            Err(SongTagServiceError::Other(anyhow!(
                "Provided songtag does not have a pic_id!"
            )))
        }
    }

    async fn download_recording(
        &self,
        song: &SongTag,
    ) -> std::result::Result<String, super::service::SongTagServiceError<Self::Error>> {
        // this function is to get the url for downloading, which in migu does not require extra fetching
        // so if its available, use it, otherwise report "NotSupported"
        if let Some(UrlTypes::FreeDownloadable(url)) = song.url.as_ref() {
            return Ok(url.clone());
        }

        Err(SongTagServiceError::NotSupported(
            SongTagServiceErrorWhere::DownloadRecording,
            Self::display_name(),
        ))
    }
}
