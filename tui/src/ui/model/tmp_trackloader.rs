use anyhow::{Context, Result};
use termusiclib::podcast::db::Database as DBPod;
use termusiclib::track::Track;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::{JoinHandle, JoinSet},
};

use crate::ui::{model::TxToMain, msg::Msg, track_cache::SharedTrackCache, track_id::TUITrackId};

pub type TrackLoadActorSender = UnboundedSender<TMPTrackLoadMsg>;

#[derive(Debug, PartialEq)]
pub enum TMPTrackLoadMsg {
    Track(TUITrackId),
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
            TMPTrackLoadMsg::Track(tuitrack_id) => self.load_track(tuitrack_id).await?,
            TMPTrackLoadMsg::PinnedVec(tuitrack_ids) => {
                self.load_pinned_tracks(tuitrack_ids).await?;
            }
        }

        Ok(())
    }

    /// Handle the [`LoadTrack`](TMPTrackLoadMsg::LoadTrack) message.
    async fn load_track(&self, id: TUITrackId) -> Result<()> {
        let track = Self::load_single_track(self.db_pod.clone(), id).await?;

        self.cache.write().insert_new(track);
        self.send_response(Msg::ForceRedraw);

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

    /// Handle the [`LoadPinned`](TMPTrackLoadMsg::LoadPinned) message.
    async fn load_pinned_tracks(&self, ids: Vec<TUITrackId>) -> Result<()> {
        let mut set = JoinSet::new();

        // a simple "drop(cache)" does not satisfy clippy here, so a block is used
        {
            let mut cache = self.cache.write();

            for id in ids {
                // if it already exist, no need to fetch again
                if let Some(track) = cache.try_get_cached(&id) {
                    cache.insert_new_pinned(track);
                    continue;
                }

                set.spawn(Self::load_single_track(self.db_pod.clone(), id));
            }
        }

        let res = set.join_all().await;

        let mut cache = self.cache.write();

        for res in res {
            let track = res?;

            cache.insert_new_pinned(track);
        }

        self.send_response(Msg::ForceRedraw);

        Ok(())
    }

    #[inline]
    fn send_response(&self, msg: Msg) {
        let _ = self.tx_main.send(msg);
    }
}
