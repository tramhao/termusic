use crate::source::SourceStream;
use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use reqwest::Client;
use std::{
    io,
    pin::Pin,
    str::FromStr,
    task::{self, Poll},
};
use tracing::{info, warn};

pub struct HttpStream {
    stream: Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Unpin + Send + Sync>,
    client: Client,
    content_length: Option<u64>,
    url: reqwest::Url,
}

impl Stream for HttpStream {
    type Item = Result<Bytes, reqwest::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.stream).poll_next(cx)
    }
}

#[async_trait]
impl SourceStream for HttpStream {
    type Url = reqwest::Url;
    type Error = reqwest::Error;

    async fn create(url: Self::Url) -> io::Result<Self> {
        let client = Client::new();
        info!("Requesting content length");
        let response = client
            .get(url.as_str())
            .send()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
        let mut content_length = None;
        if let Some(length) = response.headers().get(reqwest::header::CONTENT_LENGTH) {
            let length = u64::from_str(
                length
                    .to_str()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?,
            )
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            info!("Got content length {length}");
            content_length = Some(length);
        } else {
            warn!("Content length header missing");
        }
        let stream = response.bytes_stream();
        Ok(Self {
            stream: Box::new(stream),
            client,
            content_length,
            url,
        })
    }

    async fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    async fn seek_range(&mut self, start: u64, end: Option<u64>) -> io::Result<()> {
        info!("Seeking: {start}-{end:?}");
        let response = self
            .client
            .get(self.url.as_str())
            .header(
                "Range",
                format!(
                    "bytes={start}-{}",
                    end.map(|e| e.to_string()).unwrap_or_default()
                ),
            )
            .send()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
        if !response.status().is_success() {
            return response
                .error_for_status()
                .map(|_| ())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()));
        }
        self.stream = Box::new(response.bytes_stream());
        info!("Done seeking");
        Ok(())
    }
}
