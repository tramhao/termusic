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
mod youtube_options;
use crate::{
    config::Termusic,
    song::Song,
    ui::{Application, Id, Msg, StatusLine},
    VERSION,
};

use crate::player::GStreamer;
use crate::ui::components::{
    draw_area_in, draw_area_top_right, DeleteConfirmInputPopup, DeleteConfirmRadioPopup,
    ErrorPopup, GlobalListener, HelpPopup, Label, LibrarySearchInputPopup, LibrarySearchTablePopup,
    Lyric, MessagePopup, MusicLibrary, Playlist, Progress, QuitPopup, YoutubeSearchInputPopup,
    YoutubeSearchTablePopup,
};
use crate::ui::{Loop, Status};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
#[cfg(feature = "cover")]
use std::process::Child;
use std::sync::mpsc::{self, Receiver, Sender};
#[cfg(feature = "cover")]
use std::sync::RwLock;
use std::thread::{self, sleep};
use std::time::{Duration, Instant};
use tui_realm_treeview::Tree;
use tuirealm::props::{Alignment, AttrValue, Attribute, Color, PropPayload, TextModifiers};
use tuirealm::terminal::TerminalBridge;
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::tui::widgets::Clear;
use tuirealm::{EventListenerCfg, NoUserEvent};
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
    pub time_pos: u64,
    pub lyric_line: String,
    youtube_options: YoutubeOptions,
    sender: Sender<UpdateComponents>,
    receiver: Receiver<UpdateComponents>,
    pub sender_playlist_items: Sender<VecDeque<Song>>,
    receiver_playlist_items: Receiver<VecDeque<Song>>,

    #[cfg(feature = "cover")]
    ueberzug: RwLock<Option<Child>>,
}

impl Model {
    pub fn new(config: &Termusic) -> Self {
        let full_path = shellexpand::tilde(&config.music_dir);
        let p: &Path = Path::new(full_path.as_ref());
        let tree = Tree::new(Self::dir_tree(p, MAX_DEPTH));

        let mut player = GStreamer::new();
        player.set_volume(config.volume);
        let (tx, rx): (Sender<UpdateComponents>, Receiver<UpdateComponents>) = mpsc::channel();
        let (tx2, rx2): (Sender<VecDeque<Song>>, Receiver<VecDeque<Song>>) = mpsc::channel();

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
            time_pos: 0,
            lyric_line: String::new(),
            youtube_options: YoutubeOptions::new(),
            sender: tx,
            receiver: rx,
            sender_playlist_items: tx2,
            receiver_playlist_items: rx2,
            #[cfg(feature = "cover")]
            ueberzug: RwLock::new(None),
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

    fn init_app(tree: &Tree) -> Application<Id, Msg, NoUserEvent> {
        // Setup application
        // NOTE: NoUserEvent is a shorthand to tell tui-realm we're not going to use any custom user event
        // NOTE: the event listener is configured to use the default crossterm input listener and to raise a Tick event each second
        // which we will use to update the clock

        let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
            EventListenerCfg::default()
                .default_input_listener(Duration::from_millis(30))
                .poll_timeout(Duration::from_millis(1000))
                .tick_interval(Duration::from_secs(1)),
        );
        assert!(app
            .mount(Id::Library, Box::new(MusicLibrary::new(tree, None)), vec![])
            .is_ok());
        assert!(app
            .mount(Id::Playlist, Box::new(Playlist::default()), vec![])
            .is_ok());
        assert!(app
            .mount(Id::Progress, Box::new(Progress::default()), vec![])
            .is_ok());
        assert!(app
            .mount(Id::Lyric, Box::new(Lyric::default()), vec![])
            .is_ok());
        assert!(app
            .mount(
                Id::Label,
                Box::new(
                    Label::default()
                        .text(format!("Press <CTRL+H> for help. Version: {}", VERSION,))
                        .alignment(Alignment::Left)
                        .background(Color::Reset)
                        .foreground(Color::Cyan)
                        .modifiers(TextModifiers::BOLD),
                ),
                Vec::default(),
            )
            .is_ok());
        // Mount counters
        assert!(app
            .mount(
                Id::GlobalListener,
                Box::new(GlobalListener::default()),
                Self::subscribe(),
            )
            .is_ok());
        // Active letter counter
        assert!(app.active(&Id::Library).is_ok());
        app
    }

    pub fn view(&mut self) {
        if self.redraw {
            self.redraw = false;
            self.last_redraw = Instant::now();
            assert!(self
                .terminal
                .raw_mut()
                .draw(|f| {
                    let chunks_main = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
                        .split(f.size());
                    let chunks_left = Layout::default()
                        .direction(Direction::Horizontal)
                        .margin(0)
                        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)].as_ref())
                        .split(chunks_main[0]);
                    let chunks_right = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints(
                            [
                                Constraint::Min(2),
                                Constraint::Length(3),
                                Constraint::Length(4),
                            ]
                            .as_ref(),
                        )
                        .split(chunks_left[1]);

                    // app.view(&Id::Progress, f, chunks_right[1]);

                    self.app.view(&Id::Library, f, chunks_left[0]);
                    self.app.view(&Id::Playlist, f, chunks_right[0]);
                    self.app.view(&Id::Progress, f, chunks_right[1]);
                    self.app.view(&Id::Lyric, f, chunks_right[2]);
                    self.app.view(&Id::Label, f, chunks_main[1]);
                    // -- popups
                    if self.app.mounted(&Id::QuitPopup) {
                        let popup = draw_area_in(f.size(), 30, 10);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::QuitPopup, f, popup);
                    } else if self.app.mounted(&Id::ErrorPopup) {
                        let popup = draw_area_in(f.size(), 50, 15);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::ErrorPopup, f, popup);
                    } else if self.app.mounted(&Id::HelpPopup) {
                        let popup = draw_area_in(f.size(), 60, 90);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::HelpPopup, f, popup);
                    } else if self.app.mounted(&Id::DeleteConfirmRadioPopup) {
                        let popup = draw_area_in(f.size(), 30, 10);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::DeleteConfirmRadioPopup, f, popup);
                    } else if self.app.mounted(&Id::DeleteConfirmInputPopup) {
                        let popup = draw_area_in(f.size(), 30, 10);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::DeleteConfirmInputPopup, f, popup);
                    } else if self.app.mounted(&Id::LibrarySearchInput) {
                        let popup = draw_area_in(f.size(), 76, 60);
                        f.render_widget(Clear, popup);
                        let popup_chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints(
                                [
                                    Constraint::Length(3), // Input form
                                    Constraint::Min(2),    // Yes/No
                                ]
                                .as_ref(),
                            )
                            .split(popup);
                        self.app.view(&Id::LibrarySearchInput, f, popup_chunks[0]);
                        self.app.view(&Id::LibrarySearchTable, f, popup_chunks[1]);
                    } else if self.app.mounted(&Id::YoutubeSearchInputPopup) {
                        let popup = draw_area_in(f.size(), 30, 10);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::YoutubeSearchInputPopup, f, popup);
                    } else if self.app.mounted(&Id::YoutubeSearchTablePopup) {
                        let popup = draw_area_in(f.size(), 60, 70);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::YoutubeSearchTablePopup, f, popup);
                    }

                    if self.app.mounted(&Id::MessagePopup) {
                        let popup = draw_area_top_right(f.size(), 32, 15);
                        f.render_widget(Clear, popup);
                        self.app.view(&Id::MessagePopup, f, popup);
                    }
                })
                .is_ok());
        }
    }
    // Mount error and give focus to it
    pub fn mount_error_popup(&mut self, err: &str) {
        // pub fn mount_error_popup(&mut self, err: impl ToString) {
        assert!(self
            .app
            .remount(
                Id::ErrorPopup,
                Box::new(ErrorPopup::new(err.to_string())),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::ErrorPopup).is_ok());
        self.app.lock_subs();
    }
    /// Mount quit popup
    pub fn mount_quit_popup(&mut self) {
        assert!(self
            .app
            .remount(Id::QuitPopup, Box::new(QuitPopup::default()), vec![])
            .is_ok());
        assert!(self.app.active(&Id::QuitPopup).is_ok());
        self.app.lock_subs();
    }
    /// Mount help popup
    pub fn mount_help_popup(&mut self) {
        assert!(self
            .app
            .remount(Id::HelpPopup, Box::new(HelpPopup::default()), vec![])
            .is_ok());
        assert!(self.app.active(&Id::HelpPopup).is_ok());
        self.app.lock_subs();
    }

    pub fn mount_confirm_radio(&mut self) {
        assert!(self
            .app
            .remount(
                Id::DeleteConfirmRadioPopup,
                Box::new(DeleteConfirmRadioPopup::default()),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::DeleteConfirmRadioPopup).is_ok());
        self.app.lock_subs();
    }

    pub fn mount_confirm_input(&mut self) {
        assert!(self
            .app
            .remount(
                Id::DeleteConfirmInputPopup,
                Box::new(DeleteConfirmInputPopup::default()),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::DeleteConfirmInputPopup).is_ok());
        self.app.lock_subs();
    }

    pub fn mount_search_library(&mut self) {
        assert!(self
            .app
            .remount(
                Id::LibrarySearchInput,
                Box::new(LibrarySearchInputPopup::default()),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::LibrarySearchTable,
                Box::new(LibrarySearchTablePopup::default()),
                vec![]
            )
            .is_ok());

        assert!(self.app.active(&Id::LibrarySearchInput).is_ok());
        self.app.lock_subs();
    }

    pub fn mount_youtube_search_input(&mut self) {
        assert!(self
            .app
            .remount(
                Id::YoutubeSearchInputPopup,
                Box::new(YoutubeSearchInputPopup::default()),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::YoutubeSearchInputPopup).is_ok());
        self.app.lock_subs();
    }

    pub fn mount_youtube_search_table(&mut self) {
        assert!(self
            .app
            .remount(
                Id::YoutubeSearchTablePopup,
                Box::new(YoutubeSearchTablePopup::default()),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::YoutubeSearchTablePopup).is_ok());
        self.app.lock_subs();
    }

    pub fn next_song(&mut self) {
        if self.playlist_items.is_empty() {
            return;
        }
        self.time_pos = 0;
        if let Some(song) = self.playlist_items.pop_front() {
            if let Some(file) = song.file() {
                self.player.add_and_play(file);
            }
            match self.config.loop_mode {
                Loop::Playlist => self.playlist_items.push_back(song.clone()),
                Loop::Single => self.playlist_items.push_front(song.clone()),
                Loop::Queue => {}
            }
            self.sync_playlist();
            self.current_song = Some(song);
            self.update_photo();
            self.update_progress_title();
            self.update_playing_song();
        }
    }

    pub fn previous_song(&mut self) {
        if let Loop::Single | Loop::Queue = self.config.loop_mode {
            return;
        }

        if self.playlist_items.is_empty() {
            return;
        }

        if let Some(song) = self.playlist_items.pop_back() {
            self.playlist_items.push_front(song);
        }
        if let Some(song) = self.playlist_items.pop_back() {
            self.playlist_items.push_front(song);
        }
        self.next_song();
    }

    pub fn play_pause(&mut self) {
        if self.player.is_paused() {
            self.status = Some(Status::Running);
            self.player.resume();
        } else {
            self.status = Some(Status::Paused);
            self.player.pause();
        }
    }

    pub fn seek(&mut self, offset: i64) {
        self.player.seek(offset).ok();
        self.update_progress();
    }
    // change status bar text to indicate the downloading state
    pub fn update_components(&mut self) {
        if let Ok(update_components_state) = self.receiver.try_recv() {
            match update_components_state {
                UpdateComponents::DownloadRunning => {
                    self.update_status_line(StatusLine::Running);
                }
                UpdateComponents::DownloadSuccess => {
                    self.update_status_line(StatusLine::Success);
                }
                UpdateComponents::DownloadCompleted(Some(file)) => {
                    self.sync_library(Some(file.as_str()));
                    self.update_status_line(StatusLine::Default);
                }
                UpdateComponents::DownloadCompleted(None) => {
                    self.sync_library(None);
                    self.update_status_line(StatusLine::Default);
                }
                UpdateComponents::DownloadErrDownload(error_message) => {
                    self.mount_error_popup(format!("download failed: {}", error_message).as_str());
                    self.update_status_line(StatusLine::Error);
                }
                UpdateComponents::DownloadErrEmbedData => {
                    // This case will not happen in main activity
                }
                UpdateComponents::YoutubeSearchSuccess(y) => {
                    self.youtube_options = y;
                    self.sync_youtube_options();
                    self.redraw = true;
                }
                UpdateComponents::YoutubeSearchFail(e) => {
                    self.mount_error_popup(&e);
                }
                UpdateComponents::MessageShow((title, text)) => {
                    self.mount_message(&title, &text);
                }
                UpdateComponents::MessageHide((title, text)) => {
                    self.umount_message(&title, &text);
                } // _ => {}
            }
        };
    }

    // change status bar text to indicate the downloading state
    pub fn update_status_line(&mut self, s: StatusLine) {
        match s {
            StatusLine::Default => {
                let text = format!("Press <CTRL+H> for help. Version: {}", crate::VERSION);
                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Text, AttrValue::String(text))
                    .is_ok());
                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Color, AttrValue::Color(Color::Cyan))
                    .is_ok());
                assert!(self
                    .app
                    .attr(
                        &Id::Label,
                        Attribute::Background,
                        AttrValue::Color(Color::Reset)
                    )
                    .is_ok());
            }
            StatusLine::Running => {
                let text = " Downloading...".to_string();
                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Text, AttrValue::String(text))
                    .is_ok());
                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Color, AttrValue::Color(Color::Black))
                    .is_ok());
                assert!(self
                    .app
                    .attr(
                        &Id::Label,
                        Attribute::Background,
                        AttrValue::Color(Color::Yellow)
                    )
                    .is_ok());
            }
            StatusLine::Success => {
                let text = " Download Success!".to_string();

                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Text, AttrValue::String(text))
                    .is_ok());
                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Color, AttrValue::Color(Color::Black))
                    .is_ok());
                assert!(self
                    .app
                    .attr(
                        &Id::Label,
                        Attribute::Background,
                        AttrValue::Color(Color::Green)
                    )
                    .is_ok());
            }
            StatusLine::Error => {
                let text = " Download Error!".to_string();

                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Text, AttrValue::String(text))
                    .is_ok());
                assert!(self
                    .app
                    .attr(&Id::Label, Attribute::Color, AttrValue::Color(Color::White))
                    .is_ok());
                assert!(self
                    .app
                    .attr(
                        &Id::Label,
                        Attribute::Background,
                        AttrValue::Color(Color::Red)
                    )
                    .is_ok());
            }
        }
    }
    // update playlist items when loading
    pub fn update_playlist_items(&mut self) {
        if let Ok(playlist_items) = self.receiver_playlist_items.try_recv() {
            self.playlist_items = playlist_items;
            self.sync_playlist();
            self.redraw = true;
        }
    }

    // show a popup for playing song
    pub fn update_playing_song(&self) {
        if let Some(song) = &self.current_song {
            let name = song.name().unwrap_or("Unknown Song");
            self.show_message_timeout("Current Playing", name, None);
        }
    }

    pub fn show_message_timeout(&self, title: &str, text: &str, time_out: Option<u64>) {
        let tx = self.sender.clone();
        let title_string = title.to_string();
        let text_string = text.to_string();
        let time_out = time_out.unwrap_or(5);

        thread::spawn(move || {
            tx.send(UpdateComponents::MessageShow((
                title_string.clone(),
                text_string.clone(),
            )))
            .ok();
            sleep(Duration::from_secs(time_out));
            tx.send(UpdateComponents::MessageHide((title_string, text_string)))
                .ok();
        });
    }

    pub fn mount_message(&mut self, title: &str, text: &str) {
        assert!(self
            .app
            .remount(
                Id::MessagePopup,
                Box::new(MessagePopup::new(title, text)),
                vec![]
            )
            .is_ok());
        // assert!(self.app.active(&Id::ErrorPopup).is_ok());
    }

    /// ### `umount_message`
    ///
    /// Umount error message
    pub fn umount_message(&mut self, _title: &str, text: &str) {
        // Text(Payload(Vec(TextSpan))): set paragraph text
        if let Ok(Some(AttrValue::Payload(PropPayload::Vec(spans)))) =
            self.app.query(&Id::MessagePopup, Attribute::Text)
        {
            if let Some(display_text) = spans.get(0) {
                let d = display_text.clone().unwrap_text_span().content;
                if text.eq(&d) {
                    self.app.umount(&Id::MessagePopup).ok();
                }
            }
        }

        // if let Some(props) = self.view.get_props(COMPONENT_TEXT_MESSAGE) {
        //     if let Some(PropPayload::Vec(spans)) = props.own.get("spans") {
        //         if let Some(display_text) = spans.get(0) {
        //             if text == display_text.unwrap_text_span().content {
        //                 self.view.umount(COMPONENT_TEXT_MESSAGE);
        //             }
        //         }
        //     }
        // }
    }

    // fn update_duration(&mut self) {
    //     let (_new_prog, _time_pos, duration) = self.player.get_progress();
    //     if let Some(song) = &mut self.current_song {
    //         let diff = song.duration().as_secs().checked_sub(duration as u64);
    //         if let Some(d) = diff {
    //             if d > 1 {
    //                 let _drop = song.update_duration();
    //             }
    //         } else {
    //             let _drop = song.update_duration();
    //         }
    //     }
    // }
}
