mod color;
mod general;
mod ke_input;
mod ke_select;
mod update;
mod view;

use crate::config::Settings;
use crate::ui::model::ConfigEditorLayout;
use crate::ui::{ConfigEditorMsg, Msg};
pub use color::*;
pub use general::*;
pub use ke_input::*;
pub use ke_select::*;

use tui_realm_stdlib::{Radio, Span};
use tuirealm::props::{Alignment, BorderSides, BorderType, Borders, Color, TextSpan};
use tuirealm::{
    command::{Cmd, CmdResult, Direction},
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

#[derive(MockComponent)]
pub struct CEHeader {
    component: Radio,
}

impl CEHeader {
    pub fn new(layout: &ConfigEditorLayout, config: &Settings) -> Self {
        Self {
            component: Radio::default()
                .borders(
                    Borders::default()
                        .color(Color::Cyan) // This color is not working perhaps because no focus
                        .modifiers(BorderType::Double)
                        .sides(BorderSides::BOTTOM),
                )
                .choices(&[
                    "General Configuration",
                    "Themes and Colors",
                    "Keys Global",
                    "Keys Other",
                ])
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::Yellow),
                )
                .value(match layout {
                    ConfigEditorLayout::General => 0,
                    ConfigEditorLayout::Color => 1,
                    ConfigEditorLayout::Key1 => 2,
                    ConfigEditorLayout::Key2 => 3,
                }),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEHeader {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct Footer {
    component: Span,
}

impl Footer {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Span::default().spans(&[
                TextSpan::new("<CTRL+S>").bold().fg(config
                    .style_color_symbol
                    .library_highlight()
                    .unwrap_or(Color::Cyan)),
                TextSpan::new(" Save parameters "),
                TextSpan::new("<ESC>").bold().fg(config
                    .style_color_symbol
                    .library_highlight()
                    .unwrap_or(Color::Cyan)),
                TextSpan::new(" Exit "),
                TextSpan::new("<TAB>").bold().fg(config
                    .style_color_symbol
                    .library_highlight()
                    .unwrap_or(Color::Cyan)),
                TextSpan::new(" Change panel "),
                TextSpan::new("<UP/DOWN>").bold().fg(config
                    .style_color_symbol
                    .library_highlight()
                    .unwrap_or(Color::Cyan)),
                TextSpan::new(" Change field "),
                TextSpan::new("<ENTER>").bold().fg(config
                    .style_color_symbol
                    .library_highlight()
                    .unwrap_or(Color::Cyan)),
                TextSpan::new(" Select theme/Preview symbol "),
            ]),
        }
    }
}

impl Component<Msg, NoUserEvent> for Footer {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct ConfigSavePopup {
    component: Radio,
    config: Settings,
}

impl ConfigSavePopup {
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
                .title(" Config changed. Do you want to save? ", Alignment::Center)
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0),
            config: config.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigSavePopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),

            Event::Keyboard(key) if key == self.config.keys.global_left.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.config.keys.global_right.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.config.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.config.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.config.keys.global_quit.key_event() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigSaveCancel))
            }
            Event::Keyboard(key) if key == self.config.keys.global_esc.key_event() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigSaveCancel))
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
            Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigSaveCancel))
        } else if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(1)))
        ) {
            Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigSaveOk))
        } else {
            Some(Msg::None)
        }
    }
}
