mod download_tracker;
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
use crate::CombinedSettings;
use download_tracker::DownloadTracker;
use termusiclib::config::v2::tui::keys::Keys;
use termusiclib::config::v2::tui::theme::ThemeWrap;
use termusiclib::library_db::{DataBase, SearchCriteria};
use termusiclib::types::{Id, Msg, SearchLyricState, YoutubeOptions};
use termusiclib::xywh;

use termusiclib::track::{MediaType, Track};
#[cfg(all(feature = "cover-ueberzug", not(target_os = "windows")))]
use termusiclib::ueberzug::UeInstance;

use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};
use termusiclib::config::{ServerOverlay, SharedServerSettings, SharedTuiSettings};
use termusiclib::library_db::TrackDB;
use termusiclib::podcast::{db::Database as DBPod, Podcast, PodcastFeed};
use termusiclib::songtag::SongTag;
use termusiclib::taskpool::TaskPool;
use termusiclib::utils::get_app_config_path;
use termusicplayback::{PlayerCmd, Playlist};
use tokio::sync::mpsc::UnboundedSender;
use tui_realm_treeview::Tree;
use tuirealm::event::NoUserEvent;
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalBridge};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TermusicLayout {
    TreeView,
    DataBase,
    Podcast,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum ConfigEditorLayout {
    General,
    Color,
    Key1,
    Key2,
}

/// All data specific to the Music Library Widget / View
#[derive(Debug)]
pub struct MusicLibraryData {
    /// Current Path that the library-tree is in
    pub tree_path: PathBuf,
    /// Tree of the Music Library widget
    pub tree: Tree<String>,
    /// The Node that a yank & paste was started on
    pub yanked_node_id: Option<String>,
}

/// All data specific to the Database Widget / View
#[derive(Debug)]
pub struct DatabaseWidgetData {
    /// Criteria to search for
    pub criteria: SearchCriteria,
    /// Criteria Search results `(criteria -> this)`
    pub search_results: Vec<String>,
    /// Results of the critea results search `(criteria -> search_results -> this)`
    pub search_tracks: Vec<TrackDB>,
}

impl DatabaseWidgetData {
    /// Reset all search Vectors to a new empty Vector
    ///
    /// (removing allocations)
    pub fn reset_search_results(&mut self) {
        // Reset instead of ".clear" as "clear" does not remove capacity and might not be used again and could potentially be large
        self.search_results = Vec::new();
        self.search_tracks = Vec::new();
    }
}

/// All data specific to the Podcast Widget / View
#[derive(Debug)]
pub struct PodcastWidgetData {
    /// Loaded and displayed Podcast list
    pub podcasts: Vec<Podcast>,
    /// Selected podcast index
    pub podcasts_index: usize,
    /// Podcast Database
    pub db_podcast: DBPod,
    /// Podcast search results
    pub search_results: Option<Vec<PodcastFeed>>,
}

/// All data specific to the Config Editor Widget / View
#[derive(Debug)]
pub struct ConfigEditorData {
    /// All possible themes that could be selected
    pub themes: Vec<String>,
    /// The Theme to edit to preview before saving
    pub theme: ThemeWrap,
    /// The Keybindings to preview before saving
    pub key_config: Keys,
    /// The current tab in the config editor
    pub layout: ConfigEditorLayout,
    /// Indicator to prompt a save on config editor exit
    pub config_changed: bool,
}

pub struct Model {
    /// Indicates that the application must quit
    pub quit: bool,
    /// Tells whether to redraw interface
    pub redraw: bool,
    last_redraw: Instant,
    pub app: Application<Id, Msg, NoUserEvent>,
    /// Used to draw to terminal
    pub terminal: TerminalBridge<CrosstermTerminalAdapter>,
    pub tx_to_main: Sender<Msg>,
    pub rx_to_main: Receiver<Msg>,
    /// Sender for Player Commands
    pub cmd_tx: UnboundedSender<PlayerCmd>,

    pub config_tui: SharedTuiSettings,
    pub config_server: SharedServerSettings,
    pub db: DataBase,

    pub layout: TermusicLayout,
    pub library: MusicLibraryData,
    pub dw: DatabaseWidgetData,
    pub podcast: PodcastWidgetData,
    pub config_editor: ConfigEditorData,

    /// Clone of `playlist.current_track`, but kept around when playlist goes empty but song is still playing
    pub current_song: Option<Track>,
    pub tageditor_song: Option<Track>,
    pub time_pos: Duration,
    pub lyric_line: String,
    pub playlist: Playlist,

    #[cfg(all(feature = "cover-ueberzug", not(target_os = "windows")))]
    pub ueberzug_instance: UeInstance,
    pub viuer_supported: ViuerSupported,
    pub xywh: xywh::Xywh,

    youtube_options: YoutubeOptions,
    pub songtag_options: Vec<SongTag>,
    pub sender_songtag: Sender<SearchLyricState>,
    pub receiver_songtag: Receiver<SearchLyricState>,
    pub download_tracker: DownloadTracker,
    /// Taskpool to limit number of active network requests
    ///
    /// Currently only used for podcast sync & download
    pub taskpool: TaskPool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ViuerSupported {
    #[cfg(feature = "cover-viuer-kitty")]
    Kitty,
    #[cfg(feature = "cover-viuer-iterm")]
    ITerm,
    #[cfg(feature = "cover-viuer-sixel")]
    Sixel,
    NotSupported,
}

fn get_viuer_support() -> ViuerSupported {
    #[cfg(feature = "cover-viuer-kitty")]
    if viuer::KittySupport::None != viuer::get_kitty_support() {
        return ViuerSupported::Kitty;
    }
    #[cfg(feature = "cover-viuer-iterm")]
    if viuer::is_iterm_supported() {
        return ViuerSupported::ITerm;
    }
    #[cfg(feature = "cover-viuer-sixel")]
    if viuer::is_sixel_supported() {
        return ViuerSupported::Sixel;
    }

    ViuerSupported::NotSupported
}

impl Model {
    pub async fn new(config: CombinedSettings, cmd_tx: UnboundedSender<PlayerCmd>) -> Self {
        let CombinedSettings {
            server: config_server,
            tui: config_tui,
        } = config;
        let path = Self::get_full_path_from_config(&config_server.read());
        // TODO: refactor music library tree to be Paths instead?
        let tree = Tree::new(Self::library_dir_tree(
            &path,
            config_server.read().get_library_scan_depth(),
        ));

        let (tx3, rx3): (Sender<SearchLyricState>, Receiver<SearchLyricState>) = mpsc::channel();

        let viuer_supported = get_viuer_support();
        let db = DataBase::new(&config_server.read()).expect("Open Library Database");
        let db_criteria = SearchCriteria::Artist;
        let terminal = TerminalBridge::new_crossterm().expect("Could not initialize terminal");

        #[cfg(all(feature = "cover-ueberzug", not(target_os = "windows")))]
        let ueberzug_instance = UeInstance::default();
        let db_path = get_app_config_path().expect("failed to get podcast db path.");

        let db_podcast = DBPod::new(&db_path).expect("error connecting to podcast db.");

        let podcasts = db_podcast
            .get_podcasts()
            .expect("failed to get podcasts from db.");
        let taskpool = TaskPool::new(usize::from(
            config_server
                .read()
                .settings
                .podcast
                .concurrent_downloads_max
                .get(),
        ));
        let (tx_to_main, rx_to_main) = mpsc::channel();

        let playlist = Playlist::new(config_server.clone()).unwrap_or_default();
        let app = Self::init_app(&tree, &config_tui);

        // This line is required, in order to show the playing message for the first track
        // playlist.set_current_track_index(0);

        let ce_theme = config_tui.read().settings.theme.clone();
        let xywh = xywh::Xywh::from(&config_tui.read().settings.coverart);

        Self {
            app,
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
            terminal,
            config_server,
            config_tui,
            // current_song: None,
            tageditor_song: None,
            time_pos: Duration::default(),
            lyric_line: String::new(),

            library: MusicLibraryData {
                tree_path: path,
                tree,
                yanked_node_id: None,
            },
            // TODO: Consider making YoutubeOptions async and use async reqwest in YoutubeOptions
            // and avoid this `spawn_blocking` call.
            youtube_options: tokio::task::spawn_blocking(YoutubeOptions::default)
                .await
                .expect("Failed to initialize YoutubeOptions in a blocking task due to a panic"),
            #[cfg(all(feature = "cover-ueberzug", not(target_os = "windows")))]
            ueberzug_instance,
            songtag_options: vec![],
            sender_songtag: tx3,
            receiver_songtag: rx3,
            viuer_supported,
            db,
            layout: TermusicLayout::TreeView,
            dw: DatabaseWidgetData {
                criteria: db_criteria,
                search_results: Vec::new(),
                search_tracks: Vec::new(),
            },
            podcast: PodcastWidgetData {
                podcasts,
                podcasts_index: 0,
                db_podcast,
                search_results: None,
            },
            config_editor: ConfigEditorData {
                themes: Vec::new(),
                theme: ce_theme,
                key_config: Keys::default(),
                layout: ConfigEditorLayout::General,
                config_changed: false,
            },
            taskpool,
            tx_to_main,
            rx_to_main,
            download_tracker: DownloadTracker::default(),
            playlist,
            cmd_tx,
            current_song: None,
            xywh,
        }
    }

    #[inline]
    pub fn get_combined_settings(&self) -> CombinedSettings {
        CombinedSettings {
            server: self.config_server.clone(),
            tui: self.config_tui.clone(),
        }
    }

    /// Get the first music directory or the cli provided music dir resolved
    pub fn get_full_path_from_config(config: &ServerOverlay) -> PathBuf {
        let mut full_path = String::new();

        if let Some(first_music_dir) = config.get_first_music_dir() {
            full_path = shellexpand::path::tilde(first_music_dir)
                .to_string_lossy()
                .to_string();
        }

        PathBuf::from(full_path)
    }

    pub fn init_config(&mut self) {
        if let Err(e) = Self::theme_select_save() {
            self.mount_error_popup(e.context("theme save"));
        }
        self.mount_label_help();
        self.db.sync_database(&self.library.tree_path);
        self.playlist_sync();
    }

    /// Initialize terminal
    pub fn init_terminal(&mut self) {
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic| {
            let mut terminal_clone =
                TerminalBridge::new_crossterm().expect("Could not initialize terminal");
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
