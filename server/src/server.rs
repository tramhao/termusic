mod cli;
mod logger;
mod music_player_service;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context as _, Result};
use clap::Parser;
use music_player_service::MusicPlayerService;
use parking_lot::Mutex;
use termusiclib::config::v2::server::config_extra::ServerConfigVersionedDefaulted;
use termusiclib::config::v2::server::{ComProtocol, ScanDepth};
use termusiclib::config::{new_shared_server_settings, ServerOverlay, SharedServerSettings};
use termusiclib::player::music_player_server::MusicPlayerServer;
use termusiclib::player::{GetProgressResponse, PlayerProgress, PlayerTime, RunningStatus};
use termusiclib::track::MediaTypesSimple;
use termusiclib::{podcast, utils};
use termusicplayback::{
    Backend, BackendSelect, GeneralPlayer, PlayerCmd, PlayerCmdReciever, PlayerCmdSender,
    PlayerTrait, Playlist, SharedPlaylist, SpeedSigned, VolumeSigned,
};
use tokio::runtime::Handle;
use tokio::select;
use tokio::sync::{broadcast, oneshot};
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;
use tonic::transport::server::TcpIncoming;
use tonic::transport::Server;

#[macro_use]
extern crate log;

pub const MAX_DEPTH: usize = 4;
pub const VOLUME_STEP: VolumeSigned = 5;
pub const SPEED_STEP: SpeedSigned = 1;

/// Stats for the music player responses
#[derive(Debug, Clone, PartialEq)]
struct PlayerStats {
    pub progress: PlayerProgress,
    pub current_track_index: u64,
    pub status: u32,
    pub volume: u16,
    pub speed: i32,
    pub gapless: bool,
    pub current_track_updated: bool,
    pub radio_title: String,
}

impl PlayerStats {
    pub fn new() -> Self {
        Self {
            progress: PlayerProgress {
                position: None,
                total_duration: None,
            },
            current_track_index: 0,
            status: 1,
            volume: 0,
            speed: 10,
            gapless: true,
            current_track_updated: false,
            radio_title: String::new(),
        }
    }

    pub fn as_getprogress_response(&self) -> GetProgressResponse {
        GetProgressResponse {
            progress: Some(self.as_playertime()),
            current_track_index: self.current_track_index,
            status: self.status,
            volume: u32::from(self.volume),
            speed: self.speed,
            gapless: self.gapless,
            current_track_updated: self.current_track_updated,
            radio_title: self.radio_title.clone(),
        }
    }

    pub fn as_playertime(&self) -> PlayerTime {
        self.progress.into()
    }
}

fn main() -> Result<()> {
    // print error to the log and then throw it
    if let Err(err) = actual_main() {
        error!("Error: {:?}", err);
        return Err(err);
    }

    Ok(())
}

#[tokio::main]
async fn actual_main() -> Result<()> {
    let args = cli::Args::parse();
    let _ = logger::setup(&args);
    let config = get_config(&args)?;

    if let Some(action) = args.action {
        return execute_action(action, &config);
    }

    info!("Server starting...");

    // do this before anything else so that we exit early on invalid/unavailable backends
    let backend = {
        let config_backend = config.settings.player.backend.try_into()?;

        if let Some(backend) = args.backend {
            trace!("Backend from CLI");
            backend.into()
        } else {
            trace!("Backend from Config");
            config_backend
        }
    };

    let config = new_shared_server_settings(config);
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = PlayerCmdSender::new(cmd_tx);
    // Note that the channel size might quickly become too low if there is a massive delete (like removing the non-existent tracks from the playlist)
    let (stream_tx, _) = broadcast::channel(10);

    let playlist =
        Playlist::new_shared(&config, stream_tx.clone()).context("Failed to load playlist")?;

    let music_player_service: MusicPlayerService = MusicPlayerService::new(
        cmd_tx.clone(),
        stream_tx.clone(),
        config.clone(),
        playlist.clone(),
    );
    let playerstats = music_player_service.player_stats.clone();

    let cmd_tx_ctrlc = cmd_tx.clone();
    let cmd_tx_ticker = cmd_tx.clone();

    ctrlc::set_handler(move || {
        cmd_tx_ctrlc
            .send(PlayerCmd::Quit)
            .expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");

    let service_cancel_token = CancellationToken::new();

    let join_handle =
        start_service(&config, music_player_service, service_cancel_token.clone()).await?;

    let tokio_handle = Handle::current();

    let cancel_token = service_cancel_token.clone();
    let playlist_c = playlist.clone();
    start_playlist_save_interval(tokio_handle.clone(), cancel_token, playlist_c);

    let (player_handle_os_tx, player_handle_os_rx) = oneshot::channel();
    let player_handle = std::thread::Builder::new()
        .name("main player loop".into())
        .spawn(move || {
            let _guard = tokio_handle.enter();
            let res = player_loop(
                backend,
                cmd_tx,
                cmd_rx,
                config,
                playerstats,
                stream_tx,
                playlist,
            );
            let _ = player_handle_os_tx.send(res);
        })?;

    ticker_thread(cmd_tx_ticker)?;

    info!("Server ready");

    // await the oneshot completing in a async fashion
    player_handle_os_rx.await??;
    // do this *after* the oneshot, because this is a blocking operation
    // and by doing this after the oneshot we can be sure the thread is actually exited, or exiting
    let _ = player_handle.join();

    // ensure cleanup of the service tasks happens before main exits
    service_cancel_token.cancel();
    let _ = join_handle.await;

    // Graceful exit log
    info!("Bye");

    Ok(())
}

const PLAYLIST_SAVE_INTERVAL: Duration = Duration::from_secs(30);

/// Spawn a task to periodically save the playlist to disk, if modified.
fn start_playlist_save_interval(
    handle: Handle,
    cancel_token: CancellationToken,
    playlist: SharedPlaylist,
) {
    handle.spawn(async move {
        let mut timer = tokio::time::interval_at(
            Instant::now() + PLAYLIST_SAVE_INTERVAL,
            PLAYLIST_SAVE_INTERVAL,
        );
        loop {
            select! {
                _ = timer.tick() => {
                    match playlist.write().save_if_modified() {
                        Err(err) => warn!("Error saving playlist in interval: {err:#?}"),
                        Ok(true) => debug!("Saved playlist in interval"),
                        Ok(false) => ()
                    }
                },
                _ = cancel_token.cancelled() => {
                    break;
                }
            }
        }
    });
}

/// Start the [`MusicPlayerService`] with the according transport protocol.
async fn start_service(
    config: &SharedServerSettings,
    music_player_service: MusicPlayerService,
    cancel_token: CancellationToken,
) -> Result<JoinHandle<Result<(), tonic::transport::Error>>> {
    // otherwise the MutexGuard would be held across await points
    let protocol = config.read().settings.com.protocol;
    let handle = match protocol {
        ComProtocol::HTTP => {
            let (tcp_stream, addr) = tcp_stream(config).await?;
            info!("Server listening on {addr}");

            tokio::spawn(
                Server::builder()
                    .add_service(MusicPlayerServer::new(music_player_service))
                    .serve_with_incoming_shutdown(tcp_stream, cancel_token.cancelled_owned()),
            )
        }
        #[cfg(unix)]
        ComProtocol::UDS => {
            // TODO: unlink socket file if it already exists
            let (uds_stream, addr) = uds::uds_stream(config).await?;
            info!("Server listening on {addr}");

            tokio::spawn(
                Server::builder()
                    .add_service(MusicPlayerServer::new(music_player_service))
                    .serve_with_incoming_shutdown(uds_stream, cancel_token.cancelled_owned()),
            )
        }
        #[cfg(not(unix))]
        ComProtocol::UDS => {
            // runtime error to not plaster "cfg(unix)" everywhere and because the default for those systems is "HTTP"
            // and windows in tonic(and lower) will support uds soon-ish
            unimplemented!("UDS/Unix Domain Sockets are only implemented for unix targets")
        }
    };

    Ok(handle)
}

/// Create the TCP Stream for HTTP requests.
async fn tcp_stream(config: &SharedServerSettings) -> Result<(TcpIncoming, SocketAddr)> {
    let addr = SocketAddr::from(&config.read().settings.com);

    // workaround to print address once sever "actually" is started and address is known
    // see https://github.com/hyperium/tonic/issues/351
    let tcp_listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Error binding address: {}", addr))?;

    // workaround as "TcpIncoming" does not provide a function to get the address
    let socket_addr = tcp_listener.local_addr()?;

    let stream = TcpIncoming::from(tcp_listener).with_nodelay(Some(true));

    Ok((stream, socket_addr))
}

#[cfg(unix)]
mod uds {
    use std::{
        io,
        pin::Pin,
        task::{Context, Poll},
    };

    use anyhow::{Context as _, Result};
    use termusiclib::config::SharedServerSettings;
    use tokio::net::{UnixListener, UnixStream};
    use tokio_stream::Stream;

    /// Create the UDS Stream for UDS requests.
    pub async fn uds_stream(config: &SharedServerSettings) -> Result<(UnixListenerStream, String)> {
        let path = &config.read().settings.com.socket_path;

        // if the file already exists, tokio will error with "Address already in use"
        // not using async here because of MutexGuard and being before anything important
        if path.exists() {
            warn!("Socket Path {} already exists, unlinking!", path.display());
            let _ = std::fs::remove_file(path);
        }

        let path_str = path.display().to_string();
        let uds = UnixListener::bind(path).with_context(|| path_str.clone())?;

        let stream = UnixListenerStream::new(uds);

        Ok((stream, path_str))
    }

    /// A wrapper around [`UnixListener`] that implements [`Stream`].
    ///
    /// Copied from [`tokio_stream::wrappers::UnixListenerStream`], which is licensed MIT.
    ///
    /// Modified because the normal implementation does not remove the socket on drop.
    #[derive(Debug)]
    #[cfg_attr(docsrs, doc(cfg(all(unix, feature = "net"))))]
    pub struct UnixListenerStream {
        inner: UnixListener,
    }

    impl UnixListenerStream {
        /// Create a new `UnixListenerStream`.
        pub fn new(listener: UnixListener) -> Self {
            Self { inner: listener }
        }
    }

    impl Stream for UnixListenerStream {
        type Item = io::Result<UnixStream>;

        fn poll_next(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Option<io::Result<UnixStream>>> {
            match self.inner.poll_accept(cx) {
                Poll::Ready(Ok((stream, _))) => Poll::Ready(Some(Ok(stream))),
                Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
                Poll::Pending => Poll::Pending,
            }
        }
    }

    impl AsRef<UnixListener> for UnixListenerStream {
        fn as_ref(&self) -> &UnixListener {
            &self.inner
        }
    }

    impl AsMut<UnixListener> for UnixListenerStream {
        fn as_mut(&mut self) -> &mut UnixListener {
            &mut self.inner
        }
    }

    impl Drop for UnixListenerStream {
        fn drop(&mut self) {
            // unlink socket file as it is not done so by default
            let tmp = self.inner.local_addr().ok();
            if let Some(val) = tmp.as_ref().and_then(|v| v.as_pathname()) {
                let _ = std::fs::remove_file(val);
            }
        }
    }
}

/// The main player loop where we handle all events
fn player_loop(
    backend: BackendSelect,
    cmd_tx: PlayerCmdSender,
    mut cmd_rx: PlayerCmdReciever,
    config: SharedServerSettings,
    playerstats: Arc<Mutex<PlayerStats>>,
    stream_tx: termusicplayback::StreamTX,
    playlist: SharedPlaylist,
) -> Result<()> {
    let mut player = GeneralPlayer::new_backend(backend, config, cmd_tx, stream_tx, playlist)?;
    while let Some((cmd, cb)) = cmd_rx.blocking_recv() {
        #[allow(unreachable_patterns)]
        match cmd {
            PlayerCmd::AboutToFinish => {
                info!("about to finish signal received");
                let playlist = player.playlist.read();
                if !playlist.is_empty()
                    && !playlist.has_next_track()
                    && player.config.read().settings.player.gapless
                {
                    drop(playlist);
                    player.enqueue_next_from_playlist();
                }
            }
            PlayerCmd::Quit => {
                info!("PlayerCmd::Quit received");
                // to have a consistent last position
                player.pause();
                player.player_save_last_position();
                if let Err(e) = player.playlist.write().save() {
                    error!("error when saving playlist: {e}");
                };
                // clear out all currently queued rodio sources
                // without this, on rusty backend, may keep the process around until the last source has been consumed
                player.stop();
                if let Err(e) =
                    ServerConfigVersionedDefaulted::save_config_path(&player.config.read().settings)
                {
                    error!("error when saving config: {e}");
                };
                return Ok(());
            }
            PlayerCmd::CycleLoop => {
                player.config.write().settings.player.loop_mode =
                    player.playlist.write().cycle_loop_mode();
            }
            PlayerCmd::Eos => {
                info!("Eos received");
                let mut playlist = player.playlist.write();
                if playlist.is_empty() {
                    drop(playlist);
                    player.stop();
                    continue;
                }
                debug!(
                    "current track index: {:?}",
                    playlist.get_current_track_index()
                );
                playlist.clear_current_track();
                drop(playlist);
                player.start_play();
                debug!(
                    "playing index is: {}",
                    player.playlist.read().get_current_track_index()
                );
            }
            PlayerCmd::GetProgress => {}
            PlayerCmd::SkipPrevious => {
                info!("skip to previous track");
                player.player_save_last_position();
                player.previous();
            }
            PlayerCmd::ReloadConfig => {
                if let Err(err) = player.reload_config() {
                    error!("Reloading config failed, using old: {:#?}", err);
                }
            }
            PlayerCmd::ReloadPlaylist => {
                player.playlist.write().reload_tracks().ok();
            }
            PlayerCmd::SeekBackward => {
                player.seek_relative(false);
                let mut p_tick = playerstats.lock();
                if let Some(progress) = player.get_progress() {
                    p_tick.progress = progress
                }
            }
            PlayerCmd::SeekForward => {
                player.seek_relative(true);
                let mut p_tick = playerstats.lock();
                if let Some(progress) = player.get_progress() {
                    p_tick.progress = progress
                }
            }
            PlayerCmd::SkipNext => {
                info!("skip to next track.");
                player.player_save_last_position();
                player.next();
            }
            PlayerCmd::SpeedDown => {
                let new_speed = player.add_speed(-SPEED_STEP);
                info!("after speed down: {}", new_speed);
                player.config.write().settings.player.speed = new_speed;
                let mut p_tick = playerstats.lock();
                p_tick.speed = new_speed;
            }

            PlayerCmd::SpeedUp => {
                let new_speed = player.add_speed(SPEED_STEP);
                info!("after speed up: {}", new_speed);
                player.config.write().settings.player.speed = new_speed;
                let mut p_tick = playerstats.lock();
                p_tick.speed = new_speed;
            }
            PlayerCmd::Tick => {
                // info!("tick received");
                player.mpris_handle_events();
                let mut p_tick = playerstats.lock();
                let mut playlist = player.playlist.read();
                p_tick.status = playlist.status().as_u32();
                // branch to auto-start playing if status is "stopped"(not paused) and playlist is not empty anymore
                if playlist.status() == RunningStatus::Stopped {
                    if playlist.is_empty() {
                        continue;
                    }
                    debug!(
                        "current track index: {:?}",
                        playlist.get_current_track_index()
                    );
                    drop(playlist);
                    let mut playlist = player.playlist.write();
                    playlist.clear_current_track();
                    playlist.proceed_false();
                    drop(playlist);
                    player.start_play();
                    continue;
                }
                if let Some(progress) = player.get_progress() {
                    p_tick.progress = progress;
                    // the following function is "mut", which does not like having the immutable borrow to "playlist"
                    // so we have to unlock first then later re-acquire the handle for later parts
                    drop(playlist);
                    player.mpris_update_progress(&p_tick.progress);
                    playlist = player.playlist.read();
                }
                if player.current_track_updated {
                    p_tick.current_track_index =
                        u64::try_from(playlist.get_current_track_index()).unwrap();
                    p_tick.current_track_updated = player.current_track_updated;
                    player.current_track_updated = false;
                }
                if let Some(track) = playlist.current_track() {
                    // if only one backend is enabled, rust will complain that it is the only thing that happens
                    #[allow(irrefutable_let_patterns)]
                    if MediaTypesSimple::LiveRadio == track.media_type() {
                        // TODO: consider changing "radio_title" and "media_title" to be consistent
                        p_tick.radio_title = player.media_info().media_title.unwrap_or_default();

                        if let Backend::Rusty(ref mut backend) = player.backend {
                            p_tick.progress.total_duration = Some(Duration::from_secs(
                                ((*backend.radio_downloaded.lock() as f32 * 44100.0
                                    / 1000000.0
                                    / 1024.0)
                                    * (backend.speed() as f32 / 10.0))
                                    as u64,
                            ));
                        }

                        #[cfg(feature = "gst")]
                        if let Backend::GStreamer(_) = player.backend {
                            // p_tick.duration = player.backend.get_buffer_duration();
                            // error!("buffer duration: {}", p_tick.duration);
                            p_tick.progress.total_duration = Some(
                                p_tick.progress.position.unwrap_or_default()
                                    + Duration::from_secs(20),
                            );
                        }
                    }
                }
            }
            PlayerCmd::ToggleGapless => {
                let new_gapless = player.toggle_gapless();
                let mut p_tick = playerstats.lock();
                p_tick.gapless = new_gapless;
            }
            PlayerCmd::TogglePause => {
                info!("player toggled pause");
                player.toggle_pause();
                let mut p_tick = playerstats.lock();
                p_tick.status = player.playlist.read().status().as_u32();
            }
            PlayerCmd::VolumeDown => {
                info!("before volumedown: {}", player.volume());
                let new_volume = player.add_volume(-VOLUME_STEP);
                player.config.write().settings.player.volume = new_volume;
                info!("after volumedown: {}", new_volume);
                let mut p_tick = playerstats.lock();
                p_tick.volume = new_volume;
                player.mpris_volume_update();
            }
            PlayerCmd::VolumeUp => {
                info!("before volumeup: {}", player.volume());
                let new_volume = player.add_volume(VOLUME_STEP);
                player.config.write().settings.player.volume = new_volume;
                info!("after volumeup: {}", new_volume);
                let mut p_tick = playerstats.lock();
                p_tick.volume = new_volume;
                player.mpris_volume_update();
            }
            PlayerCmd::Pause => {
                player.pause();
            }
            PlayerCmd::Play => {
                player.resume();
            }

            PlayerCmd::PlaylistPlaySpecific(info) => {
                info!(
                    "play specific track, idx: {} id: {:#?}",
                    info.track_index, info.id
                );
                player.player_save_last_position();
                if let Err(err) = player.playlist.write().play_specific(&info) {
                    error!("Error setting specific track to play: {err}");
                }
                player.next();
            }
            PlayerCmd::PlaylistAddTrack(info) => {
                if let Err(err) = player.playlist.write().add_tracks(info, &player.db_podcast) {
                    error!("Error adding tracks: {err}");
                }
            }
            PlayerCmd::PlaylistRemoveTrack(info) => {
                if let Err(err) = player.playlist.write().remove_tracks(info) {
                    error!("Error removing tracks: {err}");
                }
            }
            PlayerCmd::PlaylistClear => {
                player.playlist.write().clear();
            }
            PlayerCmd::PlaylistSwapTrack(info) => {
                if let Err(err) = player.playlist.write().swap_tracks(&info) {
                    error!("Error swapping tracks: {err}");
                }
            }
            PlayerCmd::PlaylistShuffle => {
                player.playlist.write().shuffle();
            }
            PlayerCmd::PlaylistRemoveDeletedTracks => {
                player.playlist.write().remove_deleted_items();
            }
        }

        cb.call();
    }

    Ok(())
}

/// Spawn the thread that periodically sends [`PlayerCmd::Tick`]
fn ticker_thread(cmd_tx: PlayerCmdSender) -> Result<()> {
    std::thread::Builder::new()
        .name("ticker".into())
        .spawn(move || {
            while cmd_tx.send(PlayerCmd::Tick).is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        })?;

    Ok(())
}

fn get_config(args: &cli::Args) -> Result<ServerOverlay> {
    let config = ServerConfigVersionedDefaulted::from_config_path()?.into_settings();

    let max_depth = args.max_depth.map(ScanDepth::Limited);

    let music_dir = if let Some(ref dir) = args.music_directory {
        Some(get_path(dir).context("cli provided music-dir")?)
    } else {
        None
    };

    let overlay = ServerOverlay {
        settings: config,
        music_dir_overwrite: music_dir,
        disable_discord_status: args.disable_discord,
        library_scan_depth: max_depth,
    };

    Ok(overlay)
}

fn get_path(dir: &Path) -> Result<PathBuf> {
    let mut path = dir.to_path_buf();

    if path.exists() {
        if !path.has_root() {
            if let Ok(p_base) = std::env::current_dir() {
                path = p_base.join(path);
            }
        }

        if let Ok(p_canonical) = path.canonicalize() {
            path = p_canonical;
        }

        return Ok(path);
    }

    bail!("Error: non-existing directory '{}'", dir.display());
}

fn execute_action(action: cli::Action, config: &ServerOverlay) -> Result<()> {
    match action {
        cli::Action::Import { file } => {
            println!("need to import from file {}", file.display());

            let path = get_path(&file).context("import cli file-path")?;
            let config_dir_path =
                utils::get_app_config_path().context("getting app-config-path")?;

            podcast::import_from_opml(&config_dir_path, &config.settings.podcast, &path)
                .context("import opml")?;
        }
        cli::Action::Export { file } => {
            println!("need to export to file {}", file.display());
            let path = utils::absolute_path(&file)?;
            let config_dir_path =
                utils::get_app_config_path().context("getting app-config-path")?;
            podcast::export_to_opml(&config_dir_path, &path).context("export opml")?;
        }
    };

    Ok(())
}
