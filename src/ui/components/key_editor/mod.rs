mod ke_input;
mod ke_select;
use crate::ui::{KEMsg, Msg};
pub use ke_input::KEGlobalQuitInput;
pub use ke_select::KEGlobalQuit;
use serde::{Deserialize, Serialize};
use tui_realm_stdlib::{Radio, Table};
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::props::{Alignment, BorderType, Borders, Color, TableBuilder, TextSpan};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers, NoUserEvent},
    Component, Event, MockComponent, State, StateValue,
};

#[derive(Clone, Deserialize, Serialize)]
pub struct Keys {
    pub global_esc: KeyBind,
    pub global_quit: KeyBind,
    pub global_left: KeyBind,
    pub global_down: KeyBind,
    pub global_up: KeyBind,
    pub global_right: KeyBind,
    pub global_goto_top: KeyBind,
    pub global_goto_bottom: KeyBind,
    pub global_player_toggle_pause: KeyBind,
    pub global_player_next: KeyBind,
    pub global_player_previous: KeyBind,
    pub global_player_volume_plus_1: KeyBind,
    pub global_player_volume_plus_2: KeyBind,
    pub global_player_volume_minus_1: KeyBind,
    pub global_player_volume_minus_2: KeyBind,
    pub global_help: KeyBind,
    pub global_player_seek_forward: KeyBind,
    pub global_player_seek_backward: KeyBind,
    pub global_lyric_adjust_forward: KeyBind,
    pub global_lyric_adjust_backward: KeyBind,
    pub global_lyric_cycle: KeyBind,
    pub global_color_editor_open: KeyBind,
    pub global_key_editor_open: KeyBind,
    pub library_load_dir: KeyBind,
    pub library_delete: KeyBind,
    pub library_yank: KeyBind,
    pub library_paste: KeyBind,
    pub library_search: KeyBind,
    pub library_search_youtube: KeyBind,
    pub library_tag_editor_open: KeyBind,
    pub playlist_delete: KeyBind,
    pub playlist_delete_all: KeyBind,
    pub playlist_shuffle: KeyBind,
    pub playlist_mode_cycle: KeyBind,
    pub playlist_play_selected: KeyBind,
    pub playlist_add_front: KeyBind,
    pub playlist_search: KeyBind,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct KeyBind {
    code: Key,
    modifiers: KeyModifiers,
}

impl std::fmt::Display for KeyBind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let code_string = if let Key::Char(char) = self.code {
            char.to_string()
        } else {
            format!("{:?}", self.code)
        };
        match self.modifiers {
            KeyModifiers::NONE => write!(f, "{}", code_string),
            KeyModifiers::SHIFT => write!(f, "{}", code_string.to_uppercase()),
            KeyModifiers::CONTROL => write!(f, "CTRL+{}", code_string),
            KeyModifiers::ALT => write!(f, "ALT+{}", code_string),
            _ => write!(f, "Wrong Modifiers"),
        }
    }
}

impl KeyBind {
    pub const fn key_event(&self) -> KeyEvent {
        KeyEvent {
            code: self.code,
            modifiers: self.modifiers,
        }
    }
    pub const fn modifier(&self) -> usize {
        match self.modifiers {
            KeyModifiers::NONE => 0,
            KeyModifiers::SHIFT => 1,
            KeyModifiers::CONTROL => 2,
            KeyModifiers::ALT => 3,
            _ => 0,
        }
    }

    pub fn key(&self) -> String {
        match self.code {
            Key::Backspace => "Backspace".to_string(),
            Key::Enter => "Enter".to_string(),
            Key::Left => "Left".to_string(),
            Key::Right => "Right".to_string(),
            Key::Up => "Up".to_string(),
            Key::Down => "Down".to_string(),
            Key::Home => "Home".to_string(),
            Key::End => "End".to_string(),
            Key::PageUp => "PageUp".to_string(),
            Key::PageDown => "PageDown".to_string(),
            Key::Tab => "Tab".to_string(),
            Key::BackTab => "BackTab".to_string(),
            Key::Delete => "Delete".to_string(),
            Key::Insert => "Insert".to_string(),
            Key::Function(int) => format!("F{}", int),
            Key::Char(char) => format!("{}", char),
            Key::Null => "Null".to_string(),
            Key::Esc => "Esc".to_string(),
        }
    }
}

impl Default for Keys {
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        Self {
            global_esc: KeyBind {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            },
            global_quit: KeyBind {
                code: Key::Char('q'),
                modifiers: KeyModifiers::NONE,
            },
            global_left: KeyBind {
                code: Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            },
            global_down: KeyBind {
                code: Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            },
            global_up: KeyBind {
                code: Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            },
            global_right: KeyBind {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            },
            global_goto_top: KeyBind {
                code: Key::Char('g'),
                modifiers: KeyModifiers::NONE,
            },
            global_goto_bottom: KeyBind {
                code: Key::Char('G'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_player_toggle_pause: KeyBind {
                code: Key::Char(' '),
                modifiers: KeyModifiers::NONE,
            },
            global_player_next: KeyBind {
                code: Key::Char('n'),
                modifiers: KeyModifiers::NONE,
            },
            global_player_previous: KeyBind {
                code: Key::Char('N'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_player_volume_plus_1: KeyBind {
                code: Key::Char('+'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_player_volume_plus_2: KeyBind {
                code: Key::Char('='),
                modifiers: KeyModifiers::NONE,
            },
            global_player_volume_minus_1: KeyBind {
                code: Key::Char('_'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_player_volume_minus_2: KeyBind {
                code: Key::Char('-'),
                modifiers: KeyModifiers::NONE,
            },
            global_help: KeyBind {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            },
            global_player_seek_forward: KeyBind {
                code: Key::Char('f'),
                modifiers: KeyModifiers::NONE,
            },
            global_player_seek_backward: KeyBind {
                code: Key::Char('b'),
                modifiers: KeyModifiers::NONE,
            },
            global_lyric_adjust_forward: KeyBind {
                code: Key::Char('F'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_lyric_adjust_backward: KeyBind {
                code: Key::Char('B'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_lyric_cycle: KeyBind {
                code: Key::Char('T'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_color_editor_open: KeyBind {
                code: Key::Char('C'),
                modifiers: KeyModifiers::SHIFT,
            },
            global_key_editor_open: KeyBind {
                code: Key::Char('K'),
                modifiers: KeyModifiers::SHIFT,
            },
            library_load_dir: KeyBind {
                code: Key::Char('L'),
                modifiers: KeyModifiers::SHIFT,
            },
            library_delete: KeyBind {
                code: Key::Char('d'),
                modifiers: KeyModifiers::NONE,
            },
            library_yank: KeyBind {
                code: Key::Char('y'),
                modifiers: KeyModifiers::NONE,
            },
            library_paste: KeyBind {
                code: Key::Char('p'),
                modifiers: KeyModifiers::NONE,
            },
            library_search: KeyBind {
                code: Key::Char('/'),
                modifiers: KeyModifiers::NONE,
            },
            library_search_youtube: KeyBind {
                code: Key::Char('s'),
                modifiers: KeyModifiers::NONE,
            },
            library_tag_editor_open: KeyBind {
                code: Key::Char('t'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_delete: KeyBind {
                code: Key::Char('d'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_delete_all: KeyBind {
                code: Key::Char('D'),
                modifiers: KeyModifiers::SHIFT,
            },
            playlist_shuffle: KeyBind {
                code: Key::Char('s'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_mode_cycle: KeyBind {
                code: Key::Char('m'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_play_selected: KeyBind {
                code: Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_add_front: KeyBind {
                code: Key::Char('a'),
                modifiers: KeyModifiers::NONE,
            },
            playlist_search: KeyBind {
                code: Key::Char('/'),
                modifiers: KeyModifiers::NONE,
            },
        }
    }
}

#[derive(MockComponent)]
pub struct KERadioOk {
    component: Radio,
}
impl Default for KERadioOk {
    fn default() -> Self {
        Self {
            component: Radio::default()
                .foreground(Color::Yellow)
                // .background(Color::Black)
                .borders(
                    Borders::default()
                        .color(Color::Yellow)
                        .modifiers(BorderType::Rounded),
                )
                // .title("Additional operation:", Alignment::Left)
                .rewind(true)
                .choices(&["Save and Close"])
                .value(0),
        }
    }
}

impl Component<Msg, NoUserEvent> for KERadioOk {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::KeyEditor(KEMsg::RadioOkBlurDown))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::KeyEditor(KEMsg::RadioOkBlurUp)),
            Event::Keyboard(KeyEvent {
                code: Key::Esc | Key::Char('q'),
                ..
            }) => return Some(Msg::KeyEditor(KEMsg::KeyEditorCloseCancel)),

            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => return Some(Msg::KeyEditor(KEMsg::HelpPopupShow)),
            // Event::Keyboard(KeyEvent {
            //     code: Key::Left | Key::Char('h' | 'j'),
            //     ..
            // }) => self.perform(Cmd::Move(Direction::Left)),
            // Event::Keyboard(KeyEvent {
            //     code: Key::Right | Key::Char('l' | 'k'),
            //     ..
            // }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => return None,
        };
        if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(0)))
        ) {
            return Some(Msg::KeyEditor(KEMsg::KeyEditorCloseOk));
        }
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct KEHelpPopup {
    component: Table,
}

impl Default for KEHelpPopup {
    fn default() -> Self {
        Self {
            component: Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Green),
                )
                // .foreground(Color::Yellow)
                // .background(Color::Black)
                .title("Help: Esc or Enter to exit.", Alignment::Center)
                .scroll(false)
                // .highlighted_color(Color::LightBlue)
                // .highlighted_str("\u{1f680}")
                // .highlighted_str("🚀")
                // .rewind(true)
                .step(4)
                .row_height(1)
                .headers(&["Key", "Function"])
                .column_spacing(3)
                .widths(&[30, 70])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::new("<TAB>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Switch focus"))
                        .add_row()
                        .add_col(TextSpan::new("<ESC> or <q>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Exit without saving"))
                        .add_row()
                        .add_col(TextSpan::new("Theme Select").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(TextSpan::new("<h,j,k,l,g,G>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Move cursor(vim style)"))
                        .add_row()
                        .add_col(TextSpan::new("<ENTER>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("load a theme for preview"))
                        .add_row()
                        .add_col(TextSpan::new("Color Select").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(TextSpan::new("<ENTER>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("select a color"))
                        .add_row()
                        .add_col(TextSpan::new("<h,j>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Move cursor(vim style)"))
                        .add_row()
                        .add_col(
                            TextSpan::new("Highlight String")
                                .bold()
                                .fg(Color::LightYellow),
                        )
                        .add_row()
                        .add_col(TextSpan::new("").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("You can paste symbol, or input."))
                        .add_row()
                        .add_col(TextSpan::new("<ENTER>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("preview unicode symbol."))
                        .build(),
                ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEHelpPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter | Key::Esc,
                ..
            }) => Some(Msg::KeyEditor(KEMsg::HelpPopupClose)),
            _ => None,
        }
    }
}
