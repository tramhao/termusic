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
use super::ColorConfig;
use crate::ui::components::StyleColorSymbol;
use crate::ui::{CEMsg, IdColorEditor, Msg};
use lazy_static::lazy_static;
use regex::Regex;
use std::convert::From;
use tui_realm_stdlib::{Label, Select};
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders, Color, Style, TextModifiers};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

lazy_static! {
    /**
     * Regex matches:
     * - group 1: Red
     * - group 2: Green
     * - group 3: Blue
     */
    static ref COLOR_HEX_REGEX: Regex = Regex::new(r"#(:?[0-9a-fA-F]{2})(:?[0-9a-fA-F]{2})(:?[0-9a-fA-F]{2})").unwrap();
}

const COLOR_LIST: [ColorConfig; 19] = [
    ColorConfig::Reset,
    ColorConfig::Foreground,
    ColorConfig::Background,
    ColorConfig::Black,
    ColorConfig::Red,
    ColorConfig::Green,
    ColorConfig::Yellow,
    ColorConfig::Blue,
    ColorConfig::Magenta,
    ColorConfig::Cyan,
    ColorConfig::White,
    ColorConfig::LightBlack,
    ColorConfig::LightRed,
    ColorConfig::LightGreen,
    ColorConfig::LightYellow,
    ColorConfig::LightBlue,
    ColorConfig::LightMagenta,
    ColorConfig::LightCyan,
    ColorConfig::LightWhite,
];

#[derive(MockComponent)]
pub struct CESelectColor {
    component: Select,
    id: IdColorEditor,
    style_color_symbol: StyleColorSymbol,
    on_key_shift: Msg,
    on_key_backshift: Msg,
}

impl CESelectColor {
    pub fn new(
        name: &str,
        id: IdColorEditor,
        color: Color,
        style_color_symbol: &StyleColorSymbol,
        on_key_shift: Msg,
        on_key_backshift: Msg,
    ) -> Self {
        let init_value = Self::init_color_select(&id, style_color_symbol);
        let mut choices = vec![];
        for color in &COLOR_LIST {
            choices.push(String::from(color.clone()));
        }
        Self {
            component: Select::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(color),
                )
                .foreground(color)
                .title(name, Alignment::Left)
                .rewind(false)
                .inactive(Style::default().bg(color))
                .highlighted_color(Color::LightGreen)
                .highlighted_str(">> ")
                .choices(&choices)
                .value(init_value),
            id,
            style_color_symbol: style_color_symbol.clone(),
            on_key_shift,
            on_key_backshift,
        }
    }

    const fn init_color_select(id: &IdColorEditor, style_color_symbol: &StyleColorSymbol) -> usize {
        match *id {
            IdColorEditor::LibraryForeground => style_color_symbol.library_foreground.as_usize(),
            IdColorEditor::LibraryBackground => style_color_symbol.library_background.as_usize(),
            IdColorEditor::LibraryBorder => style_color_symbol.library_border.as_usize(),
            IdColorEditor::LibraryHighlight => style_color_symbol.library_highlight.as_usize(),
            IdColorEditor::PlaylistForeground => style_color_symbol.playlist_foreground.as_usize(),
            IdColorEditor::PlaylistBackground => style_color_symbol.playlist_background.as_usize(),
            IdColorEditor::PlaylistBorder => style_color_symbol.playlist_border.as_usize(),
            IdColorEditor::PlaylistHighlight => style_color_symbol.playlist_highlight.as_usize(),
            IdColorEditor::ProgressForeground => style_color_symbol.progress_foreground.as_usize(),
            IdColorEditor::ProgressBackground => style_color_symbol.progress_background.as_usize(),
            IdColorEditor::ProgressBorder => style_color_symbol.progress_border.as_usize(),
            IdColorEditor::LyricForeground => style_color_symbol.lyric_foreground.as_usize(),
            IdColorEditor::LyricBackground => style_color_symbol.lyric_background.as_usize(),
            IdColorEditor::LyricBorder => style_color_symbol.lyric_border.as_usize(),

            _ => 0,
        }
    }

    fn update_color(&mut self, index: usize) -> Msg {
        if let Some(color_config) = COLOR_LIST.get(index) {
            let color = color_config
                .color(&self.style_color_symbol.alacritty_theme)
                .unwrap_or(Color::Red);
            self.attr(Attribute::Foreground, AttrValue::Color(color));
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
                AttrValue::Style(Style::default().bg(color)),
            );
            Msg::ColorEditor(CEMsg::ColorChanged(self.id.clone(), color_config.clone()))
        } else {
            self.attr(Attribute::Foreground, AttrValue::Color(Color::Red));
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
                AttrValue::Style(Style::default().bg(Color::Red)),
            );

            Msg::None
        }
    }
}

impl Component<Msg, NoUserEvent> for CESelectColor {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(self.on_key_shift.clone())
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backshift.clone()),
            Event::Keyboard(KeyEvent {
                code: Key::Esc | Key::Char('q'),
                ..
            }) => match self.state() {
                State::One(_) => return Some(Msg::ColorEditor(CEMsg::ColorEditorCloseCancel)),
                _ => self.perform(Cmd::Cancel),
            },
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => return Some(Msg::ColorEditor(CEMsg::HelpPopupShow)),

            Event::Keyboard(KeyEvent {
                code: Key::Down | Key::Char('j'),
                ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up | Key::Char('k'),
                ..
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::Usize(index))) => {
                Some(self.update_color(index))
                // Some(Msg::TESelectLyricOk(COLOR_LIST[index]))
            }
            _ => Some(Msg::None),
        }
    }
}

#[derive(MockComponent)]
pub struct CELibraryTitle {
    component: Label,
}

impl Default for CELibraryTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text("Library style"),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELibraryTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct CELibraryForeground {
    component: CESelectColor,
}

impl CELibraryForeground {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Foreground",
                IdColorEditor::LibraryForeground,
                style_color_symbol
                    .library_foreground()
                    .unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::LibraryForegroundBlurDown),
                Msg::ColorEditor(CEMsg::LibraryForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELibraryForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CELibraryBackground {
    component: CESelectColor,
}

impl CELibraryBackground {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Background",
                IdColorEditor::LibraryBackground,
                style_color_symbol
                    .library_background()
                    .unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::LibraryBackgroundBlurDown),
                Msg::ColorEditor(CEMsg::LibraryBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELibraryBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CELibraryBorder {
    component: CESelectColor,
}

impl CELibraryBorder {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Border",
                IdColorEditor::LibraryBorder,
                style_color_symbol.library_border().unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::LibraryBorderBlurDown),
                Msg::ColorEditor(CEMsg::LibraryBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELibraryBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CELibraryHighlight {
    component: CESelectColor,
}

impl CELibraryHighlight {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Highlight",
                IdColorEditor::LibraryHighlight,
                style_color_symbol
                    .library_highlight()
                    .unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::LibraryHighlightBlurDown),
                Msg::ColorEditor(CEMsg::LibraryHighlightBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELibraryHighlight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CEPlaylistTitle {
    component: Label,
}

impl Default for CEPlaylistTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text("Playlist style"),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEPlaylistTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct CEPlaylistForeground {
    component: CESelectColor,
}

impl CEPlaylistForeground {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Foreground",
                IdColorEditor::PlaylistForeground,
                style_color_symbol
                    .playlist_foreground()
                    .unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::PlaylistForegroundBlurDown),
                Msg::ColorEditor(CEMsg::PlaylistForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEPlaylistForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CEPlaylistBackground {
    component: CESelectColor,
}

impl CEPlaylistBackground {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Background",
                IdColorEditor::PlaylistBackground,
                style_color_symbol
                    .playlist_background()
                    .unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::PlaylistBackgroundBlurDown),
                Msg::ColorEditor(CEMsg::PlaylistBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEPlaylistBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CEPlaylistBorder {
    component: CESelectColor,
}

impl CEPlaylistBorder {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Border",
                IdColorEditor::PlaylistBorder,
                style_color_symbol.playlist_border().unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::PlaylistBorderBlurDown),
                Msg::ColorEditor(CEMsg::PlaylistBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEPlaylistBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CEPlaylistHighlight {
    component: CESelectColor,
}

impl CEPlaylistHighlight {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Highlight",
                IdColorEditor::PlaylistHighlight,
                style_color_symbol
                    .playlist_highlight()
                    .unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::PlaylistHighlightBlurDown),
                Msg::ColorEditor(CEMsg::PlaylistHighlightBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEPlaylistHighlight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CEProgressTitle {
    component: Label,
}

impl Default for CEProgressTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text("Progress style"),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEProgressTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct CEProgressForeground {
    component: CESelectColor,
}

impl CEProgressForeground {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Foreground",
                IdColorEditor::ProgressForeground,
                style_color_symbol
                    .progress_foreground()
                    .unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::ProgressForegroundBlurDown),
                Msg::ColorEditor(CEMsg::ProgressForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEProgressForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CEProgressBackground {
    component: CESelectColor,
}

impl CEProgressBackground {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Background",
                IdColorEditor::ProgressBackground,
                style_color_symbol
                    .progress_background()
                    .unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::ProgressBackgroundBlurDown),
                Msg::ColorEditor(CEMsg::ProgressBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEProgressBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CEProgressBorder {
    component: CESelectColor,
}

impl CEProgressBorder {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Border",
                IdColorEditor::ProgressBorder,
                style_color_symbol.progress_border().unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::ProgressBorderBlurDown),
                Msg::ColorEditor(CEMsg::ProgressBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEProgressBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CELyricTitle {
    component: Label,
}

impl Default for CELyricTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text("Lyric style"),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELyricTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct CELyricForeground {
    component: CESelectColor,
}

impl CELyricForeground {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Foreground",
                IdColorEditor::LyricForeground,
                style_color_symbol.lyric_foreground().unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::LyricForegroundBlurDown),
                Msg::ColorEditor(CEMsg::LyricForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELyricForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CELyricBackground {
    component: CESelectColor,
}

impl CELyricBackground {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Background",
                IdColorEditor::LyricBackground,
                style_color_symbol.lyric_background().unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::LyricBackgroundBlurDown),
                Msg::ColorEditor(CEMsg::LyricBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELyricBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct CELyricBorder {
    component: CESelectColor,
}

impl CELyricBorder {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: CESelectColor::new(
                "Border",
                IdColorEditor::LyricBorder,
                style_color_symbol.lyric_border().unwrap_or(Color::Blue),
                style_color_symbol,
                Msg::ColorEditor(CEMsg::LyricBorderBlurDown),
                Msg::ColorEditor(CEMsg::LyricBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for CELyricBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

/// ### `parse_hex_color`
///
/// Try to parse a color in hex format, such as:
///
///     - #f0ab05
///     - #AA33BC
pub fn parse_hex_color(color: &str) -> Option<Color> {
    COLOR_HEX_REGEX.captures(color).map(|groups| {
        Color::Rgb(
            u8::from_str_radix(groups.get(1).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
            u8::from_str_radix(groups.get(2).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
            u8::from_str_radix(groups.get(3).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
        )
    })
}
