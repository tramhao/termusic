use termusiclib::config::{SharedTuiSettings, TuiOverlay};
use termusiclib::types::Msg;
use tui_realm_stdlib::Radio;
use tuirealm::{
    command::{Cmd, CmdResult, Direction},
    event::{Key, KeyEvent},
    props::{Alignment, BorderType, Borders, Color, PropPayload, PropValue},
    AttrValue, Attribute, Event, MockComponent, State, StateValue,
};

use crate::ui::model::UserEvent;

/// Struct for the Style of the [`YNConfirm`]
#[derive(Debug, Clone, PartialEq)]
pub struct YNConfirmStyle {
    pub foreground_color: Color,
    pub background_color: Color,
    pub border_color: Color,
    pub title_alignment: Alignment,
}

/// A Common [`MockComponent`] for `No/Yes` Popups
#[derive(MockComponent)]
pub struct YNConfirm {
    component: Radio,
    config: SharedTuiSettings,
}

impl YNConfirm {
    /// Create a new instance with custom colors
    pub fn new_with_cb<F: FnOnce(&TuiOverlay) -> YNConfirmStyle, T: Into<String>>(
        config: SharedTuiSettings,
        title: T,
        cb: F,
    ) -> Self {
        let component = {
            let config = config.read();
            let style = cb(&config);
            Radio::default()
                .foreground(style.foreground_color)
                .background(style.background_color)
                .borders(
                    Borders::default()
                        .color(style.border_color)
                        .modifiers(BorderType::Rounded),
                )
                .title(title, style.title_alignment)
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0)
        };

        Self { component, config }
    }

    /// Basically [`Component::on`](tuirealm::Component::on) but with custom extra parameters
    ///
    /// `on_y` corresponds to pressing `Yes` and `on_n` to pressing `No`
    #[allow(clippy::needless_pass_by_value)]
    pub fn on(&mut self, ev: Event<UserEvent>, on_y: Msg, on_n: Msg) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(on_n),
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),

            Event::Keyboard(key) if key == keys.navigation_keys.left.get() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.right.get() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.up.get() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.down.get() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == keys.quit.get() => return Some(on_n),
            Event::Keyboard(key) if key == keys.escape.get() => return Some(on_n),
            Event::Keyboard(KeyEvent {
                code: Key::Char('y'),
                ..
            }) => {
                // ordering is 0 = No, 1 = Yes
                self.component.attr(
                    Attribute::Value,
                    AttrValue::Payload(PropPayload::One(PropValue::Usize(1))),
                );
                self.perform(Cmd::Submit)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('n'),
                ..
            }) => {
                // ordering is 0 = No, 1 = Yes
                self.component.attr(
                    Attribute::Value,
                    AttrValue::Payload(PropPayload::One(PropValue::Usize(0))),
                );
                self.perform(Cmd::Submit)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => return None,
        };

        match cmd_result {
            CmdResult::Submit(State::One(StateValue::Usize(0))) => Some(on_n),
            CmdResult::Submit(State::One(StateValue::Usize(1))) => Some(on_y),
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}
