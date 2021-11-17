use crate::Msg;
use tui_realm_treeview::TreeView;
use tuirealm::command::{Cmd, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders};
use tuirealm::tui::style::Color;
use tuirealm::{Component, Event, MockComponent, NoUserEvent};

#[derive(MockComponent)]
pub struct AddressInput {
    component: TreeView,
}

impl Default for AddressInput {
    fn default() -> Self {
        Self {
            component: TreeView::default()
                .foreground(Color::LightBlue)
                .borders(
                    Borders::default()
                        .color(Color::LightBlue)
                        .modifiers(BorderType::Rounded),
                )
                .title("Music Library", Alignment::Left),
        }
    }
}

impl Component<Msg, NoUserEvent> for AddressInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Type(ch)),
            Event::Keyboard(KeyEvent {
                code: Key::Left,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Delete,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Delete),
            _ => return None,
        };
        Some(Msg::None)
    }
}
