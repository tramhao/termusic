use anyhow::Result;
use futures_util::StreamExt;
use termusiclib::player::music_player_client::MusicPlayerClient;
use tokio::sync::mpsc::{self};
use tonic::transport::Channel;
use tuirealm::application::PollStrategy;

use crate::CombinedSettings;
use crate::ui::server_req_actor::ServerRequestActor;
use model::Model;
use music_player_client::Playback;
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
}

impl UI {
    /// Create a new [`UI`] instance
    pub async fn new(config: CombinedSettings, client: MusicPlayerClient<Channel>) -> Result<Self> {
        let mut playback = Playback::new(client);

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let stream_updates = playback.subscribe_to_stream_updates().await?;

        let mut model = Model::new(config, cmd_tx, stream_updates.boxed());
        model.init();

        ServerRequestActor::start_actor(playback, cmd_rx, model.tx_to_main.clone());

        Ok(Self { model })
    }

    /// Handle terminal init & finalize and start the UI Loop.
    // TODO: remove this function?
    pub fn run(&mut self) -> Result<()> {
        self.run_inner()
    }

    /// Main Loop function.
    ///
    /// This function does NOT handle initializing and finializing the terminal.
    #[allow(clippy::unnecessary_wraps)] // to easily change if it ever becomes required again
    fn run_inner(&mut self) -> Result<()> {
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
}
