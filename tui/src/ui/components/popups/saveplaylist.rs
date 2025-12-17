use std::path::PathBuf;

use anyhow::Result;
use termusiclib::config::{SharedTuiSettings, TuiOverlay};
use tuirealm::{
    Component, Event, MockComponent, State, StateValue,
    command::{Cmd, CmdResult, Direction, Position},
    event::{Key, KeyEvent, KeyModifiers},
    props::{Alignment, BorderType, Borders, InputType},
};

use super::{YNConfirm, YNConfirmStyle};
use crate::ui::components::vendored::tui_realm_stdlib_input::Input;
use crate::ui::ids::Id;
use crate::ui::model::{Model, UserEvent};
use crate::ui::msg::{Msg, SavePlaylistMsg};

#[derive(MockComponent)]
pub struct SavePlaylistPopup {
    component: Input,
}

impl SavePlaylistPopup {
    pub fn new(config: &TuiOverlay) -> Self {
        let settings = &config.settings;
        Self {
            component: Input::default()
                .foreground(settings.theme.fallback_foreground())
                .background(settings.theme.fallback_background())
                .borders(
                    Borders::default()
                        .color(settings.theme.fallback_border())
                        .modifiers(BorderType::Rounded),
                )
                // .invalid_style(Style::default().fg(Color::Red))
                .input_type(InputType::Text)
                .title(" Save Playlist as: (Enter to confirm) ", Alignment::Left),
        }
    }
}

impl Component<Msg, UserEvent> for SavePlaylistPopup {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
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
                return Some(Msg::SavePlaylist(SavePlaylistMsg::CloseCancel));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => match self.component.state() {
                State::One(StateValue::String(input_string)) => {
                    return Some(Msg::SavePlaylist(SavePlaylistMsg::CloseOk(PathBuf::from(
                        input_string,
                    ))));
                }
                _ => CmdResult::None,
            },
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::String(input_string))) => Some(
                Msg::SavePlaylist(SavePlaylistMsg::Update(PathBuf::from(input_string))),
            ),
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

#[derive(MockComponent)]
pub struct SavePlaylistConfirmPopup {
    component: YNConfirm,
    full_path: PathBuf,
}

impl SavePlaylistConfirmPopup {
    pub fn new(config: SharedTuiSettings, filename: PathBuf) -> Self {
        let component = YNConfirm::new_with_cb(config, " Playlist exists. Overwrite? ", |config| {
            YNConfirmStyle {
                foreground_color: config.settings.theme.important_popup_foreground(),
                background_color: config.settings.theme.important_popup_background(),
                border_color: config.settings.theme.important_popup_border(),
                title_alignment: Alignment::Center,
            }
        });

        Self {
            component,
            full_path: filename,
        }
    }
}

impl Component<Msg, UserEvent> for SavePlaylistConfirmPopup {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::SavePlaylist(SavePlaylistMsg::OverwriteOk(self.full_path.clone())),
            Msg::SavePlaylist(SavePlaylistMsg::OverwriteCancel),
        )
    }
}

impl Model {
    pub fn mount_save_playlist(&mut self) -> Result<()> {
        assert!(
            self.app
                .remount(
                    Id::SavePlaylistPopup,
                    Box::new(SavePlaylistPopup::new(&self.config_tui.read())),
                    vec![]
                )
                .is_ok()
        );

        self.remount_save_playlist_label(&PathBuf::new())?;
        assert!(self.app.active(&Id::SavePlaylistPopup).is_ok());
        Ok(())
    }

    pub fn umount_save_playlist(&mut self) {
        if self.app.mounted(&Id::SavePlaylistPopup) {
            assert!(self.app.umount(&Id::SavePlaylistPopup).is_ok());
            assert!(self.app.umount(&Id::SavePlaylistLabel).is_ok());
        }
    }

    pub fn mount_save_playlist_confirm(&mut self, filename: PathBuf) {
        assert!(
            self.app
                .remount(
                    Id::SavePlaylistConfirm,
                    Box::new(SavePlaylistConfirmPopup::new(
                        self.config_tui.clone(),
                        filename
                    )),
                    vec![]
                )
                .is_ok()
        );
        assert!(self.app.active(&Id::SavePlaylistConfirm).is_ok());
    }

    pub fn umount_save_playlist_confirm(&mut self) {
        if self.app.mounted(&Id::SavePlaylistConfirm) {
            assert!(self.app.umount(&Id::SavePlaylistConfirm).is_ok());
        }
    }
}
