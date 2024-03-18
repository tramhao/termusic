use crate::source::SourceStream;
use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use parking_lot::Mutex;
use reqwest::Client;
use std::sync::Arc;
use std::{
    io,
    pin::Pin,
    str::FromStr,
    task::{self, Poll},
};
use tracing::{info, warn};

const STONG_TITLE_ERROR: &str = "Error Please Try Again";
const CHUNKS_BEFORE_START: u8 = 20;

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

    async fn create(
        url: Self::Url,
        is_radio: bool,
        radio_title: Arc<Mutex<String>>,
    ) -> io::Result<Self> {
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
        let mut count_down = CHUNKS_BEFORE_START;
        let url_inside = url.clone();
        if is_radio {
            let mut should_restart = true;
            let client_inside = Client::new();
            tokio::spawn(async move {
                loop {
                    let mut response = match client_inside
                        .get(url_inside.as_str())
                        .header("icy-metadata", "1")
                        .send()
                        .await
                    {
                        Ok(t) => t,
                        Err(_) => {
                            *radio_title.lock() = STONG_TITLE_ERROR.to_string();
                            continue;
                        }
                    };
                    if let Some(header_value) = response.headers().get("content-type") {
                        if header_value.to_str().unwrap_or_default() != "audio/mpeg" {
                            *radio_title.lock() = STONG_TITLE_ERROR.to_string();
                            continue;
                        }
                    } else {
                        *radio_title.lock() = STONG_TITLE_ERROR.to_string();
                        continue;
                    }
                    let meta_interval: usize =
                        if let Some(header_value) = response.headers().get("icy-metaint") {
                            header_value
                                .to_str()
                                .unwrap_or_default()
                                .parse()
                                .unwrap_or_default()
                        } else {
                            0
                        };
                    let mut counter = meta_interval;
                    let mut awaiting_metadata_size = false;
                    let mut metadata_size: u8 = 0;
                    let mut awaiting_metadata = false;
                    let mut metadata: Vec<u8> = Vec::new();
                    let mut title_string = String::new();
                    while let Some(chunk) = response.chunk().await.expect("Couldn't get next chunk")
                    {
                        for byte in &chunk {
                            if meta_interval != 0 {
                                if awaiting_metadata_size {
                                    awaiting_metadata_size = false;
                                    metadata_size = *byte * 16;
                                    if metadata_size == 0 {
                                        counter = meta_interval;
                                    } else {
                                        awaiting_metadata = true;
                                    }
                                } else if awaiting_metadata {
                                    metadata.push(*byte);
                                    metadata_size = metadata_size.saturating_sub(1);
                                    if metadata_size == 0 {
                                        awaiting_metadata = false;
                                        let metadata_string =
                                            std::str::from_utf8(&metadata).unwrap_or("");
                                        if !metadata_string.is_empty() {
                                            const STREAM_TITLE_KEYWORD: &str = "StreamTitle='";
                                            if let Some(index) =
                                                metadata_string.find(STREAM_TITLE_KEYWORD)
                                            {
                                                let left_index = index + 13;
                                                let stream_title_substring =
                                                    &metadata_string[left_index..];
                                                if let Some(right_index) =
                                                    stream_title_substring.find('\'')
                                                {
                                                    let trimmed_song_title =
                                                        &stream_title_substring[..right_index];
                                                    title_string += " ";
                                                    title_string += trimmed_song_title;
                                                    *radio_title.lock() =
                                                        format!("Current playing: {}", title_string.trim());
                                                }
                                            }
                                        }
                                        metadata.clear();
                                        counter = meta_interval;
                                    }
                                } else {
                                    // file.write_all(&[*byte]).expect("Couldn't write to file");
                                    counter = counter.saturating_sub(1);
                                    if counter == 0 {
                                        awaiting_metadata_size = true;
                                    }
                                }
                            } else {
                                // file.write_all(&[*byte]).expect("Couldn't write to file");
                            }
                        }
                        if should_restart {
                            if count_down == 0 {
                                should_restart = false;
                                title_string = String::new();
                            } else {
                                count_down -= 1;
                            }
                        }
                    }
                }
            });
        }
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
