use crate::config::{Keys, Settings, StyleColorSymbol};
/**
 * MIT License
 *
 * tuifeed - Copyright (c) 2021 Christian Visintin
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
use crate::ui::Msg;
use tui_realm_stdlib::{Input, Paragraph, Radio, Table};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{
    Alignment, BorderType, Borders, Color, InputType, TableBuilder, TextModifiers, TextSpan,
};
use tuirealm::{Component, Event, MockComponent, State, StateValue};

#[derive(MockComponent)]
pub struct QuitPopup {
    component: Radio,
    keys: Keys,
}

impl QuitPopup {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Radio::default()
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::Yellow),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .title("Are sure you want to quit?", Alignment::Center)
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0),
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for QuitPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),

            Event::Keyboard(key) if key == self.keys.global_left.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.keys.global_right.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.keys.global_quit.key_event() => {
                return Some(Msg::QuitPopupCloseCancel)
            }
            Event::Keyboard(key) if key == self.keys.global_esc.key_event() => {
                return Some(Msg::QuitPopupCloseCancel)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => return None,
        };
        if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(0)))
        ) {
            Some(Msg::QuitPopupCloseCancel)
        } else if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(1)))
        ) {
            Some(Msg::QuitPopupCloseOk)
        } else {
            Some(Msg::None)
        }
    }
}

#[derive(MockComponent)]
pub struct ErrorPopup {
    component: Paragraph,
}

impl ErrorPopup {
    pub fn new<S: AsRef<str>>(msg: S) -> Self {
        Self {
            component: Paragraph::default()
                .borders(
                    Borders::default()
                        .color(Color::Red)
                        .modifiers(BorderType::Rounded),
                )
                .foreground(Color::Red)
                // .background(Color::Black)
                .modifiers(TextModifiers::BOLD)
                .alignment(Alignment::Center)
                .text(vec![TextSpan::from(msg.as_ref().to_string())].as_slice()),
        }
    }
}

impl Component<Msg, NoUserEvent> for ErrorPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter | Key::Esc,
                ..
            }) => Some(Msg::ErrorPopupClose),
            _ => None,
        }
    }
}

#[derive(MockComponent)]
pub struct HelpPopup {
    component: Table,
    keys: Keys,
}

impl HelpPopup {
    #[allow(clippy::too_many_lines)]
    pub fn new(config: &Settings) -> Self {
        let keys = &config.keys;
        let key_quit = format!("<{}> or <{}>", keys.global_esc, keys.global_quit);
        let key_movement = format!(
            "<{},{},{},{},{},{}>",
            keys.global_left,
            keys.global_down,
            keys.global_up,
            keys.global_right,
            keys.global_goto_top,
            keys.global_goto_bottom
        );
        let key_player_seek = format!(
            "<{}/{}>",
            keys.global_player_seek_forward, keys.global_player_seek_backward
        );
        let key_player_speed = format!(
            "<{}/{}>",
            keys.global_player_speed_up, keys.global_player_speed_down
        );
        let key_lyric_adjust = format!(
            "<{}/{}>",
            keys.global_lyric_adjust_forward, keys.global_lyric_adjust_backward
        );
        let key_player = format!(
            "<{}/{}/{}>",
            keys.global_player_next, keys.global_player_previous, keys.global_player_toggle_pause,
        );
        let key_volume = format!(
            "<{},{}/{},{}>",
            keys.global_player_volume_plus_1,
            keys.global_player_volume_plus_2,
            keys.global_player_volume_minus_1,
            keys.global_player_volume_minus_2,
        );
        let key_playlist_swap = format!("<{}/{}>", keys.playlist_swap_down, keys.playlist_swap_up,);

        Self {
            component: Table::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Green),
                    ),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Black),
                )
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
                .scroll(true)
                .title("Help: Esc or Enter to exit.", Alignment::Center)
                .rewind(false)
                .step(4)
                .row_height(1)
                .headers(&["Key", "Function"])
                .column_spacing(3)
                .widths(&[30, 70])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::new(key_quit).bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Exit"))
                        .add_row()
                        .add_col(TextSpan::new("<TAB>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Switch focus"))
                        .add_row()
                        .add_col(TextSpan::new(key_movement).bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Move cursor(vim style by default)"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.global_config_open).as_str())
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Open configuration dialogue"))
                        .add_row()
                        .add_col(TextSpan::new(key_player_seek).bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Seek forward/backward 5 seconds"))
                        .add_row()
                        .add_col(
                            TextSpan::new(key_lyric_adjust.as_str())
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Seek forward/backward 1 second for lyrics"))
                        .add_row()
                        .add_col(
                            TextSpan::new(key_player_speed.as_str())
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Playback speed up/down 10 percent"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.global_player_toggle_gapless))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Toggle gapless playback"))
                        .add_row()
                        .add_col(TextSpan::new(key_lyric_adjust).bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Before 10 seconds,adjust offset of lyrics"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.global_lyric_cycle))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Switch lyrics if more than 1 available"))
                        .add_row()
                        .add_col(TextSpan::new(key_player).bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Next/Previous/Pause current track"))
                        .add_row()
                        .add_col(TextSpan::new(key_volume).bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Increase/Decrease volume"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.global_config_open))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Open Config Editor(all configuration)"))
                        .add_row()
                        .add_col(TextSpan::new("Library").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!(
                                "<{}/{}>",
                                keys.global_right, keys.library_load_dir
                            ))
                            .bold()
                            .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Add one/all tracks to playlist"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.library_delete))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Delete track or folder"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.library_search_youtube))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Download or search track from youtube"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.library_tag_editor_open))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Open tag editor for tag and lyric download"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!(
                                "<{}/{}>",
                                keys.library_yank, keys.library_paste
                            ))
                            .bold()
                            .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Yank and Paste files"))
                        .add_row()
                        .add_col(TextSpan::new("<Enter>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Open sub directory as root"))
                        .add_row()
                        .add_col(TextSpan::new("<Backspace>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Go back to parent directory"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.library_search))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Search in library"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.library_switch_root))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Switch among several root folders"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.library_add_root))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Add new root folder"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.library_remove_root))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Remove current root from root folder list"))
                        .add_row()
                        .add_col(TextSpan::new("Playlist").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!(
                                "<{}/{}>",
                                keys.playlist_delete, keys.playlist_delete_all
                            ))
                            .bold()
                            .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Delete one/all tracks from playlist"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.playlist_play_selected))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Play selected"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.playlist_shuffle))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Randomize playlist"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.playlist_mode_cycle))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Loop mode toggle"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.playlist_add_front))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from(
                            "Add a track to the front of playlist or back",
                        ))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.playlist_search))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Search in playlist"))
                        .add_row()
                        .add_col(
                            TextSpan::new(key_playlist_swap.as_str())
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Swap track down/up in playlist"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!(
                                "<{}>/<{}>",
                                keys.playlist_cmus_tqueue, keys.playlist_cmus_lqueue
                            ))
                            .bold()
                            .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Select random tracks/albums to playlist"))
                        .add_row()
                        .add_col(TextSpan::new("Database").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!(
                                "<{}/{}>",
                                keys.global_right, keys.database_add_all
                            ))
                            .bold()
                            .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Add one/all track(s) to playlist"))
                        .add_row()
                        .add_col(
                            TextSpan::new(format!("<{}>", keys.library_search))
                                .bold()
                                .fg(Color::Cyan),
                        )
                        .add_col(TextSpan::from("Search in database"))
                        .build(),
                ),
            keys: keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for HelpPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => return Some(Msg::HelpPopupClose),

            Event::Keyboard(key) if key == self.keys.global_quit.key_event() => {
                return Some(Msg::HelpPopupClose)
            }
            Event::Keyboard(key) if key == self.keys.global_esc.key_event() => {
                return Some(Msg::HelpPopupClose)
            }

            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            _ => CmdResult::None,
        };

        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct DeleteConfirmRadioPopup {
    component: Radio,
    keys: Keys,
}

impl DeleteConfirmRadioPopup {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Radio::default()
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::LightRed),
                )
                // .background(Color::Black)
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::LightRed),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .title("Are sure you want to delete?", Alignment::Left)
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0),
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for DeleteConfirmRadioPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),

            Event::Keyboard(key) if key == self.keys.global_left.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.keys.global_right.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.keys.global_quit.key_event() => {
                return Some(Msg::DeleteConfirmCloseCancel)
            }
            Event::Keyboard(key) if key == self.keys.global_esc.key_event() => {
                return Some(Msg::DeleteConfirmCloseCancel)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => return None,
        };
        if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(0)))
        ) {
            Some(Msg::DeleteConfirmCloseCancel)
        } else if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(1)))
        ) {
            Some(Msg::DeleteConfirmCloseOk)
        } else {
            Some(Msg::None)
        }
    }
}

#[derive(MockComponent)]
pub struct DeleteConfirmInputPopup {
    component: Input,
}

impl DeleteConfirmInputPopup {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: Input::default()
                .foreground(
                    style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .background(
                    style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .borders(
                    Borders::default()
                        .color(style_color_symbol.library_border().unwrap_or(Color::Green))
                        .modifiers(BorderType::Rounded),
                )
                // .invalid_style(Style::default().fg(Color::Red))
                .input_type(InputType::Text)
                .title("Type DELETE to confirm:", Alignment::Left),
        }
    }
}

impl Component<Msg, NoUserEvent> for DeleteConfirmInputPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => self.perform(Cmd::Delete),
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE,
            }) => self.perform(Cmd::Type(ch)),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::DeleteConfirmCloseCancel);
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::String(input_string))) => {
                if input_string == *"DELETE" {
                    return Some(Msg::DeleteConfirmCloseOk);
                }
                Some(Msg::DeleteConfirmCloseCancel)
            }
            _ => Some(Msg::None),
        }

        // if cmd_result == CmdResult::Submit(State::One(StateValue::String("DELETE".to_string()))) {
        //     Some(Msg::DeleteConfirmCloseOk)
        // } else {
        //     Some(Msg::DeleteConfirmCloseCancel)
        // }
    }
}

#[derive(MockComponent)]
pub struct MessagePopup {
    component: Paragraph,
}

impl MessagePopup {
    pub fn new<S: AsRef<str>>(title: S, msg: S) -> Self {
        Self {
            component: Paragraph::default()
                .borders(
                    Borders::default()
                        .color(Color::Cyan)
                        .modifiers(BorderType::Rounded),
                )
                .foreground(Color::Green)
                // .background(Color::Black)
                .modifiers(TextModifiers::BOLD)
                .alignment(Alignment::Center)
                .title(title, Alignment::Center)
                .text(vec![TextSpan::from(msg.as_ref().to_string())].as_slice()),
        }
    }
}

impl Component<Msg, NoUserEvent> for MessagePopup {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        // match ev {
        //     _ => None,
        // }
        None
    }
}
