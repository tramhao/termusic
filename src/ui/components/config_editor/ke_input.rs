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
use crate::config::{BindingForEvent, Settings};
use crate::ui::{ConfigEditorMsg, IdConfigEditor, Msg};

use std::str;
use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType, Style};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

#[derive(MockComponent)]
pub struct KEConfigInput {
    component: Input,
    id: IdConfigEditor,
    on_key_tab: Msg,
    on_key_backtab: Msg,
    config: Settings,
}

impl KEConfigInput {
    pub fn new(
        name: &str,
        id: IdConfigEditor,
        config: &Settings,
        on_key_tab: Msg,
        on_key_backtab: Msg,
    ) -> Self {
        let init_value = Self::init_key(&id, config);
        Self {
            component: Input::default()
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
                .input_type(InputType::Text)
                .placeholder("a/b/c", Style::default().fg(Color::Rgb(128, 128, 128)))
                .title(name, Alignment::Left)
                .value(init_value),
            id,
            config: config.clone(),
            on_key_tab,
            on_key_backtab,
        }
    }

    fn init_key(id: &IdConfigEditor, config: &Settings) -> String {
        match *id {
            IdConfigEditor::GlobalQuitInput => config.keys.global_quit.key(),
            // IdConfigEditor::GlobalLeftInput => keys.global_left.key(),
            // IdConfigEditor::GlobalRightInput => keys.global_right.key(),
            // IdConfigEditor::GlobalUpInput => keys.global_up.key(),
            // IdConfigEditor::GlobalDownInput => keys.global_down.key(),
            // IdConfigEditor::GlobalGotoTopInput => keys.global_goto_top.key(),
            // IdConfigEditor::GlobalGotoBottomInput => keys.global_goto_bottom.key(),
            // IdConfigEditor::GlobalPlayerTogglePauseInput => keys.global_player_toggle_pause.key(),
            // IdConfigEditor::GlobalPlayerNextInput => keys.global_player_next.key(),
            // IdConfigEditor::GlobalPlayerPreviousInput => keys.global_player_previous.key(),
            // IdConfigEditor::GlobalHelpInput => keys.global_help.key(),
            // IdConfigEditor::GlobalVolumeUpInput => keys.global_player_volume_plus_2.key(),
            // IdConfigEditor::GlobalVolumeDownInput => keys.global_player_volume_minus_2.key(),
            // IdConfigEditor::GlobalPlayerSeekForwardInput => keys.global_player_seek_forward.key(),
            // IdConfigEditor::GlobalPlayerSeekBackwardInput => keys.global_player_seek_backward.key(),
            // IdConfigEditor::GlobalPlayerSpeedUpInput => keys.global_player_speed_up.key(),
            // IdConfigEditor::GlobalPlayerSpeedDownInput => keys.global_player_speed_down.key(),
            // IdConfigEditor::GlobalLyricAdjustForwardInput => keys.global_lyric_adjust_forward.key(),
            // IdConfigEditor::GlobalLyricAdjustBackwardInput => keys.global_lyric_adjust_backward.key(),
            // IdConfigEditor::GlobalLyricCycleInput => keys.global_lyric_cycle.key(),
            // IdConfigEditor::GlobalColorEditorInput => keys.global_color_editor_open.key(),
            // IdConfigEditor::GlobalConfigEditorInput => keys.global_key_editor_open.key(),
            // IdConfigEditor::LibraryDeleteInput => keys.library_delete.key(),
            // IdConfigEditor::LibraryLoadDirInput => keys.library_load_dir.key(),
            // IdConfigEditor::LibraryYankInput => keys.library_yank.key(),
            // IdConfigEditor::LibraryPasteInput => keys.library_paste.key(),
            // IdConfigEditor::LibrarySearchInput => keys.library_search.key(),
            // IdConfigEditor::LibrarySearchYoutubeInput => keys.library_search_youtube.key(),
            // IdConfigEditor::LibraryTagEditorInput => keys.library_tag_editor_open.key(),
            // IdConfigEditor::PlaylistPlaySelectedInput => keys.playlist_play_selected.key(),
            // IdConfigEditor::PlaylistDeleteAllInput => keys.playlist_delete_all.key(),
            // IdConfigEditor::PlaylistDeleteInput => keys.playlist_delete.key(),
            // IdConfigEditor::PlaylistShuffleInput => keys.playlist_shuffle.key(),
            // IdConfigEditor::PlaylistModeCycleInput => keys.playlist_mode_cycle.key(),
            // IdConfigEditor::PlaylistAddFrontInput => keys.playlist_add_front.key(),
            // IdConfigEditor::PlaylistSearchInput => keys.playlist_search.key(),
            // IdConfigEditor::PlaylistSwapDownInput => keys.playlist_swap_down.key(),
            // IdConfigEditor::PlaylistSwapUpInput => keys.playlist_swap_up.key(),
            // IdConfigEditor::GlobalLayoutTreeviewInput => keys.global_layout_treeview.key(),
            // IdConfigEditor::GlobalLayoutDatabaseInput => keys.global_layout_database.key(),
            // IdConfigEditor::DatabaseAddAllInput => keys.database_add_all.key(),
            // IdConfigEditor::GlobalPlayerToggleGaplessInput => keys.global_player_toggle_gapless.key(),
            _ => "".to_string(),
        }
    }

    fn update_key(&mut self, result: CmdResult) -> Msg {
        if let CmdResult::Changed(State::One(StateValue::String(codes))) = result {
            if codes.is_empty() {
                self.update_symbol_after(Color::Blue);
                return Msg::None;
            }
            if BindingForEvent::key_from_str(&codes).is_ok() {
                // success getting a unicode letter
                self.update_symbol_after(Color::Green);
                return Msg::ConfigEditor(ConfigEditorMsg::KeyChanged(self.id.clone()));
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

impl Component<Msg, NoUserEvent> for KEConfigInput {
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

            // Event::Keyboard(keyevent) if keyevent == self.keys.global_help.key_event() => {
            //     Some(Msg::ConfigEditor(ConfigEditorMsg::HelpPopupShow))
            // }
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                ..
            }) => {
                let result = self.perform(Cmd::Type(ch));
                Some(self.update_key(result))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => Some(self.on_key_tab.clone()),
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => Some(self.on_key_backtab.clone()),

            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_esc.key_event() => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel))
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
pub struct ConfigGlobalQuitInput {
    component: KEConfigInput,
}

impl ConfigGlobalQuitInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalQuitInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalQuitInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalQuitInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalQuitInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLeftInput {
    component: KEConfigInput,
}

impl ConfigGlobalLeftInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalLeftInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLeftInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLeftInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLeftInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalRightInput {
    component: KEConfigInput,
}

impl ConfigGlobalRightInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalRightInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalRightInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalRightInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalRightInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalUpInput {
    component: KEConfigInput,
}

impl ConfigGlobalUpInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalUpInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalUpInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalUpInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalUpInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalDownInput {
    component: KEConfigInput,
}

impl ConfigGlobalDownInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalDownInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalDownInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalDownInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalDownInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalGotoTopInput {
    component: KEConfigInput,
}

impl ConfigGlobalGotoTopInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalGotoTopInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalGotoTopInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalGotoTopInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalGotoTopInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalGotoBottomInput {
    component: KEConfigInput,
}

impl ConfigGlobalGotoBottomInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalGotoBottomInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalGotoBottomInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalGotoBottomInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalGotoBottomInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerTogglePauseInput {
    component: KEConfigInput,
}

impl ConfigGlobalPlayerTogglePauseInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalPlayerTogglePauseInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerTogglePauseInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerTogglePauseInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerTogglePauseInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerNextInput {
    component: KEConfigInput,
}

impl ConfigGlobalPlayerNextInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalPlayerNextInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerNextInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerNextInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerNextInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerPreviousInput {
    component: KEConfigInput,
}

impl ConfigGlobalPlayerPreviousInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalPlayerPreviousInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerPreviousInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerPreviousInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerPreviousInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalHelpInput {
    component: KEConfigInput,
}

impl ConfigGlobalHelpInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalHelpInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalHelpInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalHelpInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalHelpInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalVolumeUpInput {
    component: KEConfigInput,
}

impl ConfigGlobalVolumeUpInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalVolumeUpInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalVolumeUpInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalVolumeUpInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalVolumeUpInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalVolumeDownInput {
    component: KEConfigInput,
}

impl ConfigGlobalVolumeDownInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalVolumeDownInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalVolumeDownInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalVolumeDownInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalVolumeDownInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerSeekForwardInput {
    component: KEConfigInput,
}

impl ConfigGlobalPlayerSeekForwardInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalPlayerSeekForwardInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSeekForwardInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSeekForwardInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerSeekForwardInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalPlayerSeekBackwardInput {
    component: KEConfigInput,
}

impl ConfigGlobalPlayerSeekBackwardInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalPlayerSeekBackwardInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSeekBackwardInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSeekBackwardInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerSeekBackwardInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerSpeedUpInput {
    component: KEConfigInput,
}

impl ConfigGlobalPlayerSpeedUpInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalPlayerSpeedUpInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSpeedUpInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSpeedUpInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerSpeedUpInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerSpeedDownInput {
    component: KEConfigInput,
}

impl ConfigGlobalPlayerSpeedDownInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalPlayerSpeedDownInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSpeedDownInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerSpeedDownInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerSpeedDownInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLyricAdjustForwardInput {
    component: KEConfigInput,
}

impl ConfigGlobalLyricAdjustForwardInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalLyricAdjustForwardInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLyricAdjustForwardInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLyricAdjustForwardInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLyricAdjustForwardInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalLyricAdjustBackwardInput {
    component: KEConfigInput,
}

impl ConfigGlobalLyricAdjustBackwardInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalLyricAdjustBackwardInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLyricAdjustBackwardInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLyricAdjustBackwardInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLyricAdjustBackwardInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
#[derive(MockComponent)]
pub struct ConfigGlobalLyricCycleInput {
    component: KEConfigInput,
}

impl ConfigGlobalLyricCycleInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalLyricCycleInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLyricCyleInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLyricCyleInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLyricCycleInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalPlayerToggleGaplessInput {
    component: KEConfigInput,
}

impl ConfigGlobalPlayerToggleGaplessInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalPlayerToggleGaplessInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerToggleGaplessInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalPlayerToggleGaplessInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalPlayerToggleGaplessInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLayoutTreeviewInput {
    component: KEConfigInput,
}

impl ConfigGlobalLayoutTreeviewInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalLayoutTreeviewInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLayoutTreeviewInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLayoutTreeviewInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLayoutTreeviewInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigGlobalLayoutDatabaseInput {
    component: KEConfigInput,
}

impl ConfigGlobalLayoutDatabaseInput {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: KEConfigInput::new(
                "",
                IdConfigEditor::GlobalLayoutDatabaseInput,
                config,
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLayoutDatabaseInputBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::GlobalLayoutDatabaseInputBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigGlobalLayoutDatabaseInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

// #[derive(MockComponent)]
// pub struct KELibraryDeleteInput {
//     component: KEConfigInput,
// }

// impl KELibraryDeleteInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::LibraryDeleteInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryDeleteInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryDeleteInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibraryDeleteInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KELibraryLoadDirInput {
//     component: KEConfigInput,
// }

// impl KELibraryLoadDirInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::LibraryLoadDirInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryLoadDirInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryLoadDirInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibraryLoadDirInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KELibraryYankInput {
//     component: KEConfigInput,
// }

// impl KELibraryYankInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::LibraryYankInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryYankInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryYankInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibraryYankInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KELibraryPasteInput {
//     component: KEConfigInput,
// }

// impl KELibraryPasteInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::LibraryPasteInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryPasteInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryPasteInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibraryPasteInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KELibrarySearchInput {
//     component: KEConfigInput,
// }

// impl KELibrarySearchInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::LibrarySearchInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibrarySearchInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibrarySearchInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibrarySearchInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KELibrarySearchYoutubeInput {
//     component: KEConfigInput,
// }

// impl KELibrarySearchYoutubeInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::LibrarySearchYoutubeInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibrarySearchYoutubeInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibrarySearchYoutubeInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibrarySearchYoutubeInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KELibraryTagEditorInput {
//     component: KEConfigInput,
// }

// impl KELibraryTagEditorInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::LibraryTagEditorInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryTagEditorInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::LibraryTagEditorInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KELibraryTagEditorInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistDeleteInput {
//     component: KEConfigInput,
// }

// impl KEPlaylistDeleteInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::PlaylistDeleteInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistDeleteInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistDeleteInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistDeleteInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistDeleteAllInput {
//     component: KEConfigInput,
// }

// impl KEPlaylistDeleteAllInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::PlaylistDeleteAllInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistDeleteAllInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistDeleteAllInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistDeleteAllInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistShuffleInput {
//     component: KEConfigInput,
// }

// impl KEPlaylistShuffleInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::PlaylistShuffleInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistShuffleInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistShuffleInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistShuffleInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistModeCycleInput {
//     component: KEConfigInput,
// }

// impl KEPlaylistModeCycleInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::PlaylistModeCycleInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistModeCycleInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistModeCycleInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistModeCycleInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistPlaySelectedInput {
//     component: KEConfigInput,
// }

// impl KEPlaylistPlaySelectedInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::PlaylistPlaySelectedInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistPlaySelectedInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistPlaySelectedInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistPlaySelectedInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistAddFrontInput {
//     component: KEConfigInput,
// }

// impl KEPlaylistAddFrontInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::PlaylistAddFrontInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistAddFrontInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistAddFrontInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistAddFrontInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistSearchInput {
//     component: KEConfigInput,
// }

// impl KEPlaylistSearchInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::PlaylistSearchInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistSearchInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistSearchInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistSearchInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistSwapDownInput {
//     component: KEConfigInput,
// }

// impl KEPlaylistSwapDownInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::PlaylistSwapDownInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistSwapDownInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistSwapDownInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistSwapDownInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEPlaylistSwapUpInput {
//     component: KEConfigInput,
// }

// impl KEPlaylistSwapUpInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::PlaylistSwapUpInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistSwapUpInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::PlaylistSwapUpInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEPlaylistSwapUpInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }

// #[derive(MockComponent)]
// pub struct KEDatabaseAddAllInput {
//     component: KEConfigInput,
// }

// impl KEDatabaseAddAllInput {
//     pub fn new(config: &Settings) -> Self {
//         Self {
//             component: KEConfigInput::new(
//                 "",
//                 IdConfigEditor::DatabaseAddAllInput,
//                 config,
//                 Msg::ConfigEditor(ConfigEditorMsg::DatabaseAddAllInputBlurDown),
//                 Msg::ConfigEditor(ConfigEditorMsg::DatabaseAddAllInputBlurUp),
//             ),
//         }
//     }
// }

// impl Component<Msg, NoUserEvent> for KEDatabaseAddAllInput {
//     fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
//         self.component.on(ev)
//     }
// }
