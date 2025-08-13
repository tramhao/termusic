use termusiclib::config::SharedTuiSettings;
use termusiclib::types::{TEMsg, TFMsg};
use tui_realm_stdlib::Textarea;
use tuirealm::command::{Cmd, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, TextSpan};
use tuirealm::{Component, Event, MockComponent};

use crate::ui::model::UserEvent;
use crate::ui::msg::Msg;

#[derive(MockComponent)]
pub struct TETextareaLyric {
    component: Textarea,
    config: SharedTuiSettings,
}

impl TETextareaLyric {
    pub fn new(config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();
            Textarea::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(config.settings.theme.library_border()),
                )
                .foreground(config.settings.theme.library_foreground())
                .title(" Lyrics ", Alignment::Left)
                .step(4)
                .highlighted_str("\u{1f3b5}")
                // .highlighted_str("🎵")
                .text_rows([TextSpan::from("No lyrics.")])
        };
        Self { component, config }
    }
}

impl Component<Msg, UserEvent> for TETextareaLyric {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let _cmd_result = match ev {
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                return Some(Msg::TagEditor(TEMsg::TERename));
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::TextareaLyricBlurDown)));
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::TagEditor(TEMsg::TEFocus(TFMsg::TextareaLyricBlurUp))),
            Event::Keyboard(k) if k == keys.quit.get() => {
                return Some(Msg::TagEditor(TEMsg::TagEditorClose));
            }
            Event::Keyboard(k) if k == keys.escape.get() => {
                return Some(Msg::TagEditor(TEMsg::TagEditorClose));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(k) if k == keys.navigation_keys.down.get() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(k) if k == keys.navigation_keys.up.get() => {
                self.perform(Cmd::Move(Direction::Up))
            }
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

            Event::Keyboard(k) if k == keys.navigation_keys.goto_top.get() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }

            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }

            Event::Keyboard(k) if k == keys.navigation_keys.goto_bottom.get() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            _ => return None,
        };
        // "Textarea::perform" currently always returns "CmdResult::None", so always redraw on event
        // see https://github.com/veeso/tui-realm-stdlib/issues/27
        Some(Msg::ForceRedraw)
    }
}
