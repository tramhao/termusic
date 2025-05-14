use std::fmt::Write as _;
use std::path::Path;

use anyhow::{bail, Context, Result};
use pathdiff::diff_paths;
use termusiclib::new_track::MediaTypes;
use termusiclib::player::playlist_helpers::{PlaylistAddTrack, PlaylistTrackSource};
use termusiclib::player::PlaylistRemoveTrackInfo;
use termusiclib::podcast::db::Database as DBPod;
use termusiclib::utils::get_parent_folder;
use termusiclib::{config::v2::server::LoopMode, new_track::Track};

/// A Playlist with all the tracks and options
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TUIPlaylist {
    tracks: Vec<Track>,
    /// Index into `tracks`, if set
    current_track_idx: Option<usize>,
    loop_mode: LoopMode,
}

impl TUIPlaylist {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    #[must_use]
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    /// Clear the current Playlist's contents.
    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_track_idx.take();
    }

    // TODO: make this explicit with the server instead of saying "cycle"
    /// Cycle through the loop modes and return the new mode.
    ///
    /// order:
    /// [Random](LoopMode::Random) -> [Playlist](LoopMode::Playlist)
    /// [Playlist](LoopMode::Playlist) -> [Single](LoopMode::Single)
    /// [Single](LoopMode::Single) -> [Random](LoopMode::Random)
    pub fn cycle_loop_mode(&mut self) -> LoopMode {
        let new_mode = match self.loop_mode {
            LoopMode::Random => LoopMode::Playlist,
            LoopMode::Playlist => LoopMode::Single,
            LoopMode::Single => LoopMode::Random,
        };

        self.set_loop_mode(new_mode);

        self.loop_mode
    }

    /// Set a specific [`LoopMode`].
    pub fn set_loop_mode(&mut self, new_mode: LoopMode) {
        self.loop_mode = new_mode;
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

        self.tracks.swap(index_a, index_b);

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

        Self::check_same_source(&items.trackid, track_at_idx.inner(), at_index)?;

        self.remove_simple(at_index)
    }

    /// Add Paths / Urls from the music service
    ///
    /// # Errors
    ///
    /// - When invalid inputs are given (non-existing path, etc)
    pub fn add_tracks(&mut self, tracks: PlaylistAddTrack, db_pod: &DBPod) -> Result<()> {
        self.tracks.reserve(tracks.tracks.len());
        let at_index = usize::try_from(tracks.at_index).unwrap();
        if at_index >= self.len() {
            // insert tracks at the end
            for track_location in tracks.tracks {
                let track = match &track_location {
                    PlaylistTrackSource::Path(path) => Self::track_from_path(path)?,
                    PlaylistTrackSource::Url(uri) => Self::track_from_uri(uri),
                    PlaylistTrackSource::PodcastUrl(uri) => {
                        Self::track_from_podcasturi(uri, db_pod)?
                    }
                };

                self.tracks.push(track);
            }

            return Ok(());
        }
        let mut at_index = at_index;
        // insert tracks at position
        for track_location in tracks.tracks {
            let track = match &track_location {
                PlaylistTrackSource::Path(path) => Self::track_from_path(path)?,
                PlaylistTrackSource::Url(uri) => Self::track_from_uri(uri),
                PlaylistTrackSource::PodcastUrl(uri) => Self::track_from_podcasturi(uri, db_pod)?,
            };

            self.tracks.insert(at_index, track);
            at_index += 1;
        }

        Ok(())
    }

    /// Create a Track from a given Path
    fn track_from_path(path_str: &str) -> Result<Track> {
        let path = Path::new(path_str);

        // Not checking that it is a supported file, as the server checks that.

        // if !filetype_supported(path_str) {
        //     error!("unsupported filetype: {path:#?}");
        //     let p = path.to_path_buf();
        //     let ext = path.extension().map(|v| v.to_string_lossy().to_string());
        //     return Err(PlaylistAddError::UnsupportedFileType(ext, p));
        // }

        // if !path.exists() {
        //     return Err(PlaylistAddError::PathDoesNotExist(path.to_path_buf()));
        // }

        // TODO: refactor to have everything necessary send over grpc instead of having the TUI reading too
        let track =
            Track::read_track_from_path(path).with_context(|| path.display().to_string())?;

        Ok(track)
    }

    /// Create a Track from a given uri (radio only)
    fn track_from_uri(uri: &str) -> Track {
        Track::new_radio(uri)
    }

    /// Create a Track from a given podcast uri
    fn track_from_podcasturi(uri: &str, db_pod: &DBPod) -> Result<Track> {
        // TODO: refactor to have everything necessary send over grpc instead of having the TUI access to the database
        let ep = db_pod.get_episode_by_url(uri)?;
        let track = Track::from_podcast_episode(&ep);

        Ok(track)
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

        Ok(())
    }

    /// Get the current track in the playlist, if there is one.
    pub fn current_track(&self) -> Option<&Track> {
        let idx = self.current_track_idx?;

        self.tracks.get(idx)
    }

    /// Completely overwrite the tracks in this playlist.
    pub fn set_tracks(&mut self, tracks: Vec<Track>) {
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
        for track in &self.tracks {
            let file = match track.inner() {
                MediaTypes::Track(track_data) => {
                    let path_relative = diff_paths(track_data.path(), parent_folder);

                    path_relative.map_or_else(
                        || track_data.path().to_string_lossy(),
                        |v| v.to_string_lossy().to_string().into(),
                    )
                }
                MediaTypes::Radio(radio_track_data) => radio_track_data.url().into(),
                MediaTypes::Podcast(podcast_track_data) => podcast_track_data.url().into(),
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
        track_inner: &MediaTypes,
        at_index: usize,
    ) -> Result<()> {
        // Error style: "Error; expected INFO_TYPE; found PLAYLIST_TYPE"
        match (info, track_inner) {
            (PlaylistTrackSource::Path(file_url), MediaTypes::Track(id)) => {
                if Path::new(&file_url) != id.path() {
                    bail!(
                        "Path mismatch, expected \"{file_url}\" at \"{at_index}\", found \"{}\"",
                        id.path().display()
                    );
                }
            }
            (PlaylistTrackSource::Url(file_url), MediaTypes::Radio(id)) => {
                if file_url != id.url() {
                    bail!(
                        "URI mismatch, expected \"{file_url}\" at \"{at_index}\", found \"{}\"",
                        id.url()
                    );
                }
            }
            (PlaylistTrackSource::PodcastUrl(file_url), MediaTypes::Podcast(id)) => {
                if file_url != id.url() {
                    bail!(
                        "URI mismatch, expected \"{file_url}\" at \"{at_index}\", found \"{}\"",
                        id.url()
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
}
