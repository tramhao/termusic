//! ## Model
//!
//! app model
/**
 * MIT License
 *
 * termusic - Copyright (C) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
#[cfg(feature = "mpris")]
mod mpris;
mod ueberzug;
mod update;
mod view;
mod youtube_options;
use crate::{
    config::Termusic,
    song::Song,
    ui::{Application, Id, Msg},
};

use crate::player::GStreamer;
use crate::songtag::SongTag;
use crate::ui::{SearchLyricState, Status};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
#[cfg(feature = "cover")]
use std::process::Child;
use std::sync::mpsc::{self, Receiver, Sender};
#[cfg(feature = "cover")]
use std::sync::RwLock;
use std::time::{Duration, Instant};
use tui_realm_treeview::Tree;
use tuirealm::terminal::TerminalBridge;
use tuirealm::NoUserEvent;
use youtube_options::YoutubeOptions;

pub const MAX_DEPTH: usize = 3;

// TransferState is used to describe the status of download
pub enum UpdateComponents {
    DownloadRunning, // indicates progress
    DownloadSuccess,
    DownloadCompleted(Option<String>),
    DownloadErrDownload(String),
    DownloadErrEmbedData,
    MessageShow((String, String)),
    MessageHide((String, String)),
    YoutubeSearchSuccess(YoutubeOptions),
    YoutubeSearchFail(String),
}

pub struct Model {
    /// Indicates that the application must quit
    pub quit: bool,
    /// Tells whether to redraw interface
    pub redraw: bool,
    last_redraw: Instant,
    pub app: Application<Id, Msg, NoUserEvent>,
    /// Used to draw to terminal
    pub terminal: TerminalBridge,
    pub path: PathBuf,
    pub tree: Tree,
    pub playlist_items: VecDeque<Song>,
    pub config: Termusic,
    pub player: GStreamer,
    pub status: Option<Status>,
    pub yanked_node_id: Option<String>,
    pub current_song: Option<Song>,
    pub tageditor_song: Option<Song>,
    pub time_pos: u64,
    pub lyric_line: String,
    youtube_options: YoutubeOptions,
    pub sender: Sender<UpdateComponents>,
    receiver: Receiver<UpdateComponents>,
    pub sender_playlist_items: Sender<VecDeque<Song>>,
    receiver_playlist_items: Receiver<VecDeque<Song>>,
    #[cfg(feature = "cover")]
    ueberzug: RwLock<Option<Child>>,
    pub songtag_options: Vec<SongTag>,
    pub sender_songtag: Sender<SearchLyricState>,
    pub receiver_songtag: Receiver<SearchLyricState>,
}

impl Model {
    pub fn new(config: &Termusic) -> Self {
        let full_path = shellexpand::tilde(&config.music_dir);
        let p: &Path = Path::new(full_path.as_ref());
        let tree = Tree::new(Self::library_dir_tree(p, MAX_DEPTH));

        let mut player = GStreamer::new();
        player.set_volume(config.volume);
        let (tx, rx): (Sender<UpdateComponents>, Receiver<UpdateComponents>) = mpsc::channel();
        let (tx2, rx2): (Sender<VecDeque<Song>>, Receiver<VecDeque<Song>>) = mpsc::channel();
        let (tx3, rx3): (Sender<SearchLyricState>, Receiver<SearchLyricState>) = mpsc::channel();
        Self {
            app: Self::init_app(&tree),
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
            tree,
            path: p.to_path_buf(),
            terminal: TerminalBridge::new().expect("Could not initialize terminal"),
            playlist_items: VecDeque::with_capacity(100),
            config: config.clone(),
            player,
            yanked_node_id: None,
            status: None,
            current_song: None,
            tageditor_song: None,
            time_pos: 0,
            lyric_line: String::new(),
            youtube_options: YoutubeOptions::new(),
            sender: tx,
            receiver: rx,
            sender_playlist_items: tx2,
            receiver_playlist_items: rx2,
            #[cfg(feature = "cover")]
            ueberzug: RwLock::new(None),
            songtag_options: vec![],
            sender_songtag: tx3,
            receiver_songtag: rx3,
        }
    }

    /// Initialize terminal
    pub fn init_terminal(&mut self) {
        let _ = self.terminal.enable_raw_mode();
        let _ = self.terminal.enter_alternate_screen();
        let _ = self.terminal.clear_screen();
    }

    /// Finalize terminal
    pub fn finalize_terminal(&mut self) {
        let _ = self.terminal.disable_raw_mode();
        let _ = self.terminal.leave_alternate_screen();
        let _ = self.terminal.clear_screen();
    }
    /// Returns elapsed time since last redraw
    pub fn since_last_redraw(&self) -> Duration {
        self.last_redraw.elapsed()
    }
    pub fn force_redraw(&mut self) {
        self.redraw = true;
    }
}
