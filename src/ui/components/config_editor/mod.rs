use crate::ui::model::ConfigEditorLayout;
use crate::ui::{ConfigEditorMsg, Msg};
use tui_realm_stdlib::Radio;
// use tuirealm::command::{Cmd, CmdResult, Direction, Position};
// use tuirealm::props::{Alignment, BorderSides, BorderType, Borders, Color, TableBuilder, TextSpan};
use tuirealm::props::{BorderSides, Borders, Color};
use tuirealm::{
    event::{Key, KeyEvent, NoUserEvent},
    Component, Event, MockComponent,
};
// use tuirealm::{
//     event::{Key, KeyEvent, KeyModifiers, NoUserEvent},
//     Component, Event, MockComponent, State, StateValue,
// };

#[derive(MockComponent)]
pub struct ConfigEditorHeader {
    component: Radio,
}

impl ConfigEditorHeader {
    pub fn new(layout: ConfigEditorLayout) -> Self {
        Self {
            component: Radio::default()
                .borders(
                    Borders::default()
                        .color(Color::Yellow)
                        .sides(BorderSides::BOTTOM),
                )
                .choices(&["General Configuration", "Themes and Colors", "Keys"])
                .foreground(Color::Yellow)
                .value(match layout {
                    ConfigEditorLayout::General => 0,
                    ConfigEditorLayout::Color => 1,
                    ConfigEditorLayout::Key => 2,
                }),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigEditorHeader {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter | Key::Esc,
                ..
            }) => Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel)),
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout))
            }
            _ => None,
        }
    }
}
