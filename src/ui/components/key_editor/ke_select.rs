use super::{Keys, ALT_SHIFT, CONTROL_ALT, CONTROL_ALT_SHIFT, CONTROL_SHIFT};
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
use crate::ui::{IdKeyEditor, KEMsg, Msg};
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
            MyModifiers::None => KeyModifiers::NONE,
            MyModifiers::Shift => KeyModifiers::SHIFT,
            MyModifiers::Control => KeyModifiers::CONTROL,
            MyModifiers::Alt => KeyModifiers::ALT,
            MyModifiers::ControlShift => CONTROL_SHIFT,
            MyModifiers::AltShift => ALT_SHIFT,
            MyModifiers::ControlAlt => CONTROL_ALT,
            MyModifiers::ControlAltShift => CONTROL_ALT_SHIFT,
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
pub struct KESelectModifier {
    component: Select,
    id: IdKeyEditor,
    // keys: Keys,
    on_key_shift: Msg,
    on_key_backshift: Msg,
}

impl KESelectModifier {
    pub fn new(
        name: &str,
        id: IdKeyEditor,
        keys: &Keys,
        on_key_shift: Msg,
        on_key_backshift: Msg,
    ) -> Self {
        let init_value = Self::init_modifier_select(&id, keys);
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
            // keys,
            on_key_shift,
            on_key_backshift,
        }
    }

    const fn init_modifier_select(id: &IdKeyEditor, keys: &Keys) -> usize {
        match *id {
            IdKeyEditor::GlobalQuit => keys.global_quit.modifier(),
            IdKeyEditor::GlobalLeft => keys.global_left.modifier(),
            IdKeyEditor::GlobalRight => keys.global_right.modifier(),
            IdKeyEditor::GlobalUp => keys.global_up.modifier(),
            IdKeyEditor::GlobalDown => keys.global_down.modifier(),
            IdKeyEditor::GlobalGotoTop => keys.global_goto_top.modifier(),
            IdKeyEditor::GlobalGotoBottom => keys.global_goto_bottom.modifier(),
            IdKeyEditor::GlobalPlayerTogglePause => keys.global_player_toggle_pause.modifier(),
            IdKeyEditor::GlobalPlayerNext => keys.global_player_next.modifier(),
            IdKeyEditor::GlobalPlayerPrevious => keys.global_player_previous.modifier(),
            IdKeyEditor::GlobalHelp => keys.global_help.modifier(),
            IdKeyEditor::GlobalVolumeUp => keys.global_player_volume_plus_2.modifier(),
            IdKeyEditor::GlobalVolumeDown => keys.global_player_volume_minus_2.modifier(),
            IdKeyEditor::GlobalPlayerSeekForward => keys.global_player_seek_forward.modifier(),
            IdKeyEditor::GlobalPlayerSeekBackward => keys.global_player_seek_backward.modifier(),
            IdKeyEditor::GlobalLyricAdjustForward => keys.global_lyric_adjust_forward.modifier(),
            IdKeyEditor::GlobalLyricAdjustBackward => keys.global_lyric_adjust_backward.modifier(),
            IdKeyEditor::GlobalLyricCycle => keys.global_lyric_cycle.modifier(),
            IdKeyEditor::GlobalColorEditor => keys.global_color_editor_open.modifier(),
            IdKeyEditor::GlobalKeyEditor => keys.global_key_editor_open.modifier(),
            IdKeyEditor::LibraryDelete => keys.library_delete.modifier(),
            IdKeyEditor::LibraryLoadDir => keys.library_load_dir.modifier(),
            IdKeyEditor::LibraryYank => keys.library_yank.modifier(),
            IdKeyEditor::LibraryPaste => keys.library_paste.modifier(),
            IdKeyEditor::LibrarySearch => keys.library_search.modifier(),
            IdKeyEditor::LibrarySearchYoutube => keys.library_search_youtube.modifier(),
            IdKeyEditor::LibraryTagEditor => keys.library_tag_editor_open.modifier(),
            IdKeyEditor::PlaylistDelete => keys.playlist_delete.modifier(),
            IdKeyEditor::PlaylistDeleteAll => keys.playlist_delete_all.modifier(),
            IdKeyEditor::PlaylistShuffle => keys.playlist_shuffle.modifier(),
            IdKeyEditor::PlaylistSearch => keys.playlist_search.modifier(),
            IdKeyEditor::PlaylistAddFront => keys.playlist_add_front.modifier(),
            IdKeyEditor::PlaylistPlaySelected => keys.playlist_play_selected.modifier(),
            IdKeyEditor::PlaylistModeCycle => keys.playlist_mode_cycle.modifier(),
            _ => 0,
        }
    }
}

impl Component<Msg, NoUserEvent> for KESelectModifier {
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
                State::One(_) => return Some(Msg::KeyEditor(KEMsg::KeyEditorCloseCancel)),
                _ => self.perform(Cmd::Cancel),
            },

            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => return Some(Msg::KeyEditor(KEMsg::HelpPopupShow)),
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
            CmdResult::Submit(State::One(StateValue::Usize(_index))) => {
                // Some(Msg::None)
                Some(Msg::KeyEditor(KEMsg::KeyChanged(self.id.clone())))
            }
            _ => Some(Msg::None),
        }
    }
}

#[derive(MockComponent)]
pub struct KEGlobalQuit {
    component: KESelectModifier,
}

impl KEGlobalQuit {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Global Quit",
                IdKeyEditor::GlobalQuit,
                keys,
                Msg::KeyEditor(KEMsg::GlobalQuitBlurDown),
                Msg::KeyEditor(KEMsg::GlobalQuitBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalQuit {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalLeft {
    component: KESelectModifier,
}

impl KEGlobalLeft {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Global Left",
                IdKeyEditor::GlobalLeft,
                keys,
                Msg::KeyEditor(KEMsg::GlobalLeftBlurDown),
                Msg::KeyEditor(KEMsg::GlobalLeftBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalLeft {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalDown {
    component: KESelectModifier,
}

impl KEGlobalDown {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Global Down",
                IdKeyEditor::GlobalDown,
                keys,
                Msg::KeyEditor(KEMsg::GlobalDownBlurDown),
                Msg::KeyEditor(KEMsg::GlobalDownBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalDown {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct KEGlobalRight {
    component: KESelectModifier,
}

impl KEGlobalRight {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Global Right",
                IdKeyEditor::GlobalRight,
                keys,
                Msg::KeyEditor(KEMsg::GlobalRightBlurDown),
                Msg::KeyEditor(KEMsg::GlobalRightBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalRight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct KEGlobalUp {
    component: KESelectModifier,
}

impl KEGlobalUp {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Global Up",
                IdKeyEditor::GlobalUp,
                keys,
                Msg::KeyEditor(KEMsg::GlobalUpBlurDown),
                Msg::KeyEditor(KEMsg::GlobalUpBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalUp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalGotoTop {
    component: KESelectModifier,
}

impl KEGlobalGotoTop {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "G Goto Top",
                IdKeyEditor::GlobalGotoTop,
                keys,
                Msg::KeyEditor(KEMsg::GlobalGotoTopBlurDown),
                Msg::KeyEditor(KEMsg::GlobalGotoTopBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalGotoTop {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalGotoBottom {
    component: KESelectModifier,
}

impl KEGlobalGotoBottom {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "G Goto Bottom",
                IdKeyEditor::GlobalGotoBottom,
                keys,
                Msg::KeyEditor(KEMsg::GlobalGotoBottomBlurDown),
                Msg::KeyEditor(KEMsg::GlobalGotoBottomBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalGotoBottom {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalPlayerTogglePause {
    component: KESelectModifier,
}

impl KEGlobalPlayerTogglePause {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "G Pause Toggle",
                IdKeyEditor::GlobalPlayerTogglePause,
                keys,
                Msg::KeyEditor(KEMsg::GlobalPlayerTogglePauseBlurDown),
                Msg::KeyEditor(KEMsg::GlobalPlayerTogglePauseBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalPlayerTogglePause {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalPlayerNext {
    component: KESelectModifier,
}

impl KEGlobalPlayerNext {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "G Next Song",
                IdKeyEditor::GlobalPlayerNext,
                keys,
                Msg::KeyEditor(KEMsg::GlobalPlayerNextBlurDown),
                Msg::KeyEditor(KEMsg::GlobalPlayerNextBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalPlayerNext {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalPlayerPrevious {
    component: KESelectModifier,
}

impl KEGlobalPlayerPrevious {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "G Previous Song",
                IdKeyEditor::GlobalPlayerPrevious,
                keys,
                Msg::KeyEditor(KEMsg::GlobalPlayerPreviousBlurDown),
                Msg::KeyEditor(KEMsg::GlobalPlayerPreviousBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalPlayerPrevious {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalHelp {
    component: KESelectModifier,
}

impl KEGlobalHelp {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Global Help",
                IdKeyEditor::GlobalHelp,
                keys,
                Msg::KeyEditor(KEMsg::GlobalHelpBlurDown),
                Msg::KeyEditor(KEMsg::GlobalHelpBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalHelp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct KEGlobalVolumeUp {
    component: KESelectModifier,
}

impl KEGlobalVolumeUp {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Global Volume +  ",
                IdKeyEditor::GlobalVolumeUp,
                keys,
                Msg::KeyEditor(KEMsg::GlobalVolumeUpBlurDown),
                Msg::KeyEditor(KEMsg::GlobalVolumeUpBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalVolumeUp {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalVolumeDown {
    component: KESelectModifier,
}

impl KEGlobalVolumeDown {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Global Volume -  ",
                IdKeyEditor::GlobalVolumeDown,
                keys,
                Msg::KeyEditor(KEMsg::GlobalVolumeDownBlurDown),
                Msg::KeyEditor(KEMsg::GlobalVolumeDownBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalVolumeDown {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalPlayerSeekForward {
    component: KESelectModifier,
}

impl KEGlobalPlayerSeekForward {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "G Seek Forward",
                IdKeyEditor::GlobalPlayerSeekForward,
                keys,
                Msg::KeyEditor(KEMsg::GlobalPlayerSeekForwardBlurDown),
                Msg::KeyEditor(KEMsg::GlobalPlayerSeekForwardBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalPlayerSeekForward {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalPlayerSeekBackward {
    component: KESelectModifier,
}

impl KEGlobalPlayerSeekBackward {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "G Seek Backward",
                IdKeyEditor::GlobalPlayerSeekBackward,
                keys,
                Msg::KeyEditor(KEMsg::GlobalPlayerSeekBackwardBlurDown),
                Msg::KeyEditor(KEMsg::GlobalPlayerSeekBackwardBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalPlayerSeekBackward {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalLyricAdjustForward {
    component: KESelectModifier,
}

impl KEGlobalLyricAdjustForward {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "G Lyric Forward",
                IdKeyEditor::GlobalLyricAdjustForward,
                keys,
                Msg::KeyEditor(KEMsg::GlobalLyricAdjustForwardBlurDown),
                Msg::KeyEditor(KEMsg::GlobalLyricAdjustForwardBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalLyricAdjustForward {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalLyricAdjustBackward {
    component: KESelectModifier,
}

impl KEGlobalLyricAdjustBackward {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "G Lyric Backward",
                IdKeyEditor::GlobalLyricAdjustBackward,
                keys,
                Msg::KeyEditor(KEMsg::GlobalLyricAdjustBackwardBlurDown),
                Msg::KeyEditor(KEMsg::GlobalLyricAdjustBackwardBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalLyricAdjustBackward {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalLyricCycle {
    component: KESelectModifier,
}

impl KEGlobalLyricCycle {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "G Lyric Cycle",
                IdKeyEditor::GlobalLyricCycle,
                keys,
                Msg::KeyEditor(KEMsg::GlobalLyricCyleBlurDown),
                Msg::KeyEditor(KEMsg::GlobalLyricCyleBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalLyricCycle {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalColorEditor {
    component: KESelectModifier,
}

impl KEGlobalColorEditor {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "G Color Editor",
                IdKeyEditor::GlobalColorEditor,
                keys,
                Msg::KeyEditor(KEMsg::GlobalColorEditorBlurDown),
                Msg::KeyEditor(KEMsg::GlobalColorEditorBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalColorEditor {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalKeyEditor {
    component: KESelectModifier,
}

impl KEGlobalKeyEditor {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "G Key Editor",
                IdKeyEditor::GlobalKeyEditor,
                keys,
                Msg::KeyEditor(KEMsg::GlobalKeyEditorBlurDown),
                Msg::KeyEditor(KEMsg::GlobalKeyEditorBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalKeyEditor {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibraryDelete {
    component: KESelectModifier,
}

impl KELibraryDelete {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Library Delete",
                IdKeyEditor::LibraryDelete,
                keys,
                Msg::KeyEditor(KEMsg::LibraryDeleteBlurDown),
                Msg::KeyEditor(KEMsg::LibraryDeleteBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibraryDelete {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibraryLoadDir {
    component: KESelectModifier,
}

impl KELibraryLoadDir {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Library Load Dir",
                IdKeyEditor::LibraryLoadDir,
                keys,
                Msg::KeyEditor(KEMsg::LibraryLoadDirBlurDown),
                Msg::KeyEditor(KEMsg::LibraryLoadDirBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibraryLoadDir {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibraryYank {
    component: KESelectModifier,
}

impl KELibraryYank {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Library Yank",
                IdKeyEditor::LibraryYank,
                keys,
                Msg::KeyEditor(KEMsg::LibraryYankBlurDown),
                Msg::KeyEditor(KEMsg::LibraryYankBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibraryYank {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibraryPaste {
    component: KESelectModifier,
}

impl KELibraryPaste {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Library Paste",
                IdKeyEditor::LibraryPaste,
                keys,
                Msg::KeyEditor(KEMsg::LibraryPasteBlurDown),
                Msg::KeyEditor(KEMsg::LibraryPasteBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibraryPaste {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibrarySearch {
    component: KESelectModifier,
}

impl KELibrarySearch {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Library Search",
                IdKeyEditor::LibrarySearch,
                keys,
                Msg::KeyEditor(KEMsg::LibrarySearchBlurDown),
                Msg::KeyEditor(KEMsg::LibrarySearchBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibrarySearch {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibrarySearchYoutube {
    component: KESelectModifier,
}

impl KELibrarySearchYoutube {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "L Search Youtube",
                IdKeyEditor::LibrarySearchYoutube,
                keys,
                Msg::KeyEditor(KEMsg::LibrarySearchYoutubeBlurDown),
                Msg::KeyEditor(KEMsg::LibrarySearchYoutubeBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibrarySearchYoutube {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibraryTagEditor {
    component: KESelectModifier,
}

impl KELibraryTagEditor {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "L Tag Editor",
                IdKeyEditor::LibraryTagEditor,
                keys,
                Msg::KeyEditor(KEMsg::LibraryTagEditorBlurDown),
                Msg::KeyEditor(KEMsg::LibraryTagEditorBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibraryTagEditor {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistDelete {
    component: KESelectModifier,
}

impl KEPlaylistDelete {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Playlist Delete",
                IdKeyEditor::PlaylistDelete,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistDeleteBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistDeleteBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistDelete {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistDeleteAll {
    component: KESelectModifier,
}

impl KEPlaylistDeleteAll {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Playlist Delete All",
                IdKeyEditor::PlaylistDeleteAll,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistDeleteAllBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistDeleteAllBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistDeleteAll {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistShuffle {
    component: KESelectModifier,
}

impl KEPlaylistShuffle {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Playlist Shuffle",
                IdKeyEditor::PlaylistShuffle,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistShuffleBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistShuffleBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistShuffle {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistModeCycle {
    component: KESelectModifier,
}

impl KEPlaylistModeCycle {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "P Mode Cycle",
                IdKeyEditor::PlaylistModeCycle,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistModeCycleBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistModeCycleBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistModeCycle {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistPlaySelected {
    component: KESelectModifier,
}

impl KEPlaylistPlaySelected {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "P Play Selected",
                IdKeyEditor::PlaylistPlaySelected,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistPlaySelectedBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistPlaySelectedBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistPlaySelected {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistAddFront {
    component: KESelectModifier,
}

impl KEPlaylistAddFront {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Playlist Add Front",
                IdKeyEditor::PlaylistAddFront,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistAddFrontBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistAddFrontBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistAddFront {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistSearch {
    component: KESelectModifier,
}

impl KEPlaylistSearch {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KESelectModifier::new(
                "Playlist Search",
                IdKeyEditor::PlaylistSearch,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistSearchBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistSearchBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistSearch {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
