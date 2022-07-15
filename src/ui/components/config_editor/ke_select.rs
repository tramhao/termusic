use super::{ALT_SHIFT, CONTROL_ALT, CONTROL_ALT_SHIFT, CONTROL_SHIFT};
use crate::config::Settings;
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
use crate::ui::{ConfigEditorMsg, IdConfigEditor, Msg};
use tui_realm_stdlib::Select;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders, Color};
use tuirealm::{Component, Event, MockComponent, State, StateValue};

#[derive(Debug, Clone, PartialEq)]
pub enum MyModifiers {
    None,
    Shift,
    Control,
    Alt,
    ControlShift,
    AltShift,
    ControlAlt,
    ControlAltShift,
}
impl From<MyModifiers> for &'static str {
    fn from(modifier: MyModifiers) -> Self {
        match modifier {
            MyModifiers::None => "none",
            MyModifiers::Shift => "shift",
            MyModifiers::Control => "control",
            MyModifiers::Alt => "alt",
            MyModifiers::ControlShift => "ctrl_shift",
            MyModifiers::AltShift => "alt_shift",
            MyModifiers::ControlAlt => "ctrl_alt",
            MyModifiers::ControlAltShift => "ctrl_alt_shift",
        }
    }
}

impl From<MyModifiers> for String {
    fn from(modifier: MyModifiers) -> Self {
        <MyModifiers as Into<&'static str>>::into(modifier).to_owned()
    }
}

impl MyModifiers {
    pub const fn modifier(&self) -> KeyModifiers {
        match self {
            Self::None => KeyModifiers::NONE,
            Self::Shift => KeyModifiers::SHIFT,
            Self::Control => KeyModifiers::CONTROL,
            Self::Alt => KeyModifiers::ALT,
            Self::ControlShift => CONTROL_SHIFT,
            Self::AltShift => ALT_SHIFT,
            Self::ControlAlt => CONTROL_ALT,
            Self::ControlAltShift => CONTROL_ALT_SHIFT,
        }
    }
}
pub const MODIFIER_LIST: [MyModifiers; 8] = [
    MyModifiers::None,
    MyModifiers::Shift,
    MyModifiers::Control,
    MyModifiers::Alt,
    MyModifiers::ControlShift,
    MyModifiers::AltShift,
    MyModifiers::ControlAlt,
    MyModifiers::ControlAltShift,
];

#[derive(MockComponent)]
pub struct KEModifierSelect {
    component: Select,
    id: IdConfigEditor,
    config: Settings,
    on_key_tab: Msg,
    on_key_backtab: Msg,
}

impl KEModifierSelect {
    pub fn new(
        name: &str,
        id: IdConfigEditor,
        config: &Settings,
        on_key_tab: Msg,
        on_key_backtab: Msg,
    ) -> Self {
        let init_value = Self::init_modifier_select(&id, config);
        let mut choices = vec![];
        for modifier in &MODIFIER_LIST {
            choices.push(String::from(modifier.clone()));
        }
        Self {
            component: Select::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Blue),
                )
                .title(name, Alignment::Left)
                .rewind(false)
                // .inactive(Style::default().bg(Color::Green))
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightGreen),
                )
                .highlighted_str(">> ")
                .choices(&choices)
                .value(init_value),
            id,
            config: config.clone(),
            on_key_tab,
            on_key_backtab,
        }
    }

    const fn init_modifier_select(id: &IdConfigEditor, config: &Settings) -> usize {
        match *id {
            IdConfigEditor::GlobalQuit => config.keys.global_quit.modifier(),
            IdConfigEditor::GlobalLeft => config.keys.global_left.modifier(),
            IdConfigEditor::GlobalRight => config.keys.global_right.modifier(),
            IdConfigEditor::GlobalUp => config.keys.global_up.modifier(),
            IdConfigEditor::GlobalDown => config.keys.global_down.modifier(),
            IdConfigEditor::GlobalGotoTop => config.keys.global_goto_top.modifier(),
            IdConfigEditor::GlobalGotoBottom => config.keys.global_goto_bottom.modifier(),
            IdConfigEditor::GlobalPlayerTogglePause => {
                config.keys.global_player_toggle_pause.modifier()
            }
            IdConfigEditor::GlobalPlayerNext => config.keys.global_player_next.modifier(),
            IdConfigEditor::GlobalPlayerPrevious => config.keys.global_player_previous.modifier(),
            IdConfigEditor::GlobalHelp => config.keys.global_help.modifier(),
            IdConfigEditor::GlobalVolumeUp => config.keys.global_player_volume_plus_2.modifier(),
            IdConfigEditor::GlobalVolumeDown => config.keys.global_player_volume_minus_2.modifier(),
            IdConfigEditor::GlobalPlayerSeekForward => {
                config.keys.global_player_seek_forward.modifier()
            }
            IdConfigEditor::GlobalPlayerSeekBackward => {
                config.keys.global_player_seek_backward.modifier()
            }
            IdConfigEditor::GlobalPlayerSpeedUp => config.keys.global_player_speed_up.modifier(),
            IdConfigEditor::GlobalPlayerSpeedDown => {
                config.keys.global_player_speed_down.modifier()
            }
            IdConfigEditor::GlobalLyricAdjustForward => {
                config.keys.global_lyric_adjust_forward.modifier()
            }
            IdConfigEditor::GlobalLyricAdjustBackward => {
                config.keys.global_lyric_adjust_backward.modifier()
            }
            IdConfigEditor::GlobalLyricCycle => config.keys.global_lyric_cycle.modifier(),
            IdConfigEditor::GlobalLayoutTreeview => config.keys.global_layout_treeview.modifier(),
            IdConfigEditor::GlobalLayoutDatabase => config.keys.global_layout_database.modifier(),
            IdConfigEditor::GlobalPlayerToggleGapless => {
                config.keys.global_player_toggle_gapless.modifier()
            }
            IdConfigEditor::LibraryDelete => config.keys.library_delete.modifier(),
            IdConfigEditor::LibraryLoadDir => config.keys.library_load_dir.modifier(),
            IdConfigEditor::LibraryYank => config.keys.library_yank.modifier(),
            IdConfigEditor::LibraryPaste => config.keys.library_paste.modifier(),
            IdConfigEditor::LibrarySearch => config.keys.library_search.modifier(),
            IdConfigEditor::LibrarySearchYoutube => config.keys.library_search_youtube.modifier(),
            IdConfigEditor::LibraryTagEditor => config.keys.library_tag_editor_open.modifier(),
            IdConfigEditor::PlaylistDelete => config.keys.playlist_delete.modifier(),
            IdConfigEditor::PlaylistDeleteAll => config.keys.playlist_delete_all.modifier(),
            IdConfigEditor::PlaylistShuffle => config.keys.playlist_shuffle.modifier(),
            IdConfigEditor::PlaylistSearch => config.keys.playlist_search.modifier(),
            IdConfigEditor::PlaylistAddFront => config.keys.playlist_add_front.modifier(),
            IdConfigEditor::PlaylistPlaySelected => config.keys.playlist_play_selected.modifier(),
            IdConfigEditor::PlaylistModeCycle => config.keys.playlist_mode_cycle.modifier(),
            IdConfigEditor::PlaylistSwapDown => config.keys.playlist_swap_down.modifier(),
            IdConfigEditor::PlaylistSwapUp => config.keys.playlist_swap_up.modifier(),
            IdConfigEditor::DatabaseAddAll => config.keys.database_add_all.modifier(),
            IdConfigEditor::GlobalConfig => config.keys.global_config_open.modifier(),
            _ => 0,
        }
    }
}

impl Component<Msg, NoUserEvent> for KEModifierSelect {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            // Global Hotkey
            Event::Keyboard(keyevent)
                if keyevent == self.config.keys.global_config_save.key_event() =>
            {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => match self.state() {
                State::One(_) => return Some(self.on_key_tab.clone()),
                _ => self.perform(Cmd::Move(Direction::Down)),
            },
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => match self.state() {
                State::One(_) => return Some(self.on_key_backtab.clone()),
                _ => self.perform(Cmd::Move(Direction::Up)),
            },
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout));
            }
            // Local Hotkey
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_esc.key_event() => {
                match self.state() {
                    State::One(_) => return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel)),
                    _ => self.perform(Cmd::Cancel),
                }
            }
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_quit.key_event() => {
                match self.state() {
                    State::One(_) => return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel)),
                    _ => self.perform(Cmd::Cancel),
                }
            }
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_down.key_event() => {
                match self.state() {
                    State::One(_) => return Some(self.on_key_tab.clone()),
                    _ => self.perform(Cmd::Move(Direction::Down)),
                }
            }
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_up.key_event() => {
                match self.state() {
                    State::One(_) => return Some(self.on_key_backtab.clone()),
                    _ => self.perform(Cmd::Move(Direction::Up)),
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::Usize(_index))) => {
                // Some(Msg::None)
                Some(Msg::ConfigEditor(ConfigEditorMsg::KeyChanged(
                    self.id.clone(),
                )))
            }
            _ => Some(Msg::None),
        }
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalQuit {
    component: KEModifierSelect,
}

impl ConfigGlobalQuit {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Quit ",
                IdConfigEditor::GlobalQuit,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalQuitBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalQuitBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalQuit {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLeft {
    component: KEModifierSelect,
}

impl ConfigGlobalLeft {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Left ",
                IdConfigEditor::GlobalLeft,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLeftBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLeftBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLeft {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalDown {
    component: KEModifierSelect,
}

impl ConfigGlobalDown {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Down ",
                IdConfigEditor::GlobalDown,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalDownBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalDownBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalDown {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalRight {
    component: KEModifierSelect,
}

impl ConfigGlobalRight {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Right ",
                IdConfigEditor::GlobalRight,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalRightBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalRightBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalRight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalUp {
    component: KEModifierSelect,
}

impl ConfigGlobalUp {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Up ",
                IdConfigEditor::GlobalUp,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalUpBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalUpBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalUp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalGotoTop {
    component: KEModifierSelect,
}

impl ConfigGlobalGotoTop {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Goto Top ",
                IdConfigEditor::GlobalGotoTop,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalGotoTopBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalGotoTopBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalGotoTop {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalGotoBottom {
    component: KEModifierSelect,
}

impl ConfigGlobalGotoBottom {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Goto Bottom ",
                IdConfigEditor::GlobalGotoBottom,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalGotoBottomBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalGotoBottomBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalGotoBottom {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerTogglePause {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerTogglePause {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Pause Toggle ",
                IdConfigEditor::GlobalPlayerTogglePause,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerTogglePauseBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerTogglePauseBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerTogglePause {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerNext {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerNext {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Next Song ",
                IdConfigEditor::GlobalPlayerNext,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerNextBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerNextBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerNext {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerPrevious {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerPrevious {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Previous Song ",
                IdConfigEditor::GlobalPlayerPrevious,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerPreviousBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerPreviousBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerPrevious {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalHelp {
    component: KEModifierSelect,
}

impl ConfigGlobalHelp {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Help ",
                IdConfigEditor::GlobalHelp,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalHelpBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalHelpBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalHelp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalVolumeUp {
    component: KEModifierSelect,
}

impl ConfigGlobalVolumeUp {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Volume + ",
                IdConfigEditor::GlobalVolumeUp,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalVolumeUpBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalVolumeUpBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalVolumeUp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalVolumeDown {
    component: KEModifierSelect,
}

impl ConfigGlobalVolumeDown {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Volume - ",
                IdConfigEditor::GlobalVolumeDown,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalVolumeDownBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalVolumeDownBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalVolumeDown {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerSeekForward {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerSeekForward {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Seek Forward ",
                IdConfigEditor::GlobalPlayerSeekForward,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSeekForwardBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSeekForwardBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerSeekForward {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerSeekBackward {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerSeekBackward {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Seek Backward ",
                IdConfigEditor::GlobalPlayerSeekBackward,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSeekBackwardBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSeekBackwardBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerSeekBackward {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerSpeedUp {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerSpeedUp {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Speed Up ",
                IdConfigEditor::GlobalPlayerSpeedUp,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSpeedUpBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSpeedUpBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerSpeedUp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerSpeedDown {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerSpeedDown {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Speed Down ",
                IdConfigEditor::GlobalPlayerSpeedDown,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSpeedDownBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSpeedDownBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerSpeedDown {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLyricAdjustForward {
    component: KEModifierSelect,
}

impl ConfigGlobalLyricAdjustForward {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Lyric Forward ",
                IdConfigEditor::GlobalLyricAdjustForward,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLyricAdjustForwardBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLyricAdjustForwardBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLyricAdjustForward {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLyricAdjustBackward {
    component: KEModifierSelect,
}

impl ConfigGlobalLyricAdjustBackward {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Lyric Backward ",
                IdConfigEditor::GlobalLyricAdjustBackward,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLyricAdjustBackwardBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLyricAdjustBackwardBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLyricAdjustBackward {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLyricCycle {
    component: KEModifierSelect,
}

impl ConfigGlobalLyricCycle {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Lyric Cycle ",
                IdConfigEditor::GlobalLyricCycle,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLyricCyleBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLyricCyleBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLyricCycle {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLayoutTreeview {
    component: KEModifierSelect,
}

impl ConfigGlobalLayoutTreeview {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Layout Tree ",
                IdConfigEditor::GlobalLayoutTreeview,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLayoutTreeviewBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLayoutTreeviewBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLayoutTreeview {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLayoutDatabase {
    component: KEModifierSelect,
}

impl ConfigGlobalLayoutDatabase {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Layout DataBase ",
                IdConfigEditor::GlobalLayoutDatabase,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLayoutDatabaseBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLayoutDatabaseBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLayoutDatabase {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerToggleGapless {
    component: KEModifierSelect,
}

impl ConfigGlobalPlayerToggleGapless {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Gapless Toggle ",
                IdConfigEditor::GlobalPlayerToggleGapless,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerToggleGaplessBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerToggleGaplessBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerToggleGapless {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryDelete {
    component: KEModifierSelect,
}

impl ConfigLibraryDelete {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Delete ",
                IdConfigEditor::LibraryDelete,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibraryDeleteBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibraryDeleteBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryDelete {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryLoadDir {
    component: KEModifierSelect,
}

impl ConfigLibraryLoadDir {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                "Library Load Dir",
                IdConfigEditor::LibraryLoadDir,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibraryLoadDirBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibraryLoadDirBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryLoadDir {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryYank {
    component: KEModifierSelect,
}

impl ConfigLibraryYank {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Yank ",
                IdConfigEditor::LibraryYank,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibraryYankBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibraryYankBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryYank {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryPaste {
    component: KEModifierSelect,
}

impl ConfigLibraryPaste {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Paste ",
                IdConfigEditor::LibraryPaste,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibraryPasteBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibraryPasteBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryPaste {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibrarySearch {
    component: KEModifierSelect,
}

impl ConfigLibrarySearch {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Library Search ",
                IdConfigEditor::LibrarySearch,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibrarySearchBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibrarySearchBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibrarySearch {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibrarySearchYoutube {
    component: KEModifierSelect,
}

impl ConfigLibrarySearchYoutube {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " L SearchYoutube ",
                IdConfigEditor::LibrarySearchYoutube,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibrarySearchYoutubeBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibrarySearchYoutubeBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibrarySearchYoutube {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryTagEditor {
    component: KEModifierSelect,
}

impl ConfigLibraryTagEditor {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                "L Tag Editor",
                IdConfigEditor::LibraryTagEditor,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibraryTagEditorBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibraryTagEditorBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryTagEditor {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistDelete {
    component: KEModifierSelect,
}

impl ConfigPlaylistDelete {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Playlist Delete ",
                IdConfigEditor::PlaylistDelete,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistDeleteBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistDeleteBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistDelete {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistDeleteAll {
    component: KEModifierSelect,
}

impl ConfigPlaylistDeleteAll {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " P Delete All ",
                IdConfigEditor::PlaylistDeleteAll,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistDeleteAllBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistDeleteAllBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistDeleteAll {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistShuffle {
    component: KEModifierSelect,
}

impl ConfigPlaylistShuffle {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                "Playlist Shuffle",
                IdConfigEditor::PlaylistShuffle,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistShuffleBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistShuffleBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistShuffle {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistModeCycle {
    component: KEModifierSelect,
}

impl ConfigPlaylistModeCycle {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " P Mode Cycle ",
                IdConfigEditor::PlaylistModeCycle,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistModeCycleBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistModeCycleBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistModeCycle {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistPlaySelected {
    component: KEModifierSelect,
}

impl ConfigPlaylistPlaySelected {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                "P Play Selected",
                IdConfigEditor::PlaylistPlaySelected,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistPlaySelectedBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistPlaySelectedBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistPlaySelected {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistAddFront {
    component: KEModifierSelect,
}

impl ConfigPlaylistAddFront {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " P Add Front ",
                IdConfigEditor::PlaylistAddFront,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistAddFrontBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistAddFrontBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistAddFront {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistSearch {
    component: KEModifierSelect,
}

impl ConfigPlaylistSearch {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                "Playlist Search",
                IdConfigEditor::PlaylistSearch,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistSearchBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistSearchBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistSearch {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistSwapDown {
    component: KEModifierSelect,
}

impl ConfigPlaylistSwapDown {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                "P Swap Down",
                IdConfigEditor::PlaylistSwapDown,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistSwapDownBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistSwapDownBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistSwapDown {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistSwapUp {
    component: KEModifierSelect,
}

impl ConfigPlaylistSwapUp {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                "P Swap Up",
                IdConfigEditor::PlaylistSwapUp,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistSwapUpBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistSwapUpBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistSwapUp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigDatabaseAddAll {
    component: KEModifierSelect,
}

impl ConfigDatabaseAddAll {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                "DB Add All",
                IdConfigEditor::DatabaseAddAll,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::DatabaseAddAllBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::DatabaseAddAllBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigDatabaseAddAll {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalConfig {
    component: KEModifierSelect,
}

impl ConfigGlobalConfig {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEModifierSelect::new(
                " Config Editor ",
                IdConfigEditor::GlobalConfig,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalConfigBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalConfigBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalConfig {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
