use crate::config::Settings;
use crate::ui::model::ConfigEditorLayout;
use crate::ui::{ConfigEditorMsg, Msg};
use tui_realm_stdlib::{Input, Radio, Span};
// use tuirealm::command::{Cmd, CmdResult, Direction, Position};
// use tuirealm::props::{Alignment, BorderSides, BorderType, Borders, Color, TableBuilder, TextSpan};
use tuirealm::props::{
    Alignment, BorderSides, BorderType, Borders, Color, InputType, Style, TextSpan,
};
use tuirealm::{
    command::{Cmd, Direction, Position},
    event::{Key, KeyEvent, NoUserEvent},
    Component, Event, MockComponent,
};
// use tuirealm::{
//     event::{Key, KeyEvent, KeyModifiers, NoUserEvent},
//     Component, Event, MockComponent, State, StateValue,
// };

#[derive(MockComponent)]
pub struct CEHeader {
    component: Radio,
}

impl CEHeader {
    pub fn new(layout: &ConfigEditorLayout) -> Self {
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

impl Component<Msg, NoUserEvent> for CEHeader {
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

#[derive(MockComponent)]
pub struct Footer {
    component: Span,
}

impl Default for Footer {
    fn default() -> Self {
        Self {
            component: Span::default().spans(&[
                TextSpan::new("<CTRL+S>").bold().fg(Color::Cyan),
                TextSpan::new(" Save parameters "),
                TextSpan::new("<ESC>").bold().fg(Color::Cyan),
                TextSpan::new(" Exit "),
                TextSpan::new("<TAB>").bold().fg(Color::Cyan),
                TextSpan::new(" Change panel "),
                TextSpan::new("<UP/DOWN>").bold().fg(Color::Cyan),
                TextSpan::new(" Change field "),
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
pub struct MusicDir {
    component: Input,
    config: Settings,
}

impl MusicDir {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Input::default()
                .borders(
                    Borders::default()
                        .color(Color::LightGreen)
                        .modifiers(BorderType::Rounded),
                )
                .foreground(Color::LightGreen)
                .input_type(InputType::Text)
                .placeholder("~/Music", Style::default().fg(Color::Rgb(128, 128, 128)))
                .title("Root Music Directory", Alignment::Left)
                .value(&config.music_dir),
            config: config.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for MusicDir {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        handle_input_ev(
            self,
            ev,
            &config,
            Msg::ConfigEditor(ConfigEditorMsg::MusicDirBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::MusicDirBlurUp),
        )
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_input_ev(
    component: &mut dyn Component<Msg, NoUserEvent>,
    ev: Event<NoUserEvent>,
    config: &Settings,
    on_key_down: Msg,
    on_key_up: Msg,
) -> Option<Msg> {
    match ev {
        Event::Keyboard(KeyEvent {
            code: Key::Left, ..
        }) => {
            component.perform(Cmd::Move(Direction::Left));
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Right, ..
        }) => {
            component.perform(Cmd::Move(Direction::Right));
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Home, ..
        }) => {
            component.perform(Cmd::GoTo(Position::Begin));
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
            component.perform(Cmd::GoTo(Position::End));
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Delete, ..
        }) => {
            component.perform(Cmd::Cancel);
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Backspace,
            ..
        }) => {
            component.perform(Cmd::Delete);
            Some(Msg::None)
        }
        // Event::Keyboard(KeyEvent {
        //     // NOTE: escaped control sequence
        //     code: Key::Char('h') | Key::Char('r') | Key::Char('s'),
        //     modifiers: KeyModifiers::CONTROL,
        // }) => Some(Msg::None),
        Event::Keyboard(KeyEvent {
            code: Key::Char(ch),
            ..
        }) => {
            component.perform(Cmd::Type(ch));
            Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigChanged))
        }
        Event::Keyboard(KeyEvent {
            code: Key::Down, ..
        }) => Some(on_key_down),
        Event::Keyboard(KeyEvent { code: Key::Up, .. }) => Some(on_key_up),
        Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout))
        }
        Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel))
        }

        Event::Keyboard(keyevent) if keyevent == config.keys.global_config_save.key_event() => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk))
        }
        _ => None,
    }
}

#[derive(MockComponent)]
pub struct ExitConfirmation {
    component: Radio,
}

impl ExitConfirmation {
    pub fn new(enabled: bool) -> Self {
        Self {
            component: Radio::default()
                .borders(
                    Borders::default()
                        .color(Color::LightRed)
                        .modifiers(BorderType::Rounded),
                )
                .choices(&["Yes", "No"])
                .foreground(Color::LightRed)
                .rewind(true)
                .title("Show exit confirmation?", Alignment::Left)
                .value(if enabled { 0 } else { 1 }),
        }
    }
}

impl Component<Msg, NoUserEvent> for ExitConfirmation {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_radio_ev(
            self,
            ev,
            Msg::ConfigEditor(ConfigEditorMsg::ExitConfirmationBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::ExitConfirmationBlurUp),
        )
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_radio_ev(
    component: &mut dyn Component<Msg, NoUserEvent>,
    ev: Event<NoUserEvent>,
    on_key_down: Msg,
    on_key_up: Msg,
) -> Option<Msg> {
    match ev {
        Event::Keyboard(KeyEvent {
            code: Key::Left, ..
        }) => {
            component.perform(Cmd::Move(Direction::Left));
            Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigChanged))
        }
        Event::Keyboard(KeyEvent {
            code: Key::Right, ..
        }) => {
            component.perform(Cmd::Move(Direction::Right));
            Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigChanged))
        }
        Event::Keyboard(KeyEvent {
            code: Key::Down, ..
        }) => Some(on_key_down),
        Event::Keyboard(KeyEvent { code: Key::Up, .. }) => Some(on_key_up),
        _ => None,
    }
}

#[derive(MockComponent)]
pub struct PlaylistDisplaySymbol {
    component: Radio,
}

impl PlaylistDisplaySymbol {
    pub fn new(enabled: bool) -> Self {
        Self {
            component: Radio::default()
                .borders(
                    Borders::default()
                        .color(Color::LightRed)
                        .modifiers(BorderType::Rounded),
                )
                .choices(&["Yes", "No"])
                .foreground(Color::LightRed)
                .rewind(true)
                .title("Display symbol in playlist title?", Alignment::Left)
                .value(if enabled { 0 } else { 1 }),
        }
    }
}

impl Component<Msg, NoUserEvent> for PlaylistDisplaySymbol {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_radio_ev(
            self,
            ev,
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistDisplaySymbolBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistDisplaySymbolBlurUp),
        )
    }
}
