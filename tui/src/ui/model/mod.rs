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
mod update;
mod view;
mod youtube_options;
use crate::ui::Application;
use termusiclib::sqlite::{DataBase, SearchCriteria};
use termusiclib::types::{Id, Msg, SearchLyricState, YoutubeOptions};

#[cfg(all(feature = "cover-uberzug", not(target_os = "windows")))]
use termusiclib::ueberzug::UeInstance;
use termusiclib::{
    config::Settings,
    track::{MediaType, Track},
};

use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};
use termusiclib::config::{Keys, StyleColorSymbol};
use termusiclib::podcast::{db::Database as DBPod, Podcast, PodcastFeed, TaskPool};
use termusiclib::songtag::SongTag;
use termusiclib::sqlite::TrackForDB;
use termusiclib::utils::{get_app_config_path, DownloadTracker};
use termusicplayback::{PlayerCmd, Playlist, SharedSettings};
use tokio::sync::mpsc::UnboundedSender;
use tui_realm_treeview::Tree;
use tuirealm::event::NoUserEvent;
use tuirealm::terminal::TerminalBridge;

#[derive(PartialEq, Eq)]
pub enum TermusicLayout {
    TreeView,
    DataBase,
    Podcast,
}

#[derive(PartialEq, Clone, Eq)]
pub enum ConfigEditorLayout {
    General,
    Color,
    Key1,
    Key2,
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
    pub config: SharedSettings,
    pub yanked_node_id: Option<String>,
    pub current_song: Option<Track>,
    pub tageditor_song: Option<Track>,
    pub time_pos: Duration,
    pub lyric_line: String,
    youtube_options: YoutubeOptions,
    #[cfg(all(feature = "cover-uberzug", not(target_os = "windows")))]
    pub ueberzug_instance: UeInstance,
    pub songtag_options: Vec<SongTag>,
    pub sender_songtag: Sender<SearchLyricState>,
    pub receiver_songtag: Receiver<SearchLyricState>,
    pub viuer_supported: ViuerSupported,
    pub ce_themes: Vec<String>,
    pub ce_style_color_symbol: StyleColorSymbol,
    pub ke_key_config: Keys,
    pub db: DataBase,
    pub db_criteria: SearchCriteria,
    pub db_search_results: Vec<String>,
    pub db_search_tracks: Vec<TrackForDB>,
    pub layout: TermusicLayout,
    pub config_layout: ConfigEditorLayout,
    pub config_changed: bool,
    pub download_tracker: DownloadTracker,
    pub podcasts: Vec<Podcast>,
    pub podcasts_index: usize,
    pub db_podcast: DBPod,
    pub threadpool: TaskPool,
    pub tx_to_main: Sender<Msg>,
    pub rx_to_main: Receiver<Msg>,
    pub podcast_search_vec: Option<Vec<PodcastFeed>>,
    pub playlist: Playlist,
    pub cmd_tx: UnboundedSender<PlayerCmd>,
}

#[derive(Debug)]
pub enum ViuerSupported {
    Kitty,
    ITerm,
    Sixel,
    NotSupported,
}

impl Model {
    pub async fn new(config: Settings, cmd_tx: UnboundedSender<PlayerCmd>) -> Self {
        let path = Self::get_full_path_from_config(&config);
        let tree = Tree::new(Self::library_dir_tree(&path, config.max_depth_cli));

        let (tx3, rx3): (Sender<SearchLyricState>, Receiver<SearchLyricState>) = mpsc::channel();

        let mut viuer_supported = ViuerSupported::NotSupported;
        if viuer::KittySupport::None != viuer::get_kitty_support() {
            viuer_supported = ViuerSupported::Kitty;
        } else if viuer::is_iterm_supported() {
            viuer_supported = ViuerSupported::ITerm;
        } else if viuer::is_sixel_supported() {
            viuer_supported = ViuerSupported::Sixel;
        }
        let db = DataBase::new(&config);
        let db_criteria = SearchCriteria::Artist;
        let terminal = TerminalBridge::new().expect("Could not initialize terminal");
        // let viuer_supported =
        //     viuer::KittySupport::None != viuer::get_kitty_support() || viuer::is_iterm_supported();

        #[cfg(all(feature = "cover-uberzug", not(target_os = "windows")))]
        let ueberzug_instance = UeInstance::default();
        let db_path = get_app_config_path().expect("failed to get podcast db path.");

        let db_podcast = DBPod::connect(&db_path).expect("error connecting to podcast db.");

        let podcasts = db_podcast
            .get_podcasts()
            .expect("failed to get podcasts from db.");
        let threadpool = TaskPool::new(config.podcast_simultanious_download);
        let (tx_to_main, rx_to_main) = mpsc::channel();

        let config = std::sync::Arc::new(parking_lot::RwLock::new(config));

        let playlist = Playlist::new(config.clone()).unwrap_or_default();
        let app = Self::init_app(&tree, &config);

        // This line is required, in order to show the playing message for the first track
        // playlist.set_current_track_index(0);

        Self {
            app,
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
            tree,
            path,
            terminal,
            config,
            yanked_node_id: None,
            // current_song: None,
            tageditor_song: None,
            time_pos: Duration::default(),
            lyric_line: String::new(),

            // TODO: Consider making YoutubeOptions async and use async reqwest in YoutubeOptions
            // and avoid this `spawn_blocking` call.
            youtube_options: tokio::task::spawn_blocking(YoutubeOptions::default)
                .await
                .expect("Failed to initialize YoutubeOptions in a blocking task due to a panic"),
            #[cfg(all(feature = "cover-uberzug", not(target_os = "windows")))]
            ueberzug_instance,
            songtag_options: vec![],
            sender_songtag: tx3,
            receiver_songtag: rx3,
            viuer_supported,
            ce_themes: vec![],
            ce_style_color_symbol: StyleColorSymbol::default(),
            ke_key_config: Keys::default(),
            db,
            layout: TermusicLayout::TreeView,
            config_layout: ConfigEditorLayout::General,
            db_criteria,
            db_search_results: Vec::new(),
            db_search_tracks: Vec::new(),
            config_changed: false,
            podcasts,
            podcasts_index: 0,
            db_podcast,
            threadpool,
            tx_to_main,
            rx_to_main,
            download_tracker: DownloadTracker::default(),
            podcast_search_vec: None,
            playlist,
            cmd_tx,
            current_song: None,
        }
    }

    pub fn get_full_path_from_config(config: &Settings) -> PathBuf {
        let mut full_path = String::new();
        if let Some(dir) = config.music_dir.first() {
            full_path = shellexpand::tilde(dir).to_string();
        }

        if let Some(music_dir) = &config.music_dir_from_cli {
            full_path = shellexpand::tilde(music_dir).to_string();
        };
        PathBuf::from(full_path)
    }

    pub fn init_config(&mut self) {
        if let Err(e) = Self::theme_select_save() {
            self.mount_error_popup(e.context("theme save"));
        }
        self.mount_label_help();
        self.db.sync_database(&self.path);
        self.playlist_sync();
    }

    /// Initialize terminal
    pub fn init_terminal(&mut self) {
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic| {
            let mut terminal_clone = TerminalBridge::new().expect("Could not initialize terminal");
            let _drop = terminal_clone.disable_raw_mode();
            let _drop = terminal_clone.leave_alternate_screen();
            original_hook(panic);
        }));
        let _drop = self.terminal.enable_raw_mode();
        let _drop = self.terminal.enter_alternate_screen();
        let _drop = self.terminal.clear_screen();
    }

    /// Finalize terminal
    pub fn finalize_terminal(&mut self) {
        let _drop = self.terminal.disable_raw_mode();
        let _drop = self.terminal.leave_alternate_screen();
    }
    /// Returns elapsed time since last redraw
    pub fn since_last_redraw(&self) -> Duration {
        self.last_redraw.elapsed()
    }
    pub fn force_redraw(&mut self) {
        self.redraw = true;
    }

    pub fn run(&mut self) {
        self.command(&PlayerCmd::GetProgress);
        self.progress_update_title();
        self.lyric_update_title();
    }

    pub fn player_sync_playlist(&mut self) -> Result<()> {
        self.playlist.save()?;
        self.command(&PlayerCmd::ReloadPlaylist);
        Ok(())
    }

    pub fn player_update_current_track_after(&mut self) {
        self.time_pos = Duration::default();
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(e.context("update_photo"));
        };
        self.progress_update_title();
        self.lyric_update_title();
        self.update_playing_song();
    }

    pub fn player_toggle_pause(&mut self) {
        if self.playlist.is_empty() && self.playlist.current_track().is_none() {
            return;
        }

        self.command(&PlayerCmd::TogglePause);
        // self.progress_update_title();
    }

    pub fn player_previous(&mut self) {
        self.command(&PlayerCmd::SkipPrevious);
    }

    pub fn command(&mut self, cmd: &PlayerCmd) {
        if let Err(e) = self.cmd_tx.send(cmd.clone()) {
            self.mount_error_popup((anyhow!(e)).context(format!("{cmd:?}")));
        }
    }

    pub fn is_radio(&self) -> bool {
        if let Some(track) = self.playlist.current_track() {
            if track.media_type == MediaType::LiveRadio {
                return true;
            }
        }
        false
    }
}
