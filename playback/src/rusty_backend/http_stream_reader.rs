use crate::source::{ReadSeekSource, Source};
use eyre::{Context, Result};
use tracing::info;

use stream_download::{http::HttpStream, source::SourceStream, StreamDownload};

#[derive(Debug)]
pub(crate) struct HttpStreamReader {
    downloader: StreamDownload,
    url: String,
    file_len: Option<u64>,
}

impl HttpStreamReader {
    pub async fn new(url: String) -> Result<Self> {
        let stream = HttpStream::create(url.parse()?)
            .await
            .wrap_err_with(|| "Error creating http stream")?;
        let file_len = stream.content_length().await;
        Ok(Self {
            url: url.clone(),
            downloader: StreamDownload::from_stream(stream)
                .wrap_err_with(|| "Error creating stream downloader")?,
            file_len,
        })
    }

    pub fn into_source(self) -> Box<dyn Source> {
        let parts: Vec<&str> = self.url.split('.').collect();
        let extension = if parts.len() > 1 {
            parts.last().map(|e| e.to_string())
        } else {
            None
        };
        info!("Using extension {extension:?}");

        Box::new(ReadSeekSource::new(
            self.downloader,
            self.file_len,
            extension,
        ))
    }
}
