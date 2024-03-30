use termusiclib::types::Msg;
use tui_realm_stdlib::Paragraph;
use tuirealm::{
    event::{Key, KeyEvent},
    props::{Alignment, BorderType, Borders, Color, TextModifiers, TextSpan},
    Component, Event, MockComponent, NoUserEvent,
};

#[derive(MockComponent)]
pub struct ErrorPopup {
    component: Paragraph,
}

impl ErrorPopup {
    pub fn new<E: Into<anyhow::Error>>(msg: E) -> Self {
        let msg = msg.into();
        error!("Displaying error popup: {msg:?}");
        // TODO: Consider changing to ":?" to output "Caused By" (and possibly backtrace) OR do a custom printing (copied from anyhow) once more than 4 lines can be displayed in height
        let msg = format!("{msg:#}");
        Self {
            component: Paragraph::default()
                .borders(
                    Borders::default()
                        .color(Color::Red)
                        .modifiers(BorderType::Rounded),
                )
                .title(" Error ", Alignment::Center)
                .foreground(Color::Red)
                // .background(Color::Black)
                .modifiers(TextModifiers::BOLD)
                .alignment(Alignment::Center)
                .text(&[TextSpan::from(msg)]/* &msg.lines().map(|v| TextSpan::from(v)).collect::<Vec<_>>() */),
        }
    }
}

impl Component<Msg, NoUserEvent> for ErrorPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter | Key::Esc,
                ..
            }) => Some(Msg::ErrorPopupClose),
            _ => None,
        }
    }
}
