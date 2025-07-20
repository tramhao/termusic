use termusiclib::config::SharedTuiSettings;
use termusiclib::types::{Msg, TEMsg, TFMsg};
use tui_realm_stdlib::Select;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders};
use tuirealm::{Component, Event, MockComponent, State, StateValue};

use crate::ui::model::UserEvent;

#[derive(MockComponent)]
pub struct TESelectLyric {
    component: Select,
    config: SharedTuiSettings,
}

impl TESelectLyric {
    pub fn new(config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();
            Select::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(config.settings.theme.library_border()),
                )
                .foreground(config.settings.theme.library_foreground())
                .title(" Select a lyric ", Alignment::Center)
                .rewind(true)
                .highlighted_color(config.settings.theme.library_highlight())
                .highlighted_str(&config.settings.theme.style.library.highlight_symbol)
                .choices(["No Lyric"])
        };

        Self { component, config }
    }
}

impl Component<Msg, UserEvent> for TESelectLyric {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                return Some(Msg::TagEditor(TEMsg::TERename));
            }
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => match self.state() {
                State::One(_) => return Some(Msg::TagEditor(TEMsg::TagEditorClose)),
                _ => self.perform(Cmd::Cancel),
            },
            Event::Keyboard(keyevent) if keyevent == keys.escape.get() => match self.state() {
                State::One(_) => return Some(Msg::TagEditor(TEMsg::TagEditorClose)),
                _ => self.perform(Cmd::Cancel),
            },
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurDown)));
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurUp))),

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.up.get() => {
                match self.state() {
                    State::One(_) => {
                        return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurUp)));
                    }
                    _ => self.perform(Cmd::Move(Direction::Up)),
                }
            }
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.down.get() => {
                match self.state() {
                    State::One(_) => {
                        return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurDown)));
                    }
                    _ => self.perform(Cmd::Move(Direction::Down)),
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => match self.state() {
                State::One(_) => {
                    return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurDown)));
                }
                _ => self.perform(Cmd::Move(Direction::Down)),
            },
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => match self.state() {
                State::One(_) => {
                    return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::SelectLyricBlurUp)));
                }
                _ => self.perform(Cmd::Move(Direction::Up)),
            },
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::Usize(index))) => {
                Some(Msg::TagEditor(TEMsg::TESelectLyricOk(index)))
            }
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}
