use std::collections::HashSet;
use std::fmt::Write as _;
use std::path::Path;

use anyhow::{Result, bail};
use pathdiff::diff_paths;
use termusiclib::config::v2::server::LoopMode;
use termusiclib::player::PlaylistRemoveTrackInfo;
use termusiclib::player::playlist_helpers::{PlaylistAddTrack, PlaylistTrackSource};
use termusiclib::utils::get_parent_folder;
use tokio::sync::mpsc::error::SendError;

use crate::ui::model::{TMPTrackLoadMsg, TrackLoadActorSender};
use crate::ui::track_cache::{PINNED_TRACKS_LOAD, PINNED_TRACKS_MIN};
use crate::ui::track_id::TUITrackId;

/// A Playlist with all the tracks and options
#[derive(Debug, Clone)]
pub struct TUIPlaylist {
    tracks: Vec<TUITrackId>,
    /// Index into `tracks`, if set
    current_track_idx: Option<usize>,
    last_cache_idx: Option<usize>,
    loop_mode: LoopMode,

    cache_tx: TrackLoadActorSender,
}

impl TUIPlaylist {
    pub fn new(tx: TrackLoadActorSender) -> Self {
        Self {
            tracks: Vec::new(),
            current_track_idx: None,
            last_cache_idx: None,
            loop_mode: LoopMode::default(),
            cache_tx: tx,
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    #[must_use]
    pub fn tracks(&self) -> &[TUITrackId] {
        &self.tracks
    }

    /// Clear the current Playlist's contents.
    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_track_idx.take();
        self.last_cache_idx.take();
    }

    /// Set a specific [`LoopMode`].
    pub fn set_loop_mode(&mut self, new_mode: LoopMode) {
        self.loop_mode = new_mode;
    }

    /// Get the current [`LoopMode`].
    pub fn loop_mode(&self) -> LoopMode {
        self.loop_mode
    }

    /// Swap specific indexes.
    ///
    /// # Errors
    ///
    /// - if either index `a` or `b` are out-of-bounds
    pub fn swap(&mut self, index_a: usize, index_b: usize) -> Result<()> {
        // "swap" panics if a index is out-of-bounds
        if index_a.max(index_b) >= self.tracks.len() {
            bail!("Index {} not within tracks bounds", index_a.max(index_b));
        }

        // maintain the correct current track index after swap
        if let Some(current_track_idx) = self.current_track_idx {
            if current_track_idx == index_a {
                self.current_track_idx = Some(index_b);
            } else if current_track_idx == index_b {
                self.current_track_idx = Some(index_a);
            }
        }

        self.tracks.swap(index_a, index_b);

        self.check_enough_cached();

        Ok(())
    }

    /// A simple `remove`.
    ///
    /// # Errors
    ///
    /// - if the index is out-of-bounds
    pub fn remove_simple(&mut self, index: usize) -> Result<()> {
        if index >= self.len() {
            bail!("Index {index} out of bound {}", self.tracks.len());
        }

        self.tracks.remove(index);

        // Update the current playing track index.
        // TODO: maybe this can be improved somehow? Maybe by providing the current track idx in each message?
        if self.current_track_idx.is_some_and(|v| v == index) {
            let _ = self.current_track_idx.take();
        } else if let Some(old_current_idx) = self.current_track_idx
            && index < old_current_idx
        {
            self.current_track_idx = Some(old_current_idx.saturating_sub(1));
        }

        self.check_enough_cached();

        Ok(())
    }

    /// Handle a `PlaylistRemove` message from the grpc interface
    ///
    /// # Errors
    ///
    /// - if the inde is out-of-bound
    /// - if same track checks fail (desync)
    pub fn handle_grpc_remove(&mut self, items: &PlaylistRemoveTrackInfo) -> Result<()> {
        let at_index = usize::try_from(items.at_index).unwrap();
        // verify that it is the track to be removed via id matching
        let Some(track_at_idx) = self.tracks().get(at_index) else {
            // this should not happen as it is verified before the loop, but just in case
            bail!("Failed to get track at index \"{at_index}\"");
        };

        Self::check_same_source(&items.trackid, track_at_idx, at_index)?;

        self.remove_simple(at_index)
    }

    /// Add Paths / Urls from the music service
    ///
    /// # Errors
    ///
    /// - When invalid inputs are given (non-existing path, etc)
    pub fn add_tracks(&mut self, tracks: PlaylistAddTrack) -> Result<()> {
        self.tracks.reserve(tracks.tracks.len());
        let at_index = usize::try_from(tracks.at_index).unwrap();
        if at_index >= self.len() {
            // insert tracks at the end
            for track_location in tracks.tracks {
                let id = TUITrackId::from_track_source(track_location)?;

                self.tracks.push(id);
            }

            return Ok(());
        }
        // insert tracks at position
        for (at_index, track_location) in (at_index..).zip(tracks.tracks) {
            let id = TUITrackId::from_track_source(track_location)?;

            self.tracks.insert(at_index, id);
        }

        Ok(())
    }

    #[must_use]
    pub fn current_track_index(&self) -> Option<usize> {
        self.current_track_idx.as_ref().copied()
    }

    /// Set the current track index.
    ///
    /// # Errors
    ///
    /// - if the index is out-of-bounds
    pub fn set_current_track_index(&mut self, index: usize) -> Result<()> {
        if index >= self.tracks.len() {
            bail!("Index {index} out of bound {}", self.tracks.len());
        }

        self.current_track_idx = Some(index);

        self.check_enough_cached();

        Ok(())
    }

    /// Get the current track in the playlist, if there is one.
    pub fn current_track(&self) -> Option<&TUITrackId> {
        let idx = self.current_track_idx?;

        self.tracks.get(idx)
    }

    /// Completely overwrite the tracks in this playlist.
    pub fn set_tracks(&mut self, tracks: Vec<TUITrackId>) {
        self.tracks = tracks;
        // remove the current index, as it is unknown if the data is the same
        self.current_track_idx.take();
    }

    // TODO: move "save_m3u" to server-side
    /// Export the current playlist to a `.m3u` playlist file.
    ///
    /// # Errors
    ///
    /// Error could happen when writing file to local disk.
    pub fn save_m3u(&self, filename: &Path) -> Result<()> {
        if self.tracks.is_empty() {
            bail!("Unable to save since the playlist is empty.");
        }

        let parent_folder = get_parent_folder(filename);

        let m3u = self.get_m3u_file(&parent_folder);

        std::fs::write(filename, m3u)?;
        Ok(())
    }

    /// Generate the m3u's file content.
    ///
    /// All Paths are relative to the `parent_folder` directory.
    fn get_m3u_file(&self, parent_folder: &Path) -> String {
        let mut m3u = String::from("#EXTM3U\n");
        for id in &self.tracks {
            let file = match id {
                TUITrackId::Track(track_data) => {
                    let path_relative = diff_paths(track_data, parent_folder);

                    path_relative.map_or_else(
                        || track_data.to_string_lossy(),
                        |v| v.to_string_lossy().to_string().into(),
                    )
                }
                TUITrackId::Radio(radio_track_data) => radio_track_data.into(),
                TUITrackId::Podcast(podcast_track_data) => podcast_track_data.into(),
            };

            let _ = writeln!(m3u, "{file}");
        }
        m3u
    }

    /// Check that the given `info` track source matches the given `track_inner` types.
    ///
    /// # Errors
    ///
    /// if they dont match
    pub fn check_same_source(
        info: &PlaylistTrackSource,
        track_inner: &TUITrackId,
        at_index: usize,
    ) -> Result<()> {
        // Error style: "Error; expected INFO_TYPE; found PLAYLIST_TYPE"
        match (info, track_inner) {
            (PlaylistTrackSource::Path(file_url), TUITrackId::Track(id)) => {
                if Path::new(&file_url) != id {
                    bail!(
                        "Path mismatch, expected \"{file_url}\" at \"{at_index}\", found \"{}\"",
                        id.display()
                    );
                }
            }
            (PlaylistTrackSource::Url(file_url), TUITrackId::Radio(id))
            | (PlaylistTrackSource::PodcastUrl(file_url), TUITrackId::Podcast(id)) => {
                if file_url != id {
                    bail!(
                        "URI mismatch, expected \"{file_url}\" at \"{at_index}\", found \"{id}\"",
                    );
                }
            }
            (expected, got) => {
                bail!(
                    "Type mismatch, expected \"{expected:#?}\" at \"{at_index}\" found \"{got:#?}\""
                );
            }
        }

        Ok(())
    }

    /// Check if enough pinned entries are cached, if not, cache more.
    fn check_enough_cached(&self) {
        let Some(current_idx) = self.current_track_index() else {
            return;
        };
        // we dont actually know what will be next, so not much predictive caching can be done.
        if self.loop_mode == LoopMode::Random {
            return;
        }

        let range = if self.last_cache_idx.is_none() {
            let until = (current_idx + PINNED_TRACKS_LOAD.get()).min(self.tracks.len());
            current_idx..until
        } else if let Some(last_idx) = self.last_cache_idx.as_ref() {
            let max = last_idx.max(&current_idx);
            let min = last_idx.min(&current_idx);

            if max - min > PINNED_TRACKS_MIN.get() {
                if *max == current_idx {
                    // direction is increasing
                    current_idx..(current_idx + PINNED_TRACKS_LOAD.get()).min(self.tracks.len())
                } else {
                    // direction is decreasing
                    current_idx - PINNED_TRACKS_LOAD.get()..current_idx
                }
            } else {
                return;
            }
        } else {
            return;
        };

        let tracks = HashSet::from_iter(self.tracks[range].iter().map(Clone::clone));
        let _ = self.cache_tx.send(TMPTrackLoadMsg::PinnedVec(tracks));
    }

    /// Send the given message to the Track loader.
    ///
    /// This function is meant to be temorary until the track loader is integrated via protobuf.
    pub fn request_data(&self, msg: TMPTrackLoadMsg) -> Result<(), SendError<TMPTrackLoadMsg>> {
        self.cache_tx.send(msg)
    }
}
