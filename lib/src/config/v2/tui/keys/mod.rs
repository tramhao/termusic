#![allow(clippy::module_name_repetitions)]

use std::error::Error;
use std::str::CharIndices;
use std::string::ToString;
use std::{fmt::Display, iter::Peekable};

use ahash::HashMapExt;
use serde::{Deserialize, Serialize};
use tuirealm::event as tuievents;

mod conflict;
pub use conflict::KeyConflictError;
use conflict::{CheckConflict, KeyHashMap, KeyHashMapOwned, KeyPath};

use crate::once_chain;

#[derive(Debug, Clone, PartialEq)]
pub struct KeysCheckError {
    pub errored_keys: Vec<KeyConflictError>,
}

impl Display for KeysCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "There are {} Key Conflict Errors: [",
            self.errored_keys.len()
        )?;

        for err in &self.errored_keys {
            writeln!(f, "  {err},")?;
        }

        write!(f, "]")
    }
}

impl Error for KeysCheckError {}

impl From<Vec<KeyConflictError>> for KeysCheckError {
    fn from(value: Vec<KeyConflictError>) -> Self {
        Self {
            errored_keys: value,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct Keys {
    // -- Escape controls --
    /// Key to escape / close a layer (like closing a popup); never quits
    ///
    /// Global (applies everywhere, except text-input for Char's)
    pub escape: KeyBinding,
    /// Key to quit the application, also acts as "escape" if there are layers to quit
    ///
    /// Global (applies everywhere, except text-input for Char's)
    pub quit: KeyBinding,

    // -- Specifically grouped --
    #[serde(rename = "view")]
    pub select_view_keys: KeysSelectView,
    #[serde(rename = "navigation")]
    pub navigation_keys: KeysNavigation,
    #[serde(rename = "global_player")]
    pub player_keys: KeysPlayer,
    #[serde(rename = "global_lyric")]
    pub lyric_keys: KeysLyric,
    #[serde(rename = "library")]
    pub library_keys: KeysLibrary,
    #[serde(rename = "playlist")]
    pub playlist_keys: KeysPlaylist,
    #[serde(rename = "database")]
    pub database_keys: KeysDatabase,
    #[serde(rename = "podcast")]
    pub podcast_keys: KeysPodcast,
    #[serde(rename = "adjust_cover_art")]
    pub move_cover_art_keys: KeysMoveCoverArt,
    #[serde(rename = "config")]
    pub config_keys: KeysConfigEditor,
}

impl Keys {
    /// Check all the keys if they conflict with each-other
    pub fn check_keys(&self) -> Result<(), KeysCheckError> {
        let mut key_path = KeyPath::new_with_toplevel("keys");
        let mut global_keys = KeyHashMapOwned::new();

        self.check_conflict(&mut key_path, &mut global_keys)
            .map_err(KeysCheckError::from)
    }
}

impl Default for Keys {
    fn default() -> Self {
        Self {
            escape: tuievents::Key::Esc.into(),
            quit: tuievents::Key::Char('q').into(),
            select_view_keys: KeysSelectView::default(),
            navigation_keys: KeysNavigation::default(),
            player_keys: KeysPlayer::default(),
            lyric_keys: KeysLyric::default(),
            library_keys: KeysLibrary::default(),
            playlist_keys: KeysPlaylist::default(),
            database_keys: KeysDatabase::default(),
            podcast_keys: KeysPodcast::default(),
            move_cover_art_keys: KeysMoveCoverArt::default(),
            config_keys: KeysConfigEditor::default(),
        }
    }
}

impl CheckConflict for Keys {
    fn iter(&self) -> impl Iterator<Item = (&KeyBinding, &'static str)> {
        once_chain! {
            (&self.escape, "escape"),
            (&self.quit, "quit"),
        }
    }

    fn check_conflict(
        &self,
        key_path: &mut KeyPath,
        global_keys: &mut KeyHashMapOwned,
    ) -> Result<(), Vec<KeyConflictError>> {
        let mut conflicts: Vec<KeyConflictError> = Vec::new();
        let mut current_keys = KeyHashMap::new();

        // add & check direct keys, that are global everywhere
        for (key, path) in self.iter() {
            // check global first
            if let Some(existing_path) = global_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: existing_path.to_string(),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            if let Some(existing_path) = current_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: key_path.join_with_field(existing_path),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            global_keys.insert(key.clone(), key_path.join_with_field(path));
            current_keys.insert(key, path);
        }

        // -------------
        // lets do all the views first that dont rely on global player keys
        let init_len = global_keys.len(); // sanity check
        key_path.push("config");
        if let Err(new) = self.config_keys.check_conflict(key_path, global_keys) {
            conflicts.extend(new);
        }
        key_path.pop();
        // key_path.push("tag_editor");
        // if let Err(new) = self.tag_editor.check_conflict(key_path, global_keys) {
        //     conflicts.extend(new);
        // }
        // key_path.pop();

        assert_eq!(global_keys.len(), init_len); // sanity check, the above should not have added global_keys

        // -------------
        // now lets do all the ones that add global player keys
        key_path.push("view");
        if let Err(new) = self.select_view_keys.check_conflict(key_path, global_keys) {
            conflicts.extend(new);
        }
        key_path.pop();
        key_path.push("global_player");
        if let Err(new) = self.player_keys.check_conflict(key_path, global_keys) {
            conflicts.extend(new);
        }
        key_path.pop();
        key_path.push("global_lyric");
        if let Err(new) = self.lyric_keys.check_conflict(key_path, global_keys) {
            conflicts.extend(new);
        }
        key_path.pop();
        key_path.push("adjust_cover_art");
        if let Err(new) = self
            .move_cover_art_keys
            .check_conflict(key_path, global_keys)
        {
            conflicts.extend(new);
        }
        key_path.pop();

        // -------------
        // now lets do all the ones that do not add any global player keys, but need to be checked against those
        key_path.push("navigation");
        if let Err(new) = self.navigation_keys.check_conflict(key_path, global_keys) {
            conflicts.extend(new);
        }
        key_path.pop();
        key_path.push("library");
        if let Err(new) = self.library_keys.check_conflict(key_path, global_keys) {
            conflicts.extend(new);
        }
        key_path.pop();
        key_path.push("playlist");
        if let Err(new) = self.playlist_keys.check_conflict(key_path, global_keys) {
            conflicts.extend(new);
        }
        key_path.pop();
        key_path.push("podcast");
        if let Err(new) = self.podcast_keys.check_conflict(key_path, global_keys) {
            conflicts.extend(new);
        }
        key_path.pop();

        // -------------
        if !conflicts.is_empty() {
            return Err(conflicts);
        }

        Ok(())
    }
}

/// Global keys to open global views
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct KeysSelectView {
    /// Key to switch to the Music-Library view
    pub view_library: KeyBinding,
    /// Key to switch to the Database view
    pub view_database: KeyBinding,
    /// Key to switch to the Podcast view
    pub view_podcasts: KeyBinding,

    /// Key to open the Config view
    pub open_config: KeyBinding,
    /// Key to open the Help-Popup
    pub open_help: KeyBinding,
}

impl Default for KeysSelectView {
    fn default() -> Self {
        Self {
            view_library: tuievents::Key::Char('1').into(),
            view_database: tuievents::Key::Char('2').into(),
            view_podcasts: tuievents::Key::Char('3').into(),
            open_config: tuievents::KeyEvent::new(
                tuievents::Key::Char('C'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            open_help: tuievents::KeyEvent::new(
                tuievents::Key::Char('h'),
                tuievents::KeyModifiers::CONTROL,
            )
            .into(),
        }
    }
}

impl CheckConflict for KeysSelectView {
    fn iter(&self) -> impl Iterator<Item = (&KeyBinding, &'static str)> {
        once_chain! {
            (&self.view_library, "view_library"),
            (&self.view_database, "view_database"),
            (&self.view_podcasts, "view_podcasts"),

            (&self.open_config, "open_config"),
            (&self.open_help, "open_help")
        }
    }

    fn check_conflict(
        &self,
        key_path: &mut KeyPath,
        global_keys: &mut KeyHashMapOwned,
    ) -> Result<(), Vec<KeyConflictError>> {
        let mut conflicts: Vec<KeyConflictError> = Vec::new();
        let mut current_keys = KeyHashMap::new();

        for (key, path) in self.iter() {
            // check global first
            if let Some(existing_path) = global_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: existing_path.to_string(),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            if let Some(existing_path) = current_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: key_path.join_with_field(existing_path),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            global_keys.insert(key.clone(), key_path.join_with_field(path));
            current_keys.insert(key, path);
        }

        if !conflicts.is_empty() {
            return Err(conflicts);
        }

        Ok(())
    }
}

/// Global Player controls
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct KeysPlayer {
    /// Key to toggle Play/Pause
    ///
    /// Will only apply in specific widgets (like the Playlist, but not in Config)
    pub toggle_pause: KeyBinding,
    /// Key to change to the next track
    ///
    /// Will only apply in specific widgets (like the Playlist, but not in Config)
    pub next_track: KeyBinding,
    /// Key to change to the previous track
    ///
    /// Will only apply in specific widgets (like the Playlist, but not in Config)
    pub previous_track: KeyBinding,
    /// Key to increase volume (by a set amount)
    ///
    /// Will only apply in specific widgets (like the Playlist, but not in Config)
    pub volume_up: KeyBinding,
    /// Key to decrease volume (by a set amount)
    ///
    /// Will only apply in specific widgets (like the Playlist, but not in Config)
    pub volume_down: KeyBinding,
    /// Key to seek forwards (by a set amount)
    ///
    /// Will only apply in specific widgets (like the Playlist, but not in Config)
    pub seek_forward: KeyBinding,
    /// Key to seek backwards (by a set amount)
    ///
    /// Will only apply in specific widgets (like the Playlist, but not in Config)
    pub seek_backward: KeyBinding,
    /// Key to increase speed (by a set amount)
    ///
    /// Will only apply in specific widgets (like the Playlist, but not in Config)
    pub speed_up: KeyBinding,
    /// Key to decrease speed (by a set amount)
    ///
    /// Will only apply in specific widgets (like the Playlist, but not in Config)
    pub speed_down: KeyBinding,
    /// Key to toggle if track-prefetching should be enabled
    ///
    /// Will only apply in specific widgets (like the Playlist, but not in Config)
    // TODO: always enable "gapless" in rusty backend and rename option to "prefetch"
    pub toggle_prefetch: KeyBinding,

    /// Key to save the current playlist as a "m3u" playlist
    pub save_playlist: KeyBinding,
}

impl Default for KeysPlayer {
    fn default() -> Self {
        Self {
            toggle_pause: tuievents::Key::Char(' ').into(),
            next_track: tuievents::Key::Char('n').into(),
            previous_track: tuievents::KeyEvent::new(
                tuievents::Key::Char('N'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            volume_up: tuievents::Key::Char('+').into(),
            volume_down: tuievents::Key::Char('-').into(),
            seek_forward: tuievents::Key::Char('f').into(),
            seek_backward: tuievents::Key::Char('b').into(),
            speed_up: tuievents::KeyEvent::new(
                tuievents::Key::Char('f'),
                tuievents::KeyModifiers::CONTROL,
            )
            .into(),
            speed_down: tuievents::KeyEvent::new(
                tuievents::Key::Char('b'),
                tuievents::KeyModifiers::CONTROL,
            )
            .into(),
            toggle_prefetch: tuievents::KeyEvent::new(
                tuievents::Key::Char('g'),
                tuievents::KeyModifiers::CONTROL,
            )
            .into(),
            save_playlist: tuievents::KeyEvent::new(
                tuievents::Key::Char('s'),
                tuievents::KeyModifiers::CONTROL,
            )
            .into(),
        }
    }
}

impl CheckConflict for KeysPlayer {
    fn iter(&self) -> impl Iterator<Item = (&KeyBinding, &'static str)> {
        once_chain! {
            (&self.toggle_pause, "toggle_pause"),
            (&self.next_track, "next_track"),
            (&self.previous_track, "previous_track"),
            (&self.volume_up, "volume_up"),
            (&self.volume_down, "volume_down"),
            (&self.seek_forward, "seek_forward"),
            (&self.seek_backward, "seek_backward"),
            (&self.speed_up, "speed_up"),
            (&self.speed_down, "speed_down"),
            (&self.toggle_prefetch, "toggle_prefetch"),

            (&self.save_playlist, "save_playlist"),
        }
    }

    fn check_conflict(
        &self,
        key_path: &mut KeyPath,
        global_keys: &mut KeyHashMapOwned,
    ) -> Result<(), Vec<KeyConflictError>> {
        let mut conflicts: Vec<KeyConflictError> = Vec::new();
        let mut current_keys = KeyHashMap::new();

        for (key, path) in self.iter() {
            // check global first
            if let Some(existing_path) = global_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: existing_path.to_string(),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            if let Some(existing_path) = current_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: key_path.join_with_field(existing_path),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            global_keys.insert(key.clone(), key_path.join_with_field(path));
            current_keys.insert(key, path);
        }

        if !conflicts.is_empty() {
            return Err(conflicts);
        }

        Ok(())
    }
}

/// Global Lyric adjustment keys
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct KeysLyric {
    /// Key to adjust lyric offset forwards (by a set amount)
    ///
    /// Will only apply in specific widgets (like the Playlist, but not in Config)
    pub adjust_offset_forwards: KeyBinding,
    /// Key to adjust lyric offset backwards (by a set amount)
    ///
    /// Will only apply in specific widgets (like the Playlist, but not in Config)
    pub adjust_offset_backwards: KeyBinding,
    /// Key to cycle through multiple lyric frames
    ///
    /// Will only apply in specific widgets (like the Playlist, but not in Config)
    pub cycle_frames: KeyBinding,
}

impl Default for KeysLyric {
    fn default() -> Self {
        Self {
            adjust_offset_forwards: tuievents::KeyEvent::new(
                tuievents::Key::Char('F'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            adjust_offset_backwards: tuievents::KeyEvent::new(
                tuievents::Key::Char('B'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            cycle_frames: tuievents::KeyEvent::new(
                tuievents::Key::Char('T'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
        }
    }
}

impl CheckConflict for KeysLyric {
    fn iter(&self) -> impl Iterator<Item = (&KeyBinding, &'static str)> {
        once_chain! {
            (&self.adjust_offset_forwards, "adjust_offset_forwards"),
            (&self.adjust_offset_backwards, "adjust_offset_backwards"),
            (&self.cycle_frames, "cycle_frames"),
        }
    }

    fn check_conflict(
        &self,
        key_path: &mut KeyPath,
        global_keys: &mut KeyHashMapOwned,
    ) -> Result<(), Vec<KeyConflictError>> {
        let mut conflicts: Vec<KeyConflictError> = Vec::new();
        let mut current_keys = KeyHashMap::new();

        for (key, path) in self.iter() {
            // check global first
            if let Some(existing_path) = global_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: existing_path.to_string(),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            if let Some(existing_path) = current_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: key_path.join_with_field(existing_path),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            global_keys.insert(key.clone(), key_path.join_with_field(path));
            current_keys.insert(key, path);
        }

        if !conflicts.is_empty() {
            return Err(conflicts);
        }

        Ok(())
    }
}

/// Extra navigation keys (like vim keylayout)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct KeysNavigation {
    // Note: Arrow-keys will always correspond to this
    /// Key to navigate upwards (like in a list)
    pub up: KeyBinding,
    /// Key to navigate downwards (like in a list)
    pub down: KeyBinding,
    /// Key to navigate left (like closing a node in the music library)
    pub left: KeyBinding,
    /// Key to navigate right (like opening a node in the music library)
    pub right: KeyBinding,
    /// Key to navigate to the top (like in a list)
    pub goto_top: KeyBinding,
    /// Key to navigate to the bottom (like in a list)
    pub goto_bottom: KeyBinding,
}

impl Default for KeysNavigation {
    fn default() -> Self {
        // using vim-like navigation
        Self {
            up: tuievents::Key::Char('k').into(),
            down: tuievents::Key::Char('j').into(),
            left: tuievents::Key::Char('h').into(),
            right: tuievents::Key::Char('l').into(),
            goto_top: tuievents::Key::Char('g').into(),
            goto_bottom: tuievents::KeyEvent::new(
                tuievents::Key::Char('G'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
        }
    }
}

impl CheckConflict for KeysNavigation {
    fn iter(&self) -> impl Iterator<Item = (&KeyBinding, &'static str)> {
        once_chain! {
            (&self.up, "up"),
            (&self.down, "down"),
            (&self.left, "left"),
            (&self.right, "right"),
            (&self.goto_top, "goto_top"),
            (&self.goto_bottom, "goto_bottom"),
        }
    }

    fn check_conflict(
        &self,
        key_path: &mut KeyPath,
        global_keys: &mut KeyHashMapOwned,
    ) -> Result<(), Vec<KeyConflictError>> {
        let mut conflicts: Vec<KeyConflictError> = Vec::new();
        let mut current_keys = KeyHashMap::new();

        for (key, path) in self.iter() {
            // check global first
            if let Some(existing_path) = global_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: existing_path.to_string(),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            if let Some(existing_path) = current_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: key_path.join_with_field(existing_path),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            current_keys.insert(key, path);
        }

        if !conflicts.is_empty() {
            return Err(conflicts);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct KeysLibrary {
    /// Key to load the currently selected track (only if on a file node)
    pub load_track: KeyBinding,
    /// Key to load the whole directory (only if on a directory node)
    pub load_dir: KeyBinding,
    /// Key to delete the currently selected node (which can be both a track or a directory)
    pub delete: KeyBinding,
    /// Key to start moving a node to another (requires "paste" to finish move)
    pub yank: KeyBinding,
    /// Key to finish moving a node (requires "yank" to start a move)
    pub paste: KeyBinding,
    /// Key to cycle through the Music-Directories
    pub cycle_root: KeyBinding,
    /// Key to add the currently entered node as a music root
    pub add_root: KeyBinding,
    /// Key to remove the currently entered node as music root
    pub remove_root: KeyBinding,

    /// Key to open local search (root being the selected `music_dir` root)
    pub search: KeyBinding,
    /// Key to open youtube search
    pub youtube_search: KeyBinding,
    /// Key to open the tag editor on that node (only works for files)
    pub open_tag_editor: KeyBinding,
}

impl Default for KeysLibrary {
    fn default() -> Self {
        Self {
            load_track: tuievents::Key::Char('l').into(),
            load_dir: tuievents::KeyEvent::new(
                tuievents::Key::Char('L'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            delete: tuievents::Key::Char('d').into(),
            yank: tuievents::Key::Char('y').into(),
            paste: tuievents::Key::Char('p').into(),
            cycle_root: tuievents::Key::Char('o').into(),
            add_root: tuievents::Key::Char('a').into(),
            remove_root: tuievents::KeyEvent::new(
                tuievents::Key::Char('A'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            search: tuievents::Key::Char('/').into(),
            youtube_search: tuievents::Key::Char('s').into(),
            open_tag_editor: tuievents::Key::Char('t').into(),
        }
    }
}

impl CheckConflict for KeysLibrary {
    fn iter(&self) -> impl Iterator<Item = (&KeyBinding, &'static str)> {
        once_chain! {
            (&self.load_track, "load_track"),
            (&self.load_dir, "load_dir"),
            (&self.delete, "delete"),
            (&self.yank, "yank"),
            (&self.paste, "paste"),
            (&self.cycle_root, "cycle_root"),
            (&self.add_root, "add_root"),
            (&self.remove_root, "remove_root"),

            (&self.search, "search"),
            (&self.youtube_search, "youtube_search"),
            (&self.open_tag_editor, "open_tag_editor"),
        }
    }

    fn check_conflict(
        &self,
        key_path: &mut KeyPath,
        global_keys: &mut KeyHashMapOwned,
    ) -> Result<(), Vec<KeyConflictError>> {
        let mut conflicts: Vec<KeyConflictError> = Vec::new();
        let mut current_keys = KeyHashMap::new();

        for (key, path) in self.iter() {
            // check global first
            if let Some(existing_path) = global_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: existing_path.to_string(),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            if let Some(existing_path) = current_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: key_path.join_with_field(existing_path),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            current_keys.insert(key, path);
        }

        if !conflicts.is_empty() {
            return Err(conflicts);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct KeysPlaylist {
    /// Key to delete the currently selected node from the playlist
    pub delete: KeyBinding,
    /// Key to clear the playlist of all tracks
    pub delete_all: KeyBinding,
    /// Key to shuffle the playlist with all currently added tracks
    pub shuffle: KeyBinding,
    /// Key to cycle through the Loop-modes, see [`LoopMode`](super::super::server::LoopMode)
    pub cycle_loop_mode: KeyBinding,
    /// Key to play the currently selected node
    pub play_selected: KeyBinding,
    /// Key to open playlist search (searches through the songs currently in the playlist)
    pub search: KeyBinding,
    /// Key to swap currently selected track with the node above it
    pub swap_up: KeyBinding,
    /// Key to swap currently selected track with the node below it
    pub swap_down: KeyBinding,

    /// Key to add random songs to the playlist (a set amount)
    ///
    /// previously known as `cmus_tqueue`
    pub add_random_songs: KeyBinding,
    /// Key to add a random Album to the playlist
    ///
    /// previously known as `cmus_lqueue`
    // NOTE: currently this can be somewhat broken sometimes, cause unknown
    pub add_random_album: KeyBinding,
}

impl Default for KeysPlaylist {
    fn default() -> Self {
        Self {
            delete: tuievents::Key::Char('d').into(),
            delete_all: tuievents::KeyEvent::new(
                tuievents::Key::Char('D'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            shuffle: tuievents::Key::Char('r').into(),
            cycle_loop_mode: tuievents::Key::Char('m').into(),
            play_selected: tuievents::Key::Char('l').into(),
            search: tuievents::Key::Char('/').into(),
            swap_up: tuievents::KeyEvent::new(
                tuievents::Key::Char('K'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            swap_down: tuievents::KeyEvent::new(
                tuievents::Key::Char('J'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            add_random_songs: tuievents::Key::Char('s').into(),
            add_random_album: tuievents::KeyEvent::new(
                tuievents::Key::Char('S'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
        }
    }
}

impl CheckConflict for KeysPlaylist {
    fn iter(&self) -> impl Iterator<Item = (&KeyBinding, &'static str)> {
        once_chain! {
            (&self.delete, "delete"),
            (&self.delete_all, "delete_all"),
            (&self.shuffle, "shuffle"),
            (&self.cycle_loop_mode, "cycle_loop_mode"),
            (&self.play_selected, "play_selected"),
            (&self.search, "search"),
            (&self.swap_up, "swap_up"),
            (&self.swap_down, "swap_down"),

            (&self.add_random_songs, "add_random_songs"),
            (&self.add_random_album, "add_random_album"),
        }
    }

    fn check_conflict(
        &self,
        key_path: &mut KeyPath,
        global_keys: &mut KeyHashMapOwned,
    ) -> Result<(), Vec<KeyConflictError>> {
        let mut conflicts: Vec<KeyConflictError> = Vec::new();
        let mut current_keys = KeyHashMap::new();

        for (key, path) in self.iter() {
            // check global first
            if let Some(existing_path) = global_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: existing_path.to_string(),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            if let Some(existing_path) = current_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: key_path.join_with_field(existing_path),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            current_keys.insert(key, path);
        }

        if !conflicts.is_empty() {
            return Err(conflicts);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct KeysPodcast {
    /// Key to open the search for new feeds
    pub search: KeyBinding,
    /// Key to mark the currently selected podcast episode as "played"
    pub mark_played: KeyBinding,
    /// Key to mark all episodes in the current podcast as "played"
    pub mark_all_played: KeyBinding,
    /// Key to refresh the currently selected feed
    pub refresh_feed: KeyBinding,
    /// Key to refresh all added feeds
    pub refresh_all_feeds: KeyBinding,
    /// Key to download the currently selected episode
    pub download_episode: KeyBinding,
    /// Key to delete the downloaded local file of the currently selected episode
    pub delete_local_episode: KeyBinding,
    /// Key to delete the currently selected feed
    pub delete_feed: KeyBinding,
    /// Key to delete all the added feeds
    pub delete_all_feeds: KeyBinding,
}

impl Default for KeysPodcast {
    fn default() -> Self {
        Self {
            search: tuievents::Key::Char('s').into(),
            mark_played: tuievents::Key::Char('m').into(),
            mark_all_played: tuievents::KeyEvent::new(
                tuievents::Key::Char('M'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            refresh_feed: tuievents::Key::Char('r').into(),
            refresh_all_feeds: tuievents::KeyEvent::new(
                tuievents::Key::Char('R'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            download_episode: tuievents::Key::Char('d').into(),
            delete_local_episode: tuievents::KeyEvent::new(
                tuievents::Key::Char('D'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            delete_feed: tuievents::Key::Char('x').into(),
            delete_all_feeds: tuievents::KeyEvent::new(
                tuievents::Key::Char('X'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
        }
    }
}

impl CheckConflict for KeysPodcast {
    fn iter(&self) -> impl Iterator<Item = (&KeyBinding, &'static str)> {
        once_chain! {
            (&self.search, "search"),
            (&self.mark_played, "mark_played"),
            (&self.mark_all_played, "mark_all_played"),
            (&self.refresh_feed, "refresh_feed"),
            (&self.refresh_all_feeds, "refresh_all_feeds"),
            (&self.download_episode, "download_episode"),
            (&self.delete_local_episode, "delete_local_episode"),
            (&self.delete_feed, "delete_feed"),
            (&self.delete_all_feeds, "delete_all_feeds"),
        }
    }

    fn check_conflict(
        &self,
        key_path: &mut KeyPath,
        global_keys: &mut KeyHashMapOwned,
    ) -> Result<(), Vec<KeyConflictError>> {
        let mut conflicts: Vec<KeyConflictError> = Vec::new();
        let mut current_keys = KeyHashMap::new();

        for (key, path) in self.iter() {
            // check global first
            if let Some(existing_path) = global_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: existing_path.to_string(),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            if let Some(existing_path) = current_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: key_path.join_with_field(existing_path),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            current_keys.insert(key, path);
        }

        if !conflicts.is_empty() {
            return Err(conflicts);
        }

        Ok(())
    }
}

/// Keys to manipulate the Cover-Art position
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct KeysMoveCoverArt {
    /// Key to move the album cover to the left (by a set amount)
    pub move_left: KeyBinding,
    /// Key to move the album cover to the right (by a set amount)
    pub move_right: KeyBinding,
    /// Key to move the album cover up (by a set amount)
    pub move_up: KeyBinding,
    /// Key to move the album cover down (by a set amount)
    pub move_down: KeyBinding,

    /// Key to increase the cover-art size (by a set amount)
    pub increase_size: KeyBinding,
    /// Key to decrease the cover-art size (by a set amount)
    pub decrease_size: KeyBinding,

    /// Key to toggle whether the Cover-Art is or not
    pub toggle_hide: KeyBinding,
}

impl Default for KeysMoveCoverArt {
    fn default() -> Self {
        Self {
            move_left: tuievents::KeyEvent::new(
                tuievents::Key::Left,
                tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            move_right: tuievents::KeyEvent::new(
                tuievents::Key::Right,
                tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            move_up: tuievents::KeyEvent::new(
                tuievents::Key::Up,
                tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            move_down: tuievents::KeyEvent::new(
                tuievents::Key::Down,
                tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            increase_size: tuievents::KeyEvent::new(
                tuievents::Key::PageUp,
                tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            decrease_size: tuievents::KeyEvent::new(
                tuievents::Key::PageDown,
                tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
            )
            .into(),
            toggle_hide: tuievents::KeyEvent::new(
                tuievents::Key::End,
                tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
            )
            .into(),
        }
    }
}

impl CheckConflict for KeysMoveCoverArt {
    fn iter(&self) -> impl Iterator<Item = (&KeyBinding, &'static str)> {
        once_chain! {
            (&self.move_left, "move_left"),
            (&self.move_right, "move_right"),
            (&self.move_up, "move_up"),
            (&self.move_down, "move_down"),

            (&self.increase_size, "increase_size"),
            (&self.decrease_size, "decrease_size"),

            (&self.toggle_hide, "toggle_hide"),
        }
    }

    fn check_conflict(
        &self,
        key_path: &mut KeyPath,
        global_keys: &mut KeyHashMapOwned,
    ) -> Result<(), Vec<KeyConflictError>> {
        let mut conflicts: Vec<KeyConflictError> = Vec::new();
        let mut current_keys = KeyHashMap::new();

        for (key, path) in self.iter() {
            // check global first
            if let Some(existing_path) = global_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: existing_path.to_string(),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            if let Some(existing_path) = current_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: key_path.join_with_field(existing_path),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            global_keys.insert(key.clone(), key_path.join_with_field(path));
            current_keys.insert(key, path);
        }

        if !conflicts.is_empty() {
            return Err(conflicts);
        }

        Ok(())
    }
}

/// Keys for the config editor
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct KeysConfigEditor {
    /// Save the config to disk
    pub save: KeyBinding,
}

impl Default for KeysConfigEditor {
    fn default() -> Self {
        Self {
            save: tuievents::KeyEvent::new(
                tuievents::Key::Char('s'),
                tuievents::KeyModifiers::CONTROL,
            )
            .into(),
        }
    }
}

impl CheckConflict for KeysConfigEditor {
    fn iter(&self) -> impl Iterator<Item = (&KeyBinding, &'static str)> {
        once_chain! {
            (&self.save, "save"),
        }
    }

    fn check_conflict(
        &self,
        key_path: &mut KeyPath,
        global_keys: &mut KeyHashMapOwned,
    ) -> Result<(), Vec<KeyConflictError>> {
        let mut conflicts: Vec<KeyConflictError> = Vec::new();
        let mut current_keys = KeyHashMap::new();

        for (key, path) in self.iter() {
            // check global first
            if let Some(existing_path) = global_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: existing_path.to_string(),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            if let Some(existing_path) = current_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: key_path.join_with_field(existing_path),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            current_keys.insert(key, path);
        }

        if !conflicts.is_empty() {
            return Err(conflicts);
        }

        Ok(())
    }
}

/// Keys for the database view
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct KeysDatabase {
    /// Add the currently selected track to the playlist
    pub add_selected: KeyBinding,
    /// Add all tracks in the Database view "Tracks" section
    pub add_all: KeyBinding,
}

impl Default for KeysDatabase {
    fn default() -> Self {
        Self {
            add_selected: tuievents::Key::Char('l').into(),
            add_all: tuievents::KeyEvent::new(
                tuievents::Key::Char('L'),
                tuievents::KeyModifiers::SHIFT,
            )
            .into(),
        }
    }
}

impl CheckConflict for KeysDatabase {
    fn iter(&self) -> impl Iterator<Item = (&KeyBinding, &'static str)> {
        once_chain! {
            (&self.add_all, "add_all"),
        }
    }

    fn check_conflict(
        &self,
        key_path: &mut KeyPath,
        global_keys: &mut KeyHashMapOwned,
    ) -> Result<(), Vec<KeyConflictError>> {
        let mut conflicts: Vec<KeyConflictError> = Vec::new();
        let mut current_keys = KeyHashMap::new();

        for (key, path) in self.iter() {
            // check global first
            if let Some(existing_path) = global_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: existing_path.to_string(),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            if let Some(existing_path) = current_keys.get(key) {
                conflicts.push(KeyConflictError {
                    key_path_first: key_path.join_with_field(existing_path),
                    key_path_second: key_path.join_with_field(path),
                    key: key.clone(),
                });
                continue;
            }

            current_keys.insert(key, path);
        }

        if !conflicts.is_empty() {
            return Err(conflicts);
        }

        Ok(())
    }
}

// TODO: upgrade errors with what config-key has errored
// TODO: consider upgrading this with "thiserror"
/// Error for when [`Key`] parsing fails
#[derive(Debug, Clone, PartialEq)]
pub enum KeyParseError {
    /// Error when either the string is empty, or only has modifiers.
    ///
    /// Listing (`key_bind`)
    NoKeyFound(String),
    /// The Key shortcut was formatted incorrectly (like "++" or "+control")
    ///
    /// Listing (`key_bind`)
    TrailingDelimiter(String),
    /// Error when multiple keys are found (like "Q+E")
    ///
    /// Listing (`key_bind`, (`old_key`, `new_key`))
    MultipleKeys(String, (String, String)),
    /// Error when a unknown value is found (a value that could not be parsed as a key or modifier)
    ///
    /// Example being a value that is not 1 length, starts with "f" and has numbers following or is a match against [`const_keys`].
    /// like `"    "`
    ///
    /// Listing (`key_bind`)
    UnknownKey(String),
}

impl Display for KeyParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to parse Key because {}",
            // "{:#?}" debug representation is explicitly used here to escape the string contents
            match self {
                Self::NoKeyFound(val) =>
                    format!("no key was found in the mapping, given: {val:#?}"),
                Self::TrailingDelimiter(val) => format!("trailing delimiter in key: {val:#?}"),
                Self::MultipleKeys(val, keys) => format!(
                    "multiple keys were found, keys: [{}, {}], mapping: {:#?}",
                    keys.0, keys.1, val
                ),
                Self::UnknownKey(val) => format!("of unknown key in mapping: {val:#?}"),
            }
        )
    }
}

impl Error for KeyParseError {}

// Note: this could likely be optimized / improved when the std patters becomes available (to match "".split('')), see https://github.com/rust-lang/rust/issues/27721
/// A [`str::split`] replacement that works similar to `str::split(_, '+')`, but can also return the delimiter if directly followed
/// like `"control++"` separates it into `["control", "+"]`.
#[derive(Debug)]
struct SplitAtPlus<'a> {
    text: &'a str,
    chars: Peekable<CharIndices<'a>>,
    /// Track if the previous character was [`Self::DELIM`] but returned as a character across "self.next" calls
    last_char_was_returned_delim: bool,
    /// Tracker that indicates that the last char was a [`Self::DELIM`] and is used to return a trailing empty-string.
    ///
    /// For example this is wanted so that we can return a `InvalidFormat` in the actual use-case of this split type.
    ///
    /// Examples:
    /// - `"++"` -> `["+", ""]`
    /// - `"q+"` -> `["q", ""]`
    last_char_was_delim: bool,
}

impl<'a> SplitAtPlus<'a> {
    /// The Delimiter used in this custom split
    const DELIM: char = '+';

    fn new(text: &'a str) -> Self {
        Self {
            text,
            chars: text.char_indices().peekable(),
            last_char_was_returned_delim: false,
            last_char_was_delim: false,
        }
    }
}

impl<'a> Iterator for SplitAtPlus<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // loop until a start position can be found (should loop at most 2 times)
        let (start, mut prior_char) = loop {
            break match self.chars.next() {
                // pass-on if there is nothing to return anymore
                None => {
                    if self.last_char_was_delim {
                        self.last_char_was_delim = false;
                        return Some("");
                    }

                    return None;
                }
                Some((i, c)) if c == Self::DELIM => {
                    // return a "+" if not yet encountered, like:
                    // in "++control" count the first plus as a key and the second as a delimiter
                    // in "+++" count the first plus as a key, the second as a delimiter and the third as a key again
                    // in "control++" where we are at the iteration after "control+" and at the last "+"
                    if self.last_char_was_returned_delim {
                        self.last_char_was_returned_delim = false;
                        self.last_char_was_delim = true;
                        continue;
                    } else if i == 0 && self.chars.peek().is_some_and(|v| v.1 != Self::DELIM) {
                        // special case where the delimiter is the first, but not followed by another delimiter, like "+q"
                        // this is so we return a InvalidFormat later on (treat the first "+" as a delimiter instead of a key)
                        self.last_char_was_returned_delim = false;
                        self.last_char_was_delim = true;
                        return Some("");
                    }

                    self.last_char_was_returned_delim = true;
                    self.last_char_was_delim = false;
                    return Some("+");
                }
                // not a delimiter, so just pass it as the start
                // this case is for example "q+control" where "q" is the first character
                // or "control++"
                Some(v) => v,
            };
        };

        // the following should never need to be set, as "last_char_was_returned_delim" will only get set in the case above
        // and down below consumed by the "chars.next" call
        // self.last_char_was_returned_delim = false;
        self.last_char_was_delim = false;

        loop {
            prior_char = match self.chars.next() {
                // if there is no next char, return the string from the start point as there is also no delimiter
                // example "q+control" where this iteration is past the "q+" and at "control"
                None => return Some(&self.text[start..]),
                // we have run into a delimiter, so return all the text since then
                // like the first plus in "q+control"
                // also note that "chars.next()" consumes the delimiter character and so will not be returned in the next "self.next" call
                Some((end, c)) if c == Self::DELIM && prior_char != Self::DELIM => {
                    self.last_char_was_delim = true;
                    return Some(&self.text[start..end]);
                }
                // use this new char as the last_char and repeat the loop as we have not hit the end or a delimiter yet
                Some((_, c)) => c,
            }
        }
    }
}

/// Wrapper around the stored Key-Event to use custom de- and serialization
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct KeyBinding {
    pub key_event: tuievents::KeyEvent,
}

impl KeyBinding {
    /// Parse a Key with modifiers from a given string.
    ///
    /// Multiple same-modifiers are counted as one, and multiple keys are a error
    pub fn try_from_str(input: &str) -> Result<Self, KeyParseError> {
        let input = input.to_lowercase();
        let mut modifiers = tuievents::KeyModifiers::empty();
        let mut key_opt: Option<tuievents::Key> = None;

        for val in SplitAtPlus::new(&input) {
            // make a trailing "+" as a error, like "q+"
            if val.is_empty() {
                return Err(KeyParseError::TrailingDelimiter(input.clone()));
            }

            if let Ok(new_key) = KeyWrap::try_from(val) {
                let opt: &mut Option<tuievents::Key> = &mut key_opt;
                if let Some(existing_key) = opt {
                    return Err(KeyParseError::MultipleKeys(
                        input.clone(),
                        (
                            KeyWrap::from(*existing_key).to_string(),
                            new_key.to_string(),
                        ),
                    ));
                }

                *opt = Some(new_key.0);

                continue;
            }

            if let Ok(new_modifier) = SupportedModifiers::try_from(val) {
                modifiers |= new_modifier.into();

                continue;
            }

            return Err(KeyParseError::UnknownKey(val.into()));
        }

        let Some(mut code) = key_opt else {
            return Err(KeyParseError::NoKeyFound(input.clone()));
        };

        // transform the key to be upper-case if "Shift" is enabled, as that is what tuirealm will provide (and we cannot modify that)
        if modifiers.intersects(tuievents::KeyModifiers::SHIFT) {
            if let tuievents::Key::Char(v) = code {
                code = tuievents::Key::Char(v.to_ascii_uppercase());
            }
        }

        Ok(Self {
            key_event: tuievents::KeyEvent::new(code, modifiers),
        })
    }

    /// Get the inner key
    #[inline]
    pub fn get(&self) -> tuievents::KeyEvent {
        self.key_event
    }

    /// Get the Current Modifier, and the string representation of the key
    #[inline]
    pub fn mod_key(&self) -> (tuievents::KeyModifiers, String) {
        (
            self.key_event.modifiers,
            KeyWrap::from(self.key_event.code).to_string(),
        )
    }
}

impl Display for KeyBinding {
    /// Get a string from the current instance in the format of modifiers+key like "control+alt+shift+q", all lowercase
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = KeyWrap::from(self.key_event.code);
        for res in SupportedModifiers::from_keymodifiers(self.key_event.modifiers)
            .into_iter()
            .map(Into::<&str>::into)
            .map(|v| write!(f, "{v}+"))
        {
            res?;
        }

        write!(f, "{key}")
    }
}

impl TryFrom<&str> for KeyBinding {
    type Error = KeyParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl TryFrom<String> for KeyBinding {
    type Error = KeyParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_str(&value)
    }
}

impl From<KeyBinding> for String {
    fn from(value: KeyBinding) -> Self {
        value.to_string()
    }
}

/// Simple implementation to easily convert a key without modifiers to one
impl From<KeyWrap> for KeyBinding {
    fn from(value: KeyWrap) -> Self {
        Self {
            key_event: tuievents::KeyEvent::new(value.0, tuievents::KeyModifiers::empty()),
        }
    }
}

// convenience convertion for easier construction
impl From<tuievents::Key> for KeyBinding {
    fn from(value: tuievents::Key) -> Self {
        Self::from(KeyWrap(value))
    }
}

// convenience convertion for easier construction
impl From<tuievents::KeyEvent> for KeyBinding {
    fn from(value: tuievents::KeyEvent) -> Self {
        Self { key_event: value }
    }
}

/// Error for when [`SupportedKeys`] parsing fails
#[derive(Debug, Clone, PartialEq)]
pub enum KeyWrapParseError {
    Empty,
    UnknownKey(String),
}

/// Wrapper to parse and serialize a key in a defined format
#[derive(Debug, PartialEq)]
struct KeyWrap(tuievents::Key);

/// Module for defining key string in one place, instead of multiple times in multiple places
mod const_keys {
    /// Macro to not repeat yourself writing `const IDENT: &str = CONTENT`
    ///
    /// Allows usage of calling one at a time:
    ///
    /// ```
    /// const_str!(NAME, "STRING")
    /// ```
    ///
    /// or multiple at a time to even save repeated "`const_str`!" invokations:
    ///
    /// ```
    /// const_str! {
    ///     NAME1 "STRING",
    ///     NAME2 "STRING",
    /// }
    /// ```
    #[macro_export]
    macro_rules! const_str {
        (
            $(#[$outer:meta])*
            $name:ident, $content:expr
        ) => {
            $(#[$outer])*
            pub const $name: &str = $content;
        };
        (
            $(
                $(#[$outer:meta])*
                $name:ident $content:expr
            ),+ $(,)?
        ) => {
            $(const_str!{ $(#[$outer])* $name, $content })+
        }
    }

    const_str! {
        BACKSPACE "backspace",
        ENTER "enter",
        TAB "tab",
        BACKTAB "backtab",
        DELETE "delete",
        INSERT "insert",
        HOME "home",
        END "end",
        ESCAPE "escape",

        PAGEUP "pageup",
        PAGEDOWN "pagedown",

        ARROWUP "arrowup",
        ARROWDOWN "arrowdown",
        ARROWLEFT "arrowleft",
        ARROWRIGHT "arrowright",

        // special keys
        CAPSLOCK "capslock",
        SCROLLLOCK "scrolllock",
        NUMLOCK "numlock",
        PRINTSCREEN "printscreen",
        /// The "Pause/Break" key, commonly besides "PRINT" and "SCROLLLOCK"
        PAUSE "pause",

        // weird keys
        /// https://en.wikipedia.org/wiki/Null_character
        NULL "null",
        /// https://en.wikipedia.org/wiki/Menu_key
        MENU "menu",

        // aliases
        SPACE "space"
    }

    const_str! {
        CONTROL "control",
        ALT "alt",
        SHIFT "shift",
    }
}

/// This conversion expects the input to already be lowercased
impl TryFrom<&str> for KeyWrap {
    type Error = KeyWrapParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // simple alias for less code
        use tuievents::Key as TKey;
        if value.is_empty() {
            return Err(KeyWrapParseError::Empty);
        }

        if value.len() == 1 {
            // safe unwrap because we checked the length
            return Ok(Self(tuievents::Key::Char(value.chars().next().unwrap())));
        }

        // yes, this also matches F255
        if value.len() <= 4 {
            if let Some(val) = value.strip_prefix('f') {
                if let Ok(parsed) = val.parse::<u8>() {
                    // no number validation as tuirealm seems to not care
                    return Ok(Self(tuievents::Key::Function(parsed)));
                }
                // if parsing fails, just try the other keys, or report "UnknownKey"
            }
        }

        let ret = match value {
            const_keys::BACKSPACE => Self(TKey::Backspace),
            const_keys::ENTER => Self(TKey::Enter),
            const_keys::TAB => Self(TKey::Tab),
            const_keys::BACKTAB => Self(TKey::BackTab),
            const_keys::DELETE => Self(TKey::Delete),
            const_keys::INSERT => Self(TKey::Insert),
            const_keys::HOME => Self(TKey::Home),
            const_keys::END => Self(TKey::End),
            const_keys::ESCAPE => Self(TKey::Esc),

            const_keys::PAGEUP => Self(TKey::PageUp),
            const_keys::PAGEDOWN => Self(TKey::PageDown),

            const_keys::ARROWUP => Self(TKey::Up),
            const_keys::ARROWDOWN => Self(TKey::Down),
            const_keys::ARROWLEFT => Self(TKey::Left),
            const_keys::ARROWRIGHT => Self(TKey::Right),

            const_keys::CAPSLOCK => Self(TKey::CapsLock),
            const_keys::SCROLLLOCK => Self(TKey::ScrollLock),
            const_keys::NUMLOCK => Self(TKey::NumLock),
            const_keys::PRINTSCREEN => Self(TKey::PrintScreen),
            const_keys::PAUSE => Self(TKey::Pause),

            const_keys::NULL => Self(TKey::Null),
            const_keys::MENU => Self(TKey::Menu),

            // aliases
            const_keys::SPACE => Self(TKey::Char(' ')),

            v => return Err(KeyWrapParseError::UnknownKey(v.to_owned())),
        };

        Ok(ret)
    }
}

impl Display for KeyWrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            tuievents::Key::Backspace => const_keys::BACKSPACE.fmt(f),
            tuievents::Key::Enter => const_keys::ENTER.fmt(f),
            tuievents::Key::Tab => const_keys::TAB.fmt(f),
            tuievents::Key::BackTab => const_keys::BACKTAB.fmt(f),
            tuievents::Key::Delete => const_keys::DELETE.fmt(f),
            tuievents::Key::Insert => const_keys::INSERT.fmt(f),
            tuievents::Key::Home => const_keys::HOME.fmt(f),
            tuievents::Key::End => const_keys::END.fmt(f),
            tuievents::Key::Esc => const_keys::ESCAPE.fmt(f),

            tuievents::Key::PageUp => const_keys::PAGEUP.fmt(f),
            tuievents::Key::PageDown => const_keys::PAGEDOWN.fmt(f),

            tuievents::Key::Up => const_keys::ARROWUP.fmt(f),
            tuievents::Key::Down => const_keys::ARROWDOWN.fmt(f),
            tuievents::Key::Left => const_keys::ARROWLEFT.fmt(f),
            tuievents::Key::Right => const_keys::ARROWRIGHT.fmt(f),

            tuievents::Key::CapsLock => const_keys::CAPSLOCK.fmt(f),
            tuievents::Key::ScrollLock => const_keys::SCROLLLOCK.fmt(f),
            tuievents::Key::NumLock => const_keys::NUMLOCK.fmt(f),
            tuievents::Key::PrintScreen => const_keys::PRINTSCREEN.fmt(f),
            tuievents::Key::Pause => const_keys::PAUSE.fmt(f),

            tuievents::Key::Null => const_keys::NULL.fmt(f),
            tuievents::Key::Menu => const_keys::MENU.fmt(f),

            tuievents::Key::Function(v) => write!(f, "f{v}"),
            tuievents::Key::Char(v) => {
                if v == ' ' {
                    write!(f, "{}", const_keys::SPACE)
                } else {
                    v.fmt(f)
                }
            }

            // not supporting media keys as those are handled by the mpris implementation
            tuievents::Key::Media(_) => unimplemented!(),

            // i literally have no clue what key this is supposed to be
            tuievents::Key::KeypadBegin => unimplemented!(),

            // the following are new events with tuirealm 2.0, but only available in backend "termion", which we dont use
            tuievents::Key::ShiftLeft
            | tuievents::Key::AltLeft
            | tuievents::Key::CtrlLeft
            | tuievents::Key::ShiftRight
            | tuievents::Key::AltRight
            | tuievents::Key::CtrlRight
            | tuievents::Key::ShiftUp
            | tuievents::Key::AltUp
            | tuievents::Key::CtrlUp
            | tuievents::Key::ShiftDown
            | tuievents::Key::AltDown
            | tuievents::Key::CtrlDown
            | tuievents::Key::CtrlHome
            | tuievents::Key::CtrlEnd => unimplemented!(),
        }
    }
}

// convenience function to convert
impl From<tuievents::Key> for KeyWrap {
    fn from(value: tuievents::Key) -> Self {
        Self(value)
    }
}

/// All Key-Modifiers we support
///
/// It is defined here as we want a consistent config and be in control of it instead of some upstream package
#[derive(Debug, Clone, Copy /* , EnumString, IntoStaticStr */)]
enum SupportedModifiers {
    Control,
    Shift,
    Alt,
}

impl From<SupportedModifiers> for &'static str {
    fn from(value: SupportedModifiers) -> Self {
        match value {
            SupportedModifiers::Control => const_keys::CONTROL,
            SupportedModifiers::Shift => const_keys::SHIFT,
            SupportedModifiers::Alt => const_keys::ALT,
        }
    }
}

/// This conversion expects the input to already be lowercased
impl TryFrom<&str> for SupportedModifiers {
    type Error = KeyWrapParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(KeyWrapParseError::Empty);
        }

        let val = match value {
            const_keys::CONTROL => Self::Control,
            const_keys::ALT => Self::Alt,
            const_keys::SHIFT => Self::Shift,
            v => return Err(KeyWrapParseError::UnknownKey(v.to_owned())),
        };

        Ok(val)
    }
}

impl SupportedModifiers {
    /// Get a array of [`SupportedModifiers`] from the provided modifiers
    fn from_keymodifiers(modifiers: tuievents::KeyModifiers) -> Vec<Self> {
        let mut ret = Vec::with_capacity(3);

        if modifiers.contains(tuievents::KeyModifiers::CONTROL) {
            ret.push(Self::Control);
        }
        if modifiers.contains(tuievents::KeyModifiers::ALT) {
            ret.push(Self::Alt);
        }
        if modifiers.contains(tuievents::KeyModifiers::SHIFT) {
            ret.push(Self::Shift);
        }

        ret
    }
}

impl From<SupportedModifiers> for tuievents::KeyModifiers {
    fn from(value: SupportedModifiers) -> Self {
        match value {
            SupportedModifiers::Control => Self::CONTROL,
            SupportedModifiers::Shift => Self::SHIFT,
            SupportedModifiers::Alt => Self::ALT,
        }
    }
}

mod v1_interop {
    use super::{
        tuievents, KeyBinding, Keys, KeysConfigEditor, KeysDatabase, KeysLibrary, KeysLyric,
        KeysMoveCoverArt, KeysNavigation, KeysPlayer, KeysPlaylist, KeysPodcast, KeysSelectView,
    };
    use crate::config::v1;

    impl From<v1::BindingForEvent> for KeyBinding {
        fn from(value: v1::BindingForEvent) -> Self {
            let code = if let tuievents::Key::Char(char) = value.code {
                // transform the key to be upper-case if "Shift" is enabled, as that is what tuirealm will provide (and we cannot modify that)
                if value.modifier.intersects(tuievents::KeyModifiers::SHIFT) {
                    tuievents::Key::Char(char.to_ascii_uppercase())
                } else {
                    tuievents::Key::Char(char.to_ascii_lowercase())
                }
            } else {
                value.code
            };
            Self::from(tuievents::KeyEvent {
                code,
                modifiers: value.modifier,
            })
        }
    }

    impl From<v1::Keys> for Keys {
        #[allow(clippy::too_many_lines)]
        fn from(value: v1::Keys) -> Self {
            // extra case because the v1 defaults have conflicting keys
            let podcast_delete_feed_key =
                if value.podcast_episode_download == value.podcast_delete_feed {
                    KeysPodcast::default().delete_feed
                } else {
                    value.podcast_delete_feed.into()
                };
            let podcast_delete_delete_all_eq = match (
                value.podcast_delete_all_feeds.code,
                value.podcast_delete_feed.code,
            ) {
                // the old impl had lowercase and uppercase characters, need to compare them equally
                (tuievents::Key::Char(left), tuievents::Key::Char(right)) => {
                    left.eq_ignore_ascii_case(&right)
                }
                (left, right) => left == right,
            };
            let podcast_delete_all_feeds_key = if value.podcast_episode_download
                == value.podcast_delete_feed
                && podcast_delete_delete_all_eq
            {
                KeysPodcast::default().delete_all_feeds
            } else {
                value.podcast_delete_all_feeds.into()
            };
            // need to change it here too because the v1 default is "x", which had been changed to "d+shift" to be a upgrade over "download"
            let podcast_delete_episode_key = if podcast_delete_feed_key
                == value.podcast_episode_delete_file.key_event().into()
            {
                KeysPodcast::default().delete_local_episode
            } else {
                value.podcast_episode_delete_file.into()
            };

            // this only really applies to volume_down_1 had "_+SHIFT", but now volume_down_2 is actually used instead
            // fixup the old broken way where volume down 1 was by default set to "shift+_", which actually never fires and only fires "_"
            let player_volume_down_key = {
                let old = value.global_player_volume_minus_2;
                if old.code == tuievents::Key::Char('_')
                    && old.modifier.intersects(tuievents::KeyModifiers::SHIFT)
                {
                    KeyBinding::from(tuievents::KeyEvent::new(
                        tuievents::Key::Char('_'),
                        tuievents::KeyModifiers::NONE,
                    ))
                } else {
                    old.into()
                }
            };

            Self {
                escape: value.global_esc.into(),
                quit: value.global_quit.into(),
                select_view_keys: KeysSelectView {
                    view_library: value.global_layout_treeview.into(),
                    view_database: value.global_layout_database.into(),
                    view_podcasts: value.global_layout_podcast.into(),
                    open_config: value.global_config_open.into(),
                    open_help: value.global_help.into(),
                },
                navigation_keys: KeysNavigation {
                    up: value.global_up.into(),
                    down: value.global_down.into(),
                    left: value.global_left.into(),
                    right: value.global_right.into(),
                    goto_top: value.global_goto_top.into(),
                    goto_bottom: value.global_goto_bottom.into(),
                },
                player_keys: KeysPlayer {
                    toggle_pause: value.global_player_toggle_pause.into(),
                    next_track: value.global_player_next.into(),
                    previous_track: value.global_player_previous.into(),
                    volume_up: value.global_player_volume_plus_2.into(),
                    volume_down: player_volume_down_key,
                    seek_forward: value.global_player_seek_forward.into(),
                    seek_backward: value.global_player_seek_backward.into(),
                    speed_up: value.global_player_speed_up.into(),
                    speed_down: value.global_player_speed_down.into(),
                    toggle_prefetch: value.global_player_toggle_gapless.into(),
                    save_playlist: value.global_save_playlist.into(),
                },
                lyric_keys: KeysLyric {
                    adjust_offset_forwards: value.global_lyric_adjust_forward.into(),
                    adjust_offset_backwards: value.global_lyric_adjust_backward.into(),
                    cycle_frames: value.global_lyric_cycle.into(),
                },
                library_keys: KeysLibrary {
                    // this is weird, but the previous implementation used "global_right" as the loading key to not conflict
                    load_track: value.global_right.into(),
                    load_dir: value.library_load_dir.into(),
                    delete: value.library_delete.into(),
                    yank: value.library_yank.into(),
                    paste: value.library_paste.into(),
                    cycle_root: value.library_switch_root.into(),
                    add_root: value.library_add_root.into(),
                    remove_root: value.library_remove_root.into(),
                    search: value.library_search.into(),
                    youtube_search: value.library_search_youtube.into(),
                    open_tag_editor: value.library_tag_editor_open.into(),
                },
                playlist_keys: KeysPlaylist {
                    delete: value.playlist_delete.into(),
                    delete_all: value.playlist_delete_all.into(),
                    shuffle: value.playlist_shuffle.into(),
                    cycle_loop_mode: value.playlist_mode_cycle.into(),
                    play_selected: value.playlist_play_selected.into(),
                    search: value.playlist_search.into(),
                    swap_up: value.playlist_swap_up.into(),
                    swap_down: value.playlist_swap_down.into(),
                    add_random_songs: value.playlist_add_random_tracks.into(),
                    add_random_album: value.playlist_add_random_album.into(),
                },
                database_keys: KeysDatabase {
                    // this is weird, but the previous implementation used "global_right" as the loading key to not conflict
                    add_selected: value.global_right.into(),
                    add_all: value.database_add_all.into(),
                },
                podcast_keys: KeysPodcast {
                    search: value.podcast_search_add_feed.into(),
                    mark_played: value.podcast_mark_played.into(),
                    mark_all_played: value.podcast_mark_all_played.into(),
                    refresh_feed: value.podcast_refresh_feed.into(),
                    refresh_all_feeds: value.podcast_refresh_all_feeds.into(),
                    download_episode: value.podcast_episode_download.into(),
                    delete_local_episode: podcast_delete_episode_key,
                    delete_feed: podcast_delete_feed_key,
                    delete_all_feeds: podcast_delete_all_feeds_key,
                },
                move_cover_art_keys: KeysMoveCoverArt {
                    move_left: value.global_xywh_move_left.into(),
                    move_right: value.global_xywh_move_right.into(),
                    move_up: value.global_xywh_move_up.into(),
                    move_down: value.global_xywh_move_down.into(),
                    increase_size: value.global_xywh_zoom_in.into(),
                    decrease_size: value.global_xywh_zoom_out.into(),
                    toggle_hide: value.global_xywh_hide.into(),
                },
                config_keys: KeysConfigEditor {
                    save: value.config_save.into(),
                },
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use pretty_assertions::assert_eq;
        use v1::BindingForEvent;

        #[allow(clippy::too_many_lines)] // this test just requires a lot of fields
        #[test]
        fn should_convert_default_without_error() {
            let converted: Keys = v1::Keys::default().into();

            // this is all checked by themself (and then fully) so that if there is a error, you actually get a better error than a bunch of long text
            let expected_select_view_keys = KeysSelectView {
                view_library: tuievents::Key::Char('1').into(),
                view_database: tuievents::Key::Char('2').into(),
                view_podcasts: tuievents::Key::Char('3').into(),
                open_config: tuievents::KeyEvent::new(
                    tuievents::Key::Char('C'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                open_help: tuievents::KeyEvent::new(
                    tuievents::Key::Char('h'),
                    tuievents::KeyModifiers::CONTROL,
                )
                .into(),
            };
            assert_eq!(converted.select_view_keys, expected_select_view_keys);

            let expected_navigation_keys = KeysNavigation {
                up: tuievents::Key::Char('k').into(),
                down: tuievents::Key::Char('j').into(),
                left: tuievents::Key::Char('h').into(),
                right: tuievents::Key::Char('l').into(),
                goto_top: tuievents::Key::Char('g').into(),
                goto_bottom: tuievents::KeyEvent::new(
                    tuievents::Key::Char('G'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
            };
            assert_eq!(converted.navigation_keys, expected_navigation_keys);

            let expected_player_keys = KeysPlayer {
                toggle_pause: tuievents::Key::Char(' ').into(),
                next_track: tuievents::Key::Char('n').into(),
                previous_track: tuievents::KeyEvent::new(
                    tuievents::Key::Char('N'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                // volume_up and volume_down have different default key-bindings in v2
                volume_up: tuievents::KeyEvent::new(
                    tuievents::Key::Char('='),
                    tuievents::KeyModifiers::NONE,
                )
                .into(),
                volume_down: tuievents::KeyEvent::new(
                    tuievents::Key::Char('_'),
                    tuievents::KeyModifiers::NONE,
                )
                .into(),
                seek_forward: tuievents::Key::Char('f').into(),
                seek_backward: tuievents::Key::Char('b').into(),
                speed_up: tuievents::KeyEvent::new(
                    tuievents::Key::Char('f'),
                    tuievents::KeyModifiers::CONTROL,
                )
                .into(),
                speed_down: tuievents::KeyEvent::new(
                    tuievents::Key::Char('b'),
                    tuievents::KeyModifiers::CONTROL,
                )
                .into(),
                toggle_prefetch: tuievents::KeyEvent::new(
                    tuievents::Key::Char('g'),
                    tuievents::KeyModifiers::CONTROL,
                )
                .into(),
                save_playlist: tuievents::KeyEvent::new(
                    tuievents::Key::Char('s'),
                    tuievents::KeyModifiers::CONTROL,
                )
                .into(),
            };
            assert_eq!(converted.player_keys, expected_player_keys);

            let expected_lyric_keys = KeysLyric {
                adjust_offset_forwards: tuievents::KeyEvent::new(
                    tuievents::Key::Char('F'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                adjust_offset_backwards: tuievents::KeyEvent::new(
                    tuievents::Key::Char('B'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                cycle_frames: tuievents::KeyEvent::new(
                    tuievents::Key::Char('T'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
            };
            assert_eq!(converted.lyric_keys, expected_lyric_keys);

            let expected_library_keys = KeysLibrary {
                load_track: tuievents::Key::Char('l').into(),
                load_dir: tuievents::KeyEvent::new(
                    tuievents::Key::Char('L'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                delete: tuievents::Key::Char('d').into(),
                yank: tuievents::Key::Char('y').into(),
                paste: tuievents::Key::Char('p').into(),
                cycle_root: tuievents::Key::Char('o').into(),
                add_root: tuievents::Key::Char('a').into(),
                remove_root: tuievents::KeyEvent::new(
                    tuievents::Key::Char('A'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                search: tuievents::Key::Char('/').into(),
                youtube_search: tuievents::Key::Char('s').into(),
                open_tag_editor: tuievents::Key::Char('t').into(),
            };
            assert_eq!(converted.library_keys, expected_library_keys);

            let expected_playlist_keys = KeysPlaylist {
                delete: tuievents::Key::Char('d').into(),
                delete_all: tuievents::KeyEvent::new(
                    tuievents::Key::Char('D'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                shuffle: tuievents::Key::Char('r').into(),
                cycle_loop_mode: tuievents::Key::Char('m').into(),
                play_selected: tuievents::Key::Char('l').into(),
                search: tuievents::Key::Char('/').into(),
                swap_up: tuievents::KeyEvent::new(
                    tuievents::Key::Char('K'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                swap_down: tuievents::KeyEvent::new(
                    tuievents::Key::Char('J'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                add_random_songs: tuievents::Key::Char('s').into(),
                add_random_album: tuievents::KeyEvent::new(
                    tuievents::Key::Char('S'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
            };
            assert_eq!(converted.playlist_keys, expected_playlist_keys);

            let expected_database_keys = KeysDatabase {
                add_selected: tuievents::Key::Char('l').into(),
                add_all: tuievents::KeyEvent::new(
                    tuievents::Key::Char('L'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
            };
            assert_eq!(converted.database_keys, expected_database_keys);

            let expected_podcast_keys = KeysPodcast {
                search: tuievents::Key::Char('s').into(),
                mark_played: tuievents::Key::Char('m').into(),
                mark_all_played: tuievents::KeyEvent::new(
                    tuievents::Key::Char('M'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                refresh_feed: tuievents::Key::Char('r').into(),
                refresh_all_feeds: tuievents::KeyEvent::new(
                    tuievents::Key::Char('R'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                download_episode: tuievents::Key::Char('d').into(),
                delete_local_episode: tuievents::KeyEvent::new(
                    tuievents::Key::Char('D'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                delete_feed: tuievents::Key::Char('x').into(),
                delete_all_feeds: tuievents::KeyEvent::new(
                    tuievents::Key::Char('X'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
            };
            assert_eq!(converted.podcast_keys, expected_podcast_keys);

            let expected_move_cover_art_keys = KeysMoveCoverArt {
                move_left: tuievents::KeyEvent::new(
                    tuievents::Key::Left,
                    tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                move_right: tuievents::KeyEvent::new(
                    tuievents::Key::Right,
                    tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                move_up: tuievents::KeyEvent::new(
                    tuievents::Key::Up,
                    tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                move_down: tuievents::KeyEvent::new(
                    tuievents::Key::Down,
                    tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                increase_size: tuievents::KeyEvent::new(
                    tuievents::Key::PageUp,
                    tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                decrease_size: tuievents::KeyEvent::new(
                    tuievents::Key::PageDown,
                    tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                toggle_hide: tuievents::KeyEvent::new(
                    tuievents::Key::End,
                    tuievents::KeyModifiers::CONTROL | tuievents::KeyModifiers::SHIFT,
                )
                .into(),
            };
            assert_eq!(converted.move_cover_art_keys, expected_move_cover_art_keys);

            let expected_config_editor_keys = KeysConfigEditor {
                save: tuievents::KeyEvent::new(
                    tuievents::Key::Char('s'),
                    tuievents::KeyModifiers::CONTROL,
                )
                .into(),
            };
            assert_eq!(converted.config_keys, expected_config_editor_keys);

            let expected_keys = Keys {
                escape: tuievents::Key::Esc.into(),
                quit: tuievents::Key::Char('q').into(),
                select_view_keys: expected_select_view_keys,
                navigation_keys: expected_navigation_keys,
                player_keys: expected_player_keys,
                lyric_keys: expected_lyric_keys,
                library_keys: expected_library_keys,
                playlist_keys: expected_playlist_keys,
                database_keys: expected_database_keys,
                podcast_keys: expected_podcast_keys,
                move_cover_art_keys: expected_move_cover_art_keys,
                config_keys: expected_config_editor_keys,
            };

            assert_eq!(converted, expected_keys);

            assert_eq!(Ok(()), expected_keys.check_keys());
        }

        #[test]
        fn should_fixup_old_volume_default() {
            let converted: Keys = {
                let v1 = v1::Keys {
                    global_player_volume_minus_2: BindingForEvent {
                        code: tuievents::Key::Char('_'),
                        modifier: tuievents::KeyModifiers::SHIFT,
                    },
                    ..v1::Keys::default()
                };

                v1.into()
            };

            let expected_player_keys = KeysPlayer {
                toggle_pause: tuievents::Key::Char(' ').into(),
                next_track: tuievents::Key::Char('n').into(),
                previous_track: tuievents::KeyEvent::new(
                    tuievents::Key::Char('N'),
                    tuievents::KeyModifiers::SHIFT,
                )
                .into(),
                // volume_up and volume_down have different default key-bindings in v2
                volume_up: tuievents::KeyEvent::new(
                    tuievents::Key::Char('='),
                    tuievents::KeyModifiers::NONE,
                )
                .into(),
                volume_down: tuievents::KeyEvent::new(
                    tuievents::Key::Char('_'),
                    tuievents::KeyModifiers::NONE,
                )
                .into(),
                seek_forward: tuievents::Key::Char('f').into(),
                seek_backward: tuievents::Key::Char('b').into(),
                speed_up: tuievents::KeyEvent::new(
                    tuievents::Key::Char('f'),
                    tuievents::KeyModifiers::CONTROL,
                )
                .into(),
                speed_down: tuievents::KeyEvent::new(
                    tuievents::Key::Char('b'),
                    tuievents::KeyModifiers::CONTROL,
                )
                .into(),
                toggle_prefetch: tuievents::KeyEvent::new(
                    tuievents::Key::Char('g'),
                    tuievents::KeyModifiers::CONTROL,
                )
                .into(),
                save_playlist: tuievents::KeyEvent::new(
                    tuievents::Key::Char('s'),
                    tuievents::KeyModifiers::CONTROL,
                )
                .into(),
            };
            assert_eq!(converted.player_keys, expected_player_keys);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod split_at_plus {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn should_do_nothing_at_empty() {
            assert_eq!(
                Vec::<&str>::new(),
                SplitAtPlus::new("").collect::<Vec<&str>>()
            );
        }

        #[test]
        fn should_treat_one_as_key() {
            assert_eq!(vec!["+"], SplitAtPlus::new("+").collect::<Vec<&str>>());
        }

        #[test]
        fn should_parse_with_non_delim_last() {
            assert_eq!(
                vec!["+", "control"],
                SplitAtPlus::new("++control").collect::<Vec<&str>>()
            );
        }

        #[test]
        fn should_parse_with_non_delim_first() {
            assert_eq!(
                vec!["control", "+"],
                SplitAtPlus::new("control++").collect::<Vec<&str>>()
            );
        }

        #[test]
        fn should_parse_with_multiple_with_delim() {
            assert_eq!(
                vec!["+", "+"],
                SplitAtPlus::new("+++").collect::<Vec<&str>>()
            );
        }

        #[test]
        fn should_parse_with_only_delim() {
            assert_eq!(
                vec!["q", "control"],
                SplitAtPlus::new("q+control").collect::<Vec<&str>>()
            );
        }

        #[test]
        fn should_treat_without_delim() {
            assert_eq!(
                vec!["control"],
                SplitAtPlus::new("control").collect::<Vec<&str>>()
            );
        }

        #[test]
        fn should_return_trailing_empty_string_on_delim_last() {
            assert_eq!(vec!["+", ""], SplitAtPlus::new("++").collect::<Vec<&str>>());
            assert_eq!(
                vec!["control", ""],
                SplitAtPlus::new("control+").collect::<Vec<&str>>()
            );
        }

        #[test]
        fn should_parse_non_delim_delim_non_delim() {
            assert_eq!(
                vec!["control", "+", "shift"],
                SplitAtPlus::new("control+++shift").collect::<Vec<&str>>()
            );
        }

        #[test]
        fn should_treat_delim_followed_by_key_as_trailing() {
            assert_eq!(vec!["", "q"], SplitAtPlus::new("+q").collect::<Vec<&str>>());
        }
    }

    mod key_wrap {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn should_parse_function_keys() {
            assert_eq!(
                KeyWrap(tuievents::Key::Function(10)),
                KeyWrap::try_from("f10").unwrap()
            );
            assert_eq!(
                KeyWrap(tuievents::Key::Function(0)),
                KeyWrap::try_from("f0").unwrap()
            );
            assert_eq!(
                KeyWrap(tuievents::Key::Function(255)),
                KeyWrap::try_from("f255").unwrap()
            );
        }

        #[test]
        fn should_parse_char() {
            assert_eq!(
                KeyWrap(tuievents::Key::Char('q')),
                KeyWrap::try_from("q").unwrap()
            );
            assert_eq!(
                KeyWrap(tuievents::Key::Char('w')),
                KeyWrap::try_from("w").unwrap()
            );
            assert_eq!(
                KeyWrap(tuievents::Key::Char('.')),
                KeyWrap::try_from(".").unwrap()
            );
            assert_eq!(
                KeyWrap(tuievents::Key::Char('@')),
                KeyWrap::try_from("@").unwrap()
            );

            // space alias
            assert_eq!(
                KeyWrap(tuievents::Key::Char(' ')),
                KeyWrap::try_from("space").unwrap()
            );
        }

        #[test]
        fn should_serialize_function_keys() {
            assert_eq!(&"f10", &KeyWrap(tuievents::Key::Function(10)).to_string());
            assert_eq!(&"f0", &KeyWrap(tuievents::Key::Function(0)).to_string());
            assert_eq!(&"f255", &KeyWrap(tuievents::Key::Function(255)).to_string());
        }

        #[test]
        fn should_serialize_char() {
            assert_eq!(&"q", &KeyWrap(tuievents::Key::Char('q')).to_string());
            assert_eq!(&"w", &KeyWrap(tuievents::Key::Char('w')).to_string());
            assert_eq!(&".", &KeyWrap(tuievents::Key::Char('.')).to_string());
            assert_eq!(&"@", &KeyWrap(tuievents::Key::Char('@')).to_string());

            // space
            assert_eq!(&"space", &KeyWrap(tuievents::Key::Char(' ')).to_string());
        }
    }

    mod key_binding {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn should_parse_keys_simple() {
            // all modifiers
            assert_eq!(
                KeyBinding::from(tuievents::KeyEvent::new(
                    tuievents::Key::Char('Q'),
                    tuievents::KeyModifiers::all()
                )),
                KeyBinding::try_from("CONTROL+ALT+SHIFT+Q").unwrap()
            );

            // no modifiers
            assert_eq!(
                KeyBinding::from(tuievents::KeyEvent::new(
                    tuievents::Key::Char('q'),
                    tuievents::KeyModifiers::empty()
                )),
                KeyBinding::try_from("Q").unwrap()
            );

            // multiple of the same modifier
            assert_eq!(
                KeyBinding::from(tuievents::KeyEvent::new(
                    tuievents::Key::Char('q'),
                    tuievents::KeyModifiers::CONTROL
                )),
                KeyBinding::try_from("CONTROL+CONTROL+CONTROL+Q").unwrap()
            );
        }

        #[test]
        fn should_error_on_multiple_keys() {
            assert_eq!(
                Err(KeyParseError::MultipleKeys(
                    "q+s".to_owned(),
                    ("q".to_owned(), "s".to_string())
                )),
                KeyBinding::try_from("Q+S")
            );
        }

        #[test]
        fn should_serialize() {
            // all modifiers
            assert_eq!(
                "control+alt+shift+q",
                KeyBinding::from(tuievents::KeyEvent::new(
                    tuievents::Key::Char('q'),
                    tuievents::KeyModifiers::all()
                ))
                .to_string()
            );

            // only control
            assert_eq!(
                "control+q",
                KeyBinding::from(tuievents::KeyEvent::new(
                    tuievents::Key::Char('q'),
                    tuievents::KeyModifiers::CONTROL
                ))
                .to_string()
            );

            // only alt
            assert_eq!(
                "alt+q",
                KeyBinding::from(tuievents::KeyEvent::new(
                    tuievents::Key::Char('q'),
                    tuievents::KeyModifiers::ALT
                ))
                .to_string()
            );

            // only shift
            assert_eq!(
                "shift+q",
                KeyBinding::from(tuievents::KeyEvent::new(
                    tuievents::Key::Char('q'),
                    tuievents::KeyModifiers::SHIFT
                ))
                .to_string()
            );

            // no modifiers
            assert_eq!(
                "q",
                KeyBinding::from(tuievents::KeyEvent::new(
                    tuievents::Key::Char('q'),
                    tuievents::KeyModifiers::empty()
                ))
                .to_string()
            );
        }

        #[test]
        fn should_allow_special_keys() {
            // we currently split with a delimiter of "+", but it should still be available
            assert_eq!(
                KeyBinding::from(tuievents::KeyEvent::new(
                    tuievents::Key::Char('+'),
                    tuievents::KeyModifiers::empty()
                )),
                KeyBinding::try_from("+").unwrap()
            );

            // just some extra tests
            assert_eq!(
                KeyBinding::from(tuievents::KeyEvent::new(
                    tuievents::Key::Char('-'),
                    tuievents::KeyModifiers::empty()
                )),
                KeyBinding::try_from("-").unwrap()
            );

            assert_eq!(
                KeyBinding::from(tuievents::KeyEvent::new(
                    tuievents::Key::Char(' '),
                    tuievents::KeyModifiers::empty()
                )),
                KeyBinding::try_from(" ").unwrap()
            );
        }

        #[test]
        fn should_not_allow_invalid_formats() {
            // empty string
            assert_eq!(
                Err(KeyParseError::NoKeyFound(String::new())),
                KeyBinding::try_from("")
            );

            // multiple spaces
            assert_eq!(
                Err(KeyParseError::UnknownKey("   ".to_owned())),
                KeyBinding::try_from("   ")
            );

            // this could either mean key "+" plus invalid, or invalid plus "+" key
            assert_eq!(
                Err(KeyParseError::TrailingDelimiter("++".to_owned())),
                KeyBinding::try_from("++")
            );

            // trailing delimiter
            assert_eq!(
                Err(KeyParseError::TrailingDelimiter("control+".to_owned())),
                KeyBinding::try_from("control+")
            );

            // first trailing delimiter
            assert_eq!(
                Err(KeyParseError::TrailingDelimiter("+control".to_owned())),
                KeyBinding::try_from("+control")
            );
        }
    }

    mod keys {
        use figment::{
            providers::{Format, Toml},
            Figment,
        };
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn should_parse_default_keys() {
            let serialized = toml::to_string(&Keys::default()).unwrap();

            let parsed: Keys = Figment::new()
                .merge(Toml::string(&serialized))
                .extract()
                .unwrap();

            assert_eq!(Keys::default(), parsed);
        }

        #[test]
        fn should_not_conflict_on_default() {
            assert_eq!(Ok(()), Keys::default().check_keys());
        }

        #[test]
        fn should_not_conflict_on_different_view() {
            // check that views that would not conflict do not conflict
            let mut keys = Keys::default();
            keys.library_keys.delete = tuievents::Key::Delete.into();
            keys.podcast_keys.delete_feed = tuievents::Key::Delete.into();

            assert_eq!(Ok(()), keys.check_keys());
        }

        #[test]
        fn should_err_on_global_key_conflict() {
            // check that views that would not conflict do not conflict
            let mut keys = Keys::default();
            keys.select_view_keys.view_podcasts = tuievents::Key::Delete.into();
            keys.podcast_keys.delete_feed = tuievents::Key::Delete.into();

            assert_eq!(
                Err(KeysCheckError {
                    errored_keys: vec![KeyConflictError {
                        key_path_first: "keys.view.view_podcasts".into(),
                        key_path_second: "keys.podcast.delete_feed".into(),
                        key: tuievents::Key::Delete.into()
                    }]
                }),
                keys.check_keys()
            );
        }
    }
}
