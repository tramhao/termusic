use crate::config::{Keys, Settings};
use crate::ui::{DBMsg, Id, Model, Msg};
use tui_realm_stdlib::List;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::{Alignment, BorderType, TableBuilder, TextSpan};
use tuirealm::props::{Borders, Color};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers, NoUserEvent},
    AttrValue, Attribute, Component, Event, MockComponent, State, StateValue,
};

#[derive(MockComponent)]
pub struct Podcast {
    component: List,
    on_key_tab: Msg,
    on_key_backtab: Msg,
    keys: Keys,
}

impl Podcast {
    pub fn new(config: &Settings, on_key_tab: Msg, on_key_backtab: Msg) -> Self {
        Self {
            component: List::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .title(" Podcasts: ", Alignment::Left)
                .scroll(true)
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
                .rewind(false)
                .step(4)
                .scroll(true)
                .rows(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Artist"))
                        .add_row()
                        .add_col(TextSpan::from("Album"))
                        .add_row()
                        .add_col(TextSpan::from("Genre"))
                        .add_row()
                        .add_col(TextSpan::from("Directory"))
                        .build(),
                ),
            on_key_tab,
            on_key_backtab,
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for Podcast {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => {
                if let Some(AttrValue::Table(t)) = self.query(Attribute::Content) {
                    if let State::One(StateValue::Usize(index)) = self.state() {
                        if index >= t.len() - 1 {
                            return Some(self.on_key_tab.clone());
                        }
                    }
                }
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                if let Some(AttrValue::Table(t)) = self.query(Attribute::Content) {
                    if let State::One(StateValue::Usize(index)) = self.state() {
                        if index >= t.len() - 1 {
                            return Some(self.on_key_tab.clone());
                        }
                    }
                }
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::DataBase(DBMsg::SearchResult(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::DataBase(DBMsg::SearchTracksBlurDown))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backtab.clone()),

            Event::Keyboard(keyevent) if keyevent == self.keys.library_search.key_event() => {
                return Some(Msg::GeneralSearch(crate::ui::GSMsg::PopupShowDatabase))
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct Episode {
    component: List,
    on_key_tab: Msg,
    on_key_backtab: Msg,
    keys: Keys,
}

impl Episode {
    pub fn new(config: &Settings, on_key_tab: Msg, on_key_backtab: Msg) -> Self {
        Self {
            component: List::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .title(" Episodes: ", Alignment::Left)
                .scroll(true)
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
                .rewind(false)
                .step(4)
                .scroll(true)
                .rows(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Artist"))
                        .add_row()
                        .add_col(TextSpan::from("Album"))
                        .add_row()
                        .add_col(TextSpan::from("Genre"))
                        .add_row()
                        .add_col(TextSpan::from("Directory"))
                        .build(),
                ),
            on_key_tab,
            on_key_backtab,
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for Episode {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => {
                if let Some(AttrValue::Table(t)) = self.query(Attribute::Content) {
                    if let State::One(StateValue::Usize(index)) = self.state() {
                        if index >= t.len() - 1 {
                            return Some(self.on_key_tab.clone());
                        }
                    }
                }
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                if let Some(AttrValue::Table(t)) = self.query(Attribute::Content) {
                    if let State::One(StateValue::Usize(index)) = self.state() {
                        if index >= t.len() - 1 {
                            return Some(self.on_key_tab.clone());
                        }
                    }
                }
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::DataBase(DBMsg::SearchResult(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::DataBase(DBMsg::SearchTracksBlurDown))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backtab.clone()),

            Event::Keyboard(keyevent) if keyevent == self.keys.library_search.key_event() => {
                return Some(Msg::GeneralSearch(crate::ui::GSMsg::PopupShowDatabase))
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

impl Model {}
