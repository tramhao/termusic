mod ke_input;
mod ke_select;

use crate::ui::{Id, IdKeyEditor, KEMsg, Model, Msg};
use anyhow::{bail, Result};
pub use ke_input::*;
pub use ke_select::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tui_realm_stdlib::{Radio, Table};
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::props::{
    Alignment, AttrValue, Attribute, BorderType, Borders, Color, TableBuilder, TextSpan,
};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers, NoUserEvent},
    Component, Event, MockComponent, State, StateValue,
};

pub const CONTROL_SHIFT: KeyModifiers =
    KeyModifiers::from_bits_truncate(KeyModifiers::CONTROL.bits() | KeyModifiers::SHIFT.bits());
pub const ALT_SHIFT: KeyModifiers =
    KeyModifiers::from_bits_truncate(KeyModifiers::ALT.bits() | KeyModifiers::SHIFT.bits());
pub const CONTROL_ALT: KeyModifiers =
    KeyModifiers::from_bits_truncate(KeyModifiers::ALT.bits() | KeyModifiers::CONTROL.bits());
pub const CONTROL_ALT_SHIFT: KeyModifiers = KeyModifiers::from_bits_truncate(
    KeyModifiers::ALT.bits() | KeyModifiers::CONTROL.bits() | KeyModifiers::SHIFT.bits(),
);

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
    pub global_player_speed_up: KeyBind,
    pub global_player_speed_down: KeyBind,
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
    pub code: Key,
    pub modifiers: KeyModifiers,
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
            // KeyModifiers::NONE => 0,
            KeyModifiers::SHIFT => 1,
            KeyModifiers::CONTROL => 2,
            KeyModifiers::ALT => 3,
            CONTROL_SHIFT => 4,
            ALT_SHIFT => 5,
            CONTROL_ALT => 6,
            CONTROL_ALT_SHIFT => 7,
            _ => 0,
            // _ => 0,
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

    pub fn key_from_str(str: &str) -> Result<Key> {
        if str.starts_with('F') {
            let mut chars = str.chars();
            chars.next();
            // chars.as_str();
            let my_int = u8::from_str(chars.as_str()).unwrap_or(12);
            return Ok(Key::Function(my_int));
        }
        if str.len() < 2 {
            let mut chars = str.chars();
            let char = chars.next().unwrap();
            return Ok(Key::Char(char));
        }
        let str_lower_case = str.to_lowercase();
        let special_key = match str_lower_case.as_ref() {
            "backspace" => Key::Backspace,
            "enter" => Key::Enter,
            "left" => Key::Left,
            "right" => Key::Right,
            "up" => Key::Up,
            "down" => Key::Down,
            "home" => Key::Home,
            "end" => Key::End,
            "pageup" => Key::PageUp,
            "pagedown" => Key::PageDown,
            "tab" => Key::Tab,
            "backtab" => Key::BackTab,
            "delete" => Key::Delete,
            "insert" => Key::Insert,
            "esc" => Key::Esc,
            "null" => Key::Null,
            &_ => bail!("Error key configured"),
        };
        Ok(special_key)
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
            global_player_speed_up: KeyBind {
                code: Key::Char('f'),
                modifiers: KeyModifiers::CONTROL,
            },
            global_player_speed_down: KeyBind {
                code: Key::Char('b'),
                modifiers: KeyModifiers::CONTROL,
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
                        .add_col(TextSpan::new("<TAB> <Shift-TAB>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Switch focus"))
                        .add_row()
                        .add_col(TextSpan::new("<ESC> or <q>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Exit without saving"))
                        .add_row()
                        .add_col(
                            TextSpan::new("Modifier Select")
                                .bold()
                                .fg(Color::LightYellow),
                        )
                        .add_row()
                        .add_col(TextSpan::new("<j,k>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Move cursor(vim style)"))
                        .add_row()
                        .add_col(TextSpan::new("<ENTER>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("select a Modifier"))
                        .add_row()
                        .add_col(TextSpan::new("Key input").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(TextSpan::new("").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("You can input 1 char, or key name."))
                        .add_row()
                        .add_col(TextSpan::new("<Key Name>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("backspace/enter/left/right/up/down"))
                        .add_row()
                        .add_col(TextSpan::new("<Key Name>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("home/end/pageup/pagedown/tab/backtab"))
                        .add_row()
                        .add_col(TextSpan::new("<Key Name>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("delete/insert/esc"))
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

impl Model {
    #[allow(clippy::too_many_lines)]
    pub fn update_key_editor_key_changed(&mut self, id: &IdKeyEditor) {
        match id {
            IdKeyEditor::GlobalQuit | IdKeyEditor::GlobalQuitInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalQuit,
                    IdKeyEditor::GlobalQuitInput,
                );
                self.ke_key_config.global_quit = KeyBind { code, modifiers }
            }
            IdKeyEditor::GlobalLeft | IdKeyEditor::GlobalLeftInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalLeft,
                    IdKeyEditor::GlobalLeftInput,
                );
                self.ke_key_config.global_left = KeyBind { code, modifiers }
            }
            IdKeyEditor::GlobalRight | IdKeyEditor::GlobalRightInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalRight,
                    IdKeyEditor::GlobalRightInput,
                );
                self.ke_key_config.global_right = KeyBind { code, modifiers }
            }
            IdKeyEditor::GlobalUp | IdKeyEditor::GlobalUpInput => {
                let (code, modifiers) = self
                    .extract_key_mod_and_code(IdKeyEditor::GlobalUp, IdKeyEditor::GlobalUpInput);
                self.ke_key_config.global_up = KeyBind { code, modifiers }
            }

            IdKeyEditor::GlobalDown | IdKeyEditor::GlobalDownInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalDown,
                    IdKeyEditor::GlobalDownInput,
                );
                self.ke_key_config.global_down = KeyBind { code, modifiers }
            }
            IdKeyEditor::GlobalGotoTop | IdKeyEditor::GlobalGotoTopInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalGotoTop,
                    IdKeyEditor::GlobalGotoTopInput,
                );
                self.ke_key_config.global_goto_top = KeyBind { code, modifiers }
            }
            IdKeyEditor::GlobalGotoBottom | IdKeyEditor::GlobalGotoBottomInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalGotoBottom,
                    IdKeyEditor::GlobalGotoBottomInput,
                );
                self.ke_key_config.global_goto_bottom = KeyBind { code, modifiers }
            }
            IdKeyEditor::GlobalPlayerTogglePause | IdKeyEditor::GlobalPlayerTogglePauseInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerTogglePause,
                    IdKeyEditor::GlobalPlayerTogglePauseInput,
                );
                self.ke_key_config.global_player_toggle_pause = KeyBind { code, modifiers }
            }
            IdKeyEditor::GlobalPlayerNext | IdKeyEditor::GlobalPlayerNextInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerNext,
                    IdKeyEditor::GlobalPlayerNextInput,
                );
                self.ke_key_config.global_player_next = KeyBind { code, modifiers }
            }
            IdKeyEditor::GlobalPlayerPrevious | IdKeyEditor::GlobalPlayerPreviousInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerPrevious,
                    IdKeyEditor::GlobalPlayerPreviousInput,
                );
                self.ke_key_config.global_player_previous = KeyBind { code, modifiers }
            }

            IdKeyEditor::GlobalHelp | IdKeyEditor::GlobalHelpInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalHelp,
                    IdKeyEditor::GlobalHelpInput,
                );
                self.ke_key_config.global_help = KeyBind { code, modifiers }
            }
            IdKeyEditor::GlobalVolumeUp | IdKeyEditor::GlobalVolumeUpInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalVolumeUp,
                    IdKeyEditor::GlobalVolumeUpInput,
                );
                self.ke_key_config.global_player_volume_plus_2 = KeyBind { code, modifiers }
            }
            IdKeyEditor::GlobalVolumeDown | IdKeyEditor::GlobalVolumeDownInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalVolumeDown,
                    IdKeyEditor::GlobalVolumeDownInput,
                );
                self.ke_key_config.global_player_volume_minus_2 = KeyBind { code, modifiers }
            }

            IdKeyEditor::GlobalPlayerSeekForward | IdKeyEditor::GlobalPlayerSeekForwardInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerSeekForward,
                    IdKeyEditor::GlobalPlayerSeekForwardInput,
                );
                self.ke_key_config.global_player_seek_forward = KeyBind { code, modifiers }
            }

            IdKeyEditor::GlobalPlayerSeekBackward | IdKeyEditor::GlobalPlayerSeekBackwardInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerSeekBackward,
                    IdKeyEditor::GlobalPlayerSeekBackwardInput,
                );
                self.ke_key_config.global_player_seek_backward = KeyBind { code, modifiers }
            }

            IdKeyEditor::GlobalPlayerSpeedUp | IdKeyEditor::GlobalPlayerSpeedUpInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerSpeedUp,
                    IdKeyEditor::GlobalPlayerSpeedUpInput,
                );
                self.ke_key_config.global_player_speed_up = KeyBind { code, modifiers }
            }

            IdKeyEditor::GlobalPlayerSpeedDown | IdKeyEditor::GlobalPlayerSpeedDownInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerSpeedDown,
                    IdKeyEditor::GlobalPlayerSpeedDownInput,
                );
                self.ke_key_config.global_player_speed_down = KeyBind { code, modifiers }
            }

            IdKeyEditor::GlobalLyricAdjustForward | IdKeyEditor::GlobalLyricAdjustForwardInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalLyricAdjustForward,
                    IdKeyEditor::GlobalLyricAdjustForwardInput,
                );
                self.ke_key_config.global_lyric_adjust_forward = KeyBind { code, modifiers }
            }

            IdKeyEditor::GlobalLyricAdjustBackward
            | IdKeyEditor::GlobalLyricAdjustBackwardInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalLyricAdjustBackward,
                    IdKeyEditor::GlobalLyricAdjustBackwardInput,
                );
                self.ke_key_config.global_lyric_adjust_backward = KeyBind { code, modifiers }
            }

            IdKeyEditor::GlobalLyricCycle | IdKeyEditor::GlobalLyricCycleInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalLyricCycle,
                    IdKeyEditor::GlobalLyricCycleInput,
                );
                self.ke_key_config.global_lyric_cycle = KeyBind { code, modifiers }
            }
            IdKeyEditor::GlobalColorEditor | IdKeyEditor::GlobalColorEditorInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalColorEditor,
                    IdKeyEditor::GlobalColorEditorInput,
                );
                self.ke_key_config.global_color_editor_open = KeyBind { code, modifiers }
            }
            IdKeyEditor::GlobalKeyEditor | IdKeyEditor::GlobalKeyEditorInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalKeyEditor,
                    IdKeyEditor::GlobalKeyEditorInput,
                );
                self.ke_key_config.global_key_editor_open = KeyBind { code, modifiers }
            }

            IdKeyEditor::LibraryDelete | IdKeyEditor::LibraryDeleteInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibraryDelete,
                    IdKeyEditor::LibraryDeleteInput,
                );
                self.ke_key_config.library_delete = KeyBind { code, modifiers }
            }
            IdKeyEditor::LibraryLoadDir | IdKeyEditor::LibraryLoadDirInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibraryLoadDir,
                    IdKeyEditor::LibraryLoadDirInput,
                );
                self.ke_key_config.library_load_dir = KeyBind { code, modifiers }
            }
            IdKeyEditor::LibraryYank | IdKeyEditor::LibraryYankInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibraryYank,
                    IdKeyEditor::LibraryYankInput,
                );
                self.ke_key_config.library_yank = KeyBind { code, modifiers }
            }

            IdKeyEditor::LibraryPaste | IdKeyEditor::LibraryPasteInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibraryPaste,
                    IdKeyEditor::LibraryPasteInput,
                );
                self.ke_key_config.library_paste = KeyBind { code, modifiers }
            }

            IdKeyEditor::LibrarySearch | IdKeyEditor::LibrarySearchInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibrarySearch,
                    IdKeyEditor::LibrarySearchInput,
                );
                self.ke_key_config.library_search = KeyBind { code, modifiers }
            }
            IdKeyEditor::LibrarySearchYoutube | IdKeyEditor::LibrarySearchYoutubeInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibrarySearchYoutube,
                    IdKeyEditor::LibrarySearchYoutubeInput,
                );
                self.ke_key_config.library_search_youtube = KeyBind { code, modifiers }
            }

            IdKeyEditor::LibraryTagEditor | IdKeyEditor::LibraryTagEditorInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibraryTagEditor,
                    IdKeyEditor::LibraryTagEditorInput,
                );
                self.ke_key_config.library_tag_editor_open = KeyBind { code, modifiers }
            }
            IdKeyEditor::PlaylistDelete | IdKeyEditor::PlaylistDeleteInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistDelete,
                    IdKeyEditor::PlaylistDeleteInput,
                );
                self.ke_key_config.playlist_delete = KeyBind { code, modifiers }
            }
            IdKeyEditor::PlaylistDeleteAll | IdKeyEditor::PlaylistDeleteAllInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistDeleteAll,
                    IdKeyEditor::PlaylistDeleteAllInput,
                );
                self.ke_key_config.playlist_delete_all = KeyBind { code, modifiers }
            }
            IdKeyEditor::PlaylistShuffle | IdKeyEditor::PlaylistShuffleInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistShuffle,
                    IdKeyEditor::PlaylistShuffleInput,
                );
                self.ke_key_config.playlist_shuffle = KeyBind { code, modifiers }
            }
            IdKeyEditor::PlaylistModeCycle | IdKeyEditor::PlaylistModeCycleInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistModeCycle,
                    IdKeyEditor::PlaylistModeCycleInput,
                );
                self.ke_key_config.playlist_mode_cycle = KeyBind { code, modifiers }
            }
            IdKeyEditor::PlaylistPlaySelected | IdKeyEditor::PlaylistPlaySelectedInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistPlaySelected,
                    IdKeyEditor::PlaylistPlaySelectedInput,
                );
                self.ke_key_config.playlist_play_selected = KeyBind { code, modifiers }
            }

            IdKeyEditor::PlaylistAddFront | IdKeyEditor::PlaylistAddFrontInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistAddFront,
                    IdKeyEditor::PlaylistAddFrontInput,
                );
                self.ke_key_config.playlist_add_front = KeyBind { code, modifiers }
            }

            IdKeyEditor::PlaylistSearch | IdKeyEditor::PlaylistSearchInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistSearch,
                    IdKeyEditor::PlaylistSearchInput,
                );
                self.ke_key_config.playlist_search = KeyBind { code, modifiers }
            }

            _ => {}
        }
    }

    fn extract_key_mod_and_code(
        &mut self,
        id_select: IdKeyEditor,
        id_input: IdKeyEditor,
    ) -> (Key, KeyModifiers) {
        let mut code = Key::Null;
        let mut modifier = KeyModifiers::CONTROL;
        self.update_key_input_by_modifier(id_select.clone(), id_input.clone());
        if let Ok(State::One(StateValue::Usize(index))) = self.app.state(&Id::KeyEditor(id_select))
        {
            modifier = MODIFIER_LIST[index].modifier();
            if let Ok(State::One(StateValue::String(codes))) =
                self.app.state(&Id::KeyEditor(id_input))
            {
                if let Ok(c) = KeyBind::key_from_str(&codes) {
                    code = c;
                }
            }
        }
        (code, modifier)
    }
    fn update_key_input_by_modifier(&mut self, id_select: IdKeyEditor, id_input: IdKeyEditor) {
        if let Ok(State::One(StateValue::Usize(index))) = self.app.state(&Id::KeyEditor(id_select))
        {
            let modifier = MODIFIER_LIST[index].modifier();
            if let Ok(State::One(StateValue::String(codes))) =
                self.app.state(&Id::KeyEditor(id_input.clone()))
            {
                if modifier.bits() % 2 == 1 {
                    self.app
                        .attr(
                            &Id::KeyEditor(id_input),
                            Attribute::Value,
                            AttrValue::String(codes.to_uppercase()),
                        )
                        .ok();
                } else {
                    self.app
                        .attr(
                            &Id::KeyEditor(id_input),
                            Attribute::Value,
                            AttrValue::String(codes.to_lowercase()),
                        )
                        .ok();
                }
            }
        }
    }
}
