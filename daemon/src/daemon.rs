use anyhow::Result;
use std::{
    fs,
    io::{BufReader, Read, Write},
    net::Shutdown,
    os::unix::net::{UnixListener, UnixStream},
    process,
};
use termusiclib::config::Settings;
use termusicplayback::playlist::Status;
use termusicplayback::{GeneralPlayer, PlayerCmd, PlayerTrait, CONFIG, TMP_DIR};

// #[allow(clippy::manual_flatten)]
#[allow(clippy::too_many_lines)]
pub fn spawn() -> Result<()> {
    fs::create_dir_all(&*TMP_DIR).unwrap_or_default();
    let socket_file = format!("{}/socket", *TMP_DIR);
    fs::remove_file(&socket_file).unwrap_or(());
    let listener = UnixListener::bind(&socket_file).expect("What went wrong?!");

    let mut config = Settings::default();
    config.load()?;
    info!("config loaded");

    let mut player = GeneralPlayer::new(&config);

    for request in listener.incoming().flatten() {
        let mut out_stream = request.try_clone().expect("Why can't I clone this value?!");
        let buffer = BufReader::new(&request);
        let encoded: Vec<u8> = buffer.bytes().map(|r| r.unwrap_or(0)).collect();
        let command: PlayerCmd = bincode::deserialize(&encoded).expect("Error parsing request!");

        if command.is_mut() {
            match command {
                PlayerCmd::PlaySelected => {
                    info!("play selected");
                    player.player_save_last_position();
                    player.need_proceed_to_next = false;
                    player.next();
                }
                PlayerCmd::Skip => {
                    info!("skip to next track");
                    player.player_save_last_position();
                    player.next();
                }
                PlayerCmd::Previous => {
                    info!("skip to previous track");
                    player.player_save_last_position();
                    player.previous();
                }
                PlayerCmd::TogglePause => {
                    info!("toggle pause");
                    player.toggle_pause();
                }
                PlayerCmd::Eos => {
                    info!("Eos received");
                    if player.playlist.is_empty() {
                        player.stop();
                        continue;
                    }
                    debug!(
                        "current track index: {}",
                        player.playlist.get_current_track_index()
                    );
                    player.playlist.clear_current_track();
                    player.start_play();
                }

                PlayerCmd::VolumeUp => {
                    player.volume_up();
                    send_val(&mut out_stream, &player.volume());
                }
                PlayerCmd::VolumeDown => {
                    player.volume_down();
                    send_val(&mut out_stream, &player.volume());
                }

                PlayerCmd::ReloadPlaylist => {
                    player.playlist.reload_tracks().ok();
                }
                PlayerCmd::SeekForward => {
                    player.seek_relative(true);
                }

                PlayerCmd::SeekBackward => {
                    player.seek_relative(false);
                }
                PlayerCmd::SpeedUp => {
                    player.speed_up();
                    send_val(&mut out_stream, &player.speed());
                }
                PlayerCmd::SpeedDown => {
                    player.speed_down();
                    send_val(&mut out_stream, &player.speed());
                }
                PlayerCmd::Tick => {
                    // info!("start from tick event");
                    if CONFIG.use_mpris {
                        player.update_mpris();
                    }
                    if player.playlist.status() == Status::Stopped {
                        if player.playlist.is_empty() {
                            continue;
                        }
                        debug!(
                            "current track index: {}",
                            player.playlist.get_current_track_index()
                        );
                        player.playlist.clear_current_track();
                        player.need_proceed_to_next = false;
                        player.start_play();
                    }
                }
                PlayerCmd::CycleLoop => {
                    let loop_mode = player.playlist.cycle_loop_mode();
                    send_val(&mut out_stream, &loop_mode);
                }
                PlayerCmd::AboutToFinish => {
                    if !player.playlist.is_empty() && !player.playlist.has_next_track() {
                        player.enqueue_next();
                    }
                }
                PlayerCmd::DurationNext(duration) => {
                    player
                        .playlist
                        .set_next_track_duration(std::time::Duration::from_secs(duration));
                }

                _ => panic!("Invalid player action!"),
            }
        } else {
            match command {
                PlayerCmd::ProcessID => {
                    let id = process::id() as usize;
                    send_val(&mut out_stream, &id);
                }

                PlayerCmd::FetchStatus => {
                    send_val(&mut out_stream, &player.playlist.status());
                }

                PlayerCmd::GetProgress => {
                    let position = player.player.position.lock().unwrap();
                    // info!("position is: {position}");
                    let duration = player.player.total_duration.lock().unwrap();
                    let current_track_index = player.playlist.get_current_track_index();
                    #[allow(clippy::cast_possible_wrap)]
                    let d_i64 = duration.as_secs() as i64;
                    send_val(&mut out_stream, &(*position, d_i64, current_track_index));
                }

                _ => panic!("Invalid player action!"),
            }
        }
        // }
    }
    Ok(())
}

fn send_val<V: serde::Serialize + for<'de> serde::Deserialize<'de> + ?Sized>(
    stream: &mut UnixStream,
    val: &V,
) {
    let encoded = bincode::serialize(val).expect("What went wrong?!");
    if let Err(why) = stream.write_all(&encoded) {
        info!("Unable to write to socket: {why}");
    };
    stream.shutdown(Shutdown::Write).expect("What went wrong?!");
}
