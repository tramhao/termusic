// use crate::{mpris::MprisController, CONFIG, LOG, PLAYER, TMP_DIR};
use anyhow::Result;
use std::{
    fs,
    io::{BufReader, Read, Write},
    net::Shutdown,
    os::unix::net::{UnixListener, UnixStream},
    process,
};
use termusiclib::config::Settings;
use termusicplayback::{GeneralPlayer, PlayerCmd, PlayerTrait, CONFIG, TMP_DIR};

#[allow(clippy::manual_flatten)]
pub fn spawn() -> Result<()> {
    fs::create_dir_all(&*TMP_DIR).unwrap_or_default();
    let socket_file = format!("{}/socket", *TMP_DIR);
    fs::remove_file(&socket_file).unwrap_or(());
    let listener = UnixListener::bind(&socket_file).expect("What went wrong?!");

    let mut config = Settings::default();
    config.load()?;
    info!("config loaded");

    let mut player = GeneralPlayer::new(&config);

    // let mut player = GeneralPlayer::new(&config);
    player.need_proceed_to_next = false;
    player.start_play();
    info!("start play the saved playlist");

    // let mut player_mpris = PLAYER.lock();
    // if CONFIG.use_mpris {
    //     std::thread::Builder::new()
    //         .name("mpris-ctl".to_string())
    //         .spawn(|| loop {
    //             player_mpris.update_mpris();
    //             std::thread::sleep(std::time::Duration::from_millis(100));
    //         })
    //         .expect("Why didn't the thread spawn?!");
    // }
    // info!("mpris thread spawned");

    // LOG.line_basic("Startup complete!", true);
    for request in listener.incoming() {
        if let Ok(stream) = request {
            let mut out_stream = stream.try_clone().expect("Why can't I clone this value?!");
            let buffer = BufReader::new(&stream);
            let encoded: Vec<u8> = buffer.bytes().map(|r| r.unwrap_or(0)).collect();
            let command: PlayerCmd =
                bincode::deserialize(&encoded).expect("Error parsing request!");

            if command.is_mut() {
                // let mut player = PLAYER.write().expect("What went wrong?!");
                match command {
                    PlayerCmd::PlaySelected => {
                        info!("play selected");
                        player.need_proceed_to_next = false;
                        player.next();
                    }
                    PlayerCmd::Skip => {
                        info!("skip to next track");
                        player.next();
                    }
                    PlayerCmd::Previous => {
                        info!("skip to previous track");
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
                        // self.player_restore_last_position();
                    }

                    PlayerCmd::StartPlay => {
                        info!("StartPlay received");
                        if player.playlist.is_empty() {
                            player.stop();
                            continue;
                        }
                        debug!(
                            "current track index: {}",
                            player.playlist.get_current_track_index()
                        );
                        player.playlist.clear_current_track();
                        player.need_proceed_to_next = false;
                        player.start_play();
                        // self.player_restore_last_position();
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
                        if CONFIG.use_mpris {
                            player.update_mpris();
                        }
                    }
                    // PlayerCommand::Load(playlist) => player.load_list(&playlist),
                    // PlayerCommand::CycleRepeat => player.cycle_repeat(),
                    // PlayerCommand::Play => player.play(),
                    // PlayerCommand::Restart => player.restart(),
                    // PlayerCommand::Next => player.next(),
                    // PlayerCommand::Prev => player.prev(),
                    // PlayerCommand::Resume => player.resume(),
                    // PlayerCommand::Pause => player.pause(),
                    // PlayerCommand::Stop => player.stop(),
                    // PlayerCommand::Seek(time) => player.seek(time),

                    // PlayerCommand::Shuffle => {
                    //     player.shuffle_queue();
                    //     player.find_pos();
                    // }

                    // PlayerCommand::SetPos(song) => {
                    //     player.set_pos(&song);
                    //     player.find_pos();
                    // }

                    // PlayerCommand::SetQueue(playlist) => {
                    //     player.queue = playlist;
                    //     player.find_pos();
                    // }
                    _ => panic!("Invalid player action!"),
                }
            } else {
                // let player = PLAYER.read().expect("What went wrong?!");

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
                        let d_i64 = duration.as_secs() as i64;
                        send_val(&mut out_stream, &(*position, d_i64, current_track_index));
                    }

                    // PlayerCommand::Status => {
                    //     let status = PlayerStatus {
                    //         stopped: player.is_stopped(),
                    //         paused: player.is_paused(),
                    //         position: player.position,
                    //         repeat_mode: player.repeat,
                    //         state: player.state,
                    //         song_id: player.song.song_id(),
                    //     };
                    //     send_val(&mut out_stream, &status);
                    // }

                    // PlayerCommand::GetQueue => {
                    //     send_val(&mut out_stream, &player.queue);
                    // }
                    _ => panic!("Invalid player action!"),
                }
            }
        }
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
