mod ke_input;
mod ke_select;

use crate::config::{BindingForEvent, Keys};
use crate::ui::{Id, IdKeyEditor, KEMsg, Model, Msg};
pub use ke_input::*;
pub use ke_select::*;
use tui_realm_stdlib::{Radio, Table};
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::props::{
    Alignment, AttrValue, Attribute, BorderType, Borders, Color, TableBuilder, TextSpan,
};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers, NoUserEvent},
    Component, Event, MockComponent, State, StateValue,
};

pub const CONTROL_SHIFT: KeyModifiers =
    KeyModifiers::from_bits_truncate(KeyModifiers::CONTROL.bits() | KeyModifiers::SHIFT.bits());
pub const ALT_SHIFT: KeyModifiers =
    KeyModifiers::from_bits_truncate(KeyModifiers::ALT.bits() | KeyModifiers::SHIFT.bits());
pub const CONTROL_ALT: KeyModifiers =
    KeyModifiers::from_bits_truncate(KeyModifiers::ALT.bits() | KeyModifiers::CONTROL.bits());
pub const CONTROL_ALT_SHIFT: KeyModifiers = KeyModifiers::from_bits_truncate(
    KeyModifiers::ALT.bits() | KeyModifiers::CONTROL.bits() | KeyModifiers::SHIFT.bits(),
);

#[derive(MockComponent)]
pub struct KERadioOk {
    component: Radio,
}
impl Default for KERadioOk {
    fn default() -> Self {
        Self {
            component: Radio::default()
                .foreground(Color::Yellow)
                // .background(Color::Black)
                .borders(
                    Borders::default()
                        .color(Color::Yellow)
                        .modifiers(BorderType::Rounded),
                )
                // .title("Additional operation:", Alignment::Left)
                .rewind(true)
                .choices(&["Save and Close"])
                .value(0),
        }
    }
}

impl Component<Msg, NoUserEvent> for KERadioOk {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::KeyEditor(KEMsg::RadioOkBlurDown))
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::KeyEditor(KEMsg::RadioOkBlurUp)),
            Event::Keyboard(KeyEvent {
                code: Key::Esc | Key::Char('q'),
                ..
            }) => return Some(Msg::KeyEditor(KEMsg::KeyEditorCloseCancel)),

            Event::Keyboard(KeyEvent {
                code: Key::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }) => return Some(Msg::KeyEditor(KEMsg::HelpPopupShow)),

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => return None,
        };
        if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(0)))
        ) {
            return Some(Msg::KeyEditor(KEMsg::KeyEditorCloseOk));
        }
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct KEHelpPopup {
    component: Table,
}

impl KEHelpPopup {
    pub fn new(keys: &Keys) -> Self {
        let key_quit = format!("<{}> or <{}>", keys.global_esc, keys.global_quit);
        let key_movement = format!("<{},{}>", keys.global_up, keys.global_right,);
        Self {
            component: Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Green),
                )
                // .foreground(Color::Yellow)
                // .background(Color::Black)
                .title("Help: Esc or Enter to exit.", Alignment::Center)
                .scroll(false)
                // .highlighted_color(Color::LightBlue)
                // .highlighted_str("\u{1f680}")
                // .highlighted_str("🚀")
                // .rewind(true)
                .step(4)
                .row_height(1)
                .headers(&["Key", "Function"])
                .column_spacing(3)
                .widths(&[30, 70])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::new("<TAB> <Shift-TAB>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Switch focus"))
                        .add_row()
                        .add_col(TextSpan::new(key_quit).bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Exit without saving"))
                        .add_row()
                        .add_col(
                            TextSpan::new("Modifier Select")
                                .bold()
                                .fg(Color::LightYellow),
                        )
                        .add_row()
                        .add_col(TextSpan::new(key_movement).bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Move cursor(vim style by default)"))
                        .add_row()
                        .add_col(TextSpan::new("<ENTER>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("select a Modifier"))
                        .add_row()
                        .add_col(TextSpan::new("Key input").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(TextSpan::new("").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("You can input 1 char, or key name."))
                        .add_row()
                        .add_col(TextSpan::new("<Key Name>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("backspace/enter/left/right/up/down"))
                        .add_row()
                        .add_col(TextSpan::new("<Key Name>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("home/end/pageup/pagedown/tab/backtab"))
                        .add_row()
                        .add_col(TextSpan::new("<Key Name>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("delete/insert/esc"))
                        .build(),
                ),
        }
    }
}

impl Component<Msg, NoUserEvent> for KEHelpPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter | Key::Esc,
                ..
            }) => Some(Msg::KeyEditor(KEMsg::HelpPopupClose)),
            _ => None,
        }
    }
}

impl Model {
    #[allow(clippy::too_many_lines)]
    pub fn update_key_editor_key_changed(&mut self, id: &IdKeyEditor) {
        match id {
            IdKeyEditor::GlobalQuit | IdKeyEditor::GlobalQuitInput => {
                self.ke_key_config.global_quit = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::GlobalQuit,
                    IdKeyEditor::GlobalQuitInput,
                );
            }
            IdKeyEditor::GlobalLeft | IdKeyEditor::GlobalLeftInput => {
                self.ke_key_config.global_left = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::GlobalLeft,
                    IdKeyEditor::GlobalLeftInput,
                );
            }
            IdKeyEditor::GlobalRight | IdKeyEditor::GlobalRightInput => {
                self.ke_key_config.global_right = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::GlobalRight,
                    IdKeyEditor::GlobalRightInput,
                );
            }
            IdKeyEditor::GlobalUp | IdKeyEditor::GlobalUpInput => {
                self.ke_key_config.global_up = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::GlobalUp,
                    IdKeyEditor::GlobalUpInput,
                );
            }

            IdKeyEditor::GlobalDown | IdKeyEditor::GlobalDownInput => {
                self.ke_key_config.global_down = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::GlobalDown,
                    IdKeyEditor::GlobalDownInput,
                );
            }
            IdKeyEditor::GlobalGotoTop | IdKeyEditor::GlobalGotoTopInput => {
                self.ke_key_config.global_goto_top = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::GlobalGotoTop,
                    IdKeyEditor::GlobalGotoTopInput,
                );
            }
            IdKeyEditor::GlobalGotoBottom | IdKeyEditor::GlobalGotoBottomInput => {
                self.ke_key_config.global_goto_bottom = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::GlobalGotoBottom,
                    IdKeyEditor::GlobalGotoBottomInput,
                );
            }
            IdKeyEditor::GlobalPlayerTogglePause | IdKeyEditor::GlobalPlayerTogglePauseInput => {
                self.ke_key_config.global_player_toggle_pause = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalPlayerTogglePause,
                        IdKeyEditor::GlobalPlayerTogglePauseInput,
                    );
            }
            IdKeyEditor::GlobalPlayerNext | IdKeyEditor::GlobalPlayerNextInput => {
                self.ke_key_config.global_player_next = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::GlobalPlayerNext,
                    IdKeyEditor::GlobalPlayerNextInput,
                );
            }
            IdKeyEditor::GlobalPlayerPrevious | IdKeyEditor::GlobalPlayerPreviousInput => {
                self.ke_key_config.global_player_previous = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalPlayerPrevious,
                        IdKeyEditor::GlobalPlayerPreviousInput,
                    );
            }

            IdKeyEditor::GlobalHelp | IdKeyEditor::GlobalHelpInput => {
                self.ke_key_config.global_help = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::GlobalHelp,
                    IdKeyEditor::GlobalHelpInput,
                );
            }
            IdKeyEditor::GlobalVolumeUp | IdKeyEditor::GlobalVolumeUpInput => {
                self.ke_key_config.global_player_volume_plus_2 = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalVolumeUp,
                        IdKeyEditor::GlobalVolumeUpInput,
                    );
            }
            IdKeyEditor::GlobalVolumeDown | IdKeyEditor::GlobalVolumeDownInput => {
                self.ke_key_config.global_player_volume_minus_2 = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalVolumeDown,
                        IdKeyEditor::GlobalVolumeDownInput,
                    );
            }

            IdKeyEditor::GlobalPlayerSeekForward | IdKeyEditor::GlobalPlayerSeekForwardInput => {
                self.ke_key_config.global_player_seek_forward = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalPlayerSeekForward,
                        IdKeyEditor::GlobalPlayerSeekForwardInput,
                    );
            }

            IdKeyEditor::GlobalPlayerSeekBackward | IdKeyEditor::GlobalPlayerSeekBackwardInput => {
                self.ke_key_config.global_player_seek_backward = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalPlayerSeekBackward,
                        IdKeyEditor::GlobalPlayerSeekBackwardInput,
                    );
            }

            IdKeyEditor::GlobalPlayerSpeedUp | IdKeyEditor::GlobalPlayerSpeedUpInput => {
                self.ke_key_config.global_player_speed_up = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalPlayerSpeedUp,
                        IdKeyEditor::GlobalPlayerSpeedUpInput,
                    );
            }

            IdKeyEditor::GlobalPlayerSpeedDown | IdKeyEditor::GlobalPlayerSpeedDownInput => {
                self.ke_key_config.global_player_speed_down = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalPlayerSpeedDown,
                        IdKeyEditor::GlobalPlayerSpeedDownInput,
                    );
            }

            IdKeyEditor::GlobalLyricAdjustForward | IdKeyEditor::GlobalLyricAdjustForwardInput => {
                self.ke_key_config.global_lyric_adjust_forward = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalLyricAdjustForward,
                        IdKeyEditor::GlobalLyricAdjustForwardInput,
                    );
            }

            IdKeyEditor::GlobalLyricAdjustBackward
            | IdKeyEditor::GlobalLyricAdjustBackwardInput => {
                self.ke_key_config.global_lyric_adjust_backward = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalLyricAdjustBackward,
                        IdKeyEditor::GlobalLyricAdjustBackwardInput,
                    );
            }

            IdKeyEditor::GlobalLyricCycle | IdKeyEditor::GlobalLyricCycleInput => {
                self.ke_key_config.global_lyric_cycle = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::GlobalLyricCycle,
                    IdKeyEditor::GlobalLyricCycleInput,
                );
            }
            IdKeyEditor::GlobalColorEditor | IdKeyEditor::GlobalColorEditorInput => {
                self.ke_key_config.global_color_editor_open = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalColorEditor,
                        IdKeyEditor::GlobalColorEditorInput,
                    );
            }
            IdKeyEditor::GlobalKeyEditor | IdKeyEditor::GlobalKeyEditorInput => {
                self.ke_key_config.global_key_editor_open = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalKeyEditor,
                        IdKeyEditor::GlobalKeyEditorInput,
                    );
            }

            IdKeyEditor::LibraryDelete | IdKeyEditor::LibraryDeleteInput => {
                self.ke_key_config.library_delete = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::LibraryDelete,
                    IdKeyEditor::LibraryDeleteInput,
                );
            }
            IdKeyEditor::LibraryLoadDir | IdKeyEditor::LibraryLoadDirInput => {
                self.ke_key_config.library_load_dir = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::LibraryLoadDir,
                    IdKeyEditor::LibraryLoadDirInput,
                );
            }
            IdKeyEditor::LibraryYank | IdKeyEditor::LibraryYankInput => {
                self.ke_key_config.library_yank = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::LibraryYank,
                    IdKeyEditor::LibraryYankInput,
                );
            }

            IdKeyEditor::LibraryPaste | IdKeyEditor::LibraryPasteInput => {
                self.ke_key_config.library_paste = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::LibraryPaste,
                    IdKeyEditor::LibraryPasteInput,
                );
            }

            IdKeyEditor::LibrarySearch | IdKeyEditor::LibrarySearchInput => {
                self.ke_key_config.library_search = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::LibrarySearch,
                    IdKeyEditor::LibrarySearchInput,
                );
            }
            IdKeyEditor::LibrarySearchYoutube | IdKeyEditor::LibrarySearchYoutubeInput => {
                self.ke_key_config.library_search_youtube = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::LibrarySearchYoutube,
                        IdKeyEditor::LibrarySearchYoutubeInput,
                    );
            }

            IdKeyEditor::LibraryTagEditor | IdKeyEditor::LibraryTagEditorInput => {
                self.ke_key_config.library_tag_editor_open = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::LibraryTagEditor,
                        IdKeyEditor::LibraryTagEditorInput,
                    );
            }
            IdKeyEditor::PlaylistDelete | IdKeyEditor::PlaylistDeleteInput => {
                self.ke_key_config.playlist_delete = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::PlaylistDelete,
                    IdKeyEditor::PlaylistDeleteInput,
                );
            }
            IdKeyEditor::PlaylistDeleteAll | IdKeyEditor::PlaylistDeleteAllInput => {
                self.ke_key_config.playlist_delete_all = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::PlaylistDeleteAll,
                    IdKeyEditor::PlaylistDeleteAllInput,
                );
            }
            IdKeyEditor::PlaylistShuffle | IdKeyEditor::PlaylistShuffleInput => {
                self.ke_key_config.playlist_shuffle = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::PlaylistShuffle,
                    IdKeyEditor::PlaylistShuffleInput,
                );
            }
            IdKeyEditor::PlaylistModeCycle | IdKeyEditor::PlaylistModeCycleInput => {
                self.ke_key_config.playlist_mode_cycle = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::PlaylistModeCycle,
                    IdKeyEditor::PlaylistModeCycleInput,
                );
            }
            IdKeyEditor::PlaylistPlaySelected | IdKeyEditor::PlaylistPlaySelectedInput => {
                self.ke_key_config.playlist_play_selected = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::PlaylistPlaySelected,
                        IdKeyEditor::PlaylistPlaySelectedInput,
                    );
            }

            IdKeyEditor::PlaylistAddFront | IdKeyEditor::PlaylistAddFrontInput => {
                self.ke_key_config.playlist_add_front = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::PlaylistAddFront,
                    IdKeyEditor::PlaylistAddFrontInput,
                );
            }

            IdKeyEditor::PlaylistSearch | IdKeyEditor::PlaylistSearchInput => {
                self.ke_key_config.playlist_search = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::PlaylistSearch,
                    IdKeyEditor::PlaylistSearchInput,
                );
            }

            IdKeyEditor::PlaylistSwapDown | IdKeyEditor::PlaylistSwapDownInput => {
                self.ke_key_config.playlist_swap_down = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::PlaylistSwapDown,
                    IdKeyEditor::PlaylistSwapDownInput,
                );
            }

            IdKeyEditor::PlaylistSwapUp | IdKeyEditor::PlaylistSwapUpInput => {
                self.ke_key_config.playlist_swap_up = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::PlaylistSwapUp,
                    IdKeyEditor::PlaylistSwapUpInput,
                );
            }

            IdKeyEditor::GlobalLayoutTreeview | IdKeyEditor::GlobalLayoutTreeviewInput => {
                self.ke_key_config.global_layout_treeview = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalLayoutTreeview,
                        IdKeyEditor::GlobalLayoutTreeviewInput,
                    );
            }

            IdKeyEditor::GlobalLayoutDatabase | IdKeyEditor::GlobalLayoutDatabaseInput => {
                self.ke_key_config.global_layout_database = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalLayoutDatabase,
                        IdKeyEditor::GlobalLayoutDatabaseInput,
                    );
            }

            IdKeyEditor::DatabaseAddAll | IdKeyEditor::DatabaseAddAllInput => {
                self.ke_key_config.database_add_all = self.extract_key_mod_and_code_key_editor(
                    IdKeyEditor::DatabaseAddAll,
                    IdKeyEditor::DatabaseAddAllInput,
                );
            }

            IdKeyEditor::GlobalPlayerToggleGapless
            | IdKeyEditor::GlobalPlayerToggleGaplessInput => {
                self.ke_key_config.global_player_toggle_gapless = self
                    .extract_key_mod_and_code_key_editor(
                        IdKeyEditor::GlobalPlayerToggleGapless,
                        IdKeyEditor::GlobalPlayerToggleGaplessInput,
                    );
            }
            _ => {}
        }
    }

    fn extract_key_mod_and_code_key_editor(
        &mut self,
        id_select: IdKeyEditor,
        id_input: IdKeyEditor,
    ) -> BindingForEvent {
        let mut code = Key::Null;
        let mut modifier = KeyModifiers::CONTROL;
        self.update_key_input_by_modifier_keyeditor(id_select.clone(), id_input.clone());
        if let Ok(State::One(StateValue::Usize(index))) = self.app.state(&Id::KeyEditor(id_select))
        {
            modifier = MODIFIER_LIST[index].modifier();
            if let Ok(State::One(StateValue::String(codes))) =
                self.app.state(&Id::KeyEditor(id_input))
            {
                if let Ok(c) = BindingForEvent::key_from_str(&codes) {
                    code = c;
                }
            }
        }

        BindingForEvent { code, modifier }
    }
    fn update_key_input_by_modifier_keyeditor(
        &mut self,
        id_select: IdKeyEditor,
        id_input: IdKeyEditor,
    ) {
        if let Ok(State::One(StateValue::Usize(index))) = self.app.state(&Id::KeyEditor(id_select))
        {
            let modifier = MODIFIER_LIST[index].modifier();
            if let Ok(State::One(StateValue::String(codes))) =
                self.app.state(&Id::KeyEditor(id_input.clone()))
            {
                if modifier.bits() % 2 == 1 {
                    self.app
                        .attr(
                            &Id::KeyEditor(id_input),
                            Attribute::Value,
                            AttrValue::String(codes.to_uppercase()),
                        )
                        .ok();
                } else {
                    self.app
                        .attr(
                            &Id::KeyEditor(id_input),
                            Attribute::Value,
                            AttrValue::String(codes.to_lowercase()),
                        )
                        .ok();
                }
            }
        }
    }
}
