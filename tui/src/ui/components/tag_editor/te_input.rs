use termusiclib::config::SharedTuiSettings;
use tuirealm::command::{Cmd, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, InputType};
use tuirealm::{Component, Event, MockComponent};

use crate::ui::components::vendored::tui_realm_stdlib_input::Input;
use crate::ui::model::UserEvent;
use crate::ui::msg::{Msg, TEMsg, TFMsg};

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
    pub fn on(&mut self, ev: Event<UserEvent>, on_key_down: Msg, on_key_up: Msg) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        match ev {
            // Global Hotkeys
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                Some(Msg::TagEditor(TEMsg::Save))
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
                Some(Msg::TagEditor(TEMsg::Close))
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
            }) => Some(Msg::TagEditor(TEMsg::Search)),
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

impl Component<Msg, UserEvent> for TEInputArtist {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::TagEditor(TEMsg::Focus(TFMsg::InputArtistBlurDown)),
            Msg::TagEditor(TEMsg::Focus(TFMsg::InputArtistBlurUp)),
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

impl Component<Msg, UserEvent> for TEInputTitle {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::TagEditor(TEMsg::Focus(TFMsg::InputTitleBlurDown)),
            Msg::TagEditor(TEMsg::Focus(TFMsg::InputTitleBlurUp)),
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

impl Component<Msg, UserEvent> for TEInputAlbum {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::TagEditor(TEMsg::Focus(TFMsg::InputAlbumBlurDown)),
            Msg::TagEditor(TEMsg::Focus(TFMsg::InputAlbumBlurUp)),
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

impl Component<Msg, UserEvent> for TEInputGenre {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::TagEditor(TEMsg::Focus(TFMsg::InputGenreBlurDown)),
            Msg::TagEditor(TEMsg::Focus(TFMsg::InputGenreBlurUp)),
        )
    }
}
