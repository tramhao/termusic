use anyhow::{Context, Result};
use termusiclib::podcast::db::Database as DBPod;
use termusiclib::track::Track;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::{JoinHandle, JoinSet},
};

use crate::ui::{
    model::TxToMain,
    msg::{Msg, PLMsg},
    track_cache::SharedTrackCache,
    track_id::TUITrackId,
};

pub type TrackLoadActorSender = UnboundedSender<TMPTrackLoadMsg>;

#[derive(Debug, PartialEq)]
pub enum TMPTrackLoadMsg {
    /// Load a specific track.
    /// If the `bool` is `true`, send a [`Msg::Playlist`] with [`PLMsg::TrackNotify`] will be send.
    /// If the `bool` is `false`, send a [`Msg::ForceRedraw`].
    Track(TUITrackId, bool),
    PinnedVec(Vec<TUITrackId>),
}

/// Actor that handles all requests to the Server via GRPC.
///
/// This actor can be given commands via [`TuiCmd`] and responds on [`TxToMain`].
pub struct TrackLoadActor {
    rx_cmd: UnboundedReceiver<TMPTrackLoadMsg>,
    tx_main: TxToMain,

    cache: SharedTrackCache,
    db_pod: DBPod,
}

impl TrackLoadActor {
    /// Create and start a new [`ServerRequestActor`].
    ///
    /// To shutdown this actor, close `rx_cmd` channel.
    pub fn start_actor(
        rx_cmd: UnboundedReceiver<TMPTrackLoadMsg>,
        tx_main: TxToMain,
        cache: SharedTrackCache,
        db_pod: DBPod,
    ) -> JoinHandle<()> {
        let obj = Self {
            rx_cmd,
            tx_main,

            cache,
            db_pod,
        };

        tokio::spawn(Self::run(obj))
    }

    /// The actor loop.
    async fn run(mut actor: Self) {
        while let Some(cmd) = actor.rx_cmd.recv().await {
            if let Err(err) = actor.handle_cmd(cmd).await {
                error!("Error processing command to fetch track data: {err:#?}");
            }
        }
    }

    /// Handle all commands to the server and their responses.
    async fn handle_cmd(&mut self, cmd: TMPTrackLoadMsg) -> Result<()> {
        match cmd {
            TMPTrackLoadMsg::Track(tuitrack_id, notify) => {
                self.load_track(tuitrack_id, notify).await?;
            }
            TMPTrackLoadMsg::PinnedVec(tuitrack_ids) => {
                self.load_pinned_tracks(tuitrack_ids).await?;
            }
        }

        Ok(())
    }

    /// Handle the [`Track`](TMPTrackLoadMsg::Track) message.
    ///
    /// If `notify` is `true`, send a [`Msg::Playlist`] with [`PLMsg::TrackNotify`].
    /// If `notify` is `false`, send a [`Msg::ForceRedraw`].
    async fn load_track(&self, id: TUITrackId, notify: bool) -> Result<()> {
        let track = Self::load_single_track(self.db_pod.clone(), id).await?;

        let mut cache = self.cache.write();
        let track = cache
            .insert_new(track)
            .expect("Expected insert to not fail");

        if notify {
            self.send_response(Msg::Playlist(PLMsg::TrackNotify(track.clone())));
        } else {
            self.send_response(Msg::ForceRedraw);
        }

        Ok(())
    }

    /// Load a single [`Track`] from a [`TUITrackId`].
    async fn load_single_track(db_pod: DBPod, id: TUITrackId) -> Result<Track> {
        let track = match id {
            TUITrackId::Track(path_buf) => {
                tokio::task::spawn_blocking(move || {
                    Track::read_track_from_path(&path_buf)
                        .with_context(|| path_buf.display().to_string())
                })
                .await??
            }
            TUITrackId::Radio(url) => Track::new_radio(url),
            TUITrackId::Podcast(url) => {
                // TODO: refactor to have everything necessary send over grpc instead of having the TUI access to the database
                let ep = db_pod.get_episode_by_url(&url)?;
                Track::from_podcast_episode(&ep)
            }
        };

        Ok(track)
    }

    /// Handle the [`PinnedVec`](TMPTrackLoadMsg::PinnedVec) message.
    async fn load_pinned_tracks(&self, ids: Vec<TUITrackId>) -> Result<()> {
        let mut set = JoinSet::new();

        // a simple "drop(cache)" does not satisfy clippy here, so a block is used
        {
            let mut cache = self.cache.write();

            for id in ids {
                // if it already exist, no need to fetch again
                if let Some(track) = cache.try_get_cached(&id) {
                    // We currently dont core about the Ok value, and for the error, we dont care if it did not work
                    // A log is still put out. Ultimately, TUITrackID and Track should share the same invariants.
                    let _ = cache.insert_new_pinned(track);
                    continue;
                }

                set.spawn(Self::load_single_track(self.db_pod.clone(), id));
            }
        }

        let res = set.join_all().await;

        let mut cache = self.cache.write();

        for res in res {
            let track = res?;

            // We currently dont core about the Ok value, and for the error, we dont care if it did not work
            // A log is still put out. Ultimately, TUITrackID and Track should share the same invariants.
            let _ = cache.insert_new_pinned(track);
        }

        self.send_response(Msg::ForceRedraw);

        Ok(())
    }

    #[inline]
    fn send_response(&self, msg: Msg) {
        let _ = self.tx_main.send(msg);
    }
}
