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
#[cfg(feature = "cover")]
use termusiclib::ueberzug::UeInstance;
use termusiclib::{config::Settings, track::Track};

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};
use termusiclib::config::{Keys, StyleColorSymbol};
use termusiclib::podcast::{db::Database as DBPod, Podcast, PodcastFeed, Threadpool};
use termusiclib::songtag::SongTag;
use termusiclib::sqlite::TrackForDB;
// use termusiclib::track::MediaType;
use termusiclib::utils::{get_app_config_path, DownloadTracker};
// use termusicplayback::{GeneralPlayer, PlayerMsg, PlayerTrait};
use anyhow::Result;
use termusicplayback::{audio_cmd, PlayerCmd, Playlist, Status};
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
    pub config: Settings,
    // pub player: GeneralPlayer,
    pub yanked_node_id: Option<String>,
    // pub current_song: Option<Track>,
    pub tageditor_song: Option<Track>,
    pub time_pos: i64,
    pub lyric_line: String,
    youtube_options: YoutubeOptions,
    #[cfg(feature = "cover")]
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
    pub threadpool: Threadpool,
    pub tx_to_main: Sender<Msg>,
    pub rx_to_main: Receiver<Msg>,
    pub podcast_search_vec: Option<Vec<PodcastFeed>>,
    pub playlist: Playlist,
}

#[derive(Debug)]
pub enum ViuerSupported {
    Kitty,
    ITerm,
    // Sixel,
    NotSupported,
}

impl Model {
    pub fn new(config: &Settings) -> Self {
        let path = Self::get_full_path_from_config(config);
        let tree = Tree::new(Self::library_dir_tree(&path, config.max_depth_cli));

        let (tx3, rx3): (Sender<SearchLyricState>, Receiver<SearchLyricState>) = mpsc::channel();

        let mut viuer_supported = ViuerSupported::NotSupported;
        if viuer::KittySupport::None != viuer::get_kitty_support() {
            viuer_supported = ViuerSupported::Kitty;
        // } else if viuer::is_sixel_supported() {
        // viuer_supported = ViuerSupported::Sixel;
        } else if viuer::is_iterm_supported() {
            viuer_supported = ViuerSupported::ITerm;
        }
        let db = DataBase::new(config);
        let db_criteria = SearchCriteria::Artist;
        let app = Self::init_app(&tree, config);
        let terminal = TerminalBridge::new().expect("Could not initialize terminal");
        // let player = GeneralPlayer::new(config);
        // let viuer_supported =
        //     viuer::KittySupport::None != viuer::get_kitty_support() || viuer::is_iterm_supported();

        #[cfg(feature = "cover")]
        let ueberzug_instance = UeInstance::default();
        let db_path = get_app_config_path().expect("failed to get podcast db path.");

        let db_podcast = DBPod::connect(&db_path).expect("error connecting to podcast db.");

        let podcasts = db_podcast
            .get_podcasts()
            .expect("failed to get podcasts from db.");
        let threadpool = Threadpool::new(config.podcast_simultanious_download);
        let (tx_to_main, rx_to_main) = mpsc::channel();

        let mut playlist = Playlist::new(config).unwrap_or_default();
        // This line is required, in order to show the playing message for the first track
        playlist.set_current_track_index(usize::MAX);

        Self {
            app,
            quit: false,
            redraw: true,
            last_redraw: Instant::now(),
            tree,
            path,
            terminal,
            config: config.clone(),
            // player,
            yanked_node_id: None,
            // current_song: None,
            tageditor_song: None,
            time_pos: 0,
            lyric_line: String::new(),
            youtube_options: YoutubeOptions::default(),
            #[cfg(feature = "cover")]
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
        }
    }

    pub fn get_full_path_from_config(config: &Settings) -> PathBuf {
        let mut full_path = String::new();
        if let Some(dir) = config.music_dir.get(0) {
            full_path = shellexpand::tilde(dir).to_string();
        }

        if let Some(music_dir) = &config.music_dir_from_cli {
            full_path = shellexpand::tilde(music_dir).to_string();
        };
        PathBuf::from(full_path)
    }

    pub fn init_config(&mut self) {
        if let Err(e) = Self::theme_select_save() {
            self.mount_error_popup(format!("theme save error: {e}"));
        }
        self.mount_label_help();
        self.db.sync_database(&self.path);
        self.playlist_sync();
    }

    /// Initialize terminal
    pub fn init_terminal(&mut self) {
        let _drop = self.terminal.enable_raw_mode();
        let _drop = self.terminal.enter_alternate_screen();
        let _drop = self.terminal.clear_screen();
    }

    /// Finalize terminal
    pub fn finalize_terminal(&mut self) {
        let _drop = self.terminal.disable_raw_mode();
        let _drop = self.terminal.leave_alternate_screen();
        let _drop = self.terminal.clear_screen();
    }
    /// Returns elapsed time since last redraw
    pub fn since_last_redraw(&self) -> Duration {
        self.last_redraw.elapsed()
    }
    pub fn force_redraw(&mut self) {
        self.redraw = true;
    }

    pub fn run(&mut self) {
        match audio_cmd(PlayerCmd::FetchStatus, false) {
            Ok(status) => {
                match status {
                    Status::Running => match self.playlist.status() {
                        Status::Running => {}
                        Status::Stopped => {
                            self.playlist.set_status(status);
                            // This is to show the first album photo
                            self.player_update_current_track_after();
                        }
                        Status::Paused => {
                            self.playlist.set_status(status);
                        }
                    },
                    Status::Stopped => match self.playlist.status() {
                        Status::Running | Status::Paused => {
                            self.playlist.set_status(status);
                            // This is to clear the photo shown when stopped
                            if self.playlist.is_empty() {
                                self.player_update_current_track_after();
                                return;
                            }
                        }
                        Status::Stopped => {}
                    },
                    Status::Paused => match self.playlist.status() {
                        Status::Running | Status::Stopped => {
                            self.playlist.set_status(status);
                        }
                        Status::Paused => {}
                    },
                }
            }
            Err(e) => self.mount_error_popup(format!("Error fetch status: {e}")),
        };
        self.player_get_progress();
        self.progress_update_title();
        self.lyric_update_title();
    }

    pub fn player_get_progress(&mut self) {
        match audio_cmd::<(i64, i64, usize)>(PlayerCmd::GetProgress, false) {
            Ok((position, duration, current_track_index)) => {
                self.progress_update(position, duration);
                if current_track_index != self.playlist.get_current_track_index() {
                    info!(
                        "index from player is:{current_track_index}, index in tui is:{}",
                        self.playlist.get_current_track_index()
                    );
                    self.playlist.clear_current_track();
                    self.playlist.set_current_track_index(current_track_index);
                    self.update_layout_for_current_track();
                    self.player_update_current_track_after();

                    //             self.lyric_update_for_podcast_by_current_track();
                    //             if (self.config.speed - 10).abs() >= 1 {
                    //                 self.player.set_speed(self.config.speed);
                    //             }

                    //             if let Err(e) = self.podcast_mark_current_track_played() {
                    //                 self.mount_error_popup(format!("Error when mark episode as played: {e}"));
                    //             }
                }
            }
            Err(e) => self.mount_error_popup(format!("Error get progress: {e}")),
        };
    }

    pub fn player_sync_playlist(&mut self) -> Result<()> {
        self.playlist.save()?;
        audio_cmd(PlayerCmd::ReloadPlaylist, false)?;
        Ok(())
    }
    // pub fn player_playlist_changed(&self) -> bool {
    //     audio_cmd(PlayerCmd::CheckPlaylistChanged, false).expect("What went wrong?!")
    // }

    // pub fn player_stop(&mut self) {
    //     self.time_pos = 0;
    //     // self.player.stop();
    //     // self.player.message_tx.send(PlayerMsg::Progress(0, 60)).ok();
    //     if let Err(e) = self.update_photo() {
    //         self.mount_error_popup(format!("update photo error: {e}"));
    //     };
    //     self.progress_update_title();
    //     self.lyric_update_title();
    //     self.lyric_update();
    //     self.force_redraw();
    // }

    pub fn player_update_current_track_after(&mut self) {
        self.time_pos = 0;
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {e}"));
        };
        self.progress_update_title();
        self.lyric_update_title();
        self.update_playing_song();
    }

    pub fn player_toggle_pause(&mut self) {
        if self.playlist.is_empty() && self.playlist.current_track().is_none() {
            return;
        }
        // if self.player.is_paused() {
        //     self.player.resume();
        //     #[cfg(feature = "mpris")]
        //     self.mpris.resume();
        //     #[cfg(feature = "discord")]
        //     self.discord.resume(self.time_pos);
        // } else {
        //     self.player.pause();
        //     #[cfg(feature = "mpris")]
        //     self.mpris.pause();
        //     #[cfg(feature = "discord")]
        //     self.discord.pause();
        // }
        audio_cmd::<()>(PlayerCmd::TogglePause, false).ok();
        // self.playlist.set_status(Status::Paused);
        self.progress_update_title();
    }

    // pub fn player_seek(&mut self, offset: i64) {
    // FIXME: dirty fix for seeking when paused with symphonia,basically set it to play
    // in rusty sink code, and seek, and then set it back to pause.
    // #[cfg(not(any(feature = "mpv", feature = "gst")))]
    // let paused = self.player.is_paused();
    // #[cfg(not(any(feature = "mpv", feature = "gst")))]
    // if paused {
    //     self.player.set_volume(0);
    // }

    // self.player.seek(offset).ok();

    // #[cfg(not(any(feature = "mpv", feature = "gst")))]
    // if paused {
    //     self.force_redraw();
    //     std::thread::sleep(std::time::Duration::from_millis(50));
    //     self.player.pause();
    //     self.player.set_volume(self.config.volume);
    // }
    // }

    pub fn player_skip(&mut self) {
        if let Err(e) = audio_cmd::<()>(PlayerCmd::Skip, false) {
            self.mount_error_popup(format!("Error reload playlist: {e}"));
        }
        // self.playlist.set_current_track_index(usize::MAX);
    }

    pub fn player_previous(&mut self) {
        if let Err(e) = audio_cmd::<()>(PlayerCmd::Previous, false) {
            self.mount_error_popup(format!("Error in previous:{e}"));
        }
        self.playlist.clear_current_track();
    }
}
