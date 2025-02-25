use std::sync::mpsc::{self, Receiver, RecvError, Sender};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::PlayerTimeUnit;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use termusiclib::library_db::const_unknown::{UNKNOWN_ARTIST, UNKNOWN_TITLE};
use termusiclib::track::Track;

const APP_ID: &str = "968407067889131520";

/// Handle for communicating with the discord ipc client
#[derive(Debug)]
pub struct Rpc {
    tx: Sender<RpcCommand>,
}

enum RpcCommand {
    Update(String, String),
    Pause,
    Resume(i64),
}

impl Default for Rpc {
    fn default() -> Self {
        let client = DiscordIpcClient::new(APP_ID).unwrap();
        let (tx, rx): (Sender<RpcCommand>, Receiver<RpcCommand>) = mpsc::channel();

        std::thread::Builder::new()
            .name("discord rpc loop".into())
            .spawn(|| Self::thread_fn(client, rx))
            .expect("failed to start discord rpc loop thread");

        Self { tx }
    }
}

impl Rpc {
    /// Update the discord status track information
    pub fn update(&self, track: &Track) {
        let artist = track.artist().unwrap_or(UNKNOWN_ARTIST).to_string();
        let title = track.title().unwrap_or(UNKNOWN_TITLE).to_string();
        self.tx.send(RpcCommand::Update(artist, title)).ok();
    }

    /// Update the discord status to show that it is paused
    pub fn pause(&self) {
        self.tx.send(RpcCommand::Pause).ok();
    }

    /// Update the discord status to show that it is playing
    pub fn resume(&self, time_pos: Option<PlayerTimeUnit>) {
        // ignore clippy here, this should not be a problem, maybe rich presence will support duration in the future
        #[allow(clippy::cast_possible_wrap)]
        if let Some(time_pos) = time_pos {
            self.tx
                .send(RpcCommand::Resume(time_pos.as_secs() as i64))
                .ok();
        }
    }

    /// This function actually communicates with the discord client and is meant to run in its own thread.
    #[allow(clippy::needless_pass_by_value)]
    fn thread_fn(mut client: DiscordIpcClient, rx: Receiver<RpcCommand>) {
        let mut artist = String::new();
        let mut title = String::new();

        loop {
            let msg = match rx.recv() {
                Err(RecvError) => {
                    info!("No senders for discord updates anymore, closing discord connection");
                    break;
                }
                Ok(v) => v,
            };

            if !reconnect(&mut client) {
                // if connecting to the discord rpc fails, ignore the current command

                // likely for better status we should keep a state and try to reconnect, but also still handle all the commands send here
                continue;
            }

            match msg {
                RpcCommand::Update(artist_cmd, title_cmd) => {
                    let assets = activity::Assets::new()
                        .large_image("termusic")
                        .large_text("terminal music player written in Rust");
                    // .small_text(state);
                    let time = if let Ok(v) = i64::try_from(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    ) {
                        v
                    } else {
                        warn!(
                            "SystemTime to i64 failed, discord interface cant handle this number"
                        );
                        0
                    };
                    let timestamp = activity::Timestamps::new().start(time);
                    // .end(self.time + self.duration);

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
                RpcCommand::Pause => {
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
                RpcCommand::Resume(time_pos) => {
                    let assets = activity::Assets::new()
                        .large_image("termusic")
                        .large_text("terminal music player written in Rust");

                    let time = if let Ok(v) = i64::try_from(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    ) {
                        v
                    } else {
                        warn!(
                            "SystemTime to i64 failed, discord interface cant handle this number"
                        );
                        0
                    };
                    let timestamp = activity::Timestamps::new().start(time - time_pos);

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
            }
        }
    }
}

const RETRIES: u8 = 3;

/// Try to connect the given client, with [`RETRIES`] amount of retries.
///
/// Returns `true` if connected, `false` otherwise
fn reconnect(client: &mut DiscordIpcClient) -> bool {
    let mut tries = 0;

    while tries < RETRIES {
        tries += 1;
        if client.connect().is_ok() {
            return true;
        }
        sleep(Duration::from_secs(2));
    }

    false
}
