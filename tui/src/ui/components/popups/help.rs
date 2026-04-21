use std::fmt::Write as _;

use termusiclib::config::v2::tui::keys::KeyBinding;
use termusiclib::config::v2::tui::theme::styles::ColorTermusic;
use termusiclib::config::{SharedTuiSettings, TuiOverlay};
use tui_realm_stdlib::components::Table;
use tui_realm_stdlib::prop_ext::CommonHighlight;
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::Event;
use tuirealm::props::{HorizontalAlignment, LineStatic, Title};
use tuirealm::{
    command::{Cmd, CmdResult, Direction, Position},
    event::{Key, KeyEvent, KeyModifiers},
    props::{BorderType, Borders, Style, TableBuilder},
};

use crate::ui::ids::Id;
use crate::ui::model::{Model, UserEvent};
use crate::ui::msg::{HelpPopupMsg, Msg};

#[derive(Component)]
pub struct HelpPopup {
    component: Table,
    config: SharedTuiSettings,
}

impl HelpPopup {
    /// Generate a consistent keybinding string from the given keybindings.
    fn key(config: &TuiOverlay, keys: &[&KeyBinding]) -> LineStatic {
        let mut text = String::new();
        for (idx, key) in keys.iter().enumerate() {
            if idx > 0 {
                text.push_str(", ");
            }
            let _ = write!(text, "<{key}>");
        }
        // TODO: make this somehow configurable; at least this now uses the actual theme instead of terminal
        LineStatic::styled(
            text,
            Style::new().bold().fg(config
                .settings
                .theme
                .get_color_from_theme(ColorTermusic::Cyan)),
        )
    }

    /// Generate a consistent key explanation comment.
    fn comment<T: Into<LineStatic>>(text: T) -> LineStatic {
        text.into()
    }

    /// Generate a consistent header element
    fn header<T: Into<LineStatic>>(config: &TuiOverlay, text: T) -> LineStatic {
        text.into().style(
            Style::new()
                .bold()
                .fg(config.settings.theme.library_highlight()),
        )
    }

    #[allow(clippy::too_many_lines)]
    pub fn new(config: SharedTuiSettings) -> Self {
        // The following cannot be constants due to ".into()" not being allowed in a constant context
        let focus_keys = &[
            &KeyEvent::new(Key::Tab, KeyModifiers::NONE).into(),
            &KeyEvent::new(Key::Tab, KeyModifiers::SHIFT).into(),
        ];
        let enter = &[&KeyEvent::new(Key::Enter, KeyModifiers::NONE).into()];
        let backspace = &[&KeyEvent::new(Key::Backspace, KeyModifiers::NONE).into()];

        let component = {
            let config = config.read();
            let keys = &config.settings.keys;
            Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(config.settings.theme.fallback_border()),
                )
                .inactive(Style::new().bg(config.settings.theme.library_background()))
                .foreground(config.settings.theme.fallback_foreground())
                .background(config.settings.theme.fallback_background())
                .highlight_style(
                    CommonHighlight::default()
                        .style
                        .fg(config.settings.theme.fallback_highlight()),
                )
                .highlight_str(config.settings.theme.style.library.highlight_symbol.clone())
                .scroll(true)
                .title(
                    Title::from(" Help: Esc or Enter to exit ")
                        .alignment(HorizontalAlignment::Center),
                )
                .rewind(false)
                .step(4)
                .row_height(1)
                .headers(["Key", "Function"])
                .column_spacing(3)
                .widths(&[40, 60])
                .table(
                    TableBuilder::default()
                        .add_col(Self::header(&config, "Global"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.escape, &keys.quit]))
                        .add_col(Self::comment("Exit"))
                        .add_row()
                        .add_col(Self::key(&config, focus_keys))
                        .add_col(Self::comment("Switch Focus"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[
                                &keys.navigation_keys.left,
                                &keys.navigation_keys.right,
                                &keys.navigation_keys.up,
                                &keys.navigation_keys.down,
                                &keys.navigation_keys.goto_top,
                                &keys.navigation_keys.goto_bottom,
                            ],
                        ))
                        .add_col(Self::comment("Move cursor(vim style by default)"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[
                                &keys.player_keys.seek_forward,
                                &keys.player_keys.seek_backward,
                            ],
                        ))
                        .add_col(Self::comment("Seek forward/backward 5 seconds"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[
                                &keys.lyric_keys.adjust_offset_forwards,
                                &keys.lyric_keys.adjust_offset_backwards,
                            ],
                        ))
                        .add_col(Self::comment("Seek forward/backward 1 second for lyrics"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[&keys.player_keys.speed_up, &keys.player_keys.speed_down],
                        ))
                        .add_col(Self::comment("Playback speed up/down 10 percent"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.player_keys.toggle_prefetch]))
                        .add_col(Self::comment("Toggle gapless playback"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[
                                &keys.lyric_keys.adjust_offset_forwards,
                                &keys.lyric_keys.adjust_offset_backwards,
                            ],
                        ))
                        .add_col(Self::comment("Before 10 seconds,adjust offset of lyrics"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.lyric_keys.cycle_frames]))
                        .add_col(Self::comment("Switch lyrics if more than 1 available"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[
                                &keys.player_keys.next_track,
                                &keys.player_keys.previous_track,
                                &keys.player_keys.toggle_pause,
                            ],
                        ))
                        .add_col(Self::comment("Next/Previous/Pause current track"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[&keys.player_keys.volume_up, &keys.player_keys.volume_down],
                        ))
                        .add_col(Self::comment("Increase/Decrease volume"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.select_view_keys.open_config]))
                        .add_col(Self::comment("Open Config Editor(all configuration)"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.player_keys.save_playlist]))
                        .add_col(Self::comment("Save Playlist to m3u"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.select_view_keys.view_library]))
                        .add_col(Self::comment("Switch layout to treeview"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.select_view_keys.view_database]))
                        .add_col(Self::comment("Switch layout to database"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.select_view_keys.view_podcasts]))
                        .add_col(Self::comment("Switch layout to podcast"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[
                                &keys.move_cover_art_keys.move_left,
                                &keys.move_cover_art_keys.move_right,
                            ],
                        ))
                        .add_col(Self::comment("Move album cover left/right"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[
                                &keys.move_cover_art_keys.move_up,
                                &keys.move_cover_art_keys.move_down,
                            ],
                        ))
                        .add_col(Self::comment("Move album cover up/down"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[
                                &keys.move_cover_art_keys.increase_size,
                                &keys.move_cover_art_keys.decrease_size,
                            ],
                        ))
                        .add_col(Self::comment("Zoom in/out album cover"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.move_cover_art_keys.toggle_hide]))
                        .add_col(Self::comment("Hide/Show album cover"))
                        .add_row()
                        .add_col(Self::header(&config, "Library"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[&keys.library_keys.load_track, &keys.library_keys.load_dir],
                        ))
                        .add_col(Self::comment("Add one/all tracks to playlist"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.library_keys.delete]))
                        .add_col(Self::comment("Delete track or folder"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.library_keys.youtube_search]))
                        .add_col(Self::comment("Search or download track from youtube"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.library_keys.open_tag_editor]))
                        .add_col(Self::comment("Open tag editor for tag and lyric download"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[&keys.library_keys.yank, &keys.library_keys.paste],
                        ))
                        .add_col(Self::comment("Yank and Paste files"))
                        .add_row()
                        .add_col(Self::key(&config, enter))
                        .add_col(Self::comment("Open sub directory as root"))
                        .add_row()
                        .add_col(Self::key(&config, backspace))
                        .add_col(Self::comment("Go back to parent directory"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.library_keys.search]))
                        .add_col(Self::comment("Search in library"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.library_keys.cycle_root]))
                        .add_col(Self::comment("Switch among several root folders"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.library_keys.add_root]))
                        .add_col(Self::comment("Add new root folder"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.library_keys.remove_root]))
                        .add_col(Self::comment("Remove current root from root folder list"))
                        .add_row()
                        .add_col(Self::header(&config, "Playlist"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[&keys.playlist_keys.delete, &keys.playlist_keys.delete_all],
                        ))
                        .add_col(Self::comment("Delete one/all tracks from playlist"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.playlist_keys.play_selected]))
                        .add_col(Self::comment("Play selected"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.playlist_keys.shuffle]))
                        .add_col(Self::comment("Randomize playlist"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.playlist_keys.cycle_loop_mode]))
                        .add_col(Self::comment("Loop mode cycle"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.playlist_keys.search]))
                        .add_col(Self::comment("Search in playlist"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[&keys.playlist_keys.swap_down, &keys.playlist_keys.swap_up],
                        ))
                        .add_col(Self::comment("Swap track down/up in playlist"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[
                                &keys.playlist_keys.add_random_songs,
                                &keys.playlist_keys.add_random_album,
                            ],
                        ))
                        .add_col(Self::comment("Select random tracks/albums to playlist"))
                        .add_row()
                        .add_col(Self::header(&config, "Database"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[
                                &keys.database_keys.add_selected,
                                &keys.database_keys.add_all,
                            ],
                        ))
                        .add_col(Self::comment("Add one/all track(s) to playlist"))
                        .add_row()
                        // TODO: add search key to database
                        .add_col(Self::key(&config, &[&keys.library_keys.search]))
                        .add_col(Self::comment("Search in database"))
                        .add_row()
                        .add_col(Self::header(&config, "Podcast"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.podcast_keys.search]))
                        .add_col(Self::comment("Feeds: search for new feeds"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[
                                &keys.podcast_keys.delete_feed,
                                &keys.podcast_keys.delete_all_feeds,
                            ],
                        ))
                        .add_col(Self::comment("Feeds : delete one/all feeds"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[
                                &keys.podcast_keys.refresh_feed,
                                &keys.podcast_keys.refresh_all_feeds,
                            ],
                        ))
                        .add_col(Self::comment("Feeds : refresh one/all feeds"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[
                                &keys.podcast_keys.mark_played,
                                &keys.podcast_keys.mark_all_played,
                            ],
                        ))
                        .add_col(Self::comment("Episode: Mark one/all episodes played"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.podcast_keys.download_episode]))
                        .add_col(Self::comment("Episode: Download episode"))
                        .add_row()
                        .add_col(Self::key(
                            &config,
                            &[&keys.podcast_keys.delete_local_episode],
                        ))
                        .add_col(Self::comment("Episode: delete episode local file"))
                        .add_row()
                        .add_col(Self::key(&config, &[&keys.library_keys.search]))
                        .add_col(Self::comment("Search through added Feeds / Episodes"))
                        .build(),
                )
        };

        Self { component, config }
    }
}

impl AppComponent<Msg, UserEvent> for HelpPopup {
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            // navigation
            Event::Keyboard(key) if key == keys.navigation_keys.down.get() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.up.get() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),
            // fast navigation
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == keys.navigation_keys.goto_top.get() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.goto_bottom.get() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::End)),

            // close popup
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::HelpPopup(HelpPopupMsg::Close)),

            Event::Keyboard(key) if key == keys.quit.get() => {
                return Some(Msg::HelpPopup(HelpPopupMsg::Close));
            }
            Event::Keyboard(key) if key == keys.escape.get() => {
                return Some(Msg::HelpPopup(HelpPopupMsg::Close));
            }

            _ => CmdResult::NoChange,
        };

        match cmd_result {
            CmdResult::NoChange => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

impl Model {
    /// Mount help popup
    pub fn mount_help_popup(&mut self) {
        assert!(
            self.app
                .remount(
                    Id::HelpPopup,
                    Box::new(HelpPopup::new(self.config_tui.clone())),
                    vec![]
                )
                .is_ok()
        );
        self.update_photo().ok();
        assert!(self.app.active(&Id::HelpPopup).is_ok());
    }
}
