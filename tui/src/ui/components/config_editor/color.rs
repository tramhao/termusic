/**
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
use anyhow::Result;
use termusiclib::config::SharedTuiSettings;
use termusiclib::config::v2::tui::theme::ThemeWrap;
use termusiclib::config::v2::tui::theme::styles::ColorTermusic;
use tui_realm_stdlib::{Label, Select, Table};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{
    Alignment, BorderType, Borders, Color, InputType, Style, TableBuilder, TextModifiers, TextSpan,
};
use tuirealm::ratatui::style::Modifier;
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

use crate::ui::components::vendored::tui_realm_stdlib_input::Input;
use crate::ui::ids::{Id, IdCETheme, IdConfigEditor};
use crate::ui::model::{Model, UserEvent};
use crate::ui::msg::{ConfigEditorMsg, KFMsg, Msg};

const COLOR_LIST: [ColorTermusic; 19] = [
    ColorTermusic::Reset,
    ColorTermusic::Foreground,
    ColorTermusic::Background,
    ColorTermusic::Black,
    ColorTermusic::Red,
    ColorTermusic::Green,
    ColorTermusic::Yellow,
    ColorTermusic::Blue,
    ColorTermusic::Magenta,
    ColorTermusic::Cyan,
    ColorTermusic::White,
    ColorTermusic::LightBlack,
    ColorTermusic::LightRed,
    ColorTermusic::LightGreen,
    ColorTermusic::LightYellow,
    ColorTermusic::LightBlue,
    ColorTermusic::LightMagenta,
    ColorTermusic::LightCyan,
    ColorTermusic::LightWhite,
];

#[derive(MockComponent)]
pub struct CEThemeSelectTable {
    component: Table,
    config: SharedTuiSettings,
}

impl CEThemeSelectTable {
    pub fn new(config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();
            Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(config.settings.theme.fallback_border()),
                )
                .foreground(config.settings.theme.fallback_foreground())
                .background(config.settings.theme.fallback_background())
                .title(" Themes: <Enter> to preview ", Alignment::Left)
                .scroll(true)
                .highlighted_color(config.settings.theme.fallback_highlight())
                .highlighted_str(&config.settings.theme.style.library.highlight_symbol)
                .rewind(true)
                .step(4)
                .row_height(1)
                .headers(["index", "Theme Name"])
                .column_spacing(1)
                .widths(&[18, 82])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty"))
                        .add_col(TextSpan::from("Empty Queue"))
                        .add_col(TextSpan::from("Empty"))
                        .build(),
                )
        };

        Self { component, config }
    }
}

impl Component<Msg, UserEvent> for CEThemeSelectTable {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            // Global Hotkeys
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk));
            }
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel));
            }
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel));
            }

            // Local Hotkeys
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.down.get() => {
                self.perform(Cmd::Move(Direction::Down))
            }

            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.up.get() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.goto_top.get() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.goto_bottom.get() => {
                self.perform(Cmd::GoTo(Position::End))
            }

            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)));
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)));
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::ConfigEditor(ConfigEditorMsg::ThemeSelectLoad(index)));
                }
                CmdResult::None
            }

            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

#[derive(MockComponent)]
pub struct CEColorSelect {
    component: Select,
    id: IdCETheme,
    config: SharedTuiSettings,
}

impl CEColorSelect {
    /// Get a new instance for a color selector.
    ///
    /// # Panics
    ///
    /// The only IDs expected are color IDs, everything else(like `*Symbol` or `*Label`) will panic.
    pub fn new(name: &str, id: IdCETheme, color: Color, config: SharedTuiSettings) -> Self {
        let init_value = Self::init_color_select(id, &config.read().settings.theme);
        let mut choices = Vec::new();
        for color in &COLOR_LIST {
            choices.push(color.as_ref());
        }
        Self {
            component: Select::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(color),
                )
                // .foreground(color)
                .background(color)
                .title(name, Alignment::Left)
                .rewind(false)
                .inactive(Style::default().add_modifier(Modifier::BOLD).bg(color))
                .highlighted_color(Color::LightGreen)
                .highlighted_str(">> ")
                .choices(choices)
                .value(init_value),
            id,
            config,
        }
    }

    /// Get the current color index in the current theme for the given ID.
    ///
    /// # Panics
    ///
    /// The only IDs expected are color IDs, everything else will panic.
    const fn init_color_select(id: IdCETheme, theme: &ThemeWrap) -> usize {
        match id {
            IdCETheme::LibraryForeground => theme.style.library.foreground_color.as_usize(),
            IdCETheme::LibraryBackground => theme.style.library.background_color.as_usize(),
            IdCETheme::LibraryBorder => theme.style.library.border_color.as_usize(),
            IdCETheme::LibraryHighlight => theme.style.library.highlight_color.as_usize(),
            IdCETheme::PlaylistForeground => theme.style.playlist.foreground_color.as_usize(),
            IdCETheme::PlaylistBackground => theme.style.playlist.background_color.as_usize(),
            IdCETheme::PlaylistBorder => theme.style.playlist.border_color.as_usize(),
            IdCETheme::PlaylistHighlight => theme.style.playlist.highlight_color.as_usize(),
            IdCETheme::ProgressForeground => theme.style.progress.foreground_color.as_usize(),
            IdCETheme::ProgressBackground => theme.style.progress.background_color.as_usize(),
            IdCETheme::ProgressBorder => theme.style.progress.border_color.as_usize(),
            IdCETheme::LyricForeground => theme.style.lyric.foreground_color.as_usize(),
            IdCETheme::LyricBackground => theme.style.lyric.background_color.as_usize(),
            IdCETheme::LyricBorder => theme.style.lyric.border_color.as_usize(),
            IdCETheme::ImportantPopupForeground => {
                theme.style.important_popup.foreground_color.as_usize()
            }
            IdCETheme::ImportantPopupBackground => {
                theme.style.important_popup.background_color.as_usize()
            }
            IdCETheme::ImportantPopupBorder => theme.style.important_popup.border_color.as_usize(),
            IdCETheme::FallbackForeground => theme.style.fallback.foreground_color.as_usize(),
            IdCETheme::FallbackBackground => theme.style.fallback.background_color.as_usize(),
            IdCETheme::FallbackBorder => theme.style.fallback.border_color.as_usize(),
            IdCETheme::FallbackHighlight => theme.style.fallback.highlight_color.as_usize(),

            // explicitly handle all cases
            IdCETheme::ThemeSelectTable
            | IdCETheme::LibraryHighlightSymbol
            | IdCETheme::LibraryLabel
            | IdCETheme::LyricLabel
            | IdCETheme::PlaylistHighlightSymbol
            | IdCETheme::PlaylistLabel
            | IdCETheme::CurrentlyPlayingTrackSymbol
            | IdCETheme::ProgressLabel
            | IdCETheme::ImportantPopupLabel
            | IdCETheme::FallbackLabel => unreachable!(),
        }
    }

    fn update_color(&mut self, index: usize) -> Msg {
        if let Some(color_config) = COLOR_LIST.get(index) {
            let color = self
                .config
                .read()
                .settings
                .theme
                .get_color_from_theme(*color_config);
            // self.attr(Attribute::Foreground, AttrValue::Color(color));
            self.attr(Attribute::Background, AttrValue::Color(color));
            self.attr(
                Attribute::Borders,
                AttrValue::Borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(color),
                ),
            );
            self.attr(
                Attribute::FocusStyle,
                AttrValue::Style(Style::default().add_modifier(Modifier::BOLD).bg(color)),
            );
            Msg::ConfigEditor(ConfigEditorMsg::ColorChanged(
                IdConfigEditor::Theme(self.id),
                *color_config,
            ))
        } else {
            self.attr(Attribute::Background, AttrValue::Color(Color::Red));
            self.attr(
                Attribute::Borders,
                AttrValue::Borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Red),
                ),
            );
            self.attr(
                Attribute::FocusStyle,
                AttrValue::Style(Style::default().add_modifier(Modifier::BOLD).bg(Color::Red)),
            );

            Msg::ForceRedraw
        }
    }
}

impl Component<Msg, UserEvent> for CEColorSelect {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            // Global Hotkeys
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk));
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout));
            }

            Event::Keyboard(key) if key == keys.escape.get() => match self.state() {
                State::One(_) => return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel)),
                _ => self.perform(Cmd::Cancel),
            },
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => match self.state() {
                State::One(_) => return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel)),
                _ => self.perform(Cmd::Cancel),
            },

            Event::Keyboard(key) if key == keys.navigation_keys.up.get() => match self.state() {
                State::One(_) => {
                    return Some(Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)));
                }
                _ => self.perform(Cmd::Move(Direction::Up)),
            },

            Event::Keyboard(key) if key == keys.navigation_keys.down.get() => match self.state() {
                State::One(_) => {
                    return Some(Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)));
                }
                _ => self.perform(Cmd::Move(Direction::Down)),
            },

            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => match self.state() {
                State::One(_) => {
                    return Some(Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)));
                }
                _ => self.perform(Cmd::Move(Direction::Up)),
            },

            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => match self.state() {
                State::One(_) => {
                    return Some(Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next)));
                }
                _ => self.perform(Cmd::Move(Direction::Down)),
            },

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                // "Select" returns "None" as a result for "Submit" when transitioning from "closed" to "open" state.
                // But does return something when transitioning from "open" to "closed" state.
                match self.perform(Cmd::Submit) {
                    CmdResult::None => return Some(Msg::ForceRedraw),
                    v => v,
                }
            }
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::Usize(index))) => {
                Some(self.update_color(index))
            }
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

#[derive(MockComponent)]
pub struct CEStyleTitle {
    component: Label,
}

impl CEStyleTitle {
    pub fn new(config: &SharedTuiSettings, text: &str) -> Self {
        let config_tui = config.read_recursive();

        Self {
            component: Label::default()
                .foreground(config_tui.settings.theme.lyric_foreground())
                .background(config_tui.settings.theme.lyric_background())
                .modifiers(TextModifiers::BOLD)
                .text(text),
        }
    }
}

impl Component<Msg, UserEvent> for CEStyleTitle {
    fn on(&mut self, _ev: Event<UserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct ConfigInputHighlight {
    component: Input,
    id: IdConfigEditor,
    config: SharedTuiSettings,
}

impl ConfigInputHighlight {
    pub fn new(name: &str, id: IdConfigEditor, config: SharedTuiSettings) -> Self {
        let config_r = config.read();
        // TODO: this should likely not be here, because it is a runtime error if it is unhandled
        let highlight_str = match id {
            IdConfigEditor::Theme(IdCETheme::LibraryHighlightSymbol) => {
                &config_r.settings.theme.style.library.highlight_symbol
            }
            IdConfigEditor::Theme(IdCETheme::PlaylistHighlightSymbol) => {
                &config_r.settings.theme.style.playlist.highlight_symbol
            }
            IdConfigEditor::Theme(IdCETheme::CurrentlyPlayingTrackSymbol) => {
                &config_r.settings.theme.style.playlist.current_track_symbol
            }
            _ => todo!("Unhandled IdConfigEditor Variant: {:#?}", id),
        };
        let component = Input::default()
            .borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(config_r.settings.theme.library_border()),
            )
            // .foreground(color)
            .background(config_r.settings.theme.library_background())
            .inactive(Style::new().bg(config_r.settings.theme.library_background()))
            .input_type(InputType::Text)
            .placeholder(
                "1f984/1f680/1f8a5",
                Style::default().fg(Color::Rgb(128, 128, 128)),
            )
            .title(name, Alignment::Left)
            .value(highlight_str);

        drop(config_r);
        Self {
            component,
            id,
            config,
        }
    }
    fn update_symbol(&mut self, result: CmdResult) -> Msg {
        if let CmdResult::Changed(State::One(StateValue::String(symbol))) = result.clone() {
            if symbol.is_empty() {
                let color = self.config.read().settings.theme.library_border();
                self.update_symbol_after(color);
                return Msg::ForceRedraw;
            }
            if let Some(s) = Self::string_to_unicode_char(&symbol) {
                // success getting a unicode letter
                self.update_symbol_after(Color::Green);
                return Msg::ConfigEditor(ConfigEditorMsg::SymbolChanged(self.id, s.to_string()));
            }
            // fail to get a unicode letter
            self.update_symbol_after(Color::Red);
        }

        // press enter to see preview
        if let CmdResult::Submit(State::One(StateValue::String(symbol))) = result
            && let Some(s) = Self::string_to_unicode_char(&symbol)
        {
            self.attr(Attribute::Value, AttrValue::String(s.to_string()));
            self.update_symbol_after(Color::Green);
            return Msg::ConfigEditor(ConfigEditorMsg::SymbolChanged(self.id, s.to_string()));
        }
        Msg::ForceRedraw
    }
    fn update_symbol_after(&mut self, color: Color) {
        self.attr(Attribute::Foreground, AttrValue::Color(color));
        self.attr(
            Attribute::Borders,
            AttrValue::Borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(color),
            ),
        );
    }
    fn string_to_unicode_char(s: &str) -> Option<char> {
        // Do something more appropriate to find the actual number
        // let number = &s[..];

        u32::from_str_radix(s, 16)
            .ok()
            .and_then(std::char::from_u32)
    }
}

impl Component<Msg, UserEvent> for ConfigInputHighlight {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        match ev {
            // Global Hotkeys
            Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout))
            }
            Event::Keyboard(keyevent) if keyevent == keys.escape.get() => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel))
            }
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
                let result = self.perform(Cmd::Cancel);
                Some(self.update_symbol(result))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => {
                let result = self.perform(Cmd::Delete);
                Some(self.update_symbol(result))
            }

            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            }) => {
                let result = self.perform(Cmd::Type(ch));
                Some(self.update_symbol(result))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => Some(Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Next))),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::Theme(KFMsg::Previous)))
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                let result = self.perform(Cmd::Submit);
                Some(self.update_symbol(result))
            }
            _ => None,
        }
    }
}

// --- Section Library Style ---

#[inline]
fn library_title(config: &SharedTuiSettings) -> CEStyleTitle {
    CEStyleTitle::new(config, " Library style ")
}

#[inline]
fn library_fg(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.library_foreground();
    CEColorSelect::new(" Foreground ", IdCETheme::LibraryForeground, color, config)
}

#[inline]
fn library_bg(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.library_background();
    CEColorSelect::new(" Background ", IdCETheme::LibraryBackground, color, config)
}

#[inline]
fn library_border(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.library_border();
    CEColorSelect::new(" Border ", IdCETheme::LibraryBorder, color, config)
}

#[inline]
fn library_highlight(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.library_highlight();
    CEColorSelect::new(" Highlight ", IdCETheme::LibraryHighlight, color, config)
}

// --- Section Playlist Style ---

#[inline]
fn playlist_title(config: &SharedTuiSettings) -> CEStyleTitle {
    CEStyleTitle::new(config, " Playlist style ")
}

#[inline]
fn playlist_fg(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.playlist_foreground();
    CEColorSelect::new(" Foreground ", IdCETheme::PlaylistForeground, color, config)
}

#[inline]
fn playlist_bg(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.playlist_background();
    CEColorSelect::new(" Background ", IdCETheme::PlaylistBackground, color, config)
}

#[inline]
fn playlist_border(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.playlist_border();
    CEColorSelect::new(" Border ", IdCETheme::PlaylistBorder, color, config)
}

#[inline]
fn playlist_highlight(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.playlist_highlight();
    CEColorSelect::new(" Highlight ", IdCETheme::PlaylistHighlight, color, config)
}

// --- Section Progress Style ---

#[inline]
fn progress_title(config: &SharedTuiSettings) -> CEStyleTitle {
    CEStyleTitle::new(config, " Progress style ")
}

#[inline]
fn progress_fg(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.progress_foreground();
    CEColorSelect::new(" Foreground ", IdCETheme::ProgressForeground, color, config)
}

#[inline]
fn progress_bg(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.progress_background();
    CEColorSelect::new(" Background ", IdCETheme::ProgressBackground, color, config)
}

#[inline]
fn progress_border(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.progress_border();
    CEColorSelect::new(" Border ", IdCETheme::ProgressBorder, color, config)
}

// --- Section Lyric Style ---

#[inline]
fn lyric_title(config: &SharedTuiSettings) -> CEStyleTitle {
    CEStyleTitle::new(config, " Lyric style ")
}

#[inline]
fn lyric_fg(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.lyric_foreground();
    CEColorSelect::new(" Foreground ", IdCETheme::LyricForeground, color, config)
}

#[inline]
fn lyric_bg(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.lyric_background();
    CEColorSelect::new(" Background ", IdCETheme::LyricBackground, color, config)
}

#[inline]
fn lyric_border(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.lyric_border();
    CEColorSelect::new(" Border ", IdCETheme::LyricBorder, color, config)
}

// --- Section Important Popup Style ---

#[inline]
fn important_popup_title(config: &SharedTuiSettings) -> CEStyleTitle {
    CEStyleTitle::new(config, " Important Popup style ")
}

#[inline]
fn important_popup_fg(config: SharedTuiSettings) -> CEColorSelect {
    let color = config
        .read_recursive()
        .settings
        .theme
        .important_popup_foreground();
    CEColorSelect::new(
        " Foreground ",
        IdCETheme::ImportantPopupForeground,
        color,
        config,
    )
}

#[inline]
fn important_popup_bg(config: SharedTuiSettings) -> CEColorSelect {
    let color = config
        .read_recursive()
        .settings
        .theme
        .important_popup_background();
    CEColorSelect::new(
        " Background ",
        IdCETheme::ImportantPopupBackground,
        color,
        config,
    )
}

#[inline]
fn important_popup_border(config: SharedTuiSettings) -> CEColorSelect {
    let color = config
        .read_recursive()
        .settings
        .theme
        .important_popup_border();
    CEColorSelect::new(" Border ", IdCETheme::ImportantPopupBorder, color, config)
}

// --- Section Fallback Style ---

#[inline]
fn fallback_title(config: &SharedTuiSettings) -> CEStyleTitle {
    CEStyleTitle::new(config, " Fallback style ")
}

#[inline]
fn fallback_fg(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.fallback_foreground();
    CEColorSelect::new(" Foreground ", IdCETheme::FallbackForeground, color, config)
}

#[inline]
fn fallback_bg(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.fallback_background();
    CEColorSelect::new(" Background ", IdCETheme::FallbackBackground, color, config)
}

#[inline]
fn fallback_border(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.fallback_border();
    CEColorSelect::new(" Border ", IdCETheme::FallbackBorder, color, config)
}

#[inline]
fn fallback_highlight(config: SharedTuiSettings) -> CEColorSelect {
    let color = config.read_recursive().settings.theme.fallback_highlight();
    CEColorSelect::new(" Highlight ", IdCETheme::FallbackHighlight, color, config)
}

// --- Section Highlight Symbols ---

#[inline]
fn library_hg_symbol(config: SharedTuiSettings) -> ConfigInputHighlight {
    ConfigInputHighlight::new(
        " Highlight Symbol ",
        IdConfigEditor::Theme(IdCETheme::LibraryHighlightSymbol),
        config,
    )
}

#[inline]
fn playlist_hg_symbol(config: SharedTuiSettings) -> ConfigInputHighlight {
    ConfigInputHighlight::new(
        " Highlight Symbol ",
        IdConfigEditor::Theme(IdCETheme::PlaylistHighlightSymbol),
        config,
    )
}

#[inline]
fn currently_playing_track_symbol(config: SharedTuiSettings) -> ConfigInputHighlight {
    ConfigInputHighlight::new(
        " Current Track Symbol ",
        IdConfigEditor::Theme(IdCETheme::CurrentlyPlayingTrackSymbol),
        config,
    )
}

impl Model {
    /// Mount / Remount the Config-Editor's Second Page, the Theme, Color & Symbol Options
    #[allow(clippy::too_many_lines)]
    pub(super) fn remount_config_color(
        &mut self,
        config: &SharedTuiSettings,
        theme_idx: Option<usize>,
    ) -> Result<()> {
        // Mount color page
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ThemeSelectTable)),
            Box::new(CEThemeSelectTable::new(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LibraryLabel)),
            Box::new(library_title(config)),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LibraryForeground)),
            Box::new(library_fg(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LibraryBackground)),
            Box::new(library_bg(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LibraryBorder)),
            Box::new(library_border(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LibraryHighlight)),
            Box::new(library_highlight(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::PlaylistLabel)),
            Box::new(playlist_title(config)),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::PlaylistForeground)),
            Box::new(playlist_fg(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::PlaylistBackground)),
            Box::new(playlist_bg(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::PlaylistBorder)),
            Box::new(playlist_border(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::PlaylistHighlight)),
            Box::new(playlist_highlight(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ProgressLabel)),
            Box::new(progress_title(config)),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ProgressForeground)),
            Box::new(progress_fg(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ProgressBackground)),
            Box::new(progress_bg(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ProgressBorder)),
            Box::new(progress_border(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LyricLabel)),
            Box::new(lyric_title(config)),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LyricForeground)),
            Box::new(lyric_fg(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LyricBackground)),
            Box::new(lyric_bg(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LyricBorder)),
            Box::new(lyric_border(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ImportantPopupLabel)),
            Box::new(important_popup_title(config)),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ImportantPopupForeground)),
            Box::new(important_popup_fg(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ImportantPopupBackground)),
            Box::new(important_popup_bg(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::ImportantPopupBorder)),
            Box::new(important_popup_border(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::FallbackLabel)),
            Box::new(fallback_title(config)),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::FallbackForeground)),
            Box::new(fallback_fg(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::FallbackBackground)),
            Box::new(fallback_bg(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::FallbackBorder)),
            Box::new(fallback_border(config.clone())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::FallbackHighlight)),
            Box::new(fallback_highlight(config.clone())),
            Vec::new(),
        )?;

        self.remount_config_color_symbols(config)?;

        self.theme_select_sync(theme_idx);

        Ok(())
    }

    /// Mount / Remount the Config-Editor's Second Page, Symbols
    fn remount_config_color_symbols(&mut self, config: &SharedTuiSettings) -> Result<()> {
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::LibraryHighlightSymbol)),
            Box::new(library_hg_symbol(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(IdCETheme::PlaylistHighlightSymbol)),
            Box::new(playlist_hg_symbol(config.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Theme(
                IdCETheme::CurrentlyPlayingTrackSymbol,
            )),
            Box::new(currently_playing_track_symbol(config.clone())),
            Vec::new(),
        )?;

        Ok(())
    }

    /// Unmount the Config-Editor's Second Page, the Theme, Color & Symbol Options
    pub(super) fn umount_config_color(&mut self) -> Result<()> {
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ThemeSelectTable,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LibraryLabel,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LibraryForeground,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LibraryBackground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LibraryBorder,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LibraryHighlight,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::PlaylistLabel,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::PlaylistForeground,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::PlaylistBackground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::PlaylistBorder,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::PlaylistHighlight,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ProgressLabel,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ProgressForeground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ProgressBackground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ProgressBorder,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LyricLabel,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LyricForeground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LyricBackground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LyricBorder,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ImportantPopupLabel,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ImportantPopupForeground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ImportantPopupBackground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::ImportantPopupBorder,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::FallbackLabel,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::FallbackForeground,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::FallbackBackground,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::FallbackBorder,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::FallbackHighlight,
        )))?;

        self.umount_config_color_symbols()?;

        Ok(())
    }

    /// Unmount the Config-Editor's Second Page, Symbols
    pub fn umount_config_color_symbols(&mut self) -> Result<()> {
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::LibraryHighlightSymbol,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::PlaylistHighlightSymbol,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Theme(
            IdCETheme::CurrentlyPlayingTrackSymbol,
        )))?;

        Ok(())
    }
}
