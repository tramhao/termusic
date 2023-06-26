use async_trait::async_trait;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use parking_lot::{Condvar, Mutex, RwLock, RwLockReadGuard};
use rangemap::RangeSet;
use std::{
    error::Error,
    fs::File,
    io::{self, BufWriter, Seek, SeekFrom, Write},
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};
use tap::TapFallible;
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace};

#[async_trait]
pub trait SourceStream:
    Stream<Item = Result<Bytes, Self::Error>> + Unpin + Send + Sync + Sized + 'static
{
    type Url: Send;
    type Error: Error + Send;

    async fn create(url: Self::Url) -> io::Result<Self>;
    async fn content_length(&self) -> Option<u64>;
    async fn seek_range(&mut self, start: u64, end: Option<u64>) -> io::Result<()>;
}

#[derive(Debug, Clone)]
pub struct SourceHandle {
    downloaded: Arc<RwLock<RangeSet<u64>>>,
    requested_position: Arc<AtomicI64>,
    position_reached: Arc<(Mutex<Waiter>, Condvar)>,
    content_length_retrieved: Arc<(Mutex<bool>, Condvar)>,
    content_length: Arc<AtomicI64>,
    seek_tx: mpsc::Sender<u64>,
}

impl SourceHandle {
    pub fn downloaded(&self) -> RwLockReadGuard<rangemap::RangeSet<u64>> {
        self.downloaded.read()
    }

    pub fn request_position(&self, position: u64) {
        self.requested_position
            .store(position as i64, Ordering::SeqCst);
    }

    pub fn wait_for_requested_position(&self) {
        let (mutex, cvar) = &*self.position_reached;
        let mut waiter = mutex.lock();
        if !waiter.stream_done {
            debug!("Waiting for requested position");
            cvar.wait_while(&mut waiter, |waiter| {
                !waiter.stream_done && !waiter.position_reached
            });
            if !waiter.stream_done {
                waiter.position_reached = false;
            }
            debug!("Position reached");
        }
    }

    pub fn seek(&self, position: u64) {
        self.seek_tx.try_send(position).ok();
    }

    pub fn content_length(&self) -> Option<u64> {
        let (mutex, cvar) = &*self.content_length_retrieved;
        let mut done = mutex.lock();
        if !*done {
            cvar.wait_while(&mut done, |done| !*done);
        }
        let length = self.content_length.load(Ordering::SeqCst);
        if length > -1 {
            Some(length as u64)
        } else {
            None
        }
    }
}

#[derive(Default, Debug)]
struct Waiter {
    position_reached: bool,
    stream_done: bool,
}

pub struct Source {
    writer: BufWriter<File>,
    downloaded: Arc<RwLock<RangeSet<u64>>>,
    requested_position: Arc<AtomicI64>,
    position_reached: Arc<(Mutex<Waiter>, Condvar)>,
    content_length_retrieved: Arc<(Mutex<bool>, Condvar)>,
    content_length: Arc<AtomicI64>,
    seek_tx: mpsc::Sender<u64>,
    seek_rx: mpsc::Receiver<u64>,
}

const PREFETCH_BYTES: u64 = 1024 * 256;

impl Source {
    pub fn new(tempfile: File) -> Self {
        let (seek_tx, seek_rx) = mpsc::channel(32);
        Self {
            writer: BufWriter::new(tempfile),
            downloaded: Default::default(),
            requested_position: Arc::new(AtomicI64::new(-1)),
            position_reached: Default::default(),
            content_length_retrieved: Default::default(),
            seek_tx,
            seek_rx,
            content_length: Default::default(),
        }
    }

    pub async fn download<S: SourceStream>(mut self, mut stream: S) -> io::Result<()> {
        info!("Starting file download");
        let content_length = stream.content_length().await;
        if let Some(content_length) = content_length {
            self.content_length
                .swap(content_length as i64, Ordering::SeqCst);
        } else {
            self.content_length.swap(-1, Ordering::SeqCst);
        }
        {
            let (mutex, cvar) = &*self.content_length_retrieved;
            *mutex.lock() = true;
            cvar.notify_all();
        }
        loop {
            if let Some(Ok(bytes)) = stream
                .next()
                .await
                .map(|b| b.tap_err(|e| error!("Error reading stream: {e}")))
            {
                self.writer.write_all(&bytes)?;
                let stream_position = self.writer.stream_position()?;
                trace!("Prefetch: {}/{} bytes", stream_position, PREFETCH_BYTES);
                if stream_position >= PREFETCH_BYTES {
                    self.downloaded.write().insert(0..stream_position);
                    break;
                }
            } else {
                info!("File shorter than prefetch length");
                self.writer.flush()?;
                self.downloaded
                    .write()
                    .insert(0..self.writer.stream_position()?);
                let (mutex, cvar) = &*self.position_reached;
                (mutex.lock()).stream_done = true;
                cvar.notify_all();
                return Ok(());
            }
        }
        info!("Prefetch complete");
        loop {
            tokio::select! {
                bytes = stream.next() => {
                    if let Some(Ok(bytes)) =
                        bytes.map(|b| b.tap_err(|e| error!("Error reading from stream: {e}"))) {
                        let position = self.writer.stream_position()?;
                        self.writer.write_all(&bytes)?;
                        let new_position = self.writer.stream_position()?;
                        trace!("Received response chunk. position={}", new_position);
                        self.downloaded.write().insert(position .. new_position);
                        let requested = self.requested_position.load(Ordering::SeqCst);
                        if requested > -1 {
                            debug!("downloader: requested {requested} current {}", new_position);
                        }
                        if requested > -1 && new_position as i64 >= requested {
                            info!("Notifying requested position reached: {requested}. New position: {new_position}");
                            self.requested_position.store(-1, Ordering::SeqCst);
                            let (mutex, cvar) = &*self.position_reached;
                            (mutex.lock()).position_reached = true;
                            cvar.notify_all();
                        }
                    } else {
                        info!("Stream finished downloading");
                        if let Some(content_length) = content_length {
                            let gap = {
                                let downloaded = self.downloaded.read();
                                let range = 0 .. content_length;
                                let mut gaps = downloaded.gaps(&range);
                                gaps.next()
                            };
                            if let Some(gap) = gap {
                                debug!("Downloading missing stream chunk: {gap:?}.");
                                stream.seek_range(gap.start, Some(gap.end)).await?;
                                self.writer.seek(SeekFrom::Start(gap.start))?;
                                continue;
                            }
                        }
                        self.writer.flush()?;
                        let (mutex, cvar) = &*self.position_reached;
                        (mutex.lock()).stream_done = true;
                        cvar.notify_all();
                        return Ok(());
                    }
                },
                pos = self.seek_rx.recv() => {
                    if let Some(pos) = pos {
                        debug!("Received seek position {pos}");
                        let do_seek = {
                            let downloaded = self.downloaded.read();
                            if let Some(range) = downloaded.get(&pos) {
                                !range.contains(&self.writer.stream_position()?)
                            } else {
                                true
                            }
                        };
                        if do_seek {
                            debug!("Seek position not yet downloaded");
                            stream.seek_range(pos, None).await?;
                            self.writer.seek(SeekFrom::Start(pos))?;
                        }
                    }
                }
            }
        }
    }

    pub fn source_handle(&self) -> SourceHandle {
        SourceHandle {
            downloaded: self.downloaded.clone(),
            requested_position: self.requested_position.clone(),
            position_reached: self.position_reached.clone(),
            seek_tx: self.seek_tx.clone(),
            content_length_retrieved: self.content_length_retrieved.clone(),
            content_length: self.content_length.clone(),
        }
    }
}
