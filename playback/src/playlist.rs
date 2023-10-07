use anyhow::{bail, Result};
use pathdiff::diff_utf8_paths;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
#[cfg(not(any(feature = "mpv", feature = "gst")))]
use std::time::Duration;
use termusiclib::podcast::{db::Database as DBPod, Episode};
use termusiclib::track::MediaType;
use termusiclib::{
    config::{Loop, Settings},
    track::Track,
    utils::{filetype_supported, get_app_config_path, get_parent_folder},
};

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
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

/// Iterator wrapper which generates the index used for the next track
struct NextTrackGenerator(Box<dyn Iterator<Item = usize> + Send>);
impl std::ops::Deref for NextTrackGenerator {
    type Target = Box<dyn Iterator<Item = usize> + Send>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for NextTrackGenerator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Default for NextTrackGenerator {
    fn default() -> Self {
        Self(Box::new(std::iter::empty()))
    }
}

#[derive(Default)]
pub struct Playlist {
    tracks: Vec<Track>,
    current_track_index: Option<usize>,
    pub next_track_index: usize,
    played_index: Vec<usize>,
    current_track: Option<Track>,
    next_track: Option<Track>,
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    next_track_duration: Duration,
    status: Status,
    loop_mode: Loop,
    next_track_index_generator: NextTrackGenerator,
    config: Settings,
    need_proceed_to_next: bool,
}

impl Playlist {
    /// # Errors
    /// errors could happen when reading files
    pub fn new(config: &Settings) -> Result<Self> {
        let (current_track_index, tracks) = Self::load()?;
        let loop_mode = config.player_loop_mode;
        let current_track = None;

        let next_track_generator =
            Self::new_next_track_generator(current_track_index, &tracks, &loop_mode);

        Ok(Self {
            tracks,
            next_track: None,
            #[cfg(not(any(feature = "mpv", feature = "gst")))]
            next_track_duration: Duration::from_secs(0),
            // index: Some(0),
            status: Status::Stopped,
            loop_mode,
            current_track_index,
            current_track,
            played_index: Vec::new(),
            config: config.clone(),
            next_track_index: 0,
            need_proceed_to_next: false,
            next_track_index_generator: next_track_generator,
        })
    }

    pub fn proceed(&mut self) {
        debug!("need to proceed to next: {}", self.need_proceed_to_next);
        if self.need_proceed_to_next {
            if let Some(current_track_index) = self.current_track_index {
                self.played_index.push(current_track_index);
            }
            if self.config.player_gapless && self.has_next_track() {
                self.current_track_index = Some(self.next_track_index);
                return;
            }
            self.current_track_index = self.next_track_index_generator.next();
        } else {
            self.need_proceed_to_next = true;
        }
    }

    pub fn proceed_false(&mut self) {
        self.need_proceed_to_next = false;
    }

    /// # Errors
    /// errors could happen when reading file
    pub fn load() -> Result<(Option<usize>, Vec<Track>)> {
        let mut path = get_app_config_path()?;
        path.push("playlist.log");

        let file = if let Ok(f) = File::open(path.as_path()) {
            f
        } else {
            File::create(path.as_path())?;
            File::open(path)?
        };
        let reader = BufReader::new(file);
        let mut lines: std::collections::VecDeque<_> = reader
            .lines()
            .map(|line| line.unwrap_or_else(|_| "Error".to_string()))
            .collect();

        let mut current_track_index = None;
        if let Some(index_line) = lines.pop_front() {
            if let Ok(index) = index_line.trim().parse() {
                current_track_index = Some(index);
            }
        }

        let mut playlist_items = Vec::new();
        let db_path = get_app_config_path()?;
        let db_podcast = DBPod::connect(&db_path)?;
        let podcasts = db_podcast
            .get_podcasts()
            .expect("failed to get podcasts from db.");
        for line in &lines {
            if let Ok(s) = Track::read_from_path(line, false) {
                playlist_items.push(s);
                continue;
            };
            if line.starts_with("http") {
                let mut is_podcast = false;
                'outer: for pod in &podcasts {
                    for ep in &pod.episodes {
                        if &ep.url == line {
                            is_podcast = true;
                            let track = Track::from_episode(ep);
                            playlist_items.push(track);
                            break 'outer;
                        }
                    }
                }
                if !is_podcast {
                    let track = Track::new_radio(line);
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

    /// # Errors
    /// Errors could happen when writing files
    pub fn save(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("playlist.log");

        let file = File::create(path.as_path())?;
        let mut writer = BufWriter::new(file);
        let mut bytes = Vec::new();
        bytes.extend(
            self.current_track_index
                .map_or(String::from("None"), |i| format!("{i}"))
                .as_bytes(),
        );
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

    fn new_next_track_generator(
        current_track_index: Option<usize>,
        tracks: &[Track],
        loop_mode: &Loop,
    ) -> NextTrackGenerator {
        NextTrackGenerator(match loop_mode {
            Loop::Single => match current_track_index {
                Some(current_track_index) => Box::new(std::iter::repeat(current_track_index)),
                None => Box::new(std::iter::empty()),
            },
            Loop::Playlist => Box::new((0..tracks.len()).skip(current_track_index.unwrap_or(0))),
            Loop::Random => {
                let mut rng = rand::thread_rng();
                let mut indexes: Vec<_> = (0..tracks.len()).collect();
                indexes.shuffle(&mut rng);
                Box::new(indexes.into_iter())
            }
        })
    }

    fn refresh_next_track_generator(&mut self) {
        self.next_track_index_generator =
            Self::new_next_track_generator(self.current_track_index, &self.tracks, &self.loop_mode);
    }

    pub fn previous(&mut self) {
        if !self.played_index.is_empty() {
            if let Some(index) = self.played_index.pop() {
                self.current_track_index = Some(index);
                return;
            }
        }
        match self.loop_mode {
            Loop::Single => {}
            Loop::Playlist => {
                if self.current_track_index == Some(0) {
                    self.current_track_index = Some(self.len() - 1);
                } else {
                    self.current_track_index = self.current_track_index.and_then(|i| {
                        if i == 0 {
                            self.tracks.len().checked_sub(1)
                        } else {
                            Some(i - 1)
                        }
                    });
                }
            }
            Loop::Random => {
                self.current_track_index = self.next_track_index_generator.next().or_else(|| {
                    self.refresh_next_track_generator();
                    self.next_track_index_generator.next()
                });
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
        if index < self.len() - 1 {
            let track = self.tracks.remove(index);
            self.tracks.insert(index + 1, track);
            // handle index
            if let Some(current_index) = &mut self.current_track_index {
                if index == *current_index {
                    *current_index += 1;
                } else if index == *current_index - 1 {
                    *current_index -= 1;
                }
            }
        }
    }

    pub fn swap_up(&mut self, index: usize) {
        if index > 0 {
            let track = self.tracks.remove(index);
            self.tracks.insert(index - 1, track);
            // handle index
            if let Some(current_index) = &mut self.current_track_index {
                if *current_index == index {
                    *current_index -= 1;
                } else if *current_index + 1 == index {
                    *current_index += 1;
                }
            }
        }
    }

    pub fn get_current_track(&mut self) -> Option<String> {
        let mut result = None;
        if let Some(track) = self.current_track() {
            match track.media_type {
                Some(MediaType::Music | MediaType::LiveRadio) => {
                    if let Some(file) = track.file() {
                        result = Some(file.to_string());
                    }
                }
                Some(MediaType::Podcast) => {
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
                None => {}
            }
        }
        result
    }

    pub fn fetch_next_track(&mut self) -> Option<&Track> {
        self.tracks.get(self.next_track_index_generator.next()?)
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

    pub fn cycle_loop_mode(&mut self) -> Loop {
        match self.loop_mode {
            Loop::Random => {
                self.loop_mode = Loop::Playlist;
            }
            Loop::Playlist => {
                self.loop_mode = Loop::Single;
            }
            Loop::Single => {
                self.loop_mode = Loop::Random;
            }
        };
        self.loop_mode
    }

    // export to M3U
    /// # Errors
    /// Error could happen when writing file to local disk.
    pub fn save_m3u(&self, filename: &str) -> Result<()> {
        if self.tracks.is_empty() {
            bail!("No tracks in playlist, so no need to save.");
        }

        let parent_folder = get_parent_folder(filename);

        let m3u = self.get_m3u_file(&parent_folder);

        std::fs::write(filename, m3u)?;
        Ok(())
    }

    fn get_m3u_file(&self, parent_folder: &str) -> String {
        let mut m3u = String::from("#EXTM3U\n");
        for track in &self.tracks {
            if let Some(file) = track.file() {
                let path_relative = diff_utf8_paths(file, parent_folder);

                if let Some(p) = path_relative {
                    let path = format!("{p}\n");
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
    pub fn add_playlist(&mut self, vec: Vec<&str>) -> Result<()> {
        for item in vec {
            if item.starts_with("http") {
                let track = Track::new_radio(item);
                self.tracks.push(track);
            } else if !filetype_supported(item) {
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
        if let Some(current_track_index) = &mut self.current_track_index {
            if index <= *current_track_index {
                if *current_track_index == 0 {
                    *current_track_index = 0;
                } else {
                    *current_track_index -= 1;
                }
            }
        }
        self.refresh_next_track_generator();
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_track_index = None;
    }

    pub fn shuffle(&mut self) {
        if let Some(current_track_file) = self.get_current_track() {
            self.tracks.shuffle(&mut thread_rng());
            if let Some(index) = self.find_index_from_file(&current_track_file) {
                self.current_track_index = Some(index);
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

    pub fn remove_deleted_items(&mut self) {
        if let Some(current_track_file) = self.get_current_track() {
            self.tracks
                .retain(|x| x.file().map_or(false, |p| Path::new(p).exists()));
            if let Some(new_index) = self.find_index_from_file(&current_track_file) {
                self.current_track_index = Some(new_index);
            }
        }
    }

    #[must_use]
    pub fn current_track(&self) -> Option<&Track> {
        if self.current_track.is_some() {
            return self.current_track.as_ref();
        }
        self.tracks.get(self.current_track_index?)
    }

    pub fn current_track_as_mut(&mut self) -> Option<&mut Track> {
        self.tracks.get_mut(self.current_track_index?)
    }

    pub fn clear_current_track(&mut self) {
        self.current_track = None;
    }

    #[must_use]
    pub fn get_current_track_index(&self) -> Option<usize> {
        self.current_track_index
    }

    pub fn set_current_track_index(&mut self, index: Option<usize>) {
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

    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    #[must_use]
    pub fn next_track_duration(&self) -> Duration {
        self.next_track_duration
    }

    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    pub fn set_next_track_duration(&mut self, d: Duration) {
        self.next_track_duration = d;
    }
}
