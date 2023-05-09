// use crate::{mpris::MprisController, CONFIG, LOG, PLAYER, TMP_DIR};
use crate::TMP_DIR;
use rust_utils::logging::LogLevel;
use std::{
    fs,
    io::{BufReader, Read, Write},
    net::Shutdown,
    os::unix::net::{UnixListener, UnixStream},
    process, thread,
};
use termusicplayback::{PlayerCmd, PlayerMsg};

#[allow(clippy::manual_flatten)]
pub fn spawn() {
    fs::create_dir_all(&*TMP_DIR).unwrap_or_default();
    let socket_file = format!("{}/socket", *TMP_DIR);
    fs::remove_file(&socket_file).unwrap_or(());
    let listener = UnixListener::bind(&socket_file).expect("What went wrong?!");

    // move to the next song when it ends
    thread::Builder::new()
        .name("player-ctl".to_string())
        .spawn(|| loop {
            // if let Ok(mut player) = PLAYER.try_write() {
            //     player.auto_advance();
            // }
        })
        .expect("Why didn't the thread spawn?!");

    // if CONFIG.use_mpris {
    //     thread::Builder::new()
    //         .name("mpris-ctl".to_string())
    //         .spawn(|| {
    //             let mut mpris = MprisController::new();
    //             mpris.run();
    //         })
    //         .expect("Why didn't the thread spawn?!");
    // }

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

                    // PlayerCommand::CurrentTime => {
                    //     let time = player.cur_time_secs();
                    //     send_val(&mut out_stream, &time);
                    // }

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
}

fn send_val<V: serde::Serialize + for<'de> serde::Deserialize<'de> + ?Sized>(
    stream: &mut UnixStream,
    val: &V,
) {
    let encoded = bincode::serialize(val).expect("What went wrong?!");
    if let Err(why) = stream.write_all(&encoded) {
        // LOG.line(
        //     LogLevel::Error,
        //     format!("Unable to write to socket: {why}"),
        //     false,
        // );
    };
    stream.shutdown(Shutdown::Write).expect("What went wrong?!");
}
