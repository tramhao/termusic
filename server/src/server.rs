mod music_player_service;
mod logger;
mod cli;

use std::path::Path;

use anyhow::Result;
use clap::Parser;
use music_player_service::MusicPlayerService;
use termusiclib::types::player::music_player_server::MusicPlayerServer;
use termusiclib::{config::Settings, track::TrackSource};
use termusicplayback::{GeneralPlayer, PlayerCmd, PlayerTrait};
use tonic::transport::Server;

#[macro_use]
extern crate log;

pub const MAX_DEPTH: usize = 4;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Args::parse();
    let _ = logger::setup_logger(&args);
    info!("background thread start");

    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel();

    let music_player_service: MusicPlayerService = MusicPlayerService::new(cmd_tx.clone());
    let mut config = get_config(&args)?;
    let progress_tick = music_player_service.progress.clone();

    let addr = format!("[::]:{}", config.player_port).parse()?;
    let player_handle = tokio::task::spawn_blocking(move || -> Result<()> {
        let mut player = GeneralPlayer::new(&config, cmd_tx.clone());

        let cmd_tx_tick = cmd_tx.clone();
        // Start tick thread after general player has started.
        tokio::task::spawn(async move {
            // Give TUI a bit of time to connect before we start ticking. This is due to a race
            // condition where the TUI does not subscribe fast enough which causes the player to
            // start playing music without updating the TUI.
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            loop {
                if cmd_tx_tick.send(PlayerCmd::Tick).is_err() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        });

        // *Controller* layer. This loop should handle message passing. It should not handle logic
        // that only touches `GeneralPlayer` owned resources.
        while let Some(cmd) = cmd_rx.blocking_recv() {
            #[allow(unreachable_patterns)]
            match cmd {
                PlayerCmd::AboutToFinish => {
                    info!("about to finish signal received");
                    player.handle_about_to_finish();
                }
                PlayerCmd::Quit => {
                    player.player_save_last_position();
                    if let Err(e) = player.playlist.save() {
                        error!("error when saving playlist: {e}");
                    };
                    if let Err(e) = config.save() {
                        error!("error when saving config: {e}");
                    };
                    std::process::exit(0);
                }
                PlayerCmd::CycleLoop => {
                    config.player_loop_mode = player.playlist.cycle_loop_mode();
                }
                #[cfg(not(any(feature = "mpv", feature = "gst")))]
                PlayerCmd::DurationNext(duration) => {
                    player
                        .playlist
                        .set_next_track_duration(std::time::Duration::from_secs(duration));
                }
                PlayerCmd::Eos => {
                    info!("Eos received");
                    player.handle_eos();
                    debug!(
                        "playing index is: {:?}",
                        player.playlist.get_current_track_index()
                    );
                }
                PlayerCmd::GetProgress | PlayerCmd::ProcessID => {}
                PlayerCmd::PlaySelected(index) => {
                    info!("play selected");
                    player.handle_play_selected(TrackSource::Playlist(
                        usize::try_from(index).expect("could not convert to usize"),
                    ));
                }
                PlayerCmd::SkipPrevious => {
                    info!("skip to previous track");
                    player.player_save_last_position();
                    player.previous();
                }
                PlayerCmd::ReloadConfig => {
                    config.load()?;
                    info!("config reloaded");
                    player.config = config.clone();
                }
                PlayerCmd::ReloadPlaylist => {
                    player.playlist.reload_tracks().ok();
                }
                PlayerCmd::SeekBackward => {
                    player.seek_relative(false);
                    let mut p_tick = progress_tick.lock();
                    if let Ok((position, _duration)) = player.get_progress() {
                        p_tick.position = position as u32;
                    }
                }
                PlayerCmd::SeekForward => {
                    player.seek_relative(true);
                    let mut p_tick = progress_tick.lock();
                    if let Ok((position, _duration)) = player.get_progress() {
                        p_tick.position = position as u32;
                    }
                }
                PlayerCmd::SkipNext => {
                    info!("skip to next track.");
                    player.player_save_last_position();
                    player.next();
                }
                PlayerCmd::SpeedDown => {
                    player.speed_down();
                    info!("after speed down: {}", player.speed());
                    config.player_speed = player.speed();
                    let mut p_tick = progress_tick.lock();
                    p_tick.speed = config.player_speed;
                }

                PlayerCmd::SpeedUp => {
                    player.speed_up();
                    info!("after speed up: {}", player.speed());
                    config.player_speed = player.speed();
                    let mut p_tick = progress_tick.lock();
                    p_tick.speed = config.player_speed;
                }
                PlayerCmd::Tick => {
                    // info!("tick received");
                    if config.player_use_mpris {
                        player.update_mpris();
                    }
                    player.handle_tick(&mut progress_tick.lock());
                }
                PlayerCmd::ToggleGapless => {
                    config.player_gapless = player.toggle_gapless();
                    let mut p_tick = progress_tick.lock();
                    p_tick.gapless = config.player_gapless;
                }
                PlayerCmd::TogglePause => {
                    info!("player toggled pause");
                    player.toggle_pause();
                    let mut p_tick = progress_tick.lock();
                    p_tick.status = player.playlist.status().as_u32();
                }
                PlayerCmd::VolumeDown => {
                    info!("before volumedown: {}", player.volume());
                    player.volume_down();
                    config.player_volume = player.volume();
                    info!("after volumedown: {}", player.volume());
                    let mut p_tick = progress_tick.lock();
                    p_tick.volume = config.player_volume;
                }
                PlayerCmd::VolumeUp => {
                    info!("before volumeup: {}", player.volume());
                    player.volume_up();
                    config.player_volume = player.volume();
                    info!("after volumeup: {}", player.volume());
                    let mut p_tick = progress_tick.lock();
                    p_tick.volume = config.player_volume;
                }
                PlayerCmd::Pause => {
                    player.pause();
                }
                PlayerCmd::Play => {
                    player.resume();
                }
                PlayerCmd::SubscribeToUpdates(sender) => {
                    player.subscribers.push(sender);
                }
            }
        }
        Ok(())
    });

    Server::builder()
        .add_service(MusicPlayerServer::new(music_player_service))
        .serve(addr)
        .await?;

    let _drop = player_handle.await?;

    Ok(())
}

fn get_config(args: &cli::Args) -> Result<Settings> {
    let mut config = Settings::default();
    config.load()?;

    config.disable_album_art_from_cli = args.disable_cover;
    config.disable_discord_rpc_from_cli = args.disable_discord;

    if let Some(dir) = &args.music_directory {
        config.music_dir_from_cli = get_path(dir);
    }

    config.max_depth_cli = match args.max_depth {
        Some(d) => d,
        None => MAX_DEPTH,
    };

    Ok(config)
}

fn get_path(dir: &str) -> Option<String> {
    let music_dir: Option<String>;
    let mut path = Path::new(&dir).to_path_buf();

    if path.exists() {
        if !path.has_root() {
            if let Ok(p_base) = std::env::current_dir() {
                path = p_base.join(path);
            }
        }

        if let Ok(p_canonical) = path.canonicalize() {
            path = p_canonical;
        }

        music_dir = Some(path.to_string_lossy().to_string());
    } else {
        eprintln!("Error: unknown directory '{dir}'");
        std::process::exit(0);
    }
    music_dir
}
