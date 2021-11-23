// use crate::song::Song;
use crate::ui::Msg;

use tui_realm_stdlib::ProgressBar;
use tuirealm::command::CmdResult;
use tuirealm::props::{Alignment, BorderType, Borders, Color};
use tuirealm::{Component, Event, MockComponent, NoUserEvent};

#[derive(MockComponent)]
pub struct Progress {
    component: ProgressBar,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            component: ProgressBar::default()
                .borders(
                    Borders::default()
                        .color(Color::LightMagenta)
                        .modifiers(BorderType::Rounded),
                )
                .foreground(Color::LightYellow)
                .label("Song Name")
                .title("Playing", Alignment::Center)
                .progress(0.0),
        }
    }
}

impl Component<Msg, NoUserEvent> for Progress {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _drop = match ev {
            // Event::User(UserEvent::Loaded(prog)) => {
            //     // Update
            //     let label = format!("{:02}%", (prog * 100.0) as usize);
            //     self.attr(
            //         Attribute::Value,
            //         AttrValue::Payload(PropPayload::One(PropValue::F64(prog))),
            //     );
            //     self.attr(Attribute::Text, AttrValue::String(label));
            //     CmdResult::None
            // }
            // Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => return Some(Msg::GaugeAlfaBlur),
            // Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}
