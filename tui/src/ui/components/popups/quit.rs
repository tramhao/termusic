use termusiclib::types::Msg;
use termusicplayback::SharedSettings;
use tui_realm_stdlib::Radio;
use tuirealm::{
    command::{Cmd, CmdResult, Direction},
    event::{Key, KeyEvent},
    props::{Alignment, BorderType, Borders, Color, PropPayload, PropValue},
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

#[derive(MockComponent)]
pub struct QuitPopup {
    component: Radio,
    config: SharedSettings,
}

impl QuitPopup {
    pub fn new(config: SharedSettings) -> Self {
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
                .title(" Are sure you want to quit?", Alignment::Center)
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0)
        };

        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for QuitPopup {
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
                return Some(Msg::QuitPopupCloseCancel)
            }
            Event::Keyboard(key) if key == keys.global_esc.key_event() => {
                return Some(Msg::QuitPopupCloseCancel)
            }
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
