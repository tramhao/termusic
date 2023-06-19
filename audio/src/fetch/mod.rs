use anyhow::Error;
use futures_util::{future::IntoStream, StreamExt, TryFutureExt};
use hyper::{
    client::ResponseFuture,
    header::{self, CONTENT_RANGE},
    Body, Response, StatusCode,
};
use log::debug;
use std::{
    cmp::min,
    env, fs,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use symphonia::core::io::MediaSource;
use termusiclib::utils::get_app_config_path;
use thiserror::Error;

use parking_lot::{Condvar, Mutex};
use tempfile::NamedTempFile;
use tokio::sync::{mpsc, oneshot, Semaphore};
use url::Url;

use self::{client::Client, receive::audio_file_fetch};

use crate::{
    fetch::cache::Cache,
    range_set::{Range, RangeSet},
};

pub mod client;

pub mod receive;

pub mod cache;

pub type AudioFileResult = Result<(), anyhow::Error>;

pub const MINIMUM_DOWNLOAD_SIZE: usize = 64 * 1024;

pub const MINIMUM_THROUGHPUT: usize = 8 * 1024;

pub const READ_AHEAD_BEFORE_PLAYBACK: Duration = Duration::from_secs(1);

pub const READ_AHEAD_DURING_PLAYBACK: Duration = Duration::from_secs(5);

pub const DOWNLOAD_TIMEOUT: Duration =
    Duration::from_secs((MINIMUM_DOWNLOAD_SIZE / MINIMUM_THROUGHPUT) as u64);

pub const PREFETCH_THRESHOLD_FACTOR: f32 = 4.0;

/// If the measured ping time to the Spotify server is larger than this value, it is capped
/// to avoid run-away block sizes and pre-fetching.
pub const MAXIMUM_ASSUMED_PING_TIME: Duration = Duration::from_millis(1500);

/// The ping time that is used for calculations before a ping time was actually measured.
pub const INITIAL_PING_TIME_ESTIMATE: Duration = Duration::from_millis(500);

#[derive(Error, Debug)]
pub enum AudioFileError {
    #[error("other end of channel disconnected")]
    Channel,
    #[error("required header not found")]
    Header,
    #[error("streamer received no data")]
    NoData,
    #[error("no output available")]
    Output,
    #[error("invalid status code {0}")]
    StatusCode(StatusCode),
    #[error("wait timeout exceeded")]
    WaitTimeout,
}

pub enum AudioFile {
    Cached(fs::File),
    Streaming(AudioFileStreaming),
    Local(fs::File),
}

#[derive(Debug)]
pub struct StreamingRequest {
    streamer: IntoStream<ResponseFuture>,
    initial_response: Option<Response<Body>>,
    offset: usize,
    length: usize,
}

#[derive(Clone)]
pub struct StreamLoaderController {
    channel_tx: Option<mpsc::UnboundedSender<StreamLoaderCommand>>,
    stream_shared: Option<Arc<AudioFileShared>>,
    file_size: usize,
}

impl StreamLoaderController {
    pub fn len(&self) -> usize {
        self.file_size
    }

    pub fn is_empty(&self) -> bool {
        self.file_size == 0
    }

    pub fn range_available(&self, range: Range) -> bool {
        let available = if let Some(ref shared) = self.stream_shared {
            let download_status = shared.download_status.lock();

            range.length
                <= download_status
                    .downloaded
                    .contained_length_from_value(range.start)
        } else {
            range.length <= self.len() - range.start
        };

        available
    }

    pub fn range_to_end_available(&self) -> bool {
        match self.stream_shared {
            Some(ref shared) => {
                let read_position = shared.read_position();
                self.range_available(Range::new(read_position, self.len() - read_position))
            }
            None => true,
        }
    }

    pub fn ping_time(&self) -> Option<Duration> {
        self.stream_shared.as_ref().map(|shared| shared.ping_time())
    }

    fn send_stream_loader_command(&self, command: StreamLoaderCommand) {
        if let Some(ref channel) = self.channel_tx {
            // Ignore the error in case the channel has been closed already.
            // This means that the file was completely downloaded.
            let _ = channel.send(command);
        }
    }

    pub fn fetch(&self, range: Range) {
        // signal the stream loader to fetch a range of the file
        self.send_stream_loader_command(StreamLoaderCommand::Fetch(range));
    }

    pub fn fetch_blocking(&self, mut range: Range) -> AudioFileResult {
        // signal the stream loader to tech a range of the file and block until it is loaded.

        // ensure the range is within the file's bounds.
        if range.start >= self.len() {
            range.length = 0;
        } else if range.end() > self.len() {
            range.length = self.len() - range.start;
        }

        self.fetch(range);

        if let Some(ref shared) = self.stream_shared {
            let mut download_status = shared.download_status.lock();

            while range.length
                > download_status
                    .downloaded
                    .contained_length_from_value(range.start)
            {
                if shared
                    .cond
                    .wait_for(&mut download_status, DOWNLOAD_TIMEOUT)
                    .timed_out()
                {
                    return Err(AudioFileError::WaitTimeout.into());
                }

                if range.length
                    > (download_status
                        .downloaded
                        .union(&download_status.requested)
                        .contained_length_from_value(range.start))
                {
                    // For some reason, the requested range is neither downloaded nor requested.
                    // This could be due to a network error. Request it again.
                    self.fetch(range);
                }
            }
        }

        Ok(())
    }

    pub fn fetch_next_and_wait(
        &self,
        request_length: usize,
        wait_length: usize,
    ) -> AudioFileResult {
        match self.stream_shared {
            Some(ref shared) => {
                let start = shared.read_position();

                let request_range = Range {
                    start,
                    length: request_length,
                };
                self.fetch(request_range);

                let wait_range = Range {
                    start,
                    length: wait_length,
                };
                self.fetch_blocking(wait_range)
            }
            None => Ok(()),
        }
    }

    pub fn set_random_access_mode(&self) {
        // optimise download strategy for random access
        if let Some(ref shared) = self.stream_shared {
            shared.set_download_streaming(false)
        }
    }

    pub fn set_stream_mode(&self) {
        // optimise download strategy for streaming
        if let Some(ref shared) = self.stream_shared {
            shared.set_download_streaming(true)
        }
    }

    pub fn close(&self) {
        // terminate stream loading and don't load any more data for this file.
        self.send_stream_loader_command(StreamLoaderCommand::Close);
    }

    pub fn mime_type(&self) -> Option<String> {
        if let Some(ref shared) = self.stream_shared {
            shared.get_mime_type()
        } else {
            None
        }
    }
}

pub struct AudioFileStreaming {
    read_file: fs::File,
    position: u64,
    stream_loader_command_tx: mpsc::UnboundedSender<StreamLoaderCommand>,
    shared: Arc<AudioFileShared>,
}

struct AudioFileDownloadStatus {
    requested: RangeSet,
    downloaded: RangeSet,
}

impl AudioFile {
    pub async fn open(url: &str, bytes_per_second: usize) -> Result<AudioFile, Error> {
        if Url::parse(url).is_err() {
            return Ok(AudioFile::Local(fs::File::open(url)?));
        }

        let cache = Cache::new();
        let file_id = format!("{:x}", md5::compute(url.to_owned()));
        if cache.is_file_cached(file_id.as_str()) {
            println!("File is cached: {}", file_id);
            debug!(">> File is cached: {}", file_id);
            return Ok(AudioFile::Cached(cache.open_file(file_id.as_str())?));
        }

        let (complete_tx, complete_rx) = oneshot::channel();

        let streaming = AudioFileStreaming::open(url.to_owned(), complete_tx, bytes_per_second);

        let file_id = format!("{:x}", md5::compute(url.to_owned()));

        // spawn a task to download the file
        tokio::spawn(complete_rx.map_ok(move |mut file| {
            println!("Download complete: {}", file.path().display());
            debug!(">> Download complete: {}", file.path().display());
            let cache = Cache::new();
            match cache.save_file(&file_id, &mut file) {
                Ok(_) => {
                    println!("Saved to cache: {}", file_id);
                    debug!(">> Saved to cache: {}", file_id);
                }
                Err(e) => {
                    println!("Failed to save to cache: {}", e);
                    debug!(">> Failed to save to cache: {}", e);
                }
            }
        }));

        Ok(AudioFile::Streaming(streaming.await?))
    }

    pub fn get_stream_loader_controller(&self) -> Result<StreamLoaderController, Error> {
        let controller = match self {
            AudioFile::Streaming(ref stream) => StreamLoaderController {
                channel_tx: Some(stream.stream_loader_command_tx.clone()),
                stream_shared: Some(stream.shared.clone()),
                file_size: stream.shared.file_size,
            },
            AudioFile::Cached(ref file) => StreamLoaderController {
                channel_tx: None,
                stream_shared: None,
                file_size: file.metadata()?.len() as usize,
            },
            AudioFile::Local(ref file) => StreamLoaderController {
                channel_tx: None,
                stream_shared: None,
                file_size: file.metadata()?.len() as usize,
            },
        };

        Ok(controller)
    }

    pub fn is_cached(&self) -> bool {
        matches!(self, AudioFile::Cached { .. })
    }

    pub fn is_local(&self) -> bool {
        matches!(self, AudioFile::Local { .. })
    }

    pub async fn get_mime_type(url: &str) -> Result<String, Error> {
        if Url::parse(url).is_err() {
            if !Path::new(url).exists() {
                return Err(Error::msg("File does not exist"));
            }
            match mime_guess::from_path(url).first() {
                Some(mime) => return Ok(mime.to_string()),
                None => return Err(Error::msg("No mime type found")),
            }
        }
        let mut streamer = Client::new().stream_from_url(url, 0, 512)?;
        let response = streamer.next().await.ok_or(AudioFileError::NoData)??;

        let content_type = match response.headers().get(header::CONTENT_TYPE) {
            Some(content_type) => content_type,
            None => return Err(Error::msg("No Content-Type header")),
        };

        let mime = content_type.to_str()?;

        Ok(mime.to_owned())
    }
}

impl AudioFileStreaming {
    pub async fn open(
        url: String,
        complete_tx: oneshot::Sender<NamedTempFile>,
        bytes_per_second: usize,
    ) -> Result<AudioFileStreaming, Error> {
        // When the audio file is really small, this `download_size` may turn out to be
        // larger than the audio file we're going to stream later on. This is OK; requesting
        // `Content-Range` > `Content-Length` will return the complete file with status code
        // 206 Partial Content.

        debug!(">> Downloading file: {}", url);
        let mut streamer = Client::new().stream_from_url(url.as_str(), 0, MINIMUM_DOWNLOAD_SIZE)?;

        // Get the first chunk with the headers to get the file size.
        // The remainder of that chunk with possibly also a response body is then
        // further processed in `audio_file_fetch`.
        let response = streamer.next().await.ok_or(AudioFileError::NoData)??;

        debug!(">> Got response: {:?}", response);

        let code = response.status();
        if code != StatusCode::PARTIAL_CONTENT {
            return Err(AudioFileError::StatusCode(code).into());
        }

        let header_value = response
            .headers()
            .get(CONTENT_RANGE)
            .ok_or(AudioFileError::Header)?;
        let str_value = header_value.to_str()?;
        let hyphen_index = str_value.find('-').unwrap_or_default();
        let slash_index = str_value.find('/').unwrap_or_default();
        let upper_bound: usize = str_value[hyphen_index + 1..slash_index].parse()?;
        let file_size = str_value[slash_index + 1..].parse()?;

        let content_type = match response.headers().get(header::CONTENT_TYPE) {
            Some(content_type) => content_type,
            None => return Err(Error::msg("No Content-Type header")),
        };

        let mime = content_type.to_str()?;
        let mime = mime.to_owned();

        let initial_request = StreamingRequest {
            streamer,
            initial_response: Some(response),
            offset: 0,
            length: upper_bound + 1,
        };

        let shared = Arc::new(AudioFileShared {
            url,
            file_size,
            bytes_per_second,
            cond: Condvar::new(),
            download_status: Mutex::new(AudioFileDownloadStatus {
                requested: RangeSet::new(),
                downloaded: RangeSet::new(),
            }),
            download_streaming: AtomicBool::new(false),
            download_slots: Semaphore::new(1),
            ping_time_ms: AtomicUsize::new(0),
            read_position: AtomicUsize::new(0),
            throughput: AtomicUsize::new(0),
            mime_type: mime,
        });

        let mut app_dir = std::env::temp_dir().to_string_lossy().to_string();
        if let Ok(dir) = get_app_config_path() {
            app_dir = format!("{}/cache", dir.to_string_lossy());
        }

        debug!(">> Creating temp file in {:?}", app_dir);

        let write_file = match env::consts::OS {
            "android" => NamedTempFile::new_in(app_dir)?,
            _ => NamedTempFile::new()?,
        };
        debug!(">> Created temp file: {:?}", write_file.path());
        write_file.as_file().set_len(file_size as u64)?;

        let read_file = write_file.reopen()?;

        let (stream_loader_command_tx, stream_loader_command_rx) =
            mpsc::unbounded_channel::<StreamLoaderCommand>();

        tokio::spawn(audio_file_fetch(
            shared.clone(),
            initial_request,
            write_file,
            stream_loader_command_rx,
            complete_tx,
        ));

        Ok(AudioFileStreaming {
            read_file,
            position: 0,
            stream_loader_command_tx,
            shared,
        })
    }
}

impl Read for AudioFileStreaming {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let offset = self.position as usize;

        if offset >= self.shared.file_size {
            return Ok(0);
        }

        let length = min(output.len(), self.shared.file_size - offset);
        if length == 0 {
            return Ok(0);
        }

        let length_to_request = if self.shared.is_download_streaming() {
            let length_to_request = length
                + (READ_AHEAD_DURING_PLAYBACK.as_secs_f32() * self.shared.bytes_per_second as f32)
                    as usize;

            // Due to the read-ahead stuff, we potentially request more than the actual request demanded.
            min(length_to_request, self.shared.file_size - offset)
        } else {
            length
        };

        let mut ranges_to_request = RangeSet::new();
        ranges_to_request.add_range(&Range::new(offset, length_to_request));

        let mut download_status = self.shared.download_status.lock();

        ranges_to_request.subtract_range_set(&download_status.downloaded);
        ranges_to_request.subtract_range_set(&download_status.requested);

        for &range in ranges_to_request.iter() {
            self.stream_loader_command_tx
                .send(StreamLoaderCommand::Fetch(range))
                .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err))?;
        }

        while !download_status.downloaded.contains(offset) {
            if self
                .shared
                .cond
                .wait_for(&mut download_status, DOWNLOAD_TIMEOUT)
                .timed_out()
            {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    Error::msg("Download timed out"),
                ));
            }
        }
        let available_length = download_status
            .downloaded
            .contained_length_from_value(offset);

        drop(download_status);

        self.position = self.read_file.seek(SeekFrom::Start(offset as u64))?;
        let read_len = min(length, available_length);
        let read_len = self.read_file.read(&mut output[..read_len])?;

        self.position += read_len as u64;
        self.shared.set_read_position(self.position);

        Ok(read_len)
    }
}

impl Seek for AudioFileStreaming {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        // If we are already at this position, we don't need to switch download mode.
        // These checks and locks are less expensive than interrupting streaming.
        let current_position = self.position as i64;
        let requested_pos = match pos {
            SeekFrom::Start(pos) => pos as i64,
            SeekFrom::End(pos) => self.shared.file_size as i64 - pos - 1,
            SeekFrom::Current(pos) => current_position + pos,
        };
        if requested_pos == current_position {
            return Ok(current_position as u64);
        }

        // Again if we have already downloaded this part.
        let available = self
            .shared
            .download_status
            .lock()
            .downloaded
            .contains(requested_pos as usize);

        let mut was_streaming = false;
        if !available {
            // Ensure random access mode if we need to download this part.
            // Checking whether we are streaming now is a micro-optimization
            // to save an atomic load.
            was_streaming = self.shared.is_download_streaming();
            if was_streaming {
                self.shared.set_download_streaming(false);
            }
        }

        self.position = self.read_file.seek(pos)?;
        self.shared.set_read_position(self.position);

        if !available && was_streaming {
            self.shared.set_download_streaming(true);
        }

        Ok(self.position)
    }
}

impl Read for AudioFile {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        match *self {
            AudioFile::Cached(ref mut file) => file.read(output),
            AudioFile::Streaming(ref mut file) => file.read(output),
            AudioFile::Local(ref mut file) => file.read(output),
        }
    }
}

impl Seek for AudioFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match *self {
            AudioFile::Cached(ref mut file) => file.seek(pos),
            AudioFile::Streaming(ref mut file) => file.seek(pos),
            AudioFile::Local(ref mut file) => file.seek(pos),
        }
    }
}

#[derive(Debug)]
pub enum StreamLoaderCommand {
    Fetch(Range), // signal the stream loader to fetch a range of the file
    Close,        // terminate and don't load any more data
}

struct AudioFileShared {
    url: String,
    file_size: usize,
    bytes_per_second: usize,
    cond: Condvar,
    download_status: Mutex<AudioFileDownloadStatus>,
    download_streaming: AtomicBool,
    download_slots: Semaphore,
    ping_time_ms: AtomicUsize,
    read_position: AtomicUsize,
    throughput: AtomicUsize,
    mime_type: String,
}

impl AudioFileShared {
    fn is_download_streaming(&self) -> bool {
        self.download_streaming.load(Ordering::Acquire)
    }

    fn set_download_streaming(&self, streaming: bool) {
        self.download_streaming.store(streaming, Ordering::Release)
    }

    fn ping_time(&self) -> Duration {
        let ping_time_ms = self.ping_time_ms.load(Ordering::Acquire);
        if ping_time_ms > 0 {
            Duration::from_millis(ping_time_ms as u64)
        } else {
            INITIAL_PING_TIME_ESTIMATE
        }
    }

    fn set_ping_time(&self, duration: Duration) {
        self.ping_time_ms
            .store(duration.as_millis() as usize, Ordering::Release)
    }

    fn throughput(&self) -> usize {
        self.throughput.load(Ordering::Acquire)
    }

    fn set_throughput(&self, throughput: usize) {
        self.throughput.store(throughput, Ordering::Release)
    }

    fn read_position(&self) -> usize {
        self.read_position.load(Ordering::Acquire)
    }

    fn set_read_position(&self, position: u64) {
        self.read_position
            .store(position as usize, Ordering::Release)
    }

    fn get_mime_type(&self) -> Option<String> {
        if Url::parse(&self.url).is_err() {
            if Path::new(&self.url).exists() {
                match mime_guess::from_path(&self.url).first() {
                    Some(mime) => {
                        return Some(mime.to_string());
                    }
                    None => return None,
                };
            }
        }
        Some(format!("{}", self.mime_type))
    }
}

pub struct Subfile<T: Read + Seek> {
    stream: T,
    offset: u64,
    length: u64,
}

impl<T: Read + Seek> Subfile<T> {
    pub fn new(mut stream: T, offset: u64, length: u64) -> Result<Subfile<T>, io::Error> {
        let target = SeekFrom::Start(offset);
        stream.seek(target)?;

        Ok(Subfile {
            stream,
            offset,
            length,
        })
    }
}

impl<T: Read + Seek> Read for Subfile<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<T: Read + Seek> Seek for Subfile<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let pos = match pos {
            SeekFrom::Start(offset) => SeekFrom::Start(offset + self.offset),
            SeekFrom::End(offset) => {
                if (self.length as i64 - offset) < self.offset as i64 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "newpos would be < self.offset",
                    ));
                }
                pos
            }
            _ => pos,
        };

        let newpos = self.stream.seek(pos)?;
        Ok(newpos - self.offset)
    }
}

impl<R> MediaSource for Subfile<R>
where
    R: Read + Seek + Send + Sync,
{
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.length)
    }
}
