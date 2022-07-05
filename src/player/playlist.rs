use crate::{config::get_app_config_path, track::Track};
// use anyhow::{anyhow, bail, Result};
use anyhow::Result;
use std::collections::VecDeque;
use std::fs::File;
// use std::io::{BufRead, BufReader, Write};
use std::io::{BufRead, BufReader, Write};
// use std::thread;

#[derive(Default)]
pub struct Playlist {
    pub tracks: VecDeque<Track>,
    pub current_track: Option<Track>,
    pub index: Option<usize>,
}

#[allow(unused)]
impl Playlist {
    pub fn new() -> Result<Self> {
        let tracks = Self::load()?;
        let mut current_track: Option<Track> = None;
        if let Some(track) = tracks.get(0) {
            current_track = Some(track.clone());
        }

        Ok(Self {
            tracks,
            current_track,
            index: Some(0),
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
            if let Ok(s) = Track::read_from_path(line) {
                playlist_items.push_back(s);
            };
        }

        Ok(playlist_items)
    }

    pub fn save(&mut self) -> Result<()> {
        let mut path = get_app_config_path()?;
        path.push("playlist.log");
        let mut file = File::create(path.as_path())?;
        for i in &self.tracks {
            if let Some(f) = i.file() {
                writeln!(&mut file, "{}", f)?;
            }
        }

        Ok(())
    }

    pub fn up(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        if let Some(index) = &mut self.index {
            if *index > 0 {
                *index -= 1;
            } else {
                *index = self.tracks.len() - 1;
            }
        }
    }

    pub fn down(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        if let Some(index) = &mut self.index {
            if *index + 1 < self.tracks.len() {
                *index += 1;
            } else {
                *index = 0;
            }
        }
    }

    pub fn up_with_len(&mut self, len: usize) {
        if let Some(index) = &mut self.index {
            if *index > 0 {
                *index -= 1;
            } else {
                *index = len - 1;
            }
        }
    }
    pub fn down_with_len(&mut self, len: usize) {
        if let Some(index) = &mut self.index {
            if *index + 1 < len {
                *index += 1;
            } else {
                *index = 0;
            }
        }
    }
    pub fn selected(&self) -> Option<&Track> {
        if let Some(index) = self.index {
            if let Some(item) = self.tracks.get(index) {
                return Some(item);
            }
        }
        None
    }
    pub fn index(&self) -> Option<usize> {
        self.index
    }
    pub fn len(&self) -> usize {
        self.tracks.len()
    }
    pub fn select(&mut self, i: Option<usize>) {
        self.index = i;
    }
    pub fn is_none(&self) -> bool {
        self.index.is_none()
    }
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }
    pub fn as_slice(&self) -> &VecDeque<Track> {
        &self.tracks
    }
    pub fn remove(&mut self, index: usize) {
        self.tracks.remove(index);
        let len = self.len();
        if let Some(selected) = self.index {
            if index == len && selected == len {
                self.index = Some(len.saturating_sub(1));
            } else if index == 0 && selected == 0 {
                self.index = Some(0);
            } else if len == 0 {
                self.index = None;
            }
        }
    }

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
}
