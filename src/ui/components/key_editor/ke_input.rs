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
use super::{KeyBind, Keys};
use crate::ui::{IdKeyEditor, KEMsg, Msg};

use std::str;
use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType, Style};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

#[derive(MockComponent)]
pub struct KEInput {
    component: Input,
    id: IdKeyEditor,
    on_key_shift: Msg,
    on_key_backshift: Msg,
    // keys: Keys,
}

impl KEInput {
    pub fn new(
        name: &str,
        id: IdKeyEditor,
        keys: &Keys,
        on_key_shift: Msg,
        on_key_backshift: Msg,
    ) -> Self {
        let init_value = Self::init_key(&id, keys);
        Self {
            component: Input::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Blue),
                )
                // .foreground(color)
                .input_type(InputType::Text)
                .placeholder("a/b/c", Style::default().fg(Color::Rgb(128, 128, 128)))
                .title(name, Alignment::Left)
                .value(init_value),
            id,
            // keys,
            on_key_shift,
            on_key_backshift,
        }
    }

    fn init_key(id: &IdKeyEditor, keys: &Keys) -> String {
        match *id {
            IdKeyEditor::GlobalQuitInput => keys.global_quit.key(),
            IdKeyEditor::GlobalLeftInput => keys.global_left.key(),
            IdKeyEditor::GlobalRightInput => keys.global_right.key(),
            IdKeyEditor::GlobalUpInput => keys.global_up.key(),
            IdKeyEditor::GlobalDownInput => keys.global_down.key(),
            IdKeyEditor::GlobalGotoTopInput => keys.global_goto_top.key(),
            IdKeyEditor::GlobalGotoBottomInput => keys.global_goto_bottom.key(),
            IdKeyEditor::GlobalPlayerTogglePauseInput => keys.global_player_toggle_pause.key(),
            IdKeyEditor::GlobalPlayerNextInput => keys.global_player_next.key(),
            IdKeyEditor::GlobalPlayerPreviousInput => keys.global_player_previous.key(),
            IdKeyEditor::GlobalHelpInput => keys.global_help.key(),
            IdKeyEditor::GlobalVolumeUpInput => keys.global_player_volume_plus_2.key(),
            IdKeyEditor::GlobalVolumeDownInput => keys.global_player_volume_minus_2.key(),
            IdKeyEditor::GlobalPlayerSeekForwardInput => keys.global_player_seek_forward.key(),
            IdKeyEditor::GlobalPlayerSeekBackwardInput => keys.global_player_seek_backward.key(),
            IdKeyEditor::GlobalPlayerSpeedUpInput => keys.global_player_speed_up.key(),
            IdKeyEditor::GlobalPlayerSpeedDownInput => keys.global_player_speed_down.key(),
            IdKeyEditor::GlobalLyricAdjustForwardInput => keys.global_lyric_adjust_forward.key(),
            IdKeyEditor::GlobalLyricAdjustBackwardInput => keys.global_lyric_adjust_backward.key(),
            IdKeyEditor::GlobalLyricCycleInput => keys.global_lyric_cycle.key(),
            IdKeyEditor::GlobalColorEditorInput => keys.global_color_editor_open.key(),
            IdKeyEditor::GlobalKeyEditorInput => keys.global_key_editor_open.key(),
            IdKeyEditor::LibraryDeleteInput => keys.library_delete.key(),
            IdKeyEditor::LibraryLoadDirInput => keys.library_load_dir.key(),
            IdKeyEditor::LibraryYankInput => keys.library_yank.key(),
            IdKeyEditor::LibraryPasteInput => keys.library_paste.key(),
            IdKeyEditor::LibrarySearchInput => keys.library_search.key(),
            IdKeyEditor::LibrarySearchYoutubeInput => keys.library_search_youtube.key(),
            IdKeyEditor::LibraryTagEditorInput => keys.library_tag_editor_open.key(),
            IdKeyEditor::PlaylistPlaySelectedInput => keys.playlist_play_selected.key(),
            IdKeyEditor::PlaylistDeleteAllInput => keys.playlist_delete_all.key(),
            IdKeyEditor::PlaylistDeleteInput => keys.playlist_delete.key(),
            IdKeyEditor::PlaylistShuffleInput => keys.playlist_shuffle.key(),
            IdKeyEditor::PlaylistModeCycleInput => keys.playlist_mode_cycle.key(),
            IdKeyEditor::PlaylistAddFrontInput => keys.playlist_add_front.key(),
            IdKeyEditor::PlaylistSearchInput => keys.playlist_search.key(),
            _ => "".to_string(),
        }
    }

    fn update_key(&mut self, result: CmdResult) -> Msg {
        if let CmdResult::Changed(State::One(StateValue::String(codes))) = result {
            if codes.is_empty() {
                self.update_symbol_after(Color::Blue);
                return Msg::None;
            }
            if KeyBind::key_from_str(&codes).is_ok() {
                // success getting a unicode letter
                self.update_symbol_after(Color::Green);
                return Msg::KeyEditor(KEMsg::KeyChanged(self.id.clone()));
            }
            // fail to get a good code
            self.update_symbol_after(Color::Red);
        }

        Msg::None
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
}

impl Component<Msg, NoUserEvent> for KEInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => {
                self.perform(Cmd::Move(Direction::Left));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => {
                self.perform(Cmd::Move(Direction::Right));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => {
                self.perform(Cmd::GoTo(Position::Begin));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => {
                let result = self.perform(Cmd::Cancel);
                Some(self.update_key(result))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => {
                let result = self.perform(Cmd::Delete);
                Some(self.update_key(result))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => Some(Msg::KeyEditor(KEMsg::HelpPopupShow)),

            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                ..
            }) => {
                let result = self.perform(Cmd::Type(ch));
                Some(self.update_key(result))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => Some(self.on_key_shift.clone()),
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => Some(self.on_key_backshift.clone()),

            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                Some(Msg::KeyEditor(KEMsg::KeyEditorCloseCancel))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                let result = self.perform(Cmd::Submit);
                Some(self.update_key(result))
            }
            _ => Some(Msg::None),
        }
    }
}

#[derive(MockComponent)]
pub struct KEGlobalQuitInput {
    component: KEInput,
}

impl KEGlobalQuitInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalQuitInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalQuitInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalQuitInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalQuitInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalLeftInput {
    component: KEInput,
}

impl KEGlobalLeftInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalLeftInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalLeftInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalLeftInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalLeftInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalRightInput {
    component: KEInput,
}

impl KEGlobalRightInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalRightInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalRightInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalRightInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalRightInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct KEGlobalUpInput {
    component: KEInput,
}

impl KEGlobalUpInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalUpInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalUpInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalUpInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalUpInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct KEGlobalDownInput {
    component: KEInput,
}

impl KEGlobalDownInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalDownInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalDownInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalDownInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalDownInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalGotoTopInput {
    component: KEInput,
}

impl KEGlobalGotoTopInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalGotoTopInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalGotoTopInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalGotoTopInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalGotoTopInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalGotoBottomInput {
    component: KEInput,
}

impl KEGlobalGotoBottomInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalGotoBottomInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalGotoBottomInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalGotoBottomInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalGotoBottomInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalPlayerTogglePauseInput {
    component: KEInput,
}

impl KEGlobalPlayerTogglePauseInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalPlayerTogglePauseInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalPlayerTogglePauseInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalPlayerTogglePauseInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalPlayerTogglePauseInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalPlayerNextInput {
    component: KEInput,
}

impl KEGlobalPlayerNextInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalPlayerNextInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalPlayerNextInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalPlayerNextInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalPlayerNextInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalPlayerPreviousInput {
    component: KEInput,
}

impl KEGlobalPlayerPreviousInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalPlayerPreviousInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalPlayerPreviousInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalPlayerPreviousInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalPlayerPreviousInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalHelpInput {
    component: KEInput,
}

impl KEGlobalHelpInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalHelpInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalHelpInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalHelpInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalHelpInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalVolumeUpInput {
    component: KEInput,
}

impl KEGlobalVolumeUpInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalVolumeUpInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalVolumeUpInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalVolumeUpInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalVolumeUpInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalVolumeDownInput {
    component: KEInput,
}

impl KEGlobalVolumeDownInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalVolumeDownInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalVolumeDownInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalVolumeDownInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalVolumeDownInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalPlayerSeekForwardInput {
    component: KEInput,
}

impl KEGlobalPlayerSeekForwardInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalPlayerSeekForwardInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalPlayerSeekForwardInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalPlayerSeekForwardInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalPlayerSeekForwardInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct KEGlobalPlayerSeekBackwardInput {
    component: KEInput,
}

impl KEGlobalPlayerSeekBackwardInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalPlayerSeekBackwardInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalPlayerSeekBackwardInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalPlayerSeekBackwardInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalPlayerSeekBackwardInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalPlayerSpeedUpInput {
    component: KEInput,
}

impl KEGlobalPlayerSpeedUpInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalPlayerSpeedUpInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalPlayerSpeedUpInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalPlayerSpeedUpInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalPlayerSpeedUpInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalPlayerSpeedDownInput {
    component: KEInput,
}

impl KEGlobalPlayerSpeedDownInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalPlayerSpeedDownInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalPlayerSpeedDownInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalPlayerSpeedDownInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalPlayerSpeedDownInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalLyricAdjustForwardInput {
    component: KEInput,
}

impl KEGlobalLyricAdjustForwardInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalLyricAdjustForwardInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalLyricAdjustForwardInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalLyricAdjustForwardInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalLyricAdjustForwardInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct KEGlobalLyricAdjustBackwardInput {
    component: KEInput,
}

impl KEGlobalLyricAdjustBackwardInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalLyricAdjustBackwardInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalLyricAdjustBackwardInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalLyricAdjustBackwardInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalLyricAdjustBackwardInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct KEGlobalLyricCycleInput {
    component: KEInput,
}

impl KEGlobalLyricCycleInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalLyricCycleInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalLyricCyleInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalLyricCyleInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalLyricCycleInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalColorEditorInput {
    component: KEInput,
}

impl KEGlobalColorEditorInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalColorEditorInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalColorEditorInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalColorEditorInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalColorEditorInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEGlobalKeyEditorInput {
    component: KEInput,
}

impl KEGlobalKeyEditorInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::GlobalKeyEditorInput,
                keys,
                Msg::KeyEditor(KEMsg::GlobalKeyEditorInputBlurDown),
                Msg::KeyEditor(KEMsg::GlobalKeyEditorInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEGlobalKeyEditorInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibraryDeleteInput {
    component: KEInput,
}

impl KELibraryDeleteInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::LibraryDeleteInput,
                keys,
                Msg::KeyEditor(KEMsg::LibraryDeleteInputBlurDown),
                Msg::KeyEditor(KEMsg::LibraryDeleteInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibraryDeleteInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibraryLoadDirInput {
    component: KEInput,
}

impl KELibraryLoadDirInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::LibraryLoadDirInput,
                keys,
                Msg::KeyEditor(KEMsg::LibraryLoadDirInputBlurDown),
                Msg::KeyEditor(KEMsg::LibraryLoadDirInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibraryLoadDirInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibraryYankInput {
    component: KEInput,
}

impl KELibraryYankInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::LibraryYankInput,
                keys,
                Msg::KeyEditor(KEMsg::LibraryYankInputBlurDown),
                Msg::KeyEditor(KEMsg::LibraryYankInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibraryYankInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibraryPasteInput {
    component: KEInput,
}

impl KELibraryPasteInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::LibraryPasteInput,
                keys,
                Msg::KeyEditor(KEMsg::LibraryPasteInputBlurDown),
                Msg::KeyEditor(KEMsg::LibraryPasteInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibraryPasteInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibrarySearchInput {
    component: KEInput,
}

impl KELibrarySearchInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::LibrarySearchInput,
                keys,
                Msg::KeyEditor(KEMsg::LibrarySearchInputBlurDown),
                Msg::KeyEditor(KEMsg::LibrarySearchInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibrarySearchInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibrarySearchYoutubeInput {
    component: KEInput,
}

impl KELibrarySearchYoutubeInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::LibrarySearchYoutubeInput,
                keys,
                Msg::KeyEditor(KEMsg::LibrarySearchYoutubeInputBlurDown),
                Msg::KeyEditor(KEMsg::LibrarySearchYoutubeInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibrarySearchYoutubeInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KELibraryTagEditorInput {
    component: KEInput,
}

impl KELibraryTagEditorInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::LibraryTagEditorInput,
                keys,
                Msg::KeyEditor(KEMsg::LibraryTagEditorInputBlurDown),
                Msg::KeyEditor(KEMsg::LibraryTagEditorInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KELibraryTagEditorInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistDeleteInput {
    component: KEInput,
}

impl KEPlaylistDeleteInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::PlaylistDeleteInput,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistDeleteInputBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistDeleteInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistDeleteInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistDeleteAllInput {
    component: KEInput,
}

impl KEPlaylistDeleteAllInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::PlaylistDeleteAllInput,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistDeleteAllInputBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistDeleteAllInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistDeleteAllInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistShuffleInput {
    component: KEInput,
}

impl KEPlaylistShuffleInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::PlaylistShuffleInput,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistShuffleInputBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistShuffleInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistShuffleInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistModeCycleInput {
    component: KEInput,
}

impl KEPlaylistModeCycleInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::PlaylistModeCycleInput,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistModeCycleInputBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistModeCycleInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistModeCycleInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistPlaySelectedInput {
    component: KEInput,
}

impl KEPlaylistPlaySelectedInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::PlaylistPlaySelectedInput,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistPlaySelectedInputBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistPlaySelectedInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistPlaySelectedInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistAddFrontInput {
    component: KEInput,
}

impl KEPlaylistAddFrontInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::PlaylistAddFrontInput,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistAddFrontInputBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistAddFrontInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistAddFrontInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct KEPlaylistSearchInput {
    component: KEInput,
}

impl KEPlaylistSearchInput {
    pub fn new(keys: &Keys) -> Self {
        Self {
            component: KEInput::new(
                "",
                IdKeyEditor::PlaylistSearchInput,
                keys,
                Msg::KeyEditor(KEMsg::PlaylistSearchInputBlurDown),
                Msg::KeyEditor(KEMsg::PlaylistSearchInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEPlaylistSearchInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
