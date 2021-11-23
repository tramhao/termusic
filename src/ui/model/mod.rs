//! ## Model
//!
//! app model
/**
 * MIT License
 *
 * tui-realm - Copyright (C) 2021 Christian Visintin
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
use crate::{
    config::Termusic,
    song::Song,
    ui::{Application, Id, Msg},
    VERSION,
};

use crate::player::GStreamer;
use crate::ui::components::{
    draw_area_in, draw_area_top_right, ErrorPopup, GlobalListener, Label, Lyric, MusicLibrary,
    Playlist, Progress, QuitPopup,
};
use crate::ui::{Loop, Status};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tui_realm_treeview::Tree;
use tuirealm::props::{Alignment, Color, TextModifiers};
use tuirealm::terminal::TerminalBridge;
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::tui::widgets::Clear;
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers},
    EventListenerCfg, NoUserEvent, Sub, SubClause, SubEventClause, Update,
};

pub const MAX_DEPTH: usize = 3;

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
}

impl Model {
    pub fn new(config: &Termusic) -> Self {
        let full_path = shellexpand::tilde(&config.music_dir);
        let p: &Path = Path::new(full_path.as_ref());
        let tree = Tree::new(Self::dir_tree(p, MAX_DEPTH));

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
            player: GStreamer::new(),
            status: None,
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
    /// global listener subscriptions
    fn subs() -> Vec<Sub<NoUserEvent>> {
        vec![
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Esc,
                    modifiers: KeyModifiers::NONE,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('Q'),
                    modifiers: KeyModifiers::SHIFT,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('p'),
                    modifiers: KeyModifiers::NONE,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char(' '),
                    modifiers: KeyModifiers::NONE,
                }),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('n'),
                    modifiers: KeyModifiers::NONE,
                }),
                SubClause::Always,
            ),
        ]
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
                .default_input_listener(Duration::from_millis(20))
                .poll_timeout(Duration::from_millis(10))
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
                // Vec::new(),
                Self::subs(),
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
                    }
                })
                .is_ok());
        }
    }
    // Mount error and give focus to it
    pub fn mount_error_popup(&mut self, err: impl ToString) {
        assert!(self
            .app
            .remount(
                Id::ErrorPopup,
                Box::new(ErrorPopup::new(err.to_string())),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::ErrorPopup).is_ok());
    }
    /// Mount quit popup
    pub fn mount_quit_popup(&mut self) {
        assert!(self
            .app
            .remount(Id::QuitPopup, Box::new(QuitPopup::default()), vec![])
            .is_ok());
        assert!(self.app.active(&Id::QuitPopup).is_ok());
    }
    pub fn next_song(&mut self) {
        if self.playlist_items.is_empty() {
            return;
        }
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
            // self.current_song = Some(song);
            // self.sync_playlist();
            // self.update_photo();
            // self.update_progress_title();
            // self.update_duration();
            // self.update_playing_song();
        }
    }
}

// Let's implement Update for model

impl Update<Msg> for Model {
    #[allow(clippy::too_many_lines)]
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            // Set redraw
            self.redraw = true;
            // Match message
            match msg {
                Msg::QuitPopupShow => {
                    self.mount_quit_popup();
                    None
                }
                Msg::QuitPopupClose => {
                    let _ = self.app.umount(&Id::QuitPopup);
                    None
                }
                Msg::QuitPopupCloseQuit => {
                    self.quit = true;
                    None
                }
                Msg::LibraryTreeBlur => {
                    // Give focus to letter counter
                    assert!(self.app.active(&Id::Playlist).is_ok());
                    None
                }
                Msg::PlaylistTableBlur => {
                    assert!(self.app.active(&Id::Library).is_ok());
                    None
                }
                Msg::LibraryTreeExtendDir(path) => {
                    self.extend_dir(&path, PathBuf::from(path.as_str()).as_path(), MAX_DEPTH);
                    self.reload_tree();
                    None
                }
                Msg::LibraryTreeGoToUpperDir => {
                    if let Some(parent) = self.upper_dir() {
                        self.scan_dir(parent.as_path());
                        self.reload_tree();
                    }
                    None
                }
                Msg::PlaylistAdd(current_node) => {
                    if let Err(e) = self.add_playlist(&current_node) {
                        self.mount_error_popup(format!("Application error: {}", e));
                    }
                    None
                } // _ => None,
                Msg::PlaylistDelete(index) => {
                    self.delete_item_playlist(index);
                    None
                }
                Msg::PlaylistDeleteAll => {
                    self.empty_playlist();
                    None
                }
                Msg::PlaylistShuffle => {
                    self.shuffle();
                    None
                }
                Msg::ErrorPopupClose => {
                    let _ = self.app.umount(&Id::ErrorPopup);
                    None
                }
                Msg::PlaylistLoopModeCycle => {
                    self.cycle_loop_mode();
                    None
                }
                Msg::PlayerTogglePause => {
                    self.player.toggle_pause();
                    match self.status {
                        Some(Status::Running) => self.status = Some(Status::Paused),
                        Some(Status::Paused) => self.status = Some(Status::Running),
                        _ => {}
                    }
                    None
                }
                Msg::PlaylistNextSong => {
                    self.next_song();
                    None
                }

                Msg::None => None,
            }
        } else {
            None
        }
    }
}