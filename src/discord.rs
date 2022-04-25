use crate::song::Song;
// use anyhow::Result;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

pub struct Rpc {
    client: DiscordIpcClient,
}

impl Default for Rpc {
    fn default() -> Self {
        let mut client = DiscordIpcClient::new("Termusic").ok().unwrap();

        client.connect().ok();
        Self { client }
    }
}

impl Rpc {
    pub fn update(&mut self, song: &Song) {
        self.client
            .set_activity(
                activity::Activity::new()
                    // .assets(song.album_photo().unwrap())
                    .state(song.artist().unwrap_or("Unknown Artist"))
                    .details(song.title().unwrap_or("Unknown Title")),
            )
            .ok();
    }
}

impl Drop for Rpc {
    fn drop(&mut self) {
        self.client.close().ok();
    }
}
