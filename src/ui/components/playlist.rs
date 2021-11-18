use crate::Msg;

use tui_realm_stdlib::Table;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::{Alignment, BorderType, Borders, Color, TableBuilder, TextSpan};
use tuirealm::{
    event::{Key, KeyEvent},
    Component, Event, MockComponent, NoUserEvent,
};

#[derive(MockComponent)]
pub struct Playlist {
    component: Table,
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            component: Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Thick)
                        .color(Color::Blue),
                )
                // .foreground(Color::Yellow)
                .background(Color::Black)
                .title("Playlist", Alignment::Left)
                .scroll(true)
                .highlighted_color(Color::LightBlue)
                .highlighted_str("\u{1f680}")
                // .highlighted_str("🚀")
                .rewind(true)
                .step(4)
                .row_height(1)
                .headers(&["Duration", "Artist", "Title", "Album"])
                .column_spacing(3)
                .widths(&[10, 20, 25, 45])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("KeyCode::Down"))
                        .add_col(TextSpan::from("OnKey"))
                        .add_col(TextSpan::from("Move cursor down"))
                        .add_row()
                        .add_col(TextSpan::from("KeyCode::Up"))
                        .add_col(TextSpan::from("OnKey"))
                        .add_col(TextSpan::from("Move cursor up"))
                        .add_row()
                        .add_col(TextSpan::from("KeyCode::PageDown"))
                        .add_col(TextSpan::from("OnKey"))
                        .add_col(TextSpan::from("Move cursor down by 8"))
                        .add_row()
                        .add_col(TextSpan::from("KeyCode::PageUp"))
                        .add_col(TextSpan::from("OnKey"))
                        .add_col(TextSpan::from("ove cursor up by 8"))
                        .add_row()
                        .add_col(TextSpan::from("KeyCode::End"))
                        .add_col(TextSpan::from("OnKey"))
                        .add_col(TextSpan::from("Move cursor to last item"))
                        .add_row()
                        .add_col(TextSpan::from("KeyCode::Home"))
                        .add_col(TextSpan::from("OnKey"))
                        .add_col(TextSpan::from("Move cursor to first item"))
                        .add_row()
                        .add_col(TextSpan::from("KeyCode::Char(_)"))
                        .add_col(TextSpan::from("OnKey"))
                        .add_col(TextSpan::from("Return pressed key"))
                        .build(),
                ),
        }
    }
}

impl Component<Msg, NoUserEvent> for Playlist {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _drop = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down | Key::Char('j'),
                ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up | Key::Char('k'),
                ..
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::PlaylistTableBlur)
            }
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}
