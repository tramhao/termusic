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
    pub fn update(&mut self, _song: &Song) {
        self.client
            .set_activity(activity::Activity::new().state("foo").details("bar"))
            .ok();
    }
}

impl Drop for Rpc {
    fn drop(&mut self) {
        self.client.close().ok();
    }
}
