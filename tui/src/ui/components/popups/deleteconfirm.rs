use termusiclib::config::{SharedTuiSettings, TuiOverlay};
use termusiclib::types::{Id, Msg};
use tui_realm_stdlib::Input;
use tuirealm::{
    command::{Cmd, CmdResult, Direction, Position},
    event::{Key, KeyEvent, KeyModifiers},
    props::{Alignment, BorderType, Borders, InputType},
    Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::ui::model::Model;

use super::{YNConfirm, YNConfirmStyle};

#[derive(MockComponent)]
pub struct DeleteConfirmRadioPopup {
    component: YNConfirm,
}

impl DeleteConfirmRadioPopup {
    pub fn new(config: SharedTuiSettings) -> Self {
        let component =
            YNConfirm::new_with_cb(config, " Are sure you want to delete? ", |config| {
                YNConfirmStyle {
                    foreground_color: config.settings.theme.important_popup_foreground(),
                    background_color: config.settings.theme.important_popup_background(),
                    border_color: config.settings.theme.important_popup_border(),
                    title_alignment: Alignment::Left,
                }
            });

        Self { component }
    }
}

impl Component<Msg, NoUserEvent> for DeleteConfirmRadioPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component
            .on(ev, Msg::DeleteConfirmCloseOk, Msg::DeleteConfirmCloseCancel)
    }
}

#[derive(MockComponent)]
pub struct DeleteConfirmInputPopup {
    component: Input,
}

impl DeleteConfirmInputPopup {
    pub fn new(config: &TuiOverlay) -> Self {
        let config = &config.settings;
        Self {
            component: Input::default()
                .foreground(config.theme.important_popup_foreground())
                .background(config.theme.important_popup_background())
                .borders(
                    Borders::default()
                        .color(config.theme.important_popup_border())
                        .modifiers(BorderType::Rounded),
                )
                // .invalid_style(Style::default().fg(Color::Red))
                .input_type(InputType::Text)
                .title(" Type DELETE to confirm: ", Alignment::Left),
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
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

impl Model {
    pub fn mount_confirm_radio(&mut self) {
        assert!(self
            .app
            .remount(
                Id::DeleteConfirmRadioPopup,
                Box::new(DeleteConfirmRadioPopup::new(self.config_tui.clone())),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::DeleteConfirmRadioPopup).is_ok());
    }

    pub fn mount_confirm_input(&mut self) {
        assert!(self
            .app
            .remount(
                Id::DeleteConfirmInputPopup,
                Box::new(DeleteConfirmInputPopup::new(&self.config_tui.read())),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::DeleteConfirmInputPopup).is_ok());
    }
}
