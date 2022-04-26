use crate::song::Song;
// use anyhow::Result;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

pub struct Rpc {
    client: Option<DiscordIpcClient>,
}

impl Default for Rpc {
    fn default() -> Self {
        if let Ok(mut client) = DiscordIpcClient::new("968407067889131520") {
            client.connect();
            Self {
                client: Some(client),
            }
        } else {
            Self { client: None }
        }
    }
}

impl Rpc {
    pub fn update(&mut self, song: &Song) {
        if let Some(mut client) = self.client {
            client
                .set_activity(
                    activity::Activity::new()
                        // .assets(song.album_photo().unwrap())
                        .state(song.artist().unwrap_or("Unknown Artist"))
                        .details(song.title().unwrap_or("Unknown Title")),
                )
                .ok();
        }
    }
}

impl Drop for Rpc {
    fn drop(&mut self) {
        if let Some(mut client) = &self.client {
            client.close().ok();
        }
    }
}
