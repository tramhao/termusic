use std::error::Error;
use std::fmt::{Display, Write as _};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use parking_lot::RwLock;
use pathdiff::diff_paths;
use rand::seq::SliceRandom;
use rand::Rng;
use termusiclib::config::v2::server::LoopMode;
use termusiclib::config::SharedServerSettings;
use termusiclib::player::playlist_helpers::PlaylistSwapTrack;
use termusiclib::player::playlist_helpers::PlaylistTrackSource;
use termusiclib::player::playlist_helpers::{PlaylistAddTrack, PlaylistRemoveTrackIndexed};
use termusiclib::player::PlaylistLoopModeInfo;
use termusiclib::player::PlaylistSwapInfo;
use termusiclib::player::UpdateEvents;
use termusiclib::player::UpdatePlaylistEvents;
use termusiclib::player::{PlaylistAddTrackInfo, PlaylistRemoveTrackInfo};
use termusiclib::podcast::{db::Database as DBPod, episode::Episode};
use termusiclib::track::MediaType;
use termusiclib::{
    track::Track,
    utils::{filetype_supported, get_app_config_path, get_parent_folder},
};

use crate::SharedPlaylist;
use crate::StreamTX;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum Status {
    #[default]
    Stopped,
    Running,
    Paused,
}

impl Status {
    #[must_use]
    pub fn as_u32(&self) -> u32 {
        match self {
            Status::Stopped => 0,
            Status::Running => 1,
            Status::Paused => 2,
        }
    }

    #[must_use]
    pub fn from_u32(status: u32) -> Self {
        match status {
            1 => Status::Running,
            2 => Status::Paused,
            _ => Status::Stopped,
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "Running"),
            Self::Stopped => write!(f, "Stopped"),
            Self::Paused => write!(f, "Paused"),
        }
    }
}

#[derive(Debug)]
pub struct Playlist {
    /// All tracks in the playlist
    tracks: Vec<Track>,
    /// Index into `tracks` of which the current playing track is
    current_track_index: usize,
    /// Index into `tracks` for the next track to play after the current
    next_track_index: Option<usize>,
    /// The currently playing [`Track`]. Does not need to be in `tracks`
    current_track: Option<Track>,
    /// The current playing running status of the playlist
    status: Status,
    /// The loop-/play-mode for the playlist
    loop_mode: LoopMode,
    /// Indexes into `tracks` that have been previously been played (for `previous`)
    played_index: Vec<usize>,
    /// Indicator if the playlist should advance the `current_*` and `next_*` values
    need_proceed_to_next: bool,
    stream_tx: StreamTX,
}

impl Playlist {
    /// # Errors
    /// errors could happen when reading files
    pub fn new(config: &SharedServerSettings, stream_tx: StreamTX) -> Result<Self> {
        let (current_track_index, tracks) = Self::load()?;
        // TODO: shouldnt "loop_mode" be combined with the config ones?
        let loop_mode = config.read().settings.player.loop_mode;
        let current_track = None;

        Ok(Self {
            tracks,
            status: Status::Stopped,
            loop_mode,
            current_track_index,
            current_track,
            played_index: Vec::new(),
            next_track_index: None,
            need_proceed_to_next: false,
            stream_tx,
        })
    }

    /// Create a new Playlist instance that is directly shared
    ///
    /// # Errors
    ///
    /// see [`new`](Self::new)
    pub fn new_shared(
        config: &SharedServerSettings,
        stream_tx: StreamTX,
    ) -> Result<SharedPlaylist> {
        Ok(Arc::new(RwLock::new(Self::new(config, stream_tx)?)))
    }

    /// Advance the playlist to the next track.
    pub fn proceed(&mut self) {
        debug!("need to proceed to next: {}", self.need_proceed_to_next);
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
            if line.starts_with("http") {
                let mut is_podcast = false;
                'outer: for pod in &podcasts {
                    for ep in &pod.episodes {
                        if ep.url == line.as_str() {
                            is_podcast = true;
                            let track = Track::from_episode(ep);
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
            if let Ok(track) = Track::read_from_path(&line, false) {
                playlist_items.push(track);
            }
        }

        Ok((current_track_index, playlist_items))
    }

    /// Reload the current playlist from the file. This function does not save beforehand.
    ///
    /// # Errors
    /// See [`Self::load`]
    pub fn reload_tracks(&mut self) -> Result<()> {
        let (current_track_index, tracks) = Self::load()?;
        self.tracks = tracks;
        self.current_track_index = current_track_index;
        Ok(())
    }

    /// Save the current playlist and playing index to the playlist log
    ///
    /// Path in `$config$/playlist.log`
    ///
    /// # Errors
    /// Errors could happen when writing files
    pub fn save(&mut self) -> Result<()> {
        let path = get_playlist_path()?;

        let file = File::create(&path)?;

        // If the playlist is empty, truncate the file, but dont write anything else (like a index number)
        if self.is_empty() {
            return Ok(());
        }

        let mut writer = BufWriter::new(file);
        writer.write_all(self.current_track_index.to_string().as_bytes())?;
        writer.write_all(b"\n")?;
        for track in &self.tracks {
            if let Some(f) = track.file() {
                writer.write_all(f.as_bytes())?;
                writer.write_all(b"\n")?;
            }
        }

        writer.flush()?;

        Ok(())
    }

    /// Change to the next track.
    pub fn next(&mut self) {
        self.played_index.push(self.current_track_index);
        // Note: the next index is *not* taken here, as ".proceed/next" is called first,
        // then "has_next_track" is later used to check if enqueing has used.
        if let Some(index) = self.next_track_index {
            self.current_track_index = index;
            return;
        }
        self.current_track_index = self.get_next_track_index();
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
        if !self.played_index.is_empty() {
            if let Some(index) = self.played_index.pop() {
                self.current_track_index = index;
                return;
            }
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

        self.send_stream_ev(UpdatePlaylistEvents::PlaylistSwapTracks(PlaylistSwapInfo {
            index_a,
            index_b,
        }));

        Ok(())
    }

    /// Get the current track's Path/Url.
    pub fn get_current_track(&mut self) -> Option<String> {
        let mut result = None;
        if let Some(track) = self.current_track() {
            match track.media_type {
                MediaType::Music | MediaType::LiveRadio => {
                    if let Some(file) = track.file() {
                        result = Some(file.to_string());
                    }
                }
                MediaType::Podcast => {
                    if let Some(local_file) = &track.podcast_localfile {
                        let path = Path::new(&local_file);
                        if path.exists() {
                            return Some(local_file.clone());
                        }
                    }
                    if let Some(file) = track.file() {
                        result = Some(file.to_string());
                    }
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

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.status == Status::Stopped
    }

    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.status == Status::Paused
    }

    #[must_use]
    pub fn status(&self) -> Status {
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

        self.send_stream_ev(UpdatePlaylistEvents::PlaylistLoopMode(
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
            if let Some(file) = track.file() {
                let path_relative = diff_paths(file, parent_folder);

                if let Some(path_relative) = path_relative {
                    let _ = writeln!(m3u, "{}", path_relative.display());
                }
            }
        }
        m3u
    }

    /// Add a podcast episode to the playlist.
    ///
    /// # Panics
    ///
    /// This should never happen as a podcast url has already been a string, so conversion should not fail
    pub fn add_episode(&mut self, ep: &Episode) {
        let track = Track::from_episode(ep);

        self.send_stream_ev(UpdatePlaylistEvents::PlaylistAddTrack(
            PlaylistAddTrackInfo {
                at_index: u64::try_from(self.tracks.len()).unwrap(),
                title: track.title().map(ToOwned::to_owned),
                duration: track.duration(),
                // Note: Safe unwrap, as a podcast uri is always a uri, not a path (which has been a string before)
                trackid: PlaylistTrackSource::PodcastUrl(track.file().unwrap().to_owned()),
            },
        ));

        self.tracks.push(track);
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
            return Ok(());
        }

        let track = Self::track_from_path(track_str)?;

        self.send_stream_ev(UpdatePlaylistEvents::PlaylistAddTrack(
            PlaylistAddTrackInfo {
                at_index: u64::try_from(self.tracks.len()).unwrap(),
                title: track.title().map(ToOwned::to_owned),
                duration: track.duration(),
                trackid: PlaylistTrackSource::Path(track_str.to_string()),
            },
        ));

        self.tracks.push(track);

        Ok(())
    }

    /// Add Paths / Urls from the music service
    ///
    /// # Errors
    ///
    /// see [`Self::add_track`]
    ///
    /// # Panics
    ///
    /// If `usize` cannot be converted to `u64`
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

                self.send_stream_ev(UpdatePlaylistEvents::PlaylistAddTrack(
                    PlaylistAddTrackInfo {
                        at_index: u64::try_from(self.tracks.len()).unwrap(),
                        title: track.title().map(ToOwned::to_owned),
                        duration: track.duration(),
                        trackid: track_location,
                    },
                ));

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

            self.send_stream_ev(UpdatePlaylistEvents::PlaylistAddTrack(
                PlaylistAddTrackInfo {
                    at_index: u64::try_from(at_index).unwrap(),
                    title: track.title().map(ToOwned::to_owned),
                    duration: track.duration(),
                    trackid: track_location,
                },
            ));

            self.tracks.insert(at_index, track);
            at_index += 1;
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
            bail!("at_index is higher than the length of the playlist! at_index is \"{at_index}\" and playlist length is \"{}\"", self.tracks.len());
        }

        if at_index + tracks.tracks.len().saturating_sub(1) >= self.tracks.len() {
            bail!("at_index + tracks to remove is higher than the length of the playlist! playlist lenght is \"{}\"", self.tracks.len());
        }

        for input_track in tracks.tracks {
            // verify that it is the track to be removed via id matching
            let Some(track_at_idx) = self.tracks.get(at_index) else {
                // this should not happen as it is verified before the loop, but just in case
                bail!("Failed to get track at index \"{at_index}\"");
            };

            // this unwrap could be handled better, but this should never actually happen
            let id = track_at_idx.file().unwrap();

            // Note: clippy suggested this instead of a match block
            let ((PlaylistTrackSource::Path(file_url), MediaType::Music)
            | (PlaylistTrackSource::PodcastUrl(file_url), MediaType::Podcast)
            | (PlaylistTrackSource::Url(file_url), MediaType::LiveRadio)) =
                (&input_track, track_at_idx.media_type)
            else {
                bail!(
                    "Type mismatch, expected \"{:#?}\" at \"{at_index}\" found \"{:#?}\"",
                    input_track,
                    track_at_idx
                );
            };

            if file_url != id {
                bail!("URI mismatch, expected \"{id}\" at \"{at_index}\", found \"{file_url}\"");
            }

            // verified that at index "at_index" the track is of the type and has the URI that was requested to be removed
            self.remove(at_index);

            self.send_stream_ev(UpdatePlaylistEvents::PlaylistRemoveTrack(
                PlaylistRemoveTrackInfo {
                    at_index: u64::try_from(at_index).unwrap(),
                    trackid: input_track,
                },
            ));
        }

        Ok(())
    }

    /// Create a Track from a given Path
    fn track_from_path(path_str: &str) -> Result<Track, PlaylistAddError> {
        let path = Path::new(path_str);

        if !filetype_supported(path_str) {
            error!("unsupported filetype: {path:#?}");
            let p = path.to_path_buf();
            let ext = path.extension().map(|v| v.to_string_lossy().to_string());
            return Err(PlaylistAddError::UnsupportedFileType(ext, p));
        }

        if !path.exists() {
            return Err(PlaylistAddError::PathDoesNotExist(path.to_path_buf()));
        }

        let track = Track::read_from_path(path, false)
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
        let track = Track::from_episode(&ep);

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
    pub fn remove(&mut self, index: usize) {
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
    /// This does not stop the playlist or clear [`current_track`].
    pub fn clear(&mut self) {
        self.tracks.clear();
        self.played_index.clear();
        self.next_track_index.take();
        self.current_track_index = 0;
        self.need_proceed_to_next = false;

        self.send_stream_ev(UpdatePlaylistEvents::PlaylistCleared);
    }

    /// Shuffle the playlist
    pub fn shuffle(&mut self) {
        // TODO: why does this only shuffle if there is a current track?
        if let Some(current_track_file) = self.get_current_track() {
            self.tracks.shuffle(&mut rand::rng());
            if let Some(index) = self.find_index_from_file(&current_track_file) {
                self.current_track_index = index;
            }
        }
    }

    /// Find the index in the playlist for `item`, if it exists there.
    fn find_index_from_file(&self, item: &str) -> Option<usize> {
        for (index, track) in self.tracks.iter().enumerate() {
            let Some(file) = track.file() else {
                continue;
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
    pub fn remove_deleted_items(&mut self) {
        if let Some(current_track_file) = self.get_current_track() {
            // TODO: dosnt this remove radio and podcast episodes?
            self.tracks
                .retain(|x| x.file().is_some_and(|p| Path::new(p).exists()));
            match self.find_index_from_file(&current_track_file) {
                Some(new_index) => self.current_track_index = new_index,
                None => self.current_track_index = 0,
            }
        }
    }

    /// Stop the current playlist by setting [`Status::Stopped`], preventing going to the next track
    /// and finally, stop the currently playing track.
    pub fn stop(&mut self) {
        self.set_status(Status::Stopped);
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

    /// Send stream events with consistent error handling
    fn send_stream_ev(&self, ev: UpdatePlaylistEvents) {
        // there is only one error case: no receivers
        if self
            .stream_tx
            .send(UpdateEvents::PlaylistChanged(ev))
            .is_err()
        {
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
