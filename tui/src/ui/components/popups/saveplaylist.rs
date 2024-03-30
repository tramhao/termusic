use anyhow::Result;
use termusiclib::{
    config::StyleColorSymbol,
    types::{Id, Msg},
};
use termusicplayback::SharedSettings;
use tui_realm_stdlib::{Input, Radio};
use tuirealm::{
    command::{Cmd, CmdResult, Direction, Position},
    event::{Key, KeyEvent, KeyModifiers},
    props::{Alignment, BorderType, Borders, Color, InputType},
    Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::ui::model::Model;

#[derive(MockComponent)]
pub struct SavePlaylistPopup {
    component: Input,
}

impl SavePlaylistPopup {
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
                .title("Save Playlist as: (Enter to confirm)", Alignment::Left),
        }
    }
}

impl Component<Msg, NoUserEvent> for SavePlaylistPopup {
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
            }) => {
                self.perform(Cmd::Delete);
                self.perform(Cmd::Submit)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE,
            }) => {
                self.perform(Cmd::Type(ch));
                self.perform(Cmd::Submit)
            }
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::SavePlaylistPopupCloseCancel);
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => match self.component.state() {
                State::One(StateValue::String(input_string)) => {
                    return Some(Msg::SavePlaylistPopupCloseOk(input_string))
                }
                _ => return Some(Msg::None),
            },
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::String(input_string))) => {
                Some(Msg::SavePlaylistPopupUpdate(input_string))
            }
            _ => Some(Msg::None),
        }
    }
}

#[derive(MockComponent)]
pub struct SavePlaylistConfirmPopup {
    component: Radio,
    config: SharedSettings,
    filename: String,
}

impl SavePlaylistConfirmPopup {
    pub fn new(config: SharedSettings, filename: &str) -> Self {
        let component = {
            let config = config.read();
            Radio::default()
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
                .title(
                    // format!(" Playlist {filename} exists. Overwrite? "),
                    " Playlist exists. Overwrite? ",
                    Alignment::Center,
                )
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0)
        };

        Self {
            component,
            config,
            filename: filename.to_string(),
        }
    }
}

impl Component<Msg, NoUserEvent> for SavePlaylistConfirmPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().keys;
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),

            Event::Keyboard(key) if key == keys.global_left.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == keys.global_right.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == keys.global_quit.key_event() => {
                return Some(Msg::SavePlaylistConfirmCloseCancel)
            }
            Event::Keyboard(key) if key == keys.global_esc.key_event() => {
                return Some(Msg::SavePlaylistConfirmCloseCancel)
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
            Some(Msg::SavePlaylistConfirmCloseCancel)
        } else if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(1)))
        ) {
            Some(Msg::SavePlaylistConfirmCloseOk(self.filename.clone()))
        } else {
            Some(Msg::None)
        }
    }
}

impl Model {
    pub fn mount_save_playlist(&mut self) -> Result<()> {
        assert!(self
            .app
            .remount(
                Id::SavePlaylistPopup,
                Box::new(SavePlaylistPopup::new(
                    &self.config.read().style_color_symbol
                )),
                vec![]
            )
            .is_ok());

        self.remount_save_playlist_label("")?;
        assert!(self.app.active(&Id::SavePlaylistPopup).is_ok());
        Ok(())
    }

    pub fn umount_save_playlist(&mut self) {
        if self.app.mounted(&Id::SavePlaylistPopup) {
            assert!(self.app.umount(&Id::SavePlaylistPopup).is_ok());
            assert!(self.app.umount(&Id::SavePlaylistLabel).is_ok());
        }
    }

    pub fn mount_save_playlist_confirm(&mut self, filename: &str) {
        assert!(self
            .app
            .remount(
                Id::SavePlaylistConfirm,
                Box::new(SavePlaylistConfirmPopup::new(self.config.clone(), filename)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::SavePlaylistConfirm).is_ok());
    }

    pub fn umount_save_playlist_confirm(&mut self) {
        if self.app.mounted(&Id::SavePlaylistConfirm) {
            assert!(self.app.umount(&Id::SavePlaylistConfirm).is_ok());
        }
    }
}
