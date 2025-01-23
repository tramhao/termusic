use anyhow::{bail, Context, Result};
use pathdiff::diff_paths;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use termusiclib::config::v2::server::LoopMode;
use termusiclib::config::SharedServerSettings;
use termusiclib::podcast::{db::Database as DBPod, episode::Episode};
use termusiclib::track::MediaType;
use termusiclib::{
    track::Track,
    utils::{filetype_supported, get_app_config_path, get_parent_folder},
};

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Debug)]
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

#[derive(Default, Debug)]
pub struct Playlist {
    /// All tracks in the playlist
    tracks: Vec<Track>,
    /// Index into `tracks` of which the current playing track is
    current_track_index: usize,
    /// Index into `tracks` for the next track to play after the current
    next_track_index: usize,
    /// The currently playing [`Track`]. Does not need to be in `tracks`
    current_track: Option<Track>,
    /// The next track to play after the current. Does not need to be in `tracks`
    next_track: Option<Track>,
    /// The current playing running status of the playlist
    status: Status,
    /// The loop-/play-mode for the playlist
    loop_mode: LoopMode,
    /// Indexes into `tracks` that have been previously been played (for `previous`)
    played_index: Vec<usize>,
    /// Indicator if the playlist should advance the `current_*` and `next_*` values
    need_proceed_to_next: bool,
    config: SharedServerSettings,
}

impl Playlist {
    /// # Errors
    /// errors could happen when reading files
    pub fn new(config: SharedServerSettings) -> Result<Self> {
        let (current_track_index, tracks) = Self::load()?;
        // TODO: shouldnt "loop_mode" be combined with the config ones?
        let loop_mode = config.read().settings.player.loop_mode;
        let current_track = None;

        Ok(Self {
            tracks,
            next_track: None,
            // index: Some(0),
            status: Status::Stopped,
            loop_mode,
            current_track_index,
            current_track,
            played_index: Vec::new(),
            config,
            next_track_index: 0,
            need_proceed_to_next: false,
        })
    }

    pub fn proceed(&mut self) {
        debug!("need to proceed to next: {}", self.need_proceed_to_next);
        if self.need_proceed_to_next {
            self.next();
        } else {
            self.need_proceed_to_next = true;
        }
    }

    pub fn proceed_false(&mut self) {
        self.need_proceed_to_next = false;
    }

    /// Load the playlist from the file
    ///
    /// Path in `$config$/playlist.log`
    ///
    /// # Errors
    /// errors could happen when reading file
    /// # Panics
    /// panics when error loading podcasts from db
    pub fn load() -> Result<(usize, Vec<Track>)> {
        let path = get_playlist_path()?;

        let file = if let Ok(f) = File::open(path.as_path()) {
            f
        } else {
            File::create(path.as_path())?;
            File::open(path)?
        };
        let reader = BufReader::new(file);
        let mut lines = reader
            .lines()
            .map(|line| line.unwrap_or_else(|_| "Error".to_string()));

        let mut current_track_index = 0;
        if let Some(index_line) = lines.next() {
            if let Ok(index) = index_line.trim().parse() {
                current_track_index = index;
            }
        }

        let mut playlist_items = Vec::new();
        let db_path = get_app_config_path()?;
        let db_podcast = DBPod::new(&db_path)?;
        let podcasts = db_podcast
            .get_podcasts()
            .with_context(|| "failed to get podcasts from db.")?;
        for line in lines {
            if let Ok(track) = Track::read_from_path(&line, false) {
                playlist_items.push(track);
                continue;
            };
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
            }
        }

        Ok((current_track_index, playlist_items))
    }

    /// # Errors
    /// Errors could happen when reading files
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

        let file = File::create(path.as_path())?;
        let mut writer = BufWriter::new(file);
        let mut bytes = Vec::new();
        bytes.extend(format!("{}", self.current_track_index).as_bytes());
        bytes.extend("\n".as_bytes());
        for i in &self.tracks {
            if let Some(f) = i.file() {
                bytes.extend(f.as_bytes());
                bytes.extend("\n".as_bytes());
            }
        }

        writer.write_all(&bytes)?;
        writer.flush()?;

        Ok(())
    }

    pub fn next(&mut self) {
        self.played_index.push(self.current_track_index);
        if self.config.read().settings.player.gapless && self.has_next_track() {
            self.current_track_index = self.next_track_index;
            return;
        }
        self.current_track_index = self.get_next_track_index();
    }

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

    pub fn swap_down(&mut self, index: usize) {
        if index < self.len().saturating_sub(1) {
            let track = self.tracks.remove(index);
            self.tracks.insert(index + 1, track);
            // handle index
            if index == self.current_track_index {
                self.current_track_index += 1;
            } else if index == self.current_track_index - 1 {
                self.current_track_index -= 1;
            }
        }
    }

    pub fn swap_up(&mut self, index: usize) {
        if index > 0 {
            let track = self.tracks.remove(index);
            self.tracks.insert(index - 1, track);
            // handle index
            if index == self.current_track_index {
                self.current_track_index -= 1;
            } else if index == self.current_track_index + 1 {
                self.current_track_index += 1;
            }
        }
    }

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

    pub fn fetch_next_track(&mut self) -> Option<&Track> {
        self.next_track_index = self.get_next_track_index();
        self.tracks.get(self.next_track_index)
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

    /// Cycle through the loop modes and return the new mode
    ///
    /// order:
    /// [Random](LoopMode::Random) -> [Playlist](LoopMode::Playlist)
    /// [Playlist](LoopMode::Playlist) -> [Single](LoopMode::Single)
    /// [Single](LoopMode::Single) -> [Random](LoopMode::Random)
    pub fn cycle_loop_mode(&mut self) -> LoopMode {
        match self.loop_mode {
            LoopMode::Random => {
                self.loop_mode = LoopMode::Playlist;
            }
            LoopMode::Playlist => {
                self.loop_mode = LoopMode::Single;
            }
            LoopMode::Single => {
                self.loop_mode = LoopMode::Random;
            }
        };
        self.loop_mode
    }

    /// Export the current playlist to a `.m3u` playlist file
    ///
    /// might be confused with [save](Self::save)
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

    /// Generate the m3u's file content
    ///
    /// All Paths are relative to the `parent_folder` directory
    fn get_m3u_file(&self, parent_folder: &Path) -> String {
        let mut m3u = String::from("#EXTM3U\n");
        for track in &self.tracks {
            if let Some(file) = track.file() {
                let path_relative = diff_paths(file, parent_folder);

                if let Some(path_relative) = path_relative {
                    let path = format!("{}\n", path_relative.display());
                    m3u.push_str(&path);
                }
            }
        }
        m3u
    }

    pub fn add_episode(&mut self, ep: &Episode) {
        let track = Track::from_episode(ep);
        self.tracks.push(track);
    }

    /// # Errors
    /// Error happens when track cannot be read from local file
    pub fn add_playlist<T: AsRef<str>>(&mut self, vec: &[T]) -> Result<()> {
        for item in vec.iter().map(AsRef::as_ref) {
            if item.starts_with("http") {
                let track = Track::new_radio(item);
                self.tracks.push(track);
            } else if !filetype_supported(item) {
                // TODO: add error on fail
                error!("unsupported filetype: {:#?}", item);
                continue;
            } else if PathBuf::from(item).exists() {
                let track = Track::read_from_path(item, false)?;
                self.tracks.push(track);
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn tracks(&self) -> &Vec<Track> {
        &self.tracks
    }

    pub fn remove(&mut self, index: usize) {
        self.tracks.remove(index);
        // Handle index
        if index <= self.current_track_index {
            if self.current_track_index == 0 {
                self.current_track_index = 0;
            } else {
                self.current_track_index -= 1;
            }
        }
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_track_index = 0;
    }

    pub fn shuffle(&mut self) {
        if let Some(current_track_file) = self.get_current_track() {
            self.tracks.shuffle(&mut thread_rng());
            if let Some(index) = self.find_index_from_file(&current_track_file) {
                self.current_track_index = index;
            }
        }
    }

    fn find_index_from_file(&self, item: &str) -> Option<usize> {
        for (index, track) in self.tracks.iter().enumerate() {
            if let Some(file) = track.file() {
                if file == item {
                    return Some(index);
                }
            }
        }
        None
    }

    fn get_random_index(&self) -> usize {
        let mut rng = rand::thread_rng();
        let mut random_index = self.current_track_index;
        while self.current_track_index == random_index {
            random_index = rng.gen_range(0..self.len());
        }

        random_index
    }

    pub fn remove_deleted_items(&mut self) {
        if let Some(current_track_file) = self.get_current_track() {
            self.tracks
                .retain(|x| x.file().is_some_and(|p| Path::new(p).exists()));
            match self.find_index_from_file(&current_track_file) {
                Some(new_index) => self.current_track_index = new_index,
                None => self.current_track_index = 0,
            }
        }
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
        self.next_track.as_ref()
    }

    pub fn set_next_track(&mut self, track: Option<&Track>) {
        match track {
            Some(t) => self.next_track = Some(t.clone()),
            None => self.next_track = None,
        }
    }

    #[must_use]
    pub fn has_next_track(&self) -> bool {
        self.next_track.is_some()
    }
}

const PLAYLIST_SAVE_FILENAME: &str = "playlist.log";

fn get_playlist_path() -> Result<PathBuf> {
    let mut path = get_app_config_path()?;
    path.push(PLAYLIST_SAVE_FILENAME);

    Ok(path)
}
