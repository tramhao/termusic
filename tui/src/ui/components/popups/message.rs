use termusiclib::types::Msg;
use tui_realm_stdlib::Paragraph;
use tuirealm::{
    props::{Alignment, BorderType, Borders, Color, TextModifiers, TextSpan},
    Component, Event, MockComponent, NoUserEvent,
};

#[derive(MockComponent)]
pub struct MessagePopup {
    component: Paragraph,
}

impl MessagePopup {
    pub fn new<S: AsRef<str>>(title: S, msg: S) -> Self {
        Self {
            component: Paragraph::default()
                .borders(
                    Borders::default()
                        .color(Color::Cyan)
                        .modifiers(BorderType::Rounded),
                )
                .foreground(Color::Green)
                // .background(Color::Black)
                .modifiers(TextModifiers::BOLD)
                .alignment(Alignment::Center)
                .title(title, Alignment::Center)
                .text(vec![TextSpan::from(msg.as_ref().to_string())].as_slice()),
        }
    }
}

impl Component<Msg, NoUserEvent> for MessagePopup {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}
