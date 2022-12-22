use crate::{config::get_app_config_path, track::Track};
use anyhow::{bail, Result};
use pathdiff::diff_utf8_paths;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Running,
    Stopped,
    Paused,
}

impl Default for Status {
    fn default() -> Self {
        Status::Stopped
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

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum Loop {
    Single,
    Playlist,
    Queue,
}

#[allow(clippy::non_ascii_literal)]
impl Loop {
    pub fn display(self, display_symbol: bool) -> String {
        if display_symbol {
            match self {
                Self::Single => "🔂".to_string(),
                Self::Playlist => "🔁".to_string(),
                Self::Queue => "⬇".to_string(),
            }
        } else {
            match self {
                Self::Single => "single".to_string(),
                Self::Playlist => "playlist".to_string(),
                Self::Queue => "consume".to_string(),
            }
        }
    }
}

impl Default for Loop {
    fn default() -> Self {
        Loop::Playlist
    }
}

#[derive(Default)]
pub struct Playlist {
    pub tracks: VecDeque<Track>,
    pub current_track: Option<Track>,
    pub index: Option<usize>,
    status: Status,
    loop_mode: Loop,
}

// #[allow(unused)]
impl Playlist {
    pub fn new(loop_mode: Loop) -> Result<Self> {
        let mut tracks = Self::load()?;
        let mut current_track: Option<Track> = None;
        if let Some(track) = tracks.pop_front() {
            current_track = Some(track);
        }

        Ok(Self {
            tracks,
            current_track,
            index: Some(0),
            status: Status::Stopped,
            loop_mode,
        })
    }

    pub fn load() -> Result<VecDeque<Track>> {
        let mut path = get_app_config_path()?;
        path.push("playlist.log");

        let file = if let Ok(f) = File::open(path.as_path()) {
            f
        } else {
            File::create(path.as_path())?;
            File::open(path)?
        };
        let reader = BufReader::new(file);
        let lines: Vec<_> = reader
            .lines()
            .map(|line| line.unwrap_or_else(|_| "Error".to_string()))
            .collect();

        let mut playlist_items = VecDeque::new();
        for line in &lines {
            if let Ok(s) = Track::read_from_path(line, false) {
                playlist_items.push_back(s);
            };
        }

        Ok(playlist_items)
    }

    pub fn save(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("playlist.log");

        let file = File::create(path.as_path())?;
        let mut writer = BufWriter::new(file);
        let mut bytes = Vec::new();
        if let Some(track) = &self.current_track {
            if let Some(f) = track.file() {
                bytes.extend(f.as_bytes());
                bytes.extend("\n".as_bytes());
            }
        }
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

    // pub fn up(&mut self) {
    //     if self.tracks.is_empty() {
    //         return;
    //     }

    //     if let Some(index) = &mut self.index {
    //         if *index > 0 {
    //             *index -= 1;
    //         } else {
    //             *index = self.tracks.len() - 1;
    //         }
    //     }
    // }

    // pub fn down(&mut self) {
    //     if self.tracks.is_empty() {
    //         return;
    //     }

    //     if let Some(index) = &mut self.index {
    //         if *index + 1 < self.tracks.len() {
    //             *index += 1;
    //         } else {
    //             *index = 0;
    //         }
    //     }
    // }

    // pub fn up_with_len(&mut self, len: usize) {
    //     if let Some(index) = &mut self.index {
    //         if *index > 0 {
    //             *index -= 1;
    //         } else {
    //             *index = len - 1;
    //         }
    //     }
    // }
    // pub fn down_with_len(&mut self, len: usize) {
    //     if let Some(index) = &mut self.index {
    //         if *index + 1 < len {
    //             *index += 1;
    //         } else {
    //             *index = 0;
    //         }
    //     }
    // }
    // pub fn selected(&self) -> Option<&Track> {
    //     if let Some(index) = self.index {
    //         if let Some(item) = self.tracks.get(index) {
    //             return Some(item);
    //         }
    //     }
    //     None
    // }
    // pub fn index(&self) -> Option<usize> {
    //     self.index
    // }
    pub fn len(&self) -> usize {
        self.tracks.len()
    }
    // pub fn select(&mut self, i: Option<usize>) {
    //     self.index = i;
    // }
    // pub fn is_none(&self) -> bool {
    //     self.index.is_none()
    // }
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }
    // pub fn as_slice(&self) -> &VecDeque<Track> {
    //     &self.tracks
    // }
    // pub fn remove(&mut self, index: usize) {
    //     self.tracks.remove(index);
    //     let len = self.len();
    //     if let Some(selected) = self.index {
    //         if index == len && selected == len {
    //             self.index = Some(len.saturating_sub(1));
    //         } else if index == 0 && selected == 0 {
    //             self.index = Some(0);
    //         } else if len == 0 {
    //             self.index = None;
    //         }
    //     }
    // }

    pub fn swap_down(&mut self, index: usize) {
        if index < self.len() - 1 {
            if let Some(track) = self.tracks.remove(index) {
                self.tracks.insert(index + 1, track);
            }
        }
    }

    pub fn swap_up(&mut self, index: usize) {
        if index > 0 {
            if let Some(track) = self.tracks.remove(index) {
                self.tracks.insert(index - 1, track);
            }
        }
    }

    pub fn get_current_track(&mut self) -> Option<String> {
        let mut result = None;
        if let Some(track) = &self.current_track {
            if let Some(file) = track.file() {
                result = Some(file.to_string());
            }
        }
        result
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn is_stopped(&self) -> bool {
        self.status == Status::Stopped
    }

    pub fn is_paused(&self) -> bool {
        self.status == Status::Paused
    }
    // pub fn is_running(&self) -> bool {
    //     self.status == Status::Running
    // }
    pub fn status(&self) -> Status {
        self.status
    }

    pub fn handle_current_track(&mut self) {
        // eprintln!("handle current track");

        if let Some(song) = self.tracks.pop_front() {
            match self.loop_mode {
                Loop::Playlist => self.tracks.push_back(song.clone()),
                Loop::Single => self.tracks.push_front(song.clone()),
                Loop::Queue => {}
            }
            self.current_track = Some(song);
        } else {
            self.current_track = None;
            self.set_status(Status::Stopped);
        }
    }

    pub fn cycle_loop_mode(&mut self) -> Loop {
        match self.loop_mode {
            Loop::Queue => {
                self.loop_mode = Loop::Playlist;
            }
            Loop::Playlist => {
                self.loop_mode = Loop::Single;
                if let Some(song) = self.tracks.pop_back() {
                    self.tracks.push_front(song);
                }
            }
            Loop::Single => {
                self.loop_mode = Loop::Queue;
                if let Some(song) = self.tracks.pop_front() {
                    self.tracks.push_back(song);
                }
            }
        };
        self.loop_mode
    }

    // export to M3U
    pub fn save_m3u(&self, filename: &str) -> Result<()> {
        let mut m3u = String::from("#EXTM3U\n");
        if self.tracks.is_empty() {
            bail!("No tracks in playlist, so no need to save.");
        }

        let mut parent_folder = PathBuf::new();

        let path_m3u = Path::new(filename);

        if path_m3u.is_dir() {
            parent_folder = path_m3u.to_path_buf();
        } else if let Some(parent) = path_m3u.parent() {
            parent_folder = parent.to_path_buf();
        };

        for track in &self.tracks {
            if let Some(file) = track.file() {
                let path_relative =
                    diff_utf8_paths(file, parent_folder.to_string_lossy().to_string());

                if let Some(p) = path_relative {
                    let path = format!("{p}\n");
                    m3u.push_str(&path);
                }
            }
        }
        std::fs::write(filename, m3u)?;
        Ok(())
    }
}
