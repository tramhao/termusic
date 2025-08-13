use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use id3::frame::Lyrics as Id3Lyrics;
use termusiclib::config::v2::server::ScanDepth;
use termusiclib::config::v2::tui::keys::Keys;
use termusiclib::config::v2::tui::theme::ThemeWrap;
use termusiclib::config::{ServerOverlay, SharedServerSettings, SharedTuiSettings};
use termusiclib::ids::Id;
use termusiclib::new_database::Database;
use termusiclib::new_database::track_ops::TrackRead;
use termusiclib::player::playlist_helpers::PlaylistTrackSource;
use termusiclib::player::{PlaylistTracks, RunningStatus};
use termusiclib::podcast::{Podcast, PodcastFeed, db::Database as DBPod};
use termusiclib::songtag::SongTag;
use termusiclib::songtag::lrc::Lyric;
use termusiclib::taskpool::TaskPool;
use termusiclib::track::{LyricData, MediaTypesSimple, Track};
#[cfg(all(feature = "cover-ueberzug", not(target_os = "windows")))]
use termusiclib::ueberzug::UeInstance;
use termusiclib::utils::get_app_config_path;
use termusiclib::xywh;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tui_realm_treeview::Tree;
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalBridge};

use super::components::TETrack;
use super::tui_cmd::TuiCmd;
use crate::CombinedSettings;
use crate::ui::Application;
use crate::ui::model::youtube_options::YoutubeOptions;
use crate::ui::msg::{Msg, SearchCriteria};
pub use download_tracker::DownloadTracker;
pub use user_events::UserEvent;

mod download_tracker;
mod playlist;
mod update;
mod user_events;
mod view;
pub mod youtube_options;

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
    pub search_tracks: Vec<TrackRead>,
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
    /// All possible file-themes that could be selected
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

/// Information about the playback status
#[derive(Debug, Clone)]
pub struct Playback {
    /// The Playlist with all the tracks.
    pub playlist: playlist::TUIPlaylist,
    /// The current Running Status like Playing / Paused
    status: RunningStatus,
    /// The current track, if there is one. Does not need to be in the playlist.
    current_track: Option<Track>,
    current_track_pos: Duration,
}

impl Playback {
    fn new() -> Self {
        Self {
            playlist: playlist::TUIPlaylist::default(),
            status: RunningStatus::default(),
            current_track: None,
            current_track_pos: Duration::ZERO,
        }
    }

    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.status == RunningStatus::Stopped
    }

    #[must_use]
    #[expect(dead_code)]
    pub fn is_paused(&self) -> bool {
        self.status == RunningStatus::Paused
    }

    #[must_use]
    pub fn status(&self) -> RunningStatus {
        self.status
    }

    pub fn set_status(&mut self, status: RunningStatus) {
        self.status = status;
    }

    #[must_use]
    pub fn current_track(&self) -> Option<&Track> {
        self.current_track.as_ref()
    }

    #[must_use]
    #[expect(dead_code)]
    pub fn current_track_mut(&mut self) -> Option<&mut Track> {
        self.current_track.as_mut()
    }

    pub fn clear_current_track(&mut self) {
        self.current_track.take();
    }

    pub fn set_current_track(&mut self, track: Option<Track>) {
        self.current_track = track;
    }

    /// Set the current track from the playlist, if there is one
    pub fn set_current_track_from_playlist(&mut self) {
        self.set_current_track(self.playlist.current_track().cloned());
    }

    pub fn current_track_pos(&self) -> Duration {
        self.current_track_pos
    }

    pub fn set_current_track_pos(&mut self, pos: Duration) {
        self.current_track_pos = pos;
    }

    /// Load Tracks from a GRPC response.
    ///
    /// Returns `(Position, Tracks[])`.
    ///
    /// # Errors
    ///
    /// - when converting from u64 grpc values to usize fails
    /// - when there is no track-id
    /// - when reading a Track from path or podcast database fails
    pub fn load_from_grpc(
        &mut self,
        info: PlaylistTracks,
        podcast_db: &DBPod,
    ) -> anyhow::Result<()> {
        let current_track_index = usize::try_from(info.current_track_index)
            .context("convert current_track_index(u64) to usize")?;
        let mut playlist_items = Vec::with_capacity(info.tracks.len());

        for (idx, track) in info.tracks.into_iter().enumerate() {
            let at_index_usize =
                usize::try_from(track.at_index).context("convert at_index(u64) to usize")?;
            // assume / require that the tracks are ordered correctly, if not just log a error for now
            if idx != at_index_usize {
                error!("Non-matching \"index\" and \"at_index\"!");
            }

            // this case should never happen with "termusic-server", but grpc marks them as "optional"
            let Some(id) = track.id else {
                bail!("Track does not have a id, which is required to load!");
            };

            let track = match PlaylistTrackSource::try_from(id)? {
                PlaylistTrackSource::Path(v) => Track::read_track_from_path(v)?,
                PlaylistTrackSource::Url(v) => Track::new_radio(&v),
                PlaylistTrackSource::PodcastUrl(v) => {
                    let episode = podcast_db.get_episode_by_url(&v)?;
                    Track::from_podcast_episode(&episode)
                }
            };

            playlist_items.push(track);
        }

        self.playlist.set_tracks(playlist_items);

        // the old server playlist implementation will send `current_track_index: 0`, even if there are not tracks
        // but the new TUI implementation function "set_current_track_index" will refuse to set anything if the index is out-of-bounds
        if !self.playlist.is_empty() {
            self.playlist.set_current_track_index(current_track_index)?;
        }

        self.set_current_track_from_playlist();

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtraLyricData {
    pub for_track: PathBuf,
    pub data: LyricData,
    pub selected_idx: usize,
}

impl ExtraLyricData {
    /// Cycle to the next lyric frame and parse it.
    ///
    /// Returns `Some(RawLyric)` if found.
    ///
    /// # Errors
    ///
    /// If there are no frames
    pub fn cycle_lyric(&mut self) -> Result<Option<&Id3Lyrics>> {
        if self.data.raw_lyrics.is_empty() {
            bail!("No lyric frames");
        }

        self.selected_idx += 1;
        if self.selected_idx >= self.data.raw_lyrics.len() {
            self.selected_idx = 0;
        }

        let raw_lyric = self.data.raw_lyrics.get(self.selected_idx);
        self.data.parsed_lyrics = raw_lyric.and_then(|v| Lyric::from_str(&v.text).ok());

        Ok(raw_lyric)
    }
}

pub type TxToMain = UnboundedSender<Msg>;

pub struct Model {
    /// Indicates that the application must quit
    pub quit: bool,
    /// Tells whether to redraw interface
    pub redraw: bool,
    last_redraw: Instant,
    pub app: Application<Id, Msg, UserEvent>,
    /// Used to draw to terminal
    pub terminal: TerminalBridge<CrosstermTerminalAdapter>,
    pub tx_to_main: TxToMain,
    pub rx_to_main: UnboundedReceiver<Msg>,
    /// Sender for Player Commands
    pub cmd_to_server_tx: UnboundedSender<TuiCmd>,

    pub config_tui: SharedTuiSettings,
    pub config_server: SharedServerSettings,
    pub db: Database,

    pub layout: TermusicLayout,
    pub library: MusicLibraryData,
    pub dw: DatabaseWidgetData,
    pub podcast: PodcastWidgetData,
    pub config_editor: ConfigEditorData,

    pub tageditor_song: Option<TETrack>,
    pub lyric_line: String,
    pub current_track_lyric: Option<ExtraLyricData>,
    pub playback: Playback,

    #[cfg(all(feature = "cover-ueberzug", not(target_os = "windows")))]
    pub ueberzug_instance: Option<UeInstance>,
    pub viuer_supported: ViuerSupported,
    pub xywh: xywh::Xywh,

    youtube_options: YoutubeOptions,
    pub songtag_options: Vec<SongTag>,
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
    #[allow(clippy::too_many_lines)]
    pub async fn new(config: CombinedSettings, cmd_to_server_tx: UnboundedSender<TuiCmd>) -> Self {
        let CombinedSettings {
            server: config_server,
            tui: config_tui,
        } = config;
        let path = Self::get_full_path_from_config(&config_server.read());
        // TODO: refactor music library tree to be Paths instead?
        let tree = Self::loading_tree();

        let viuer_supported = if config_tui.read().cover_features_enabled() {
            get_viuer_support()
        } else {
            ViuerSupported::NotSupported
        };

        info!("Using viuer protocol {viuer_supported:#?}");

        let db = Database::new_default_path().expect("Open Library Database");
        let db_criteria = SearchCriteria::Artist;
        let terminal = TerminalBridge::new_crossterm().expect("Could not initialize terminal");

        #[cfg(all(feature = "cover-ueberzug", not(target_os = "windows")))]
        let ueberzug_instance = if config_tui.read().cover_features_enabled()
            && viuer_supported == ViuerSupported::NotSupported
        {
            Some(UeInstance::default())
        } else {
            None
        };
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
        let (tx_to_main, rx_to_main) = unbounded_channel();

        let app = Self::init_app(&tree, &config_tui);

        // This line is required, in order to show the playing message for the first track
        // playlist.set_current_track_index(0);

        let ce_theme = config_tui.read().settings.theme.clone();
        let xywh = xywh::Xywh::from(&config_tui.read().settings.coverart);

        let download_tracker = DownloadTracker::default();

        Self::library_scan(
            tx_to_main.clone(),
            download_tracker.clone(),
            &path,
            ScanDepth::Limited(2),
            None,
        );

        Self {
            app,
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
            terminal,
            config_server,
            config_tui,
            tageditor_song: None,
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
            download_tracker,
            current_track_lyric: None,
            playback: Playback::new(),
            cmd_to_server_tx,
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
        if let Err(e) = Self::theme_extract_all() {
            self.mount_error_popup(e.context("theme save"));
        }
        self.mount_label_help();
        if let Err(err) =
            self.db
                .scan_path(&self.library.tree_path, &self.config_server.read(), false)
        {
            error!(
                "Error scanning path {:#?}: {err:#?}",
                self.library.tree_path.display()
            );
        }
        self.playlist_sync();
    }

    /// Initialize terminal
    pub fn init_terminal(&mut self) {
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic| {
            Self::hook_reset_terminal();
            original_hook(panic);
        }));
        let _drop = self.terminal.enable_raw_mode();
        let _drop = self.terminal.enter_alternate_screen();
        // required as "enter_alternate_screen" always enabled mouse-capture
        let _drop = self.terminal.disable_mouse_capture();
        let _drop = self.terminal.clear_screen();
        crate::TERMINAL_ALTERNATE_MODE.store(true, Ordering::SeqCst);
    }

    /// Finalize terminal for hooks like panic or CTRL+C
    pub fn hook_reset_terminal() {
        let mut terminal_clone =
            TerminalBridge::new_crossterm().expect("Could not initialize terminal");
        let _drop = terminal_clone.disable_raw_mode();
        let _drop = terminal_clone.leave_alternate_screen();
        crate::TERMINAL_ALTERNATE_MODE.store(false, Ordering::SeqCst);
    }

    /// Finalize terminal
    pub fn finalize_terminal(&mut self) {
        let _drop = self.terminal.disable_raw_mode();
        let _drop = self.terminal.leave_alternate_screen();
        crate::TERMINAL_ALTERNATE_MODE.store(false, Ordering::SeqCst);
    }

    /// Returns elapsed time since last redraw
    pub fn since_last_redraw(&self) -> Duration {
        self.last_redraw.elapsed()
    }

    /// Force a redraw of the entire model
    pub fn force_redraw(&mut self) {
        self.redraw = true;
    }

    /// Send a command to request the Track Progress and set the titles to the current state.
    #[inline]
    pub fn request_progress(&mut self) {
        self.command(TuiCmd::GetProgress);
    }

    /// Update all the places that need to be updated after a current track change or running status change.
    pub fn player_update_current_track_after(&mut self) {
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(e.context("update_photo"));
        }
        self.progress_update_title();
        self.lyric_update_title();
        self.update_playing_song();
    }

    /// Send a [`TogglePause`](TuiCmd::TogglePause) command, if the conditions are right.
    pub fn player_toggle_pause(&mut self) {
        if self.playback.playlist.is_empty() && self.playback.current_track().is_none() {
            return;
        }

        self.command(TuiCmd::TogglePause);
    }

    /// Send a [`SkipPrevious`](TuiCmd::SkipPrevious) command.
    pub fn player_previous(&mut self) {
        self.command(TuiCmd::SkipPrevious);
    }

    /// Send a command to the `MusicPlayerService` (via the Client)
    pub fn command(&mut self, cmd: TuiCmd) {
        if let Err(e) = self.cmd_to_server_tx.send(cmd) {
            self.mount_error_popup(anyhow!(e));
        }
    }

    pub fn is_radio(&self) -> bool {
        if let Some(track) = self.playback.current_track() {
            if track.media_type() == MediaTypesSimple::LiveRadio {
                return true;
            }
        }
        false
    }
}
