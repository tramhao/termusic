use std::{num::NonZeroUsize, sync::Arc};

use anyhow::{Result, bail};
use lru::LruCache;
use parking_lot::RwLock;
use termusiclib::track::Track;

use crate::ui::{
    model::{TMPTrackLoadMsg, TrackLoadActorSender},
    track_id::TUITrackId,
};

/// The minimal amount of pinned tracks remaining before the current track in the playlist before fetching more data.
pub const PINNED_TRACKS_MIN: NonZeroUsize = NonZeroUsize::new(5).expect("Const number");
/// The amount of tracks to load for pinned once below [`PINNED_TRACKS_MIN`].
pub const PINNED_TRACKS_LOAD: NonZeroUsize = NonZeroUsize::new(10).expect("Const number");

pub type SharedTrackCache = Arc<RwLock<TrackCache>>;

/// Cache for [`Track`]s, with the key being [`TUITrackId`].
///
/// It has `pinned` tracks, which are likely to be required very soon (like next track) and less important tracks.
#[derive(Debug, Clone)]
pub struct TrackCache {
    pinned: LruCache<TUITrackId, Arc<Track>>,

    lru: LruCache<TUITrackId, Arc<Track>>,

    tx: TrackLoadActorSender,
}

impl TrackCache {
    /// The minimal Cache size for the base cache
    pub const MIN_CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(300).expect("Const number");
    /// The size of the pinend track cache.
    const PINNED_TRACKS_SIZE: NonZeroUsize = NonZeroUsize::new(20).expect("Const number");

    /// Create a new Cache for tracks, based on [`TUITrackId`] being the key, and [`Track`] being the value.
    pub fn new(tx: TrackLoadActorSender) -> Self {
        let pinned = LruCache::new(Self::PINNED_TRACKS_SIZE);
        let lru = LruCache::new(Self::MIN_CACHE_SIZE);

        Self { pinned, lru, tx }
    }

    /// Create a new Cache which can be shared.
    pub fn new_shared(tx: TrackLoadActorSender) -> SharedTrackCache {
        Arc::new(RwLock::new(Self::new(tx)))
    }

    /// Update the non-pinned cache size.
    ///
    /// This size should always be higher than the last viewport's size.
    pub fn update_capacity(&mut self, new_capacity: NonZeroUsize) {
        self.lru.resize(new_capacity);
    }

    /// Get the current capacity of the main LRU.
    pub fn capacity(&self) -> NonZeroUsize {
        self.lru.cap()
    }

    /// Try to get a cached track for the given [`TUITrackId`].
    /// If none exists, [`None`] is returned, and the actor will fetch it in the background.
    /// Once fetched, a [`Msg::ForceRedraw`](crate::ui::msg::Msg::ForceRedraw) will be send.
    pub fn try_get_track(&mut self, id: &TUITrackId) -> Option<Arc<Track>> {
        if let Some(val) = self.try_get_cached(id) {
            return Some(val);
        }

        let _ = self.tx.send(TMPTrackLoadMsg::Track(id.clone(), false));

        None
    }

    /// Try to get a cached track for the given [`TUITrackId`].
    /// If none exists, [`None`] is returned, and the actor will fetch it in the background.
    /// Once fetched, a [`Msg::Playlist`](crate::ui::msg::Msg::Playlist) with [`PLMsg::TrackNotify`](crate::ui::msg::PLMsg::TrackNotify) will be send.
    pub fn try_get_track_notify(&mut self, id: &TUITrackId) -> Option<Arc<Track>> {
        if let Some(val) = self.try_get_cached(id) {
            return Some(val);
        }

        let _ = self.tx.send(TMPTrackLoadMsg::Track(id.clone(), true));

        None
    }

    /// Try to get a cached track for the given [`TUITrackId`].
    ///
    /// Unlike [`try_get_track`](Self::try_get_track), this function does **not** request a load.
    pub fn try_get_cached(&mut self, id: &TUITrackId) -> Option<Arc<Track>> {
        if let Some(pinned) = self.pinned.get(id) {
            // Update last used time
            let _ = self.lru.promote(id);

            return Some(pinned.clone());
        }

        if let Some(cache) = self.lru.get(id) {
            return Some(cache.clone());
        }

        None
    }

    /// Unset a specific track from all caches.
    ///
    /// For example after a tag editor edit, or the file has been deleted.
    #[allow(dead_code)] // "expect" does not work here due to usage in tests
    pub fn unset(&mut self, id: &TUITrackId) {
        let _ = self.lru.pop(id);
        let _ = self.pinned.pop(id);
    }

    /// Insert new values to be cached.
    pub fn insert_new<T: Into<Arc<Track>>>(&mut self, track: T) -> Result<&Arc<Track>> {
        let track = track.into();
        let id = match TUITrackId::from_track(&track) {
            Ok(v) => v,
            Err(err) => {
                // TODO: "Track" should also have the same invariants so that we can remove this
                error!("Failed to convert to proper track for {track:#?}: {err:#?}");
                bail!("Failed to convert to proper track for {track:#?}: {err:#?}")
            }
        };
        // dont overwrite existing value, if it already exists
        Ok(self.lru.get_or_insert(id, || track))
    }

    /// Insert new values to be pinned.
    pub fn insert_new_pinned<T: Into<Arc<Track>>>(&mut self, track: T) -> Result<&Arc<Track>> {
        let track = track.into();
        let id = match TUITrackId::from_track(&track) {
            Ok(v) => v,
            Err(err) => {
                error!("Failed to convert to proper track for {track:#?}: {err:#?}");
                bail!("Failed to convert to proper track for {track:#?}: {err:#?}")
            }
        };
        // dont overwrite existing value, if it already exists
        Ok(self.pinned.get_or_insert(id, || track))
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, sync::Arc};

    use pretty_assertions::assert_eq;
    use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

    use crate::ui::model::TMPTrackLoadMsg;

    use super::{TUITrackId, Track, TrackCache};

    fn new_cache() -> (UnboundedReceiver<TMPTrackLoadMsg>, TrackCache) {
        let (tx, rx) = unbounded_channel();
        let cache = TrackCache::new(tx);
        (rx, cache)
    }

    #[test]
    fn should_send_message_to_fetch_tracks_normal_notify() {
        let (mut rx, mut cache) = new_cache();

        assert_eq!(
            cache.try_get_track(&TUITrackId::Track(PathBuf::from("/test"))),
            None
        );
        assert_eq!(rx.len(), 1);
        assert_eq!(
            rx.blocking_recv(),
            Some(TMPTrackLoadMsg::Track(
                TUITrackId::Track(PathBuf::from("/test")),
                false
            ))
        );
    }

    #[test]
    fn should_send_message_to_fetch_tracks_with_special_notify() {
        let (mut rx, mut cache) = new_cache();

        assert_eq!(
            cache.try_get_track_notify(&TUITrackId::Track(PathBuf::from("/test"))),
            None
        );
        assert_eq!(rx.len(), 1);
        assert_eq!(
            rx.blocking_recv(),
            Some(TMPTrackLoadMsg::Track(
                TUITrackId::Track(PathBuf::from("/test")),
                true
            ))
        );
    }

    #[test]
    fn should_get_from_cache() {
        let (rx, mut cache) = new_cache();

        let url1 = "https://example.com";

        assert_eq!(
            cache.insert_new(Track::new_radio(url1)).unwrap(),
            &Arc::new(Track::new_radio(url1))
        );

        assert_eq!(
            cache.try_get_track(&TUITrackId::Radio(url1.to_string())),
            Some(Arc::new(Track::new_radio(url1)))
        );
        assert_eq!(rx.len(), 0);

        let url2 = "https://example2.com";

        assert_eq!(
            cache.insert_new_pinned(Track::new_radio(url2)).unwrap(),
            &Arc::new(Track::new_radio(url2))
        );

        assert_eq!(
            cache.try_get_track(&TUITrackId::Radio(url2.to_string())),
            Some(Arc::new(Track::new_radio(url2)))
        );
        assert_eq!(rx.len(), 0);
    }

    #[test]
    fn should_properly_unset() {
        let (rx, mut cache) = new_cache();

        let url1 = "https://example.com";

        // setup
        assert_eq!(
            cache.insert_new(Track::new_radio(url1)).unwrap(),
            &Arc::new(Track::new_radio(url1))
        );
        assert_eq!(
            cache.insert_new_pinned(Track::new_radio(url1)).unwrap(),
            &Arc::new(Track::new_radio(url1))
        );

        assert_eq!(
            cache.try_get_track(&TUITrackId::Radio(url1.to_string())),
            Some(Arc::new(Track::new_radio(url1)))
        );
        assert_eq!(rx.len(), 0);

        // verify
        cache.unset(&TUITrackId::Radio(url1.to_string()));

        assert_eq!(
            cache.try_get_track(&TUITrackId::Radio(url1.to_string())),
            None
        );
        assert_eq!(rx.len(), 1);
    }
}
