use crate::song::Song;
// use anyhow::Result;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
const APP_ID: &str = "968407067889131520";
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rpc {
    client: DiscordIpcClient,
    connected: bool,
}

impl Default for Rpc {
    fn default() -> Self {
        let mut client = DiscordIpcClient::new(APP_ID).unwrap();
        let mut connected = false;
        if let Ok(()) = client.connect() {
            connected = true;
        }
        Self { client, connected }
    }
}

impl Rpc {
    #[allow(clippy::cast_possible_wrap)]
    pub fn update(&mut self, song: &Song) {
        if self.connected {
            let assets = activity::Assets::new()
                .large_image("termusic")
                .large_text("terminal music player written in Rust");
            // .small_image(smol_image)
            // .small_text(state);

            let time_unix = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            self.client
                .set_activity(
                    activity::Activity::new()
                        .assets(assets)
                        .timestamps(activity::Timestamps::new().start(time_unix))
                        .state(song.artist().unwrap_or("Unknown Artist"))
                        .details(song.title().unwrap_or("Unknown Title")),
                )
                .ok();
        }
    }
    // pub fn update_progress(&mut self, duration: u64, progress: u64) {
    //     self.client
    //         .set_activity(
    //             activity::Activity::new()
    //                 // .assets(assets)
    //                 .state(duration)
    //                 .details(progress),
    //         )
    //         .ok();
    // }
}

impl Drop for Rpc {
    fn drop(&mut self) {
        if self.connected {
            self.client.close().ok();
        }
    }
}
