mod music_player_service;
use anyhow::Result;
use music_player_service::MusicPlayerService;
use termusiclib::config::Settings;
use termusiclib::types::player::music_player_server::MusicPlayerServer;
use termusicplayback::{GeneralPlayer, PlayerCmd, PlayerTrait};
use tonic::transport::Server;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    lovely_env_logger::init_default();
    info!("background thread start");

    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel();

    let music_player_service: MusicPlayerService = MusicPlayerService::new(cmd_tx.clone());
    let mut config = Settings::default();
    config.load()?;
    let progress_tick = music_player_service.progress.clone();

    let addr = format!("[::]:{}", config.player_port).parse()?;
    let player_handle = tokio::task::spawn_blocking(move || -> Result<()> {
        let mut player = GeneralPlayer::new(&config, cmd_tx.clone());

        let cmd_tx_tick = cmd_tx.clone();
        // Start tick thread after general player has started.
        tokio::task::spawn(async move {
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
                PlayerCmd::PlaySelected => {
                    // TODO: Currently this operation is really complex. It involves three messages.
                    // The GUI will before sending this message save the desired track index to the
                    // playlist file as `current_track_index` and then send a `PlayerCmd::ReloadPlaylist`
                    // message. This causes this controller to reload the playlist file and update
                    // the `current_track_index`. The TUI will then send this
                    // `PlayerCmd::PlaySelected` message which causes the player to set a flag which
                    // tells it on the next Eos message not to pick the `next_track`
                    // then sends a `PlayerCmd::Eos` message.
                    // On receiving the Eos message the player looks up the flag and then
                    // uses the `current_track_index` to pick the selected song.
                    //
                    // This is too complex. There should be no need to sync through a file and it
                    // should all be handled in a single message rather than 3.
                    info!("play selected");
                    player.handle_play_selected();
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
