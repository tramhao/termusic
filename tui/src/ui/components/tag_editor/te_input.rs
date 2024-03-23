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
use crate::config::Settings;
use crate::ui::{Msg, TEMsg, TFMsg};
use termusiclib::config::Keys;
use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType};
use tuirealm::{Component, Event, MockComponent};

/// Common Field Properties and event handling
#[derive(MockComponent)]
struct EditField {
    component: Input,
    config: Settings,
}

impl EditField {
    #[inline]
    pub fn new(config: Settings, title: &'static str) -> Self {
        Self {
            component: Input::default()
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Cyan),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Black),
                )
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::LightYellow),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .input_type(InputType::Text)
                .title(title, Alignment::Left),
            config,
        }
    }

    /// Basically [`Component::on`] but with custom extra parameters
    #[allow(clippy::needless_pass_by_value)]
    pub fn on(&mut self, ev: Event<NoUserEvent>, on_key_down: Msg, on_key_up: Msg) -> Option<Msg> {
        let keys = self.config.keys.clone();
        handle_input_ev(self, ev, &keys, on_key_down, on_key_up)
    }
}

#[derive(MockComponent)]
pub struct TEInputArtist {
    component: EditField,
}

impl TEInputArtist {
    pub fn new(config: Settings) -> Self {
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
    pub fn new(config: Settings) -> Self {
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

#[allow(clippy::needless_pass_by_value)]
fn handle_input_ev(
    component: &mut dyn MockComponent,
    ev: Event<NoUserEvent>,
    keys: &Keys,
    on_key_down: Msg,
    on_key_up: Msg,
) -> Option<Msg> {
    match ev {
        // Global Hotkeys
        Event::Keyboard(keyevent) if keyevent == keys.config_save.key_event() => {
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
        Event::Keyboard(keyevent) if keyevent == keys.global_esc.key_event() => {
            Some(Msg::TagEditor(TEMsg::TagEditorClose(None)))
        }

        // Local Hotkeys
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

        Event::Keyboard(KeyEvent {
            code: Key::Char(ch),
            modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE,
        }) => {
            component.perform(Cmd::Type(ch));
            Some(Msg::None)
        }

        Event::Keyboard(KeyEvent {
            code: Key::Enter, ..
        }) => Some(Msg::TagEditor(TEMsg::TESearch)),
        _ => None,
    }
}

#[derive(MockComponent)]
pub struct TEInputAlbum {
    component: EditField,
}

impl TEInputAlbum {
    pub fn new(config: Settings) -> Self {
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
    pub fn new(config: Settings) -> Self {
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
