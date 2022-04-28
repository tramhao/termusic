use crate::song::Song;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
const APP_ID: &str = "968407067889131520";
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rpc {
    client: DiscordIpcClient,
    connected: bool,
    // time: i64,
    // duration: i64,
    artist: String,
    title: String,
}

impl Default for Rpc {
    fn default() -> Self {
        let mut client = DiscordIpcClient::new(APP_ID).unwrap();
        let connected = client.connect().is_ok();

        Self {
            client,
            connected,
            // time: 0,
            // duration: 0,
            artist: String::new(),
            title: String::new(),
        }
    }
}

impl Rpc {
    #[allow(clippy::cast_possible_wrap)]
    pub fn update(&mut self, song: &Song) {
        if !self.connected {
            self.connected = self.client.connect().is_ok();
        }

        if self.connected {
            let assets = activity::Assets::new()
                .large_image("termusic")
                .large_text("terminal music player written in Rust");
            // .small_image(smol_image)
            // .small_text(state);
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            // self.duration = song.duration().as_secs() as i64;
            self.artist = song.artist().unwrap_or("Unknown Artist").to_string();
            self.title = song.title().unwrap_or("Unknown Title").to_string();
            let timestamp = activity::Timestamps::new().start(time);
            // .end(self.time + self.duration);
            self.client
                .set_activity(
                    activity::Activity::new()
                        .assets(assets)
                        .timestamps(timestamp)
                        .state(&self.artist)
                        .details(&self.title),
                )
                .ok();
        }
    }
    pub fn pause(&mut self) {
        if !self.connected {
            self.connected = self.client.connect().is_ok();
        }
        if self.connected {
            let assets = activity::Assets::new()
                .large_image("termusic")
                .large_text("terminal music player written in Rust");

            self.client
                .set_activity(
                    activity::Activity::new()
                        .assets(assets)
                        .state(&self.artist)
                        .details(format!("{}: Paused", self.title.as_str()).as_str()),
                )
                .ok();
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    pub fn resume(&mut self, time_pos: i64) {
        if !self.connected {
            self.connected = self.client.connect().is_ok();
        }
        if self.connected {
            let assets = activity::Assets::new()
                .large_image("termusic")
                .large_text("terminal music player written in Rust");

            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let timestamp = activity::Timestamps::new().start(time - time_pos);
            self.client
                .set_activity(
                    activity::Activity::new()
                        .assets(assets)
                        .timestamps(timestamp)
                        .state(&self.artist)
                        .details(&self.title),
                )
                .ok();
        }
    }
}

impl Drop for Rpc {
    fn drop(&mut self) {
        if self.connected {
            self.client.close().ok();
        }
    }
}
