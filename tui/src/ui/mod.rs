use anyhow::Result;
use futures_util::StreamExt;
use termusiclib::player::music_player_client::MusicPlayerClient;
use tokio::sync::mpsc::{self};
use tokio::task::JoinHandle;
use tonic::transport::Channel;
use tuirealm::application::PollStrategy;

use crate::CombinedSettings;
use crate::ui::server_req_actor::ServerRequestActor;
use model::Model;
pub use music_player_client::Playback;
use tui_cmd::PlaylistCmd;
use tui_cmd::TuiCmd;

pub mod components;
mod ids;
pub mod model;
mod msg;
mod music_player_client;
mod server_req_actor;
mod tui_cmd;
#[cfg(all(feature = "cover-ueberzug", not(target_os = "windows")))]
mod ueberzug;
pub mod utils;

/// The main TUI struct which handles message passing and the main-loop.
pub struct UI {
    model: Model,

    server_req_actor: JoinHandle<()>,
}

impl UI {
    /// Create a new [`UI`] instance
    pub async fn new(
        config: CombinedSettings,
        client: MusicPlayerClient<Channel>,
        layout_4: bool,
    ) -> Result<Self> {
        let mut playback = Playback::new(client);

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let stream_updates = playback.subscribe_to_stream_updates().await?;

        let mut model = Model::new(config, cmd_tx, stream_updates.boxed(), layout_4);
        model.init();

        let jh = ServerRequestActor::start_actor(playback, cmd_rx, model.tx_to_main.clone());

        Ok(Self {
            model,
            server_req_actor: jh,
        })
    }

    /// Main Loop function.
    pub fn run(&mut self) -> Result<()> {
        // load the initial playlist
        let _ = self
            .model
            .cmd_to_server_tx
            .send(TuiCmd::Playlist(PlaylistCmd::SelfReloadPlaylist));
        // initial request for all the progress states / options
        self.model.request_progress();

        // Main loop
        while !self.model.quit {
            match self.model.app.tick(PollStrategy::BlockCollectUpTo(10)) {
                Err(err) => {
                    self.model
                        .mount_error_popup((anyhow::anyhow!(err)).context("tick poll error"));
                }
                Ok(messages) if !messages.is_empty() => {
                    // NOTE: redraw if at least one msg has been processed
                    self.model.redraw = true;
                    for msg in messages {
                        self.model.update(msg);
                    }
                }
                _ => {}
            }

            self.model.ensure_quit_popup_top_most_focus();

            self.model.view();
        }

        if self
            .model
            .config_tui
            .read()
            .settings
            .behavior
            .quit_server_on_exit
        {
            self.model.cmd_to_server_tx.send(TuiCmd::QuitServer)?;
        }

        Ok(())
    }

    /// Wait until all events are handled.
    ///
    /// Currently this only waits until the [`ServerRequestActor`] is done sending events.
    pub async fn wait_until_done(self) {
        // Explicitly drop first as Model stores a ServerRequestActor channel Tx
        // and the ServerRequestActor only closes once the channel is closed and all events are handled.
        drop(self.model);

        if let Err(err) = self.server_req_actor.await {
            error!("Server Request Actor exited with a error: {err:#?}");
        }
    }
}
