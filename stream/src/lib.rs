use parking_lot::Mutex;
use source::{Source, SourceHandle, SourceStream};
use std::sync::Arc;
use std::{
    io::{self, BufReader, Read, Seek, SeekFrom},
    thread,
};
use symphonia::core::io::MediaSource;
use tap::{Tap, TapFallible};
use tempfile::NamedTempFile;
use tracing::{debug, error};

pub mod http;
pub mod source;

#[derive(Debug)]
pub struct StreamDownload {
    output_reader: BufReader<NamedTempFile>,
    handle: SourceHandle,
    is_radio: bool,
    pub radio_title: Arc<Mutex<String>>,
}

impl StreamDownload {
    pub fn new_http(
        url: reqwest::Url,
        is_radio: bool,
        radio_title: Arc<Mutex<String>>,
        radio_downloaded: Arc<Mutex<u64>>,
    ) -> io::Result<Self> {
        Self::new::<http::HttpStream>(url, is_radio, radio_title, radio_downloaded)
    }

    pub fn new<S: SourceStream>(
        url: S::Url,
        is_radio: bool,
        radio_title: Arc<Mutex<String>>,
        radio_downloaded: Arc<Mutex<u64>>,
    ) -> io::Result<Self> {
        let tempfile = tempfile::Builder::new().tempfile()?;
        let source = Source::new(tempfile.reopen()?);
        let handle = source.source_handle();
        let radio_title_inside = radio_title.clone();
        let radio_downloaded_inside1 = radio_downloaded.clone();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                let stream = S::create(url, is_radio, radio_title_inside)
                    .await
                    .tap_err(|e| error!("Error creating stream: {e}"))?;
                source.download(stream, radio_downloaded_inside1).await?;
                Ok::<_, io::Error>(())
            });
        } else {
            thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .tap_err(|e| error!("Error creating tokio runtime: {e}"))?;
                rt.block_on(async move {
                    let stream = S::create(url, is_radio, radio_title_inside)
                        .await
                        .tap_err(|e| error!("Error creating stream {e}"))?;
                    source.download(stream, radio_downloaded).await?;
                    Ok::<_, io::Error>(())
                })?;
                Ok::<_, io::Error>(())
            });
        };
        Ok(Self {
            output_reader: BufReader::new(tempfile),
            handle,
            is_radio,
            radio_title,
        })
    }

    pub fn from_stream<S: SourceStream>(
        stream: S,
        is_radio: bool,
        radio_title: Arc<Mutex<String>>,
        radio_downloaded: Arc<Mutex<u64>>,
    ) -> Result<Self, io::Error> {
        let tempfile = tempfile::Builder::new().tempfile()?;
        let source = Source::new(tempfile.reopen()?);
        let handle = source.source_handle();
        let radio_downloaded_inside1 = radio_downloaded.clone();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                source
                    .download(stream, radio_downloaded_inside1)
                    .await
                    .tap_err(|e| error!("Error downloading stream: {e}"))?;
                Ok::<_, io::Error>(())
            });
        } else {
            thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .tap_err(|e| error!("Error creating tokio runtime: {e}"))?;
                rt.block_on(async move {
                    source
                        .download(stream, radio_downloaded)
                        .await
                        .tap_err(|e| error!("Error downloading stream: {e}"))?;
                    Ok::<_, io::Error>(())
                })?;
                Ok::<_, io::Error>(())
            });
        };
        Ok(Self {
            output_reader: BufReader::new(tempfile),
            handle,
            is_radio,
            radio_title,
        })
    }
}

impl Read for StreamDownload {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        debug!("Read request buf len: {}", buf.len());
        let stream_position = self.output_reader.stream_position()?;
        let requested_position = stream_position + buf.len() as u64;
        debug!(
            "read: current position: {} requested position: {requested_position}",
            stream_position
        );
        if let Some(closest_set) = self.handle.downloaded().get(&stream_position) {
            debug!("Already downloaded {closest_set:?}");
            if closest_set.end >= requested_position {
                return self
                    .output_reader
                    .read(buf)
                    .tap(|l| debug!("Returning read length {l:?}"));
            }
        }
        self.handle.request_position(requested_position);
        debug!("waiting for position");
        self.handle.wait_for_requested_position();
        debug!(
            "reached requested position {requested_position}: stream position: {}",
            stream_position
        );
        self.output_reader
            .read(buf)
            .tap(|l| debug!("Returning read length {l:?}"))
    }
}

impl Seek for StreamDownload {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let seek_pos = match pos {
            SeekFrom::Start(pos) => {
                debug!("Seek from start: {pos}");
                pos
            }
            SeekFrom::End(pos) => {
                debug!("Seek from end: {pos}");
                if let Some(length) = self.handle.content_length() {
                    (length as i64 - 1 + pos) as u64
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Cannot seek from end when content length is unknown",
                    ));
                }
            }
            SeekFrom::Current(pos) => {
                debug!("Seek from current: {pos}");
                (self.output_reader.stream_position()? as i64 + pos) as u64
            }
        };
        if let Some(closest_set) = self.handle.downloaded().get(&seek_pos) {
            if closest_set.end >= seek_pos {
                return self.output_reader.seek(pos);
            }
        }
        self.handle.request_position(seek_pos);
        debug!(
            "seek: current position {seek_pos} requested position {:?}. waiting",
            seek_pos
        );
        self.handle.seek(seek_pos);
        self.handle.wait_for_requested_position();
        debug!("reached seek position");
        self.output_reader.seek(pos)
    }
}

impl MediaSource for StreamDownload {
    fn is_seekable(&self) -> bool {
        true
        // !self.is_radio
    }

    fn byte_len(&self) -> Option<u64> {
        self.handle.content_length()
    }
}
