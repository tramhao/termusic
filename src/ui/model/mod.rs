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

#[cfg(feature = "discord")]
use crate::discord::Rpc;
#[cfg(feature = "mpris")]
mod mpris;
mod update;
mod view;
mod youtube_options;
use crate::sqlite::{DataBase, SearchCriteria};
#[cfg(feature = "cover")]
use crate::ueberzug::UeInstance;
use crate::{
    config::Termusic,
    track::Track,
    ui::{Application, Id, Msg},
};

use crate::config::{Keys, StyleColorSymbol};
// use crate::player::{GeneralP, GeneralPl};
use crate::player::GeneralPl;
use crate::songtag::SongTag;
use crate::sqlite::TrackForDB;
use crate::ui::{SearchLyricState, Status};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};
use tui_realm_treeview::Tree;
use tuirealm::event::NoUserEvent;
use tuirealm::terminal::TerminalBridge;
use youtube_options::YoutubeOptions;

pub const MAX_DEPTH: usize = 4;

#[derive(PartialEq)]
pub enum TermusicLayout {
    TreeView,
    DataBase,
}

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
    pub config: Termusic,
    pub player: GeneralPl,
    pub yanked_node_id: Option<String>,
    pub current_song: Option<Track>,
    pub tageditor_song: Option<Track>,
    pub time_pos: i64,
    pub lyric_line: String,
    youtube_options: YoutubeOptions,
    pub sender: Sender<UpdateComponents>,
    receiver: Receiver<UpdateComponents>,
    #[cfg(feature = "cover")]
    pub ueberzug_instance: UeInstance,
    pub songtag_options: Vec<SongTag>,
    pub sender_songtag: Sender<SearchLyricState>,
    pub receiver_songtag: Receiver<SearchLyricState>,
    pub viuer_supported: ViuerSupported,
    pub ce_themes: Vec<String>,
    pub ce_style_color_symbol: StyleColorSymbol,
    pub ke_key_config: Keys,
    #[cfg(feature = "mpris")]
    pub mpris: mpris::Mpris,
    #[cfg(feature = "discord")]
    pub discord: Rpc,
    pub db: DataBase,
    pub layout: TermusicLayout,
    pub db_criteria: SearchCriteria,
    pub db_search_results: Vec<String>,
    pub db_search_tracks: Vec<TrackForDB>,
}

pub enum ViuerSupported {
    Kitty,
    ITerm,
    NotSupported,
}

impl Model {
    pub fn new(config: &Termusic) -> Self {
        let path = Self::get_full_path_from_config(config);
        let tree = Tree::new(Self::library_dir_tree(&path, MAX_DEPTH));

        let (tx, rx): (Sender<UpdateComponents>, Receiver<UpdateComponents>) = mpsc::channel();
        let (tx3, rx3): (Sender<SearchLyricState>, Receiver<SearchLyricState>) = mpsc::channel();

        let mut viuer_supported = ViuerSupported::NotSupported;
        if viuer::KittySupport::None != viuer::get_kitty_support() {
            viuer_supported = ViuerSupported::Kitty;
        } else if viuer::is_iterm_supported() {
            viuer_supported = ViuerSupported::ITerm;
        }
        let mut db = DataBase::new(config);
        db.sync_database();
        let db_criteria = SearchCriteria::Artist;
        // let viuer_supported =
        //     viuer::KittySupport::None != viuer::get_kitty_support() || viuer::is_iterm_supported();
        Self {
            app: Self::init_app(&tree, config),
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
            tree,
            path,
            terminal: TerminalBridge::new().expect("Could not initialize terminal"),
            config: config.clone(),
            player: GeneralPl::new(config),
            yanked_node_id: None,
            current_song: None,
            tageditor_song: None,
            time_pos: 0,
            lyric_line: String::new(),
            youtube_options: YoutubeOptions::new(),
            sender: tx,
            receiver: rx,
            #[cfg(feature = "cover")]
            ueberzug_instance: UeInstance::default(),
            songtag_options: vec![],
            sender_songtag: tx3,
            receiver_songtag: rx3,
            viuer_supported,
            ce_themes: vec![],
            ce_style_color_symbol: StyleColorSymbol::default(),
            ke_key_config: Keys::default(),
            #[cfg(feature = "mpris")]
            mpris: mpris::Mpris::default(),
            #[cfg(feature = "discord")]
            discord: Rpc::default(),
            db,
            layout: TermusicLayout::TreeView,
            db_criteria,
            db_search_results: Vec::new(),
            db_search_tracks: Vec::new(),
        }
    }

    pub fn get_full_path_from_config(config: &Termusic) -> PathBuf {
        let mut full_path = shellexpand::tilde(&config.music_dir);
        if let Some(music_dir) = &config.music_dir_from_cli {
            full_path = shellexpand::tilde(music_dir);
        };
        PathBuf::from(full_path.to_string())
    }

    pub fn init_config(&mut self) {
        if let Err(e) = self.theme_select_load_themes() {
            self.mount_error_popup(format!("Error load themes: {}", e).as_str());
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

    pub fn run(&mut self) {
        match self.player.status {
            Status::Stopped => {
                self.player_next(false);
                // self.player.start_play();
            }
            Status::Running | Status::Paused => {}
        }
    }
}
