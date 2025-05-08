/*
 * MIT License
 *
 * tuifeed - Copyright (c) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use crate::ui::{Msg, TEMsg, TFMsg};
use termusiclib::config::SharedTuiSettings;
use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders, InputType};
use tuirealm::{Component, Event, MockComponent};

/// Common Field Properties and event handling
#[derive(MockComponent)]
struct EditField {
    component: Input,
    config: SharedTuiSettings,
}

impl EditField {
    #[inline]
    pub fn new(config: SharedTuiSettings, title: &'static str) -> Self {
        let component = {
            let config = config.read();
            Input::default()
                .foreground(config.settings.theme.library_foreground())
                .background(config.settings.theme.library_background())
                .borders(
                    Borders::default()
                        .color(config.settings.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                .input_type(InputType::Text)
                .title(title, Alignment::Left)
        };

        Self { component, config }
    }

    /// Basically [`Component::on`] but with custom extra parameters
    #[allow(clippy::needless_pass_by_value)]
    pub fn on(&mut self, ev: Event<NoUserEvent>, on_key_down: Msg, on_key_up: Msg) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        match ev {
            // Global Hotkeys
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                Some(Msg::TagEditor(TEMsg::TERename))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down | Key::Tab,
                ..
            }) => Some(on_key_down),
            Event::Keyboard(
                KeyEvent { code: Key::Up, .. }
                | KeyEvent {
                    code: Key::BackTab,
                    modifiers: KeyModifiers::SHIFT,
                },
            ) => Some(on_key_up),
            Event::Keyboard(keyevent) if keyevent == keys.escape.get() => {
                Some(Msg::TagEditor(TEMsg::TagEditorClose))
            }

            // Local Hotkeys
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => {
                self.perform(Cmd::Move(Direction::Left));
                Some(Msg::ForceRedraw)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => {
                self.perform(Cmd::Move(Direction::Right));
                Some(Msg::ForceRedraw)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => {
                self.perform(Cmd::GoTo(Position::Begin));
                Some(Msg::ForceRedraw)
            }
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End));
                Some(Msg::ForceRedraw)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => {
                self.perform(Cmd::Cancel);
                Some(Msg::ForceRedraw)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => {
                self.perform(Cmd::Delete);
                Some(Msg::ForceRedraw)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE,
            }) => {
                self.perform(Cmd::Type(ch));
                Some(Msg::ForceRedraw)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => Some(Msg::TagEditor(TEMsg::TESearch)),
            _ => None,
        }
    }
}

#[derive(MockComponent)]
pub struct TEInputArtist {
    component: EditField,
}

impl TEInputArtist {
    pub fn new(config: SharedTuiSettings) -> Self {
        Self {
            component: EditField::new(config, " Search artist "),
        }
    }
}

impl Component<Msg, NoUserEvent> for TEInputArtist {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::TagEditor(TEMsg::TEFocus(TFMsg::InputArtistBlurDown)),
            Msg::TagEditor(TEMsg::TEFocus(TFMsg::InputArtistBlurUp)),
        )
    }
}

#[derive(MockComponent)]
pub struct TEInputTitle {
    component: EditField,
}

impl TEInputTitle {
    pub fn new(config: SharedTuiSettings) -> Self {
        Self {
            component: EditField::new(config, " Search track name "),
        }
    }
}

impl Component<Msg, NoUserEvent> for TEInputTitle {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::TagEditor(TEMsg::TEFocus(TFMsg::InputTitleBlurDown)),
            Msg::TagEditor(TEMsg::TEFocus(TFMsg::InputTitleBlurUp)),
        )
    }
}

#[derive(MockComponent)]
pub struct TEInputAlbum {
    component: EditField,
}

impl TEInputAlbum {
    pub fn new(config: SharedTuiSettings) -> Self {
        Self {
            component: EditField::new(config, " Album "),
        }
    }
}

impl Component<Msg, NoUserEvent> for TEInputAlbum {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::TagEditor(TEMsg::TEFocus(TFMsg::InputAlbumBlurDown)),
            Msg::TagEditor(TEMsg::TEFocus(TFMsg::InputAlbumBlurUp)),
        )
    }
}

#[derive(MockComponent)]
pub struct TEInputGenre {
    component: EditField,
}

impl TEInputGenre {
    pub fn new(config: SharedTuiSettings) -> Self {
        Self {
            component: EditField::new(config, " Genre "),
        }
    }
}

impl Component<Msg, NoUserEvent> for TEInputGenre {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::TagEditor(TEMsg::TEFocus(TFMsg::InputGenreBlurDown)),
            Msg::TagEditor(TEMsg::TEFocus(TFMsg::InputGenreBlurUp)),
        )
    }
}
