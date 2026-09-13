use std::{num::NonZeroUsize, sync::Arc};

use lru::LruCache;
use parking_lot::RwLock;
use termusiclib::track::Track;
use tokio::sync::mpsc::UnboundedSender;

use crate::ui::track_id::TUITrackId;

/// The minimal amount of pinned tracks remaining before the current track in the playlist before fetching more data.
const PINNED_TRACKS_MIN: NonZeroUsize = NonZeroUsize::new(5).expect("Const number");
/// The amount of tracks to load for pinned once below [`PINNED_TRACKS_MIN`].
const PINNED_TRACKS_LOAD: NonZeroUsize = NonZeroUsize::new(10).expect("Const number");
/// The size of the pinend track cache.
const PINNED_TRACKS_SIZE: NonZeroUsize = NonZeroUsize::new(15).expect("Const number");
/// The inital cache size of tracks.
const CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(200).expect("Const number");

type ActorSender = UnboundedSender<()>;

pub type SharedTrackCache = Arc<RwLock<TrackCache>>;

/// Cache for [`Track`]s, with the key being [`TUITrackId`].
///
/// It has `pinned` tracks, which are likely to be required very soon (like next track) and less important tracks.
#[derive(Debug, Clone)]
pub struct TrackCache {
    pinned: LruCache<TUITrackId, Arc<Track>>,

    lru: LruCache<TUITrackId, Arc<Track>>,

    tx: ActorSender,
}

impl TrackCache {
    /// Create a new Cache for tracks, based on [`TUITrackId`] being the key, and [`Track`] being the value.
    pub fn new(tx: ActorSender) -> Self {
        let pinned = LruCache::new(PINNED_TRACKS_SIZE);
        let lru = LruCache::new(CACHE_SIZE);

        Self { pinned, lru, tx }
    }

    /// Create a new Cache which can be shared.
    pub fn new_shared(tx: ActorSender) -> SharedTrackCache {
        Arc::new(RwLock::new(Self::new(tx)))
    }

    /// Update the non-pinned cache size.
    ///
    /// This size should always be higher than the last viewport's size.
    pub fn update_capacity(&mut self, new_capacity: NonZeroUsize) {
        self.lru.resize(new_capacity);
    }

    /// Try to get a cached track for the given [`TUITrackId`].
    /// If none exists, [`None`] is returned, and the actor will fetch it in the background.
    pub fn try_get_track(&mut self, id: &TUITrackId) -> Option<Arc<Track>> {
        if let Some(pinned) = self.pinned.get(id) {
            // Update last used time
            let _ = self.lru.promote(id);

            return Some(pinned.clone());
        }

        if let Some(cache) = self.lru.get(id) {
            return Some(cache.clone());
        }

        let _ = self.tx.send(());

        None
    }

    /// Unset a specific track from all caches.
    ///
    /// For example after a tag editor edit, or the file has been deleted.
    pub fn unset(&mut self, id: &TUITrackId) {
        let _ = self.lru.pop(id);
        let _ = self.pinned.pop(id);
    }

    /// Insert new values to be cached.
    pub fn insert_new(&mut self, data: Vec<Track>) {
        for track in data {
            let id = match TUITrackId::from_track(&track) {
                Ok(v) => v,
                Err(err) => {
                    error!("Failed to convert to proper track for {track:#?}: {err:#?}");
                    continue;
                }
            };
            // dont overwrite existing value, if it already exists
            let _ = self.lru.get_or_insert(id, || Arc::new(track));
        }
    }

    /// Insert new values to be pinned.
    pub fn insert_new_pinned(&mut self, data: Vec<Track>) {
        for track in data {
            let id = match TUITrackId::from_track(&track) {
                Ok(v) => v,
                Err(err) => {
                    error!("Failed to convert to proper track for {track:#?}: {err:#?}");
                    continue;
                }
            };
            // dont overwrite existing value, if it already exists
            let _ = self.pinned.get_or_insert(id, || Arc::new(track));
        }
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, sync::Arc};

    use pretty_assertions::assert_eq;
    use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

    use super::{TUITrackId, Track, TrackCache};

    fn new_cache() -> (UnboundedReceiver<()>, TrackCache) {
        let (tx, rx) = unbounded_channel();
        let cache = TrackCache::new(tx);
        (rx, cache)
    }

    #[test]
    fn should_send_message_to_fetch_tracks() {
        let (mut rx, mut cache) = new_cache();

        assert_eq!(
            cache.try_get_track(&TUITrackId::Track(PathBuf::from("/test"))),
            None
        );
        assert_eq!(rx.len(), 1);
        assert_eq!(rx.blocking_recv(), Some(()));
    }

    #[test]
    fn should_get_from_cache() {
        let (rx, mut cache) = new_cache();

        cache.insert_new(vec![Track::new_radio("https://example.com")]);

        assert_eq!(
            cache.try_get_track(&TUITrackId::Radio("https://example.com".to_string())),
            Some(Arc::new(Track::new_radio("https://example.com")))
        );
        assert_eq!(rx.len(), 0);

        cache.insert_new_pinned(vec![Track::new_radio("https://example2.com")]);

        assert_eq!(
            cache.try_get_track(&TUITrackId::Radio("https://example2.com".to_string())),
            Some(Arc::new(Track::new_radio("https://example2.com")))
        );
        assert_eq!(rx.len(), 0);
    }

    #[test]
    fn should_properly_unset() {
        let (rx, mut cache) = new_cache();

        // setup
        cache.insert_new(vec![Track::new_radio("https://example.com")]);
        cache.insert_new_pinned(vec![Track::new_radio("https://example.com")]);

        assert_eq!(
            cache.try_get_track(&TUITrackId::Radio("https://example.com".to_string())),
            Some(Arc::new(Track::new_radio("https://example.com")))
        );
        assert_eq!(rx.len(), 0);

        // verify
        cache.unset(&TUITrackId::Radio("https://example.com".to_string()));

        assert_eq!(
            cache.try_get_track(&TUITrackId::Radio("https://example.com".to_string())),
            None
        );
        assert_eq!(rx.len(), 1);
    }
}
