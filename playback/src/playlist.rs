use std::error::Error;
use std::fmt::{Display, Write as _};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use parking_lot::RwLock;
use pathdiff::diff_paths;
use rand::Rng;
use rand::seq::SliceRandom;
use termusiclib::config::SharedServerSettings;
use termusiclib::config::v2::server::LoopMode;
use termusiclib::player::PlaylistLoopModeInfo;
use termusiclib::player::PlaylistShuffledInfo;
use termusiclib::player::PlaylistSwapInfo;
use termusiclib::player::PlaylistTracks;
use termusiclib::player::UpdateEvents;
use termusiclib::player::UpdatePlaylistEvents;
use termusiclib::player::playlist_helpers::PlaylistPlaySpecific;
use termusiclib::player::playlist_helpers::PlaylistSwapTrack;
use termusiclib::player::playlist_helpers::PlaylistTrackSource;
use termusiclib::player::playlist_helpers::{PlaylistAddTrack, PlaylistRemoveTrackIndexed};
use termusiclib::player::{self, RunningStatus};
use termusiclib::player::{PlaylistAddTrackInfo, PlaylistRemoveTrackInfo};
use termusiclib::podcast::{db::Database as DBPod, episode::Episode};
use termusiclib::track::{MediaTypes, Track, TrackData};
use termusiclib::utils::{filetype_supported, get_app_config_path, get_parent_folder};

use crate::SharedPlaylist;
use crate::StreamTX;

#[derive(Debug)]
pub struct Playlist {
    /// All tracks in the playlist
    tracks: Vec<Track>,
    /// Index into `tracks` of which the current playing track is
    current_track_index: usize,
    /// Index into `tracks` for the next track to play after the current.
    ///
    /// Practically only used for pre-enqueue / pre-fetch / gapless.
    next_track_index: Option<usize>,
    /// The currently playing [`Track`]. Does not need to be in `tracks`
    current_track: Option<Track>,
    /// The current playing running status of the playlist
    status: RunningStatus,
    /// The loop-/play-mode for the playlist
    loop_mode: LoopMode,
    /// Indexes into `tracks` that have been previously been played (for `previous`)
    played_index: Vec<usize>,
    /// Indicator if the playlist should advance the `current_*` and `next_*` values
    need_proceed_to_next: bool,
    stream_tx: StreamTX,

    /// Indicator if we need to save the playlist for interval saving
    is_modified: bool,
}

impl Playlist {
    /// Create a new playlist instance with 0 tracks
    pub fn new(config: &SharedServerSettings, stream_tx: StreamTX) -> Self {
        // TODO: shouldn't "loop_mode" be combined with the config ones?
        let loop_mode = config.read().settings.player.loop_mode;
        let current_track = None;

        Self {
            tracks: Vec::new(),
            status: RunningStatus::Stopped,
            loop_mode,
            current_track_index: 0,
            current_track,
            played_index: Vec::new(),
            next_track_index: None,
            need_proceed_to_next: false,
            stream_tx,
            is_modified: false,
        }
    }

    /// Create a new Playlist instance that is directly shared
    ///
    /// # Errors
    ///
    /// see [`load`](Self::load)
    pub fn new_shared(
        config: &SharedServerSettings,
        stream_tx: StreamTX,
    ) -> Result<SharedPlaylist> {
        let mut playlist = Self::new(config, stream_tx);
        playlist.load_apply()?;

        Ok(Arc::new(RwLock::new(playlist)))
    }

    /// Advance the playlist to the next track.
    pub fn proceed(&mut self) {
        debug!("need to proceed to next: {}", self.need_proceed_to_next);
        self.is_modified = true;
        if self.need_proceed_to_next {
            self.next();
        } else {
            self.need_proceed_to_next = true;
        }
    }

    /// Set `need_proceed_to_next` to `false`
    pub fn proceed_false(&mut self) {
        self.need_proceed_to_next = false;
    }

    /// Load the playlist from the file.
    ///
    /// Path in `$config$/playlist.log`.
    ///
    /// Returns `(Position, Tracks[])`.
    ///
    /// # Errors
    /// - When the playlist path is not write-able
    /// - When podcasts cannot be loaded
    pub fn load() -> Result<(usize, Vec<Track>)> {
        let path = get_playlist_path()?;

        let Ok(file) = File::open(&path) else {
            // new file, nothing to parse from it
            File::create(&path)?;

            return Ok((0, Vec::new()));
        };

        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut current_track_index = 0;
        if let Some(line) = lines.next() {
            let index_line = line?;
            if let Ok(index) = index_line.trim().parse() {
                current_track_index = index;
            }
        } else {
            // empty file, nothing to parse from it
            return Ok((0, Vec::new()));
        }

        let mut playlist_items = Vec::new();
        let db_path = get_app_config_path()?;
        let db_podcast = DBPod::new(&db_path)?;
        let podcasts = db_podcast
            .get_podcasts()
            .with_context(|| "failed to get podcasts from db.")?;
        for line in lines {
            let line = line?;

            let trimmed_line = line.trim();

            // skip empty lines without trying to process them
            // skip lines that are comments (m3u-like)
            if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
                continue;
            }

            if line.starts_with("http") {
                let mut is_podcast = false;
                'outer: for pod in &podcasts {
                    for ep in &pod.episodes {
                        if ep.url == line.as_str() {
                            is_podcast = true;
                            let track = Track::from_podcast_episode(ep);
                            playlist_items.push(track);
                            break 'outer;
                        }
                    }
                }
                if !is_podcast {
                    let track = Track::new_radio(&line);
                    playlist_items.push(track);
                }
                continue;
            }
            if let Ok(track) = Track::read_track_from_path(&line) {
                playlist_items.push(track);
            }
        }

        // protect against the listed index in the playlist file not matching the elements in the playlist
        // for example lets say it has "100", but there are only 2 elements in the playlist
        let current_track_index = current_track_index.min(playlist_items.len().saturating_sub(1));

        Ok((current_track_index, playlist_items))
    }

    /// Run [`load`](Self::load), but also apply the values directly to the current instance.
    ///
    /// # Errors
    ///
    /// See [`load`](Self::load)
    pub fn load_apply(&mut self) -> Result<()> {
        let (current_track_index, tracks) = Self::load()?;
        self.current_track_index = current_track_index;
        self.tracks = tracks;
        self.is_modified = false;

        Ok(())
    }

    /// Load Tracks from a GRPC response.
    ///
    /// Returns `(Position, Tracks[])`.
    ///
    /// # Errors
    ///
    /// - when converting from u64 grpc values to usize fails
    /// - when there is no track-id
    /// - when reading a Track from path or podcast database fails
    pub fn load_from_grpc(&mut self, info: PlaylistTracks, podcast_db: &DBPod) -> Result<()> {
        let current_track_index = usize::try_from(info.current_track_index)
            .context("convert current_track_index(u64) to usize")?;
        let mut playlist_items = Vec::with_capacity(info.tracks.len());

        for (idx, track) in info.tracks.into_iter().enumerate() {
            let at_index_usize =
                usize::try_from(track.at_index).context("convert at_index(u64) to usize")?;
            // assume / require that the tracks are ordered correctly, if not just log a error for now
            if idx != at_index_usize {
                error!("Non-matching \"index\" and \"at_index\"!");
            }

            // this case should never happen with "termusic-server", but grpc marks them as "optional"
            let Some(id) = track.id else {
                bail!("Track does not have a id, which is required to load!");
            };

            let track = match PlaylistTrackSource::try_from(id)? {
                PlaylistTrackSource::Path(v) => Track::read_track_from_path(v)?,
                PlaylistTrackSource::Url(v) => Track::new_radio(&v),
                PlaylistTrackSource::PodcastUrl(v) => {
                    let episode = podcast_db.get_episode_by_url(&v)?;
                    Track::from_podcast_episode(&episode)
                }
            };

            playlist_items.push(track);
        }

        self.current_track_index = current_track_index;
        self.tracks = playlist_items;
        self.is_modified = true;

        Ok(())
    }

    /// Reload the current playlist from the file. This function does not save beforehand.
    ///
    /// This is currently 1:1 the same as [`Self::load_apply`],
    /// but has some slight different semantic meaning in that [`Self::load_apply`] is meant for a new Playlist instance.
    ///
    /// # Errors
    ///
    /// See [`Self::load`]
    pub fn reload_tracks(&mut self) -> Result<()> {
        let (current_track_index, tracks) = Self::load()?;
        self.tracks = tracks;
        self.current_track_index = current_track_index;
        self.is_modified = false;

        Ok(())
    }

    /// Save the current playlist and playing index to the playlist log
    ///
    /// Path in `$config$/playlist.log`
    ///
    /// # Errors
    ///
    /// Errors could happen when writing files
    pub fn save(&mut self) -> Result<()> {
        let path = get_playlist_path()?;

        let file = File::create(&path)?;

        // If the playlist is empty, truncate the file, but dont write anything else (like a index number)
        if self.is_empty() {
            self.is_modified = false;
            return Ok(());
        }

        let mut writer = BufWriter::new(file);
        writer.write_all(self.current_track_index.to_string().as_bytes())?;
        writer.write_all(b"\n")?;
        for track in &self.tracks {
            let id = match track.inner() {
                MediaTypes::Track(track_data) => track_data.path().to_string_lossy(),
                MediaTypes::Radio(radio_track_data) => radio_track_data.url().into(),
                MediaTypes::Podcast(podcast_track_data) => podcast_track_data.url().into(),
            };
            writeln!(writer, "{id}")?;
        }

        writer.flush()?;
        self.is_modified = false;

        Ok(())
    }

    /// Run [`Self::save`] only if [`Self::is_modified`] is `true`.
    ///
    /// This is mainly used for saving in intervals and not writing if nothing changed.
    ///
    /// Returns `true` if saving was performed.
    ///
    /// # Errors
    ///
    /// See [`Self::save`]
    pub fn save_if_modified(&mut self) -> Result<bool> {
        if self.is_modified {
            self.save()?;

            return Ok(true);
        }

        Ok(false)
    }

    /// Change to the next track.
    pub fn next(&mut self) {
        self.played_index.push(self.current_track_index);
        // Note: the next index is *not* taken here, as ".proceed/next" is called first,
        // then "has_next_track" is later used to check if enqueuing has used.
        if let Some(index) = self.next_track_index {
            self.current_track_index = index;
            return;
        }
        self.current_track_index = self.get_next_track_index();
    }

    /// Check that the given `info` track source matches the given `track_inner` types.
    ///
    /// # Errors
    ///
    /// if they dont match
    fn check_same_source(
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

    /// Skip to a specific track in the playlist
    ///
    /// # Errors
    ///
    /// - if converting u64 to usize fails
    /// - if the given info's tracks mismatch with the actual playlist
    pub fn play_specific(&mut self, info: &PlaylistPlaySpecific) -> Result<()> {
        let new_index =
            usize::try_from(info.track_index).context("convert track_index(u64) to usize")?;

        let Some(track_at_idx) = self.tracks.get(new_index) else {
            bail!("Index {new_index} is out of bound {}", self.tracks.len())
        };

        Self::check_same_source(&info.id, track_at_idx.inner(), new_index)?;

        self.played_index.push(self.current_track_index);
        self.set_next_track(None);
        self.set_current_track_index(new_index);
        self.proceed_false();
        self.is_modified = true;

        Ok(())
    }

    /// Get the next track index based on the [`LoopMode`] used.
    fn get_next_track_index(&self) -> usize {
        let mut next_track_index = self.current_track_index;
        match self.loop_mode {
            LoopMode::Single => {}
            LoopMode::Playlist => {
                next_track_index += 1;
                if next_track_index >= self.len() {
                    next_track_index = 0;
                }
            }
            LoopMode::Random => {
                next_track_index = self.get_random_index();
            }
        }
        next_track_index
    }

    /// Change to the previous track played.
    ///
    /// This uses `played_index` vec, if available, otherwise uses [`LoopMode`].
    pub fn previous(&mut self) {
        // unset next track as we now want a previous track instead of the next enqueued
        self.set_next_track(None);

        if !self.played_index.is_empty()
            && let Some(index) = self.played_index.pop()
        {
            self.current_track_index = index;
            self.is_modified = true;
            return;
        }
        match self.loop_mode {
            LoopMode::Single => {}
            LoopMode::Playlist => {
                if self.current_track_index == 0 {
                    self.current_track_index = self.len() - 1;
                } else {
                    self.current_track_index -= 1;
                }
            }
            LoopMode::Random => {
                self.current_track_index = self.get_random_index();
            }
        }
        self.is_modified = true;
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    /// Swap the `index` with the one below(+1) it, if there is one.
    pub fn swap_down(&mut self, index: usize) {
        if index < self.len().saturating_sub(1) {
            self.tracks.swap(index, index + 1);
            // handle index
            if index == self.current_track_index {
                self.current_track_index += 1;
            } else if index == self.current_track_index - 1 {
                self.current_track_index -= 1;
            }
            self.is_modified = true;
        }
    }

    /// Swap the `index` with the one above(-1) it, if there is one.
    pub fn swap_up(&mut self, index: usize) {
        if index > 0 {
            self.tracks.swap(index, index - 1);
            // handle index
            if index == self.current_track_index {
                self.current_track_index -= 1;
            } else if index == self.current_track_index + 1 {
                self.current_track_index += 1;
            }
            self.is_modified = true;
        }
    }

    /// Swap specific indexes, sends swap event.
    ///
    /// # Errors
    ///
    /// - if either index `a` or `b` are out-of-bounds
    ///
    /// # Panics
    ///
    /// If `usize` cannot be converted to `u64`
    pub fn swap(&mut self, index_a: usize, index_b: usize) -> Result<()> {
        // "swap" panics if a index is out-of-bounds
        if index_a.max(index_b) >= self.tracks.len() {
            bail!("Index {} not within tracks bounds", index_a.max(index_b));
        }

        self.tracks.swap(index_a, index_b);

        let index_a = u64::try_from(index_a).unwrap();
        let index_b = u64::try_from(index_b).unwrap();

        self.send_stream_ev_pl(UpdatePlaylistEvents::PlaylistSwapTracks(PlaylistSwapInfo {
            index_a,
            index_b,
        }));
        self.is_modified = true;

        Ok(())
    }

    /// Get the current track's Path/Url.
    // TODO: refactor this function to likely return either a consistent URI format or a enum
    // TODO: refactor to return a reference if possible
    pub fn get_current_track(&mut self) -> Option<String> {
        let mut result = None;
        if let Some(track) = self.current_track() {
            match track.inner() {
                MediaTypes::Track(track_data) => {
                    result = Some(track_data.path().to_string_lossy().to_string());
                }
                MediaTypes::Radio(radio_track_data) => {
                    result = Some(radio_track_data.url().to_string());
                }
                MediaTypes::Podcast(podcast_track_data) => {
                    result = Some(podcast_track_data.url().to_string());
                }
            }
        }
        result
    }

    /// Get the next track index and return a reference to it.
    pub fn fetch_next_track(&mut self) -> Option<&Track> {
        let next_index = self.get_next_track_index();
        self.next_track_index = Some(next_index);
        self.tracks.get(next_index)
    }

    /// Set the [`RunningStatus`] of the playlist, also sends a stream event.
    pub fn set_status(&mut self, status: RunningStatus) {
        self.status = status;
        self.send_stream_ev(UpdateEvents::PlayStateChanged {
            playing: status.as_u32(),
        });
    }

    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.status == RunningStatus::Stopped
    }

    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.status == RunningStatus::Paused
    }

    #[must_use]
    pub fn status(&self) -> RunningStatus {
        self.status
    }

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

    /// Set a specific [`LoopMode`], also sends a event that the mode changed.
    /// Only sets & sends a event if the new mode is not the same as the old one.
    pub fn set_loop_mode(&mut self, new_mode: LoopMode) {
        // dont set and dont send a event if the mode is the same
        if new_mode == self.loop_mode {
            return;
        }

        self.loop_mode = new_mode;

        self.send_stream_ev_pl(UpdatePlaylistEvents::PlaylistLoopMode(
            PlaylistLoopModeInfo::from(self.loop_mode),
        ));
    }

    /// Export the current playlist to a `.m3u` playlist file.
    ///
    /// Might be confused with [save](Self::save).
    ///
    /// # Errors
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

    /// Add a podcast episode to the playlist.
    ///
    /// # Panics
    ///
    /// This should never happen as a podcast url has already been a string, so conversion should not fail
    pub fn add_episode(&mut self, ep: &Episode) {
        let track = Track::from_podcast_episode(ep);

        let url = track.as_podcast().unwrap().url();

        self.send_stream_ev_pl(UpdatePlaylistEvents::PlaylistAddTrack(
            PlaylistAddTrackInfo {
                at_index: u64::try_from(self.tracks.len()).unwrap(),
                title: track.title().map(ToOwned::to_owned),
                duration: track.duration().unwrap_or_default(),
                // Note: Safe unwrap, as a podcast uri is always a uri, not a path (which has been a string before)
                trackid: PlaylistTrackSource::PodcastUrl(url.to_owned()),
            },
        ));

        self.tracks.push(track);
        self.is_modified = true;
    }

    /// Add many Paths/Urls to the playlist.
    ///
    /// # Errors
    /// - When invalid inputs are given
    /// - When the file(s) cannot be read correctly
    pub fn add_playlist<T: AsRef<str>>(&mut self, vec: &[T]) -> Result<(), PlaylistAddErrorVec> {
        let mut errors = PlaylistAddErrorVec::default();
        for item in vec {
            let Err(err) = self.add_track(item) else {
                continue;
            };
            errors.push(err);
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }

    /// Add a single Path/Url to the playlist
    ///
    /// # Errors
    /// - When invalid inputs are given (non-existing path, unsupported file types, etc)
    ///
    /// # Panics
    ///
    /// If `usize` cannot be converted to `u64`
    pub fn add_track<T: AsRef<str>>(&mut self, track: &T) -> Result<(), PlaylistAddError> {
        let track_str = track.as_ref();
        if track_str.starts_with("http") {
            let track = Self::track_from_uri(track_str);
            self.tracks.push(track);
            self.is_modified = true;
            return Ok(());
        }

        let track = Self::track_from_path(track_str)?;

        self.send_stream_ev_pl(UpdatePlaylistEvents::PlaylistAddTrack(
            PlaylistAddTrackInfo {
                at_index: u64::try_from(self.tracks.len()).unwrap(),
                title: track.title().map(ToOwned::to_owned),
                duration: track.duration().unwrap_or_default(),
                trackid: PlaylistTrackSource::Path(track_str.to_string()),
            },
        ));

        self.tracks.push(track);
        self.is_modified = true;

        Ok(())
    }

    /// Convert [`PlaylistTrackSource`] to [`Track`] by calling the correct functions.
    ///
    /// This mainly exist to de-duplicate this match and resulting error handling.
    fn source_to_track(track_location: &PlaylistTrackSource, db_pod: &DBPod) -> Result<Track> {
        let track = match track_location {
            PlaylistTrackSource::Path(path) => Self::track_from_path(path)?,
            PlaylistTrackSource::Url(uri) => Self::track_from_uri(uri),
            PlaylistTrackSource::PodcastUrl(uri) => Self::track_from_podcasturi(uri, db_pod)?,
        };

        Ok(track)
    }

    /// Add Paths / Urls from the music service
    ///
    /// # Errors
    ///
    /// On error on a specific track, the error will be collected and the remaining tracks will be tried to be added.
    ///
    /// - if adding a track results in a error (path not found, unsupported file types, not enough permissions, etc)
    ///
    /// # Panics
    ///
    /// If `usize` cannot be converted to `u64`
    pub fn add_tracks(
        &mut self,
        tracks: PlaylistAddTrack,
        db_pod: &DBPod,
    ) -> Result<(), PlaylistAddErrorCollection> {
        self.tracks.reserve(tracks.tracks.len());
        let at_index = usize::try_from(tracks.at_index).unwrap();
        // collect non-fatal errors to continue adding the rest of the tracks
        let mut errors: Vec<anyhow::Error> = Vec::new();

        info!(
            "Trying to add {} tracks to the playlist",
            tracks.tracks.len()
        );

        let mut added_tracks = 0;

        if at_index >= self.len() {
            // insert tracks at the end
            for track_location in tracks.tracks {
                let track = match Self::source_to_track(&track_location, db_pod) {
                    Ok(v) => v,
                    Err(err) => {
                        warn!("Error adding track: {err}");
                        errors.push(err);
                        continue;
                    }
                };

                self.send_stream_ev_pl(UpdatePlaylistEvents::PlaylistAddTrack(
                    PlaylistAddTrackInfo {
                        at_index: u64::try_from(self.tracks.len()).unwrap(),
                        title: track.title().map(ToOwned::to_owned),
                        duration: track.duration().unwrap_or_default(),
                        trackid: track_location,
                    },
                ));

                self.tracks.push(track);
                self.is_modified = true;
                added_tracks += 1;
            }
        } else {
            let mut at_index = at_index;
            // insert tracks at position
            for track_location in tracks.tracks {
                let track = match Self::source_to_track(&track_location, db_pod) {
                    Ok(v) => v,
                    Err(err) => {
                        warn!("Error adding track: {err}");
                        errors.push(err);
                        continue;
                    }
                };

                self.send_stream_ev_pl(UpdatePlaylistEvents::PlaylistAddTrack(
                    PlaylistAddTrackInfo {
                        at_index: u64::try_from(at_index).unwrap(),
                        title: track.title().map(ToOwned::to_owned),
                        duration: track.duration().unwrap_or_default(),
                        trackid: track_location,
                    },
                ));

                self.tracks.insert(at_index, track);
                self.is_modified = true;
                at_index += 1;
                added_tracks += 1;
            }
        }

        info!("Added {} tracks with {} errors", added_tracks, errors.len());

        if !errors.is_empty() {
            return Err(PlaylistAddErrorCollection::from(errors));
        }

        Ok(())
    }

    /// Remove Tracks from the music service
    ///
    /// # Errors
    ///
    /// - if the `at_index` is not within `self.tracks` bounds
    /// - if `at_index + tracks.len` is not within bounds
    /// - if the tracks type and URI mismatch
    ///
    /// # Panics
    ///
    /// If `usize` cannot be converted to `u64`
    pub fn remove_tracks(&mut self, tracks: PlaylistRemoveTrackIndexed) -> Result<()> {
        let at_index = usize::try_from(tracks.at_index).unwrap();

        if at_index >= self.tracks.len() {
            bail!(
                "at_index is higher than the length of the playlist! at_index is \"{at_index}\" and playlist length is \"{}\"",
                self.tracks.len()
            );
        }

        if at_index + tracks.tracks.len().saturating_sub(1) >= self.tracks.len() {
            bail!(
                "at_index + tracks to remove is higher than the length of the playlist! playlist length is \"{}\"",
                self.tracks.len()
            );
        }

        for input_track in tracks.tracks {
            // verify that it is the track to be removed via id matching
            let Some(track_at_idx) = self.tracks.get(at_index) else {
                // this should not happen as it is verified before the loop, but just in case
                bail!("Failed to get track at index \"{at_index}\"");
            };

            Self::check_same_source(&input_track, track_at_idx.inner(), at_index)?;

            // verified that at index "at_index" the track is of the type and has the URI that was requested to be removed
            self.handle_remove(at_index);

            self.send_stream_ev_pl(UpdatePlaylistEvents::PlaylistRemoveTrack(
                PlaylistRemoveTrackInfo {
                    at_index: u64::try_from(at_index).unwrap(),
                    trackid: input_track,
                },
            ));
        }

        Ok(())
    }

    /// Create a Track from a given Path
    #[allow(clippy::unnecessary_debug_formatting)] // we want debug information about a path (especially have it escaped)
    fn track_from_path(path_str: &str) -> Result<Track, PlaylistAddError> {
        let path = Path::new(path_str);

        if !filetype_supported(path) {
            error!("unsupported filetype: {path:#?}");
            let p = path.to_path_buf();
            let ext = path.extension().map(|v| v.to_string_lossy().to_string());
            return Err(PlaylistAddError::UnsupportedFileType(ext, p));
        }

        if !path.exists() {
            return Err(PlaylistAddError::PathDoesNotExist(path.to_path_buf()));
        }

        let track = Track::read_track_from_path(path)
            .map_err(|err| PlaylistAddError::ReadError(err, path.to_path_buf()))?;

        Ok(track)
    }

    /// Create a Track from a given uri (radio only)
    fn track_from_uri(uri: &str) -> Track {
        Track::new_radio(uri)
    }

    /// Create a Track from a given podcast uri
    fn track_from_podcasturi(uri: &str, db_pod: &DBPod) -> Result<Track> {
        let ep = db_pod.get_episode_by_url(uri)?;
        let track = Track::from_podcast_episode(&ep);

        Ok(track)
    }

    /// Swap tracks based on [`PlaylistSwapTrack`]
    ///
    /// # Errors
    ///
    /// - if either the `a` or `b` indexes are not within bounds
    /// - if the indexes cannot be converted to `usize`
    ///
    /// # Panics
    ///
    /// If `usize` cannot be converted to `u64`
    pub fn swap_tracks(&mut self, info: &PlaylistSwapTrack) -> Result<()> {
        let index_a =
            usize::try_from(info.index_a).context("Failed to convert index_a to usize")?;
        let index_b =
            usize::try_from(info.index_b).context("Failed to convert index_b to usize")?;

        self.swap(index_a, index_b)?;

        Ok(())
    }

    #[must_use]
    pub fn tracks(&self) -> &Vec<Track> {
        &self.tracks
    }

    /// Remove the track at `index`. Does not modify `current_track`.
    ///
    /// # Panics
    ///
    /// if usize cannot be converted to u64
    pub fn remove(&mut self, index: usize) {
        let Some(track) = self.tracks.get(index) else {
            error!("Index {index} out of bound {}", self.tracks.len());
            return;
        };

        let track_source = track.as_track_source();

        self.handle_remove(index);

        self.send_stream_ev_pl(UpdatePlaylistEvents::PlaylistRemoveTrack(
            PlaylistRemoveTrackInfo {
                at_index: u64::try_from(index).unwrap(),
                trackid: track_source,
            },
        ));
    }

    /// Internal common `remove` handling, does not send a event
    fn handle_remove(&mut self, index: usize) {
        self.tracks.remove(index);

        // Handle index
        if index <= self.current_track_index {
            // nothing needs to be done if the index is already 0
            if self.current_track_index != 0 {
                self.current_track_index -= 1;
            }
        }
    }

    /// Clear the current playlist.
    /// This does not stop the playlist or clear [`current_track`](Self::current_track).
    pub fn clear(&mut self) {
        self.tracks.clear();
        self.played_index.clear();
        self.next_track_index.take();
        self.current_track_index = 0;
        self.need_proceed_to_next = false;

        self.send_stream_ev_pl(UpdatePlaylistEvents::PlaylistCleared);
    }

    /// Shuffle the playlist
    ///
    /// # Panics
    ///
    /// see [`as_grpc_playlist_tracks#Errors`](Self::as_grpc_playlist_tracks)
    pub fn shuffle(&mut self) {
        let current_track_file = self.get_current_track();

        self.tracks.shuffle(&mut rand::rng());

        if let Some(current_track_file) = current_track_file
            && let Some(index) = self.find_index_from_file(&current_track_file)
        {
            self.current_track_index = index;
        }

        self.send_stream_ev_pl(UpdatePlaylistEvents::PlaylistShuffled(
            PlaylistShuffledInfo {
                tracks: self.as_grpc_playlist_tracks().unwrap(),
            },
        ));
    }

    /// Get the current tracks and state as a GRPC [`PlaylistTracks`] object.
    ///
    /// # Errors
    ///
    /// - if some track does not have a file-id
    /// - converting usize to u64 fails
    pub fn as_grpc_playlist_tracks(&self) -> Result<PlaylistTracks> {
        let tracks = self
            .tracks()
            .iter()
            .enumerate()
            .map(|(idx, track)| {
                let at_index = u64::try_from(idx).context("track index(usize) to u64")?;
                let track_source = track.as_track_source();

                Ok(player::PlaylistAddTrack {
                    at_index,
                    duration: Some(track.duration().unwrap_or_default().into()),
                    id: Some(track_source.into()),
                    optional_title: None,
                })
            })
            .collect::<Result<_>>()?;

        Ok(PlaylistTracks {
            current_track_index: u64::try_from(self.get_current_track_index())
                .context("current_track_index(usize) to u64")?,
            tracks,
        })
    }

    /// Find the index in the playlist for `item`, if it exists there.
    fn find_index_from_file(&self, item: &str) -> Option<usize> {
        for (index, track) in self.tracks.iter().enumerate() {
            let file = match track.inner() {
                MediaTypes::Track(track_data) => track_data.path().to_string_lossy(),
                MediaTypes::Radio(radio_track_data) => radio_track_data.url().into(),
                MediaTypes::Podcast(podcast_track_data) => podcast_track_data.url().into(),
            };
            if file == item {
                return Some(index);
            }
        }
        None
    }

    /// Get a random index in the playlist.
    fn get_random_index(&self) -> usize {
        let mut random_index = self.current_track_index;

        if self.len() <= 1 {
            return 0;
        }

        let mut rng = rand::rng();
        while self.current_track_index == random_index {
            random_index = rng.random_range(0..self.len());
        }

        random_index
    }

    /// Remove all tracks from the playlist that dont exist on the disk.
    ///
    /// # Panics
    ///
    /// if usize cannot be converted to u64
    pub fn remove_deleted_items(&mut self) {
        if let Some(current_track_file) = self.get_current_track() {
            let len = self.tracks.len();
            let old_tracks = std::mem::replace(&mut self.tracks, Vec::with_capacity(len));

            for track in old_tracks {
                let Some(path) = track.as_track().map(TrackData::path) else {
                    continue;
                };

                if path.exists() {
                    self.tracks.push(track);
                    continue;
                }

                let track_source = track.as_track_source();

                // the index of the playlist where this item is deleted
                // this must be the index after other indexes might have been already deleted
                // ie if 0 is deleted, then the next element is also index 0
                // also ".len" is safe to use here as it is always 1 higher than the max index of the retained elements
                let deleted_idx = self.tracks.len();

                // NOTE: this function may send many events very quickly (for example on a folder delete), which could overwhelm the broadcast channel on a low capacity value
                self.send_stream_ev_pl(UpdatePlaylistEvents::PlaylistRemoveTrack(
                    PlaylistRemoveTrackInfo {
                        at_index: u64::try_from(deleted_idx).unwrap(),
                        trackid: track_source,
                    },
                ));
                self.is_modified = true;
            }

            match self.find_index_from_file(&current_track_file) {
                Some(new_index) => self.current_track_index = new_index,
                None => self.current_track_index = 0,
            }
        }
    }

    /// Stop the current playlist by setting [`RunningStatus::Stopped`], preventing going to the next track
    /// and finally, stop the currently playing track.
    pub fn stop(&mut self) {
        self.set_status(RunningStatus::Stopped);
        self.set_next_track(None);
        self.clear_current_track();
    }

    #[must_use]
    pub fn current_track(&self) -> Option<&Track> {
        if self.current_track.is_some() {
            return self.current_track.as_ref();
        }
        self.tracks.get(self.current_track_index)
    }

    pub fn current_track_as_mut(&mut self) -> Option<&mut Track> {
        self.tracks.get_mut(self.current_track_index)
    }

    pub fn clear_current_track(&mut self) {
        self.current_track = None;
    }

    #[must_use]
    pub fn get_current_track_index(&self) -> usize {
        self.current_track_index
    }

    pub fn set_current_track_index(&mut self, index: usize) {
        self.current_track_index = index;
    }

    #[must_use]
    pub fn next_track(&self) -> Option<&Track> {
        let index = self.next_track_index?;
        self.tracks.get(index)
    }

    pub fn set_next_track(&mut self, track_idx: Option<usize>) {
        self.next_track_index = track_idx;
    }

    #[must_use]
    pub fn has_next_track(&self) -> bool {
        self.next_track_index.is_some()
    }

    /// Send Playlist stream events with consistent error handling
    fn send_stream_ev_pl(&self, ev: UpdatePlaylistEvents) {
        // there is only one error case: no receivers
        if self
            .stream_tx
            .send(UpdateEvents::PlaylistChanged(ev))
            .is_err()
        {
            debug!("Stream Event not send: No Receivers");
        }
    }

    /// Send stream events with consistent error handling
    fn send_stream_ev(&self, ev: UpdateEvents) {
        // there is only one error case: no receivers
        if self.stream_tx.send(ev).is_err() {
            debug!("Stream Event not send: No Receivers");
        }
    }
}

const PLAYLIST_SAVE_FILENAME: &str = "playlist.log";

fn get_playlist_path() -> Result<PathBuf> {
    let mut path = get_app_config_path()?;
    path.push(PLAYLIST_SAVE_FILENAME);

    Ok(path)
}

/// Error collections for [`Playlist::add_tracks`].
#[derive(Debug)]
pub struct PlaylistAddErrorCollection {
    pub errors: Vec<anyhow::Error>,
}

impl Display for PlaylistAddErrorCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "There are {} Errors adding tracks to the playlist: [",
            self.errors.len()
        )?;

        for err in &self.errors {
            writeln!(f, "  {err},")?;
        }

        write!(f, "]")
    }
}

impl Error for PlaylistAddErrorCollection {}

impl From<Vec<anyhow::Error>> for PlaylistAddErrorCollection {
    fn from(value: Vec<anyhow::Error>) -> Self {
        Self {
            errors: value.into_iter().map(|err| anyhow::anyhow!(err)).collect(),
        }
    }
}

// NOTE: this is not "thiserror" due to custom "Display" impl (the "Option" handling)
/// Error for when [`Playlist::add_track`] fails
#[derive(Debug)]
pub enum PlaylistAddError {
    /// `(FileType, Path)`
    UnsupportedFileType(Option<String>, PathBuf),
    /// `(Path)`
    PathDoesNotExist(PathBuf),
    /// Generic Error for when reading the track fails
    /// `(OriginalError, Path)`
    ReadError(anyhow::Error, PathBuf),
}

impl Display for PlaylistAddError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to add to playlist because of: {}",
            match self {
                Self::UnsupportedFileType(ext, path) => {
                    let ext = if let Some(ext) = ext {
                        format!("Some({ext})")
                    } else {
                        "None".into()
                    };
                    format!("Unsupported File type \"{ext}\" at \"{}\"", path.display())
                }
                Self::PathDoesNotExist(path) => {
                    format!("Path does not exist: \"{}\"", path.display())
                }
                Self::ReadError(err, path) => {
                    format!("{err} at \"{}\"", path.display())
                }
            }
        )
    }
}

impl Error for PlaylistAddError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::ReadError(orig, _) = self {
            return Some(orig.as_ref());
        }

        None
    }
}

/// Error for when [`Playlist::add_playlist`] fails
#[derive(Debug, Default)]
pub struct PlaylistAddErrorVec(Vec<PlaylistAddError>);

impl PlaylistAddErrorVec {
    pub fn push(&mut self, err: PlaylistAddError) {
        self.0.push(err);
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for PlaylistAddErrorVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} Error(s) happened:", self.0.len())?;
        for err in &self.0 {
            writeln!(f, "  - {err}")?;
        }

        Ok(())
    }
}

impl Error for PlaylistAddErrorVec {}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use termusiclib::{
        player::playlist_helpers::PlaylistTrackSource,
        track::{MediaTypes, PodcastTrackData, RadioTrackData, TrackData},
    };

    use super::Playlist;

    #[test]
    fn should_pass_check_info() {
        let path = "/somewhere/file.mp3".to_string();
        let path2 = PathBuf::from(&path);
        Playlist::check_same_source(
            &PlaylistTrackSource::Path(path),
            &MediaTypes::Track(TrackData::new(path2)),
            0,
        )
        .unwrap();

        let uri = "http://some.radio.com/".to_string();
        let uri2 = uri.clone();
        Playlist::check_same_source(
            &PlaylistTrackSource::Url(uri),
            &MediaTypes::Radio(RadioTrackData::new(uri2)),
            0,
        )
        .unwrap();

        let uri = "http://some.podcast.com/".to_string();
        let uri2 = uri.clone();
        Playlist::check_same_source(
            &PlaylistTrackSource::PodcastUrl(uri),
            &MediaTypes::Podcast(PodcastTrackData::new(uri2)),
            0,
        )
        .unwrap();
    }

    #[test]
    fn should_err_on_type_mismatch() {
        let path = "/somewhere/file.mp3".to_string();
        let path2 = path.clone();
        Playlist::check_same_source(
            &PlaylistTrackSource::Path(path),
            &MediaTypes::Radio(RadioTrackData::new(path2)),
            0,
        )
        .unwrap_err();
    }
}
