use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use parking_lot::Mutex;
use rodio::OutputStream;
use rodio::Source;
use sink::{Sink, SourceOptions};
use source::async_ring::{AsyncRingSource, AsyncRingSourceProvider, SeekData};
use std::num::{NonZeroU16, NonZeroUsize};
use stream_download::http::{
    reqwest::{
        header::{HeaderMap, HeaderValue},
        Client,
    },
    HttpStream,
};
use stream_download::source::SourceStream;
use stream_download::storage::bounded::BoundedStorageProvider;
use stream_download::storage::memory::MemoryStorageProvider;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings as StreamSettings, StreamDownload};
use symphonia::core::io::{
    MediaSource, MediaSourceStream, MediaSourceStreamOptions, ReadOnlySource,
};
use termusiclib::config::SharedServerSettings;
use termusiclib::track::{MediaType, Track};
use tokio::runtime::Handle;
use tokio::select;

use crate::{MediaInfo, PlayerCmd, PlayerProgress, PlayerTrait, Speed, Volume};
use decoder::buffered_source::BufferedSource;
use decoder::read_seek_source::ReadSeekSource;
use decoder::{MediaTitleRx, MediaTitleType, Symphonia};

mod decoder;
mod icy_metadata;
mod sink;
// public to bench lower modules
pub(crate) mod source;

pub type TotalDuration = Option<Duration>;
pub type ArcTotalDuration = Arc<Mutex<TotalDuration>>;

#[derive(Clone, Debug)]
pub enum PlayerInternalCmd {
    MessageOnEnd,
    /// Enqueue a new track to be played, and skip to it
    /// (Track, gapless, soundtouch)
    Play(Box<Track>, bool, bool),
    Progress(Duration),
    /// Enqueue a new track to be played, but do not skip current track
    /// (Track, gapless, soundtouch)
    QueueNext(Box<Track>, bool, bool),
    Resume,
    SeekAbsolute(Duration),
    SeekRelative(i64),
    Skip,
    Speed(i32),
    Stop,
    TogglePause,
    Volume(u16),
    Eos,
}
pub struct RustyBackend {
    volume: Arc<AtomicU16>,
    speed: i32,
    gapless: bool,
    command_tx: Sender<PlayerInternalCmd>,
    position: Arc<Mutex<Duration>>,
    total_duration: ArcTotalDuration,
    media_title: Arc<Mutex<String>>,
    pub radio_downloaded: Arc<Mutex<u64>>,
    // cmd_tx_outside: crate::PlayerCmdSender,
    config: SharedServerSettings,
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
impl RustyBackend {
    #[allow(clippy::similar_names)]
    #[allow(clippy::too_many_lines)]
    pub fn new(config: SharedServerSettings, cmd_tx: crate::PlayerCmdSender) -> Self {
        let config_read = config.read();
        let (picmd_tx, picmd_rx): (Sender<PlayerInternalCmd>, Receiver<PlayerInternalCmd>) =
            mpsc::channel();
        let picmd_tx_local = picmd_tx.clone();
        let volume = Arc::new(AtomicU16::from(config_read.settings.player.volume));
        let volume_local = volume.clone();
        let speed = config_read.settings.player.speed;
        let gapless = config_read.settings.player.gapless;
        drop(config_read);
        let position = Arc::new(Mutex::new(Duration::default()));
        let total_duration = Arc::new(Mutex::new(None));
        let total_duration_local = total_duration.clone();
        let position_local = position.clone();
        let pcmd_tx_local = cmd_tx;
        let media_title = Arc::new(Mutex::new(String::new()));
        let media_title_local = media_title.clone();
        let radio_downloaded = Arc::new(Mutex::new(100_u64));
        // let radio_downloaded_local = radio_downloaded.clone();
        // this should likely be a parameter, but works for now
        let tokio_handle = Handle::current();

        std::thread::Builder::new()
            .name("playback player loop".into())
            .spawn(move || {
                tokio_handle.block_on(player_thread(
                    total_duration_local,
                    pcmd_tx_local,
                    picmd_tx_local,
                    picmd_rx,
                    media_title_local,
                    // radio_downloaded_local,
                    position_local,
                    volume_local,
                    speed,
                ));
            })
            .expect("failed to spawn thread");

        Self {
            total_duration,
            volume,
            speed,
            gapless,
            command_tx: picmd_tx,
            position,
            media_title,
            radio_downloaded,
            // cmd_tx_outside: cmd_tx,
            config,
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn command(&self, cmd: PlayerInternalCmd) {
        if let Err(e) = self.command_tx.send(cmd.clone()) {
            error!("error in {cmd:?}: {e}");
        }
    }

    pub fn message_on_end(&self) {
        self.command(PlayerInternalCmd::MessageOnEnd);
    }
}

#[async_trait]
impl PlayerTrait for RustyBackend {
    async fn add_and_play(&mut self, track: &Track) {
        let soundtouch = self
            .config
            .read_recursive()
            .settings
            .backends
            .rusty
            .soundtouch;
        self.command(PlayerInternalCmd::Play(
            Box::new(track.clone()),
            self.gapless,
            soundtouch,
        ));
        self.resume();
    }

    fn volume(&self) -> Volume {
        self.volume.load(Ordering::SeqCst)
    }

    fn set_volume(&mut self, volume: Volume) -> Volume {
        let volume = volume.min(100);
        self.volume.store(volume, Ordering::SeqCst);
        self.command(PlayerInternalCmd::Volume(volume));

        volume
    }

    fn pause(&mut self) {
        self.command(PlayerInternalCmd::TogglePause);
    }

    fn resume(&mut self) {
        self.command(PlayerInternalCmd::Resume);
    }

    fn is_paused(&self) -> bool {
        // self.sink.is_paused()
        false
    }

    fn seek(&mut self, offset: i64) -> Result<()> {
        self.command(PlayerInternalCmd::SeekRelative(offset));
        Ok(())
    }

    #[allow(clippy::cast_possible_wrap)]
    fn seek_to(&mut self, position: Duration) {
        self.command(PlayerInternalCmd::SeekAbsolute(position));
    }

    fn set_speed(&mut self, speed: Speed) -> Speed {
        self.speed = speed;
        self.command(PlayerInternalCmd::Speed(speed));

        self.speed
    }

    fn speed(&self) -> Speed {
        self.speed
    }

    fn stop(&mut self) {
        self.command(PlayerInternalCmd::Stop);
    }

    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_wrap)]
    fn get_progress(&self) -> Option<PlayerProgress> {
        Some(PlayerProgress {
            position: Some(*self.position.lock()),
            total_duration: *self.total_duration.lock(),
        })
    }

    fn gapless(&self) -> bool {
        self.gapless
    }

    fn set_gapless(&mut self, to: bool) {
        self.gapless = to;
    }

    fn skip_one(&mut self) {
        self.command(PlayerInternalCmd::Skip);
    }

    fn enqueue_next(&mut self, track: &Track) {
        let soundtouch = self
            .config
            .read_recursive()
            .settings
            .backends
            .rusty
            .soundtouch;
        self.command(PlayerInternalCmd::QueueNext(
            Box::new(track.clone()),
            self.gapless,
            soundtouch,
        ));
    }

    fn media_info(&self) -> MediaInfo {
        let media_title_r = self.media_title.lock();
        if media_title_r.is_empty() {
            MediaInfo::default()
        } else {
            MediaInfo {
                media_title: Some(media_title_r.clone()),
            }
        }
    }
}

/// Append the `media_source` to the `sink`, while allowing different functions to run with `func` with a [`MediaTitleRx`]
fn append_to_sink_inner_media_title<F: FnOnce(&mut Symphonia, MediaTitleRx)>(
    media_source: Box<dyn MediaSource>,
    trace: &str,
    sink: &Sink,
    gapless: bool,
    soundtouch: bool,
    func: F,
) {
    let mss = MediaSourceStream::new(media_source, MediaSourceStreamOptions::default());
    match Symphonia::new_with_media_title(mss, gapless) {
        Ok((mut decoder, rx)) => {
            func(&mut decoder, rx);

            let handle = tokio::runtime::Handle::current();
            let (spec, current_frame_len) = decoder.get_spec();
            let total_duration = decoder.total_duration();
            let (prod, cons) =
                AsyncRingSource::new(spec, total_duration, current_frame_len, 0, handle.clone());

            tokio::task::spawn_blocking(move || {
                handle.block_on(decode_task(decoder, prod));
            });

            sink.append(cons, &SourceOptions { soundtouch });
            // sink.append(decoder, &SourceOptions { soundtouch });
        }
        Err(e) => error!("error decoding '{trace}' is: {e:?}"),
    }
}

/// The task that runs the decoder and writes to the ringbuffer, until a error or the consumer closes.
async fn decode_task(mut decoder: Symphonia, mut prod: AsyncRingSourceProvider) -> Option<()> {
    loop {
        // will always write the full buffer as long as the consumer is connected
        let seek_fut = prod.wait_seek();
        let exhausted_buffer = decoder.exhausted_buffer();
        let buffer = if exhausted_buffer {
            &[]
        } else {
            decoder.get_buffer_u8()
        };
        let write_fut = prod.write_data(buffer);

        select! {
            // if there is nothing to write, this future may exit immediately, causing a fast-loop until seek or dropped.
            written = write_fut, if !exhausted_buffer => {
                written.ok()?;
                decoder.advance_offset(decoder.get_buffer().len());
            },
            seek = seek_fut => {
                let seek = seek?;
                decode_task_seek_fut(&mut decoder, &mut prod, seek).await?;
            }
        }

        let spec_len = decoder.get_spec();
        if decoder.decode_once().is_none() {
            trace!("Sending EOS");
            prod.new_eos().await.ok()?;
        }
        let new_spec = decoder.get_spec();
        if spec_len != new_spec {
            prod.new_spec(new_spec.0, new_spec.1).await.ok()?;
        }
    }
}

/// Handle the result of the seek future.
///
/// This is a non-inlined function, because writing inside a macro(`select!`) is a pain.
///
/// Return [`Some`] if seek was successful, [`None`] otherwise.
async fn decode_task_seek_fut(
    decoder: &mut Symphonia,
    prod: &mut AsyncRingSourceProvider,
    seek_data: SeekData,
) -> Option<()> {
    trace!("Seeking Decoder");
    decoder.try_seek(seek_data.0).ok()?;

    let spec = decoder.get_spec();
    prod.process_seek(spec.0, spec.1, seek_data.1).await;

    Some(())
}

/// Append the `media_source` to the `sink`, while allowing different functions to run with `func`
fn append_to_sink_inner<F: FnOnce(&mut Symphonia)>(
    media_source: Box<dyn MediaSource>,
    trace: &str,
    sink: &Sink,
    gapless: bool,
    soundtouch: bool,
    func: F,
) {
    let mss = MediaSourceStream::new(media_source, MediaSourceStreamOptions::default());
    match Symphonia::new(mss, gapless) {
        Ok(mut decoder) => {
            func(&mut decoder);
            sink.append(decoder, &SourceOptions { soundtouch });
        }
        Err(e) => error!("error decoding '{trace}' is: {e:?}"),
    }
}

/// Append the `media_source` to the `sink`, while also setting `total_duration*`
///
/// Expects current thread to have a tokio handle
fn append_to_sink<MT: Fn(MediaTitleType) + Send + 'static>(
    media_source: Box<dyn MediaSource>,
    trace: &str,
    sink: &Sink,
    gapless: bool,
    total_duration_local: &ArcTotalDuration,
    soundtouch: bool,
    media_title_fn: MT,
) {
    append_to_sink_inner_media_title(
        media_source,
        trace,
        sink,
        gapless,
        soundtouch,
        |decoder, mut media_title_rx| {
            std::mem::swap(
                &mut *total_duration_local.lock(),
                &mut decoder.total_duration(),
            );

            let handle = Handle::current();

            handle.spawn(async move {
                while let Some(cmd) = media_title_rx.recv().await {
                    media_title_fn(cmd);
                }
            });
        },
    );
}

/// Append the `media_source` to the `sink`, while setting duration to be unknown (to [`None`])
fn append_to_sink_no_duration(
    media_source: Box<dyn MediaSource>,
    trace: &str,
    sink: &Sink,
    gapless: bool,
    total_duration_local: &ArcTotalDuration,
    soundtouch: bool,
) {
    append_to_sink_inner(media_source, trace, sink, gapless, soundtouch, |_| {
        // remove old stale duration
        total_duration_local.lock().take();
    });
}

/// Append the `media_source` to the `sink`, while also setting `next_duration_opt`
///
/// This is used for enqueued entries which do not start immediately
///
/// Expects current thread to have a tokio handle
fn append_to_sink_queue<MT: Fn(MediaTitleType) + Send + 'static>(
    media_source: Box<dyn MediaSource>,
    trace: &str,
    sink: &Sink,
    gapless: bool,
    // total_duration_local: &ArcTotalDuration,
    next_duration_opt: &mut Option<Duration>,
    soundtouch: bool,
    media_title_fn: MT,
) {
    append_to_sink_inner_media_title(
        media_source,
        trace,
        sink,
        gapless,
        soundtouch,
        |decoder, mut media_title_rx| {
            std::mem::swap(next_duration_opt, &mut decoder.total_duration());
            // rely on EOS message to set next duration
            sink.message_on_end();

            let handle = Handle::current();

            handle.spawn(async move {
                while let Some(cmd) = media_title_rx.recv().await {
                    media_title_fn(cmd);
                }
            });
        },
    );
}

/// Append the `media_source` to the `sink`, while also setting `next_duration_opt` to be unknown (to [`None`])
///
/// This is used for enqueued entries which do not start immediately
fn append_to_sink_queue_no_duration(
    media_source: Box<dyn MediaSource>,
    trace: &str,
    sink: &Sink,
    gapless: bool,
    // total_duration_local: &ArcTotalDuration,
    next_duration_opt: &mut Option<Duration>,
    soundtouch: bool,
) {
    append_to_sink_inner(media_source, trace, sink, gapless, soundtouch, |_| {
        // remove potential old stale duration
        next_duration_opt.take();
        // rely on EOS message to set next duration
        sink.message_on_end();
    });
}

/// Common handling of a `media_title` for a [`append_to_sink`] function
fn common_media_title_cb(media_title: Arc<Mutex<String>>) -> impl Fn(MediaTitleType) {
    move |cmd| match cmd {
        MediaTitleType::Reset => media_title.lock().clear(),
        MediaTitleType::Value(v) => *media_title.lock() = v,
    }
}

/// Player thread loop
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_pass_by_value,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]
async fn player_thread(
    total_duration: ArcTotalDuration,
    pcmd_tx: crate::PlayerCmdSender,
    picmd_tx: Sender<PlayerInternalCmd>,
    picmd_rx: Receiver<PlayerInternalCmd>,
    media_title: Arc<Mutex<String>>,
    // radio_downloaded: Arc<Mutex<u64>>,
    position: Arc<Mutex<Duration>>,
    volume_inside: Arc<AtomicU16>,
    mut speed_inside: i32,
) {
    let mut is_radio = false;

    // option to store enqueued's duration
    // note that the current implementation is only meant to have 1 enqueued next after the current playing song
    let mut next_duration_opt = None;
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&handle, picmd_tx.clone(), pcmd_tx.clone()).unwrap();
    sink.set_speed(speed_inside as f32 / 10.0);
    sink.set_volume(f32::from(volume_inside.load(Ordering::SeqCst)) / 100.0);
    loop {
        let Ok(cmd) = picmd_rx.recv() else {
            // only error can be a disconnect (no more senders)
            break;
        };

        match cmd {
            PlayerInternalCmd::Play(track, gapless, soundtouch) => {
                if let Err(err) = queue_next(
                    &track,
                    gapless,
                    &sink,
                    &mut is_radio,
                    &total_duration,
                    &mut next_duration_opt,
                    &media_title,
                    // &radio_downloaded,
                    false,
                    soundtouch,
                )
                .await
                {
                    error!("Failed to play track: {err:#?}");
                }
            }
            PlayerInternalCmd::TogglePause => {
                sink.toggle_playback();
            }
            PlayerInternalCmd::QueueNext(track, gapless, soundtouch) => {
                if let Err(err) = queue_next(
                    &track,
                    gapless,
                    &sink,
                    &mut is_radio,
                    &total_duration,
                    &mut next_duration_opt,
                    &media_title,
                    // &radio_downloaded,
                    true,
                    soundtouch,
                )
                .await
                {
                    error!("Failed to queue next track: {err:#?}");
                }
            }
            PlayerInternalCmd::Resume => {
                sink.play();
            }
            PlayerInternalCmd::Speed(speed) => {
                speed_inside = speed;
                sink.set_speed(speed_inside as f32 / 10.0);
            }
            PlayerInternalCmd::Stop => {
                sink.stop();
            }
            PlayerInternalCmd::Volume(volume) => {
                sink.set_volume(f32::from(volume) / 100.0);
                volume_inside.store(volume, Ordering::SeqCst);
            }
            PlayerInternalCmd::Skip => {
                // the sink can be empty, if for example nothing could be enqueued, so a "skip_one" would be a no-op and never send EOS, which is required to go to the next track
                if sink.is_empty() {
                    let _ = picmd_tx.send(PlayerInternalCmd::Eos);
                    let _ = pcmd_tx.send(PlayerCmd::Eos);
                } else {
                    sink.skip_one();
                }
                if sink.is_paused() {
                    sink.play();
                }
            }
            PlayerInternalCmd::Progress(new_position) => {
                // let position = sink.elapsed().as_secs() as i64;
                // error!("position in rusty backend is: {}", position);
                *position.lock() = new_position;

                // About to finish signal is a simulation of gstreamer, and used for gapless
                if !is_radio {
                    if let Some(d) = *total_duration.lock() {
                        let progress = new_position.as_secs_f64() / d.as_secs_f64();
                        if progress >= 0.5
                            && d.saturating_sub(new_position) < Duration::from_secs(2)
                        {
                            if let Err(e) = pcmd_tx.send(PlayerCmd::AboutToFinish) {
                                error!("command AboutToFinish sent failed: {e}");
                            }
                        }
                    }
                }
            }
            PlayerInternalCmd::SeekAbsolute(position) => {
                sink.seek(position);
            }
            PlayerInternalCmd::MessageOnEnd => {
                sink.message_on_end();
            }

            PlayerInternalCmd::SeekRelative(offset) => {
                let paused = sink.is_paused();
                if paused {
                    sink.set_volume(0.0);
                }
                if offset.is_positive() {
                    let new_pos = sink.elapsed().as_secs() + offset as u64;
                    if let Some(d) = *total_duration.lock() {
                        if new_pos < d.as_secs() - offset as u64 {
                            sink.seek(Duration::from_secs(new_pos));
                        }
                    }
                } else {
                    let new_pos = sink
                        .elapsed()
                        .as_secs()
                        .saturating_sub(offset.unsigned_abs());
                    sink.seek(Duration::from_secs(new_pos));
                }
                if paused {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    sink.pause();
                    sink.set_volume(f32::from(volume_inside.load(Ordering::SeqCst)) / 100.0);
                }
            }

            PlayerInternalCmd::Eos => {
                // replace the current total_duration with the next one
                // this is only present when QueueNext was used; which is only used if gapless is enabled
                if next_duration_opt.is_some() {
                    *total_duration.lock() = next_duration_opt;
                }
            }
        }
    }
}

/// Queue the given track into the [`Sink`], while also setting all of the other variables
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
async fn queue_next(
    track: &Track,
    gapless: bool,
    sink: &Sink,

    is_radio: &mut bool,
    total_duration: &ArcTotalDuration,
    next_duration_opt: &mut Option<Duration>,
    media_title: &Arc<Mutex<String>>,
    enqueue: bool,

    soundtouch: bool,
) -> Result<()> {
    let media_type = &track.media_type;
    let file_path = track
        .file()
        .ok_or_else(|| anyhow!("No file path found"))?
        .to_owned();
    match media_type {
        MediaType::Music => {
            *is_radio = false;
            let file = File::open(Path::new(&file_path)).context("Failed to open music file")?;

            if enqueue {
                append_to_sink_queue(
                    Box::new(BufferedSource::new_default_size(file)),
                    &file_path,
                    sink,
                    gapless,
                    next_duration_opt,
                    soundtouch,
                    common_media_title_cb(media_title.clone()),
                );
            } else {
                append_to_sink(
                    Box::new(BufferedSource::new_default_size(file)),
                    &file_path,
                    sink,
                    gapless,
                    total_duration,
                    soundtouch,
                    common_media_title_cb(media_title.clone()),
                );
            }

            Ok(())
        }

        MediaType::Podcast => {
            *is_radio = false;
            if let Some(file_path) = track.podcast_localfile.clone() {
                let file = File::open(Path::new(&file_path))
                    .context("Failed to open local podcast file")?;
                if enqueue {
                    append_to_sink_queue(
                        Box::new(BufferedSource::new_default_size(file)),
                        &file_path,
                        sink,
                        gapless,
                        next_duration_opt,
                        soundtouch,
                        common_media_title_cb(media_title.clone()),
                    );
                } else {
                    append_to_sink(
                        Box::new(BufferedSource::new_default_size(file)),
                        &file_path,
                        sink,
                        gapless,
                        total_duration,
                        soundtouch,
                        common_media_title_cb(media_title.clone()),
                    );
                }
                return Ok(());
            }

            let url = file_path;
            let settings = StreamSettings::default();

            let stream = HttpStream::<Client>::create(url.parse()?).await?;

            let file_len = stream.content_length();

            let reader = StreamDownload::from_stream(
                stream,
                TempStorageProvider::with_prefix(".termusic-stream-cache-"),
                settings,
            )
            .await?;

            if enqueue {
                append_to_sink_queue(
                    Box::new(ReadSeekSource::new(reader, file_len)),
                    &url,
                    sink,
                    gapless,
                    next_duration_opt,
                    soundtouch,
                    common_media_title_cb(media_title.clone()),
                );
            } else {
                append_to_sink(
                    Box::new(ReadSeekSource::new(reader, file_len)),
                    &url,
                    sink,
                    gapless,
                    total_duration,
                    soundtouch,
                    common_media_title_cb(media_title.clone()),
                );
            }
            Ok(())
        }

        MediaType::LiveRadio => {
            *is_radio = true;
            let url = file_path;
            let settings = StreamSettings::default();

            let mut headers = HeaderMap::new();
            headers.insert("icy-metadata", HeaderValue::from_static("1"));
            let client = Client::builder().default_headers(headers).build().unwrap();

            let stream = HttpStream::new(client, url.parse()?).await?;

            let meta_interval: Option<NonZeroU16> = stream
                .header("icy-metaint")
                .and_then(|v| v.parse().ok())
                .and_then(NonZeroU16::new);
            let icy_description = stream.header("icy-description").map(ToString::to_string);

            let reader = StreamDownload::from_stream(
                stream,
                BoundedStorageProvider::new(
                    MemoryStorageProvider,
                    // ensure we have enough buffer space to store the prefetch data
                    NonZeroUsize::new(usize::try_from(settings.get_prefetch_bytes() * 2)?).unwrap(),
                ),
                settings,
            )
            .await?;
            // The following comment block is useful if wanting to re-play a already downloaded stream with known data.
            // this is mainly used if not wanting to have a actual connection open, or when trying to debug offsets.
            // it is recommended to comment-out the above "reader" and "meta_interval" (including dependencies) before using this
            // // curl -H "icy-metadata: 1" -L https://tostation -o testing_stream -D testing_stream_headers
            // let reader = std::io::BufReader::new(File::open("/tmp/testing_stream").unwrap());
            // // Modify this to what the actual headers said
            // let meta_interval = 8192;

            let media_title_clone = media_title.clone();

            let cb = move |title: &str| {
                let new_title = if title.is_empty() {
                    "<no title>".to_string()
                } else {
                    title.to_string()
                };

                *media_title_clone.lock() = new_title;
            };

            // set initial title to what the header says
            if let Some(icy_description) = icy_description {
                cb(&icy_description);
            }

            let media_source: Box<dyn MediaSource> = if let Some(meta_interval) = meta_interval {
                Box::new(ReadOnlySource::new(
                    icy_metadata::FilterOutIcyMetadata::new(reader, cb, meta_interval),
                ))
            } else {
                info!("No Icy-MetaInt!");
                Box::new(ReadOnlySource::new(reader))
            };

            if enqueue {
                append_to_sink_queue_no_duration(
                    media_source,
                    &url,
                    sink,
                    gapless,
                    next_duration_opt,
                    soundtouch,
                );
            } else {
                append_to_sink_no_duration(
                    media_source,
                    &url,
                    sink,
                    gapless,
                    total_duration,
                    soundtouch,
                );
            }

            Ok(())
        }
    }
}
