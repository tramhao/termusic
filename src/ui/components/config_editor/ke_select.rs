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
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Blue),
                )
                .foreground(Color::Blue)
                .title(name, Alignment::Left)
                .rewind(false)
                // .inactive(Style::default().bg(Color::Green))
                .highlighted_color(Color::LightGreen)
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
            // IdConfigEditor::GlobalLeft => keys.global_left.modifier(),
            // IdConfigEditor::GlobalRight => keys.global_right.modifier(),
            // IdConfigEditor::GlobalUp => keys.global_up.modifier(),
            // IdConfigEditor::GlobalDown => keys.global_down.modifier(),
            // IdConfigEditor::GlobalGotoTop => keys.global_goto_top.modifier(),
            // IdConfigEditor::GlobalGotoBottom => keys.global_goto_bottom.modifier(),
            // IdConfigEditor::GlobalPlayerTogglePause => keys.global_player_toggle_pause.modifier(),
            // IdConfigEditor::GlobalPlayerNext => keys.global_player_next.modifier(),
            // IdConfigEditor::GlobalPlayerPrevious => keys.global_player_previous.modifier(),
            // IdConfigEditor::GlobalHelp => keys.global_help.modifier(),
            // IdConfigEditor::GlobalVolumeUp => keys.global_player_volume_plus_2.modifier(),
            // IdConfigEditor::GlobalVolumeDown => keys.global_player_volume_minus_2.modifier(),
            // IdConfigEditor::GlobalPlayerSeekForward => keys.global_player_seek_forward.modifier(),
            // IdConfigEditor::GlobalPlayerSeekBackward => keys.global_player_seek_backward.modifier(),
            // IdConfigEditor::GlobalPlayerSpeedUp => keys.global_player_speed_up.modifier(),
            // IdConfigEditor::GlobalPlayerSpeedDown => keys.global_player_speed_down.modifier(),
            // IdConfigEditor::GlobalLyricAdjustForward => keys.global_lyric_adjust_forward.modifier(),
            // IdConfigEditor::GlobalLyricAdjustBackward => keys.global_lyric_adjust_backward.modifier(),
            // IdConfigEditor::GlobalLyricCycle => keys.global_lyric_cycle.modifier(),
            // IdConfigEditor::GlobalColorEditor => keys.global_color_editor_open.modifier(),
            // IdConfigEditor::GlobalConfigEditor => keys.global_key_editor_open.modifier(),
            // IdConfigEditor::LibraryDelete => keys.library_delete.modifier(),
            // IdConfigEditor::LibraryLoadDir => keys.library_load_dir.modifier(),
            // IdConfigEditor::LibraryYank => keys.library_yank.modifier(),
            // IdConfigEditor::LibraryPaste => keys.library_paste.modifier(),
            // IdConfigEditor::LibrarySearch => keys.library_search.modifier(),
            // IdConfigEditor::LibrarySearchYoutube => keys.library_search_youtube.modifier(),
            // IdConfigEditor::LibraryTagEditor => keys.library_tag_editor_open.modifier(),
            // IdConfigEditor::PlaylistDelete => keys.playlist_delete.modifier(),
            // IdConfigEditor::PlaylistDeleteAll => keys.playlist_delete_all.modifier(),
            // IdConfigEditor::PlaylistShuffle => keys.playlist_shuffle.modifier(),
            // IdConfigEditor::PlaylistSearch => keys.playlist_search.modifier(),
            // IdConfigEditor::PlaylistAddFront => keys.playlist_add_front.modifier(),
            // IdConfigEditor::PlaylistPlaySelected => keys.playlist_play_selected.modifier(),
            // IdConfigEditor::PlaylistModeCycle => keys.playlist_mode_cycle.modifier(),
            // IdConfigEditor::PlaylistSwapDown => keys.playlist_swap_down.modifier(),
            // IdConfigEditor::PlaylistSwapUp => keys.playlist_swap_up.modifier(),
            // IdConfigEditor::GlobalLayoutTreeview => keys.global_layout_treeview.modifier(),
            // IdConfigEditor::GlobalLayoutDatabase => keys.global_layout_database.modifier(),
            // IdConfigEditor::DatabaseAddAll => keys.database_add_all.modifier(),
            // IdConfigEditor::GlobalPlayerToggleGapless => keys.global_player_toggle_gapless.modifier(),
            _ => 0,
        }
    }
}

impl Component<Msg, NoUserEvent> for KEModifierSelect {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(self.on_key_tab.clone())
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backtab.clone()),

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
            // Event::Keyboard(keyevent) if keyevent == self.keys.global_help.key_event() => {
            //     return Some(Msg::ConfigEditor(ConfigEditorMsg::HelpPopupShow))
            // }
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
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
                "Global Left",
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
                "Global Down",
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
                "Global Right",
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
                "Global Up",
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
                "G Goto Top",
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
                "G Goto Bottom",
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
                "G Pause Toggle",
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
                "G Next Song",
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
                "G Previous Song",
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
                "Global Help",
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
                "Global Volume +  ",
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
                "Global Volume -  ",
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
                "G Seek Forward",
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
                "G Seek Backward",
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
                "G Speed Up",
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
                "G Speed Down",
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
                "G Lyric Forward",
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
                "G LyricBackward",
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
                "G Lyric Cycle",
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
                "Player Toggle Gapless",
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

// #[derive(MockComponent)]
// pub struct KEGlobalColorEditor {
//     component: KEModifierSelect,
// }

// impl KEGlobalColorEditor {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "G Color Editor",
//                 IdConfigEditor::GlobalColorEditor,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::GlobalColorEditorBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::GlobalColorEditorBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEGlobalColorEditor {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEGlobalConfigEditor {
//     component: KEModifierSelect,
// }

// impl KEGlobalConfigEditor {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "G Key Editor",
//                 IdConfigEditor::GlobalConfigEditor,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::GlobalConfigEditorBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::GlobalConfigEditorBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEGlobalConfigEditor {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KELibraryDelete {
//     component: KEModifierSelect,
// }

// impl KELibraryDelete {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "Library Delete",
//                 IdConfigEditor::LibraryDelete,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryDeleteBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryDeleteBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibraryDelete {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KELibraryLoadDir {
//     component: KEModifierSelect,
// }

// impl KELibraryLoadDir {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "L Load Dir",
//                 IdConfigEditor::LibraryLoadDir,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryLoadDirBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryLoadDirBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibraryLoadDir {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KELibraryYank {
//     component: KEModifierSelect,
// }

// impl KELibraryYank {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "Library Yank",
//                 IdConfigEditor::LibraryYank,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryYankBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryYankBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibraryYank {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KELibraryPaste {
//     component: KEModifierSelect,
// }

// impl KELibraryPaste {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "Library Paste",
//                 IdConfigEditor::LibraryPaste,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryPasteBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryPasteBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibraryPaste {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KELibrarySearch {
//     component: KEModifierSelect,
// }

// impl KELibrarySearch {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "Library Search",
//                 IdConfigEditor::LibrarySearch,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibrarySearchBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibrarySearchBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibrarySearch {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KELibrarySearchYoutube {
//     component: KEModifierSelect,
// }

// impl KELibrarySearchYoutube {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "L SearchYoutube",
//                 IdConfigEditor::LibrarySearchYoutube,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibrarySearchYoutubeBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibrarySearchYoutubeBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibrarySearchYoutube {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KELibraryTagEditor {
//     component: KEModifierSelect,
// }

// impl KELibraryTagEditor {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "L Tag Editor",
//                 IdConfigEditor::LibraryTagEditor,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryTagEditorBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryTagEditorBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibraryTagEditor {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistDelete {
//     component: KEModifierSelect,
// }

// impl KEPlaylistDelete {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "Playlist Delete",
//                 IdConfigEditor::PlaylistDelete,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistDeleteBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistDeleteBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistDelete {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistDeleteAll {
//     component: KEModifierSelect,
// }

// impl KEPlaylistDeleteAll {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "P Delete All",
//                 IdConfigEditor::PlaylistDeleteAll,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistDeleteAllBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistDeleteAllBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistDeleteAll {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistShuffle {
//     component: KEModifierSelect,
// }

// impl KEPlaylistShuffle {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "P Shuffle",
//                 IdConfigEditor::PlaylistShuffle,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistShuffleBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistShuffleBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistShuffle {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistModeCycle {
//     component: KEModifierSelect,
// }

// impl KEPlaylistModeCycle {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "P Mode Cycle",
//                 IdConfigEditor::PlaylistModeCycle,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistModeCycleBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistModeCycleBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistModeCycle {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistPlaySelected {
//     component: KEModifierSelect,
// }

// impl KEPlaylistPlaySelected {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "P Play Selected",
//                 IdConfigEditor::PlaylistPlaySelected,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistPlaySelectedBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistPlaySelectedBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistPlaySelected {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistAddFront {
//     component: KEModifierSelect,
// }

// impl KEPlaylistAddFront {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "P Add Front",
//                 IdConfigEditor::PlaylistAddFront,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistAddFrontBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistAddFrontBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistAddFront {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistSearch {
//     component: KEModifierSelect,
// }

// impl KEPlaylistSearch {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "Playlist Search",
//                 IdConfigEditor::PlaylistSearch,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistSearchBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistSearchBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistSearch {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistSwapDown {
//     component: KEModifierSelect,
// }

// impl KEPlaylistSwapDown {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "P Swap Down",
//                 IdConfigEditor::PlaylistSwapDown,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistSwapDownBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistSwapDownBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistSwapDown {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistSwapUp {
//     component: KEModifierSelect,
// }

// impl KEPlaylistSwapUp {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "P Swap Up",
//                 IdConfigEditor::PlaylistSwapUp,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistSwapUpBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistSwapUpBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistSwapUp {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }
// #[derive(MockComponent)]
// pub struct KEDatabaseAddAll {
//     component: KEModifierSelect,
// }

// impl KEDatabaseAddAll {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEModifierSelect::new(
//                 "DB Add All",
//                 IdConfigEditor::DatabaseAddAll,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::DatabaseAddAllBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::DatabaseAddAllBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEDatabaseAddAll {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }
