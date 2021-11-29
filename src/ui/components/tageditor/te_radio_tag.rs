use crate::ui::Msg;
use tui_realm_stdlib::Radio;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent};
use tuirealm::props::{Alignment, BorderType, Borders, Color};
use tuirealm::{Component, Event, MockComponent, NoUserEvent, State, StateValue};

#[derive(MockComponent)]
pub struct TERadioTag {
    component: Radio,
}
impl Default for TERadioTag {
    fn default() -> Self {
        Self {
            component: Radio::default()
                .foreground(Color::Yellow)
                .background(Color::Black)
                .borders(
                    Borders::default()
                        .color(Color::Yellow)
                        .modifiers(BorderType::Rounded),
                )
                .title("Are sure you want to quit?", Alignment::Center)
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0),
        }
    }
}

impl Component<Msg, NoUserEvent> for TERadioTag {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left | Key::Char('h' | 'j'),
                ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right | Key::Char('l' | 'k'),
                ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
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
