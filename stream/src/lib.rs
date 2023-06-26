use source::{
    Source,
    SourceHandle,
    SourceStream,
};
use std::{
    io::{
        self,
        BufReader,
        Read,
        Seek,
        SeekFrom,
    },
    thread,
};
use tap::{
    Tap,
    TapFallible,
};
use tempfile::NamedTempFile;
use tracing::{
    debug,
    error,
};

#[cfg(feature = "http")]
pub mod http;
pub mod source;

#[derive(Debug)]
pub struct StreamDownload {
    output_reader: BufReader<NamedTempFile>,
    handle: SourceHandle,
}

impl StreamDownload {
    #[cfg(feature = "http")]
    pub fn new_http(url: reqwest::Url) -> io::Result<Self> {
        Self::new::<http::HttpStream>(url)
    }

    pub fn new<S: SourceStream>(url: S::Url) -> io::Result<Self> {
        let tempfile = tempfile::Builder::new().tempfile()?;
        let source = Source::new(tempfile.reopen()?);
        let handle = source.source_handle();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                let stream = S::create(url).await.tap_err(|e| error!("Error creating stream: {e}"))?;
                source.download(stream).await?;
                Ok::<_, io::Error>(())
            });
        } else {
            thread::spawn(move || {
                let rt =
                    tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .tap_err(|e| error!("Error creating tokio runtime: {e}"))?;
                rt.block_on(async move {
                    let stream = S::create(url).await.tap_err(|e| error!("Error creating stream {e}"))?;
                    source.download(stream).await?;
                    Ok::<_, io::Error>(())
                })?;
                Ok::<_, io::Error>(())
            });
        };
        Ok(Self {
            output_reader: BufReader::new(tempfile),
            handle,
        })
    }

    pub fn from_stream<S: SourceStream>(stream: S) -> Result<Self, io::Error> {
        let tempfile = tempfile::Builder::new().tempfile()?;
        let source = Source::new(tempfile.reopen()?);
        let handle = source.source_handle();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                source.download(stream).await.tap_err(|e| error!("Error downloading stream: {e}"))?;
                Ok::<_, io::Error>(())
            });
        } else {
            thread::spawn(move || {
                let rt =
                    tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .tap_err(|e| error!("Error creating tokio runtime: {e}"))?;
                rt.block_on(async move {
                    source.download(stream).await.tap_err(|e| error!("Error downloading stream: {e}"))?;
                    Ok::<_, io::Error>(())
                })?;
                Ok::<_, io::Error>(())
            });
        };
        Ok(Self {
            output_reader: BufReader::new(tempfile),
            handle,
        })
    }
}

impl Read for StreamDownload {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        debug!("Read request buf len: {}", buf.len());
        let stream_position = self.output_reader.stream_position()?;
        let requested_position = stream_position + buf.len() as u64;
        debug!("read: current position: {} requested position: {requested_position}", stream_position);
        if let Some(closest_set) = self.handle.downloaded().get(&stream_position) {
            debug!("Already downloaded {closest_set:?}");
            if closest_set.end >= requested_position {
                return self.output_reader.read(buf).tap(|l| debug!("Returning read length {l:?}"));
            }
        }
        self.handle.request_position(requested_position);
        debug!("waiting for position");
        self.handle.wait_for_requested_position();
        debug!("reached requested position {requested_position}: stream position: {}", stream_position);
        self.output_reader.read(buf).tap(|l| debug!("Returning read length {l:?}"))
    }
}

impl Seek for StreamDownload {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let seek_pos = match pos {
            SeekFrom::Start(pos) => {
                debug!("Seek from start: {pos}");
                pos
            },
            SeekFrom::End(pos) => {
                debug!("Seek from end: {pos}");
                if let Some(length) = self.handle.content_length() {
                    (length as i64 - 1 + pos) as u64
                } else {
                    return Err(
                        io::Error::new(
                            io::ErrorKind::Unsupported,
                            "Cannot seek from end when content length is unknown",
                        ),
                    );
                }
            },
            SeekFrom::Current(pos) => {
                debug!("Seek from current: {pos}");
                (self.output_reader.stream_position()? as i64 + pos) as u64
            },
        };
        if let Some(closest_set) = self.handle.downloaded().get(&seek_pos) {
            if closest_set.end >= seek_pos {
                return self.output_reader.seek(pos);
            }
        }
        self.handle.request_position(seek_pos);
        debug!("seek: current position {seek_pos} requested position {:?}. waiting", seek_pos);
        self.handle.seek(seek_pos);
        self.handle.wait_for_requested_position();
        debug!("reached seek position");
        self.output_reader.seek(pos)
    }
}
