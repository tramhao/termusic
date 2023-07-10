use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use termusiclib::track::Track;
const APP_ID: &str = "968407067889131520";
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Rpc {
    tx: Sender<RpcCommand>,
}

enum RpcCommand {
    Update(String, String),
    Pause,
    Resume(i64),
}

impl Default for Rpc {
    #[allow(clippy::cast_possible_wrap)]
    fn default() -> Self {
        let mut client = DiscordIpcClient::new(APP_ID).unwrap();
        let (tx, rx): (Sender<RpcCommand>, Receiver<RpcCommand>) = mpsc::channel();
        let mut artist = String::new();
        let mut title = String::new();

        std::thread::spawn(move || loop {
            match rx.try_recv() {
                Ok(RpcCommand::Update(artist_cmd, title_cmd)) => {
                    let assets = activity::Assets::new()
                        .large_image("termusic")
                        .large_text("terminal music player written in Rust");
                    // .small_image(smol_image)
                    // .small_text(state);
                    let time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    let timestamp = activity::Timestamps::new().start(time);
                    // .end(self.time + self.duration);

                    loop {
                        if client.connect().is_ok() {
                            break;
                        }
                        sleep(Duration::from_secs(2));
                    }

                    artist = artist_cmd;
                    title = title_cmd;

                    client
                        .set_activity(
                            activity::Activity::new()
                                .assets(assets)
                                .timestamps(timestamp)
                                .state(&artist)
                                .details(&title),
                        )
                        .ok();
                }
                Ok(RpcCommand::Pause) => {
                    loop {
                        if client.connect().is_ok() {
                            break;
                        }
                        sleep(Duration::from_secs(2));
                    }

                    let assets = activity::Assets::new()
                        .large_image("termusic")
                        .large_text("terminal music player written in Rust");

                    client
                        .set_activity(
                            activity::Activity::new()
                                .assets(assets)
                                .state(&artist)
                                .details(format!("{}: Paused", title.as_str()).as_str()),
                        )
                        .ok();
                }
                Ok(RpcCommand::Resume(time_pos)) => {
                    let assets = activity::Assets::new()
                        .large_image("termusic")
                        .large_text("terminal music player written in Rust");

                    let time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    let timestamp = activity::Timestamps::new().start(time - time_pos);

                    loop {
                        if client.connect().is_ok() {
                            break;
                        }
                        sleep(Duration::from_secs(2));
                    }

                    client
                        .set_activity(
                            activity::Activity::new()
                                .assets(assets)
                                .timestamps(timestamp)
                                .state(&artist)
                                .details(&title),
                        )
                        .ok();
                }
                Err(_) => {}
            }
            sleep(Duration::from_secs(1));
        });

        Self { tx }
    }
}

impl Rpc {
    pub fn update(&mut self, track: &Track) {
        let artist = track.artist().unwrap_or("Unknown Artist").to_string();
        let title = track.title().unwrap_or("Unknown Title").to_string();
        self.tx.send(RpcCommand::Update(artist, title)).ok();
    }
    pub fn pause(&mut self) {
        self.tx.send(RpcCommand::Pause).ok();
    }

    pub fn resume(&mut self, time_pos: i64) {
        self.tx.send(RpcCommand::Resume(time_pos)).ok();
    }
}

// impl Drop for Rpc {
//     fn drop(&mut self) {
//         if self.connected {
//             self.client.close().ok();
//         }
//     }
// }
