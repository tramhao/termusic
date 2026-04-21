use termusiclib::config::SharedTuiSettings;
use tui_realm_stdlib::components::Select;
use tui_realm_stdlib::prop_ext::CommonHighlight;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers};
use tuirealm::props::{BorderType, Borders, HorizontalAlignment, Style, Title};
use tuirealm::state::{State, StateValue};

use crate::ui::model::UserEvent;
use crate::ui::msg::{Msg, TEMsg, TFMsg};

#[derive(Component)]
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
                .background(config.settings.theme.library_background())
                .inactive(Style::new().bg(config.settings.theme.library_background()))
                .title(Title::from(" Select a lyric ").alignment(HorizontalAlignment::Center))
                .rewind(true)
                .highlight_style(
                    CommonHighlight::default()
                        .style
                        .bg(config.settings.theme.library_highlight()),
                )
                .highlight_str(config.settings.theme.style.library.highlight_symbol.clone())
                .choices(["No Lyric"])
        };

        Self { component, config }
    }
}

impl AppComponent<Msg, UserEvent> for TESelectLyric {
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                return Some(Msg::TagEditor(TEMsg::Save));
            }
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => match self.state() {
                State::Single(_) => return Some(Msg::TagEditor(TEMsg::Close)),
                _ => self.perform(Cmd::Cancel),
            },
            Event::Keyboard(keyevent) if keyevent == keys.escape.get() => match self.state() {
                State::Single(_) => return Some(Msg::TagEditor(TEMsg::Close)),
                _ => self.perform(Cmd::Cancel),
            },
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::SelectLyricBlurDown)));
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::SelectLyricBlurUp))),

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.up.get() => {
                match self.state() {
                    State::Single(_) => {
                        return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::SelectLyricBlurUp)));
                    }
                    _ => self.perform(Cmd::Move(Direction::Up)),
                }
            }
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.down.get() => {
                match self.state() {
                    State::Single(_) => {
                        return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::SelectLyricBlurDown)));
                    }
                    _ => self.perform(Cmd::Move(Direction::Down)),
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => match self.state() {
                State::Single(_) => {
                    return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::SelectLyricBlurDown)));
                }
                _ => self.perform(Cmd::Move(Direction::Down)),
            },
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => match self.state() {
                State::Single(_) => {
                    return Some(Msg::TagEditor(TEMsg::Focus(TFMsg::SelectLyricBlurUp)));
                }
                _ => self.perform(Cmd::Move(Direction::Up)),
            },
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::NoChange,
        };
        match cmd_result {
            CmdResult::Submit(State::Single(StateValue::Usize(index))) => {
                Some(Msg::TagEditor(TEMsg::SelectLyricOk(index)))
            }
            CmdResult::NoChange => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}
