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
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalQuit,
                    IdKeyEditor::GlobalQuitInput,
                );
                self.ke_key_config.global_quit = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::GlobalLeft | IdKeyEditor::GlobalLeftInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalLeft,
                    IdKeyEditor::GlobalLeftInput,
                );
                self.ke_key_config.global_left = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::GlobalRight | IdKeyEditor::GlobalRightInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalRight,
                    IdKeyEditor::GlobalRightInput,
                );
                self.ke_key_config.global_right = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::GlobalUp | IdKeyEditor::GlobalUpInput => {
                let (code, modifiers) = self
                    .extract_key_mod_and_code(IdKeyEditor::GlobalUp, IdKeyEditor::GlobalUpInput);
                self.ke_key_config.global_up = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::GlobalDown | IdKeyEditor::GlobalDownInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalDown,
                    IdKeyEditor::GlobalDownInput,
                );
                self.ke_key_config.global_down = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::GlobalGotoTop | IdKeyEditor::GlobalGotoTopInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalGotoTop,
                    IdKeyEditor::GlobalGotoTopInput,
                );
                self.ke_key_config.global_goto_top = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::GlobalGotoBottom | IdKeyEditor::GlobalGotoBottomInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalGotoBottom,
                    IdKeyEditor::GlobalGotoBottomInput,
                );
                self.ke_key_config.global_goto_bottom = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::GlobalPlayerTogglePause | IdKeyEditor::GlobalPlayerTogglePauseInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerTogglePause,
                    IdKeyEditor::GlobalPlayerTogglePauseInput,
                );
                self.ke_key_config.global_player_toggle_pause = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::GlobalPlayerNext | IdKeyEditor::GlobalPlayerNextInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerNext,
                    IdKeyEditor::GlobalPlayerNextInput,
                );
                self.ke_key_config.global_player_next = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::GlobalPlayerPrevious | IdKeyEditor::GlobalPlayerPreviousInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerPrevious,
                    IdKeyEditor::GlobalPlayerPreviousInput,
                );
                self.ke_key_config.global_player_previous = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::GlobalHelp | IdKeyEditor::GlobalHelpInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalHelp,
                    IdKeyEditor::GlobalHelpInput,
                );
                self.ke_key_config.global_help = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::GlobalVolumeUp | IdKeyEditor::GlobalVolumeUpInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalVolumeUp,
                    IdKeyEditor::GlobalVolumeUpInput,
                );
                self.ke_key_config.global_player_volume_plus_2 = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::GlobalVolumeDown | IdKeyEditor::GlobalVolumeDownInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalVolumeDown,
                    IdKeyEditor::GlobalVolumeDownInput,
                );
                self.ke_key_config.global_player_volume_minus_2 =
                    BindingForEvent { code, modifiers }
            }

            IdKeyEditor::GlobalPlayerSeekForward | IdKeyEditor::GlobalPlayerSeekForwardInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerSeekForward,
                    IdKeyEditor::GlobalPlayerSeekForwardInput,
                );
                self.ke_key_config.global_player_seek_forward = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::GlobalPlayerSeekBackward | IdKeyEditor::GlobalPlayerSeekBackwardInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerSeekBackward,
                    IdKeyEditor::GlobalPlayerSeekBackwardInput,
                );
                self.ke_key_config.global_player_seek_backward = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::GlobalPlayerSpeedUp | IdKeyEditor::GlobalPlayerSpeedUpInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerSpeedUp,
                    IdKeyEditor::GlobalPlayerSpeedUpInput,
                );
                self.ke_key_config.global_player_speed_up = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::GlobalPlayerSpeedDown | IdKeyEditor::GlobalPlayerSpeedDownInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalPlayerSpeedDown,
                    IdKeyEditor::GlobalPlayerSpeedDownInput,
                );
                self.ke_key_config.global_player_speed_down = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::GlobalLyricAdjustForward | IdKeyEditor::GlobalLyricAdjustForwardInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalLyricAdjustForward,
                    IdKeyEditor::GlobalLyricAdjustForwardInput,
                );
                self.ke_key_config.global_lyric_adjust_forward = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::GlobalLyricAdjustBackward
            | IdKeyEditor::GlobalLyricAdjustBackwardInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalLyricAdjustBackward,
                    IdKeyEditor::GlobalLyricAdjustBackwardInput,
                );
                self.ke_key_config.global_lyric_adjust_backward =
                    BindingForEvent { code, modifiers }
            }

            IdKeyEditor::GlobalLyricCycle | IdKeyEditor::GlobalLyricCycleInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalLyricCycle,
                    IdKeyEditor::GlobalLyricCycleInput,
                );
                self.ke_key_config.global_lyric_cycle = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::GlobalColorEditor | IdKeyEditor::GlobalColorEditorInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalColorEditor,
                    IdKeyEditor::GlobalColorEditorInput,
                );
                self.ke_key_config.global_color_editor_open = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::GlobalKeyEditor | IdKeyEditor::GlobalKeyEditorInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalKeyEditor,
                    IdKeyEditor::GlobalKeyEditorInput,
                );
                self.ke_key_config.global_key_editor_open = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::LibraryDelete | IdKeyEditor::LibraryDeleteInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibraryDelete,
                    IdKeyEditor::LibraryDeleteInput,
                );
                self.ke_key_config.library_delete = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::LibraryLoadDir | IdKeyEditor::LibraryLoadDirInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibraryLoadDir,
                    IdKeyEditor::LibraryLoadDirInput,
                );
                self.ke_key_config.library_load_dir = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::LibraryYank | IdKeyEditor::LibraryYankInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibraryYank,
                    IdKeyEditor::LibraryYankInput,
                );
                self.ke_key_config.library_yank = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::LibraryPaste | IdKeyEditor::LibraryPasteInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibraryPaste,
                    IdKeyEditor::LibraryPasteInput,
                );
                self.ke_key_config.library_paste = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::LibrarySearch | IdKeyEditor::LibrarySearchInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibrarySearch,
                    IdKeyEditor::LibrarySearchInput,
                );
                self.ke_key_config.library_search = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::LibrarySearchYoutube | IdKeyEditor::LibrarySearchYoutubeInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibrarySearchYoutube,
                    IdKeyEditor::LibrarySearchYoutubeInput,
                );
                self.ke_key_config.library_search_youtube = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::LibraryTagEditor | IdKeyEditor::LibraryTagEditorInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::LibraryTagEditor,
                    IdKeyEditor::LibraryTagEditorInput,
                );
                self.ke_key_config.library_tag_editor_open = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::PlaylistDelete | IdKeyEditor::PlaylistDeleteInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistDelete,
                    IdKeyEditor::PlaylistDeleteInput,
                );
                self.ke_key_config.playlist_delete = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::PlaylistDeleteAll | IdKeyEditor::PlaylistDeleteAllInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistDeleteAll,
                    IdKeyEditor::PlaylistDeleteAllInput,
                );
                self.ke_key_config.playlist_delete_all = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::PlaylistShuffle | IdKeyEditor::PlaylistShuffleInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistShuffle,
                    IdKeyEditor::PlaylistShuffleInput,
                );
                self.ke_key_config.playlist_shuffle = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::PlaylistModeCycle | IdKeyEditor::PlaylistModeCycleInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistModeCycle,
                    IdKeyEditor::PlaylistModeCycleInput,
                );
                self.ke_key_config.playlist_mode_cycle = BindingForEvent { code, modifiers }
            }
            IdKeyEditor::PlaylistPlaySelected | IdKeyEditor::PlaylistPlaySelectedInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistPlaySelected,
                    IdKeyEditor::PlaylistPlaySelectedInput,
                );
                self.ke_key_config.playlist_play_selected = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::PlaylistAddFront | IdKeyEditor::PlaylistAddFrontInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistAddFront,
                    IdKeyEditor::PlaylistAddFrontInput,
                );
                self.ke_key_config.playlist_add_front = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::PlaylistSearch | IdKeyEditor::PlaylistSearchInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistSearch,
                    IdKeyEditor::PlaylistSearchInput,
                );
                self.ke_key_config.playlist_search = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::PlaylistSwapDown | IdKeyEditor::PlaylistSwapDownInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistSwapDown,
                    IdKeyEditor::PlaylistSwapDownInput,
                );
                self.ke_key_config.playlist_swap_down = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::PlaylistSwapUp | IdKeyEditor::PlaylistSwapUpInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::PlaylistSwapUp,
                    IdKeyEditor::PlaylistSwapUpInput,
                );
                self.ke_key_config.playlist_swap_up = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::GlobalLayoutTreeview | IdKeyEditor::GlobalLayoutTreeviewInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalLayoutTreeview,
                    IdKeyEditor::GlobalLayoutTreeviewInput,
                );
                self.ke_key_config.global_layout_treeview = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::GlobalLayoutDatabase | IdKeyEditor::GlobalLayoutDatabaseInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::GlobalLayoutDatabase,
                    IdKeyEditor::GlobalLayoutDatabaseInput,
                );
                self.ke_key_config.global_layout_database = BindingForEvent { code, modifiers }
            }

            IdKeyEditor::DatabaseAddAll | IdKeyEditor::DatabaseAddAllInput => {
                let (code, modifiers) = self.extract_key_mod_and_code(
                    IdKeyEditor::DatabaseAddAll,
                    IdKeyEditor::DatabaseAddAllInput,
                );
                self.ke_key_config.database_add_all = BindingForEvent { code, modifiers }
            }
            _ => {}
        }
    }

    fn extract_key_mod_and_code(
        &mut self,
        id_select: IdKeyEditor,
        id_input: IdKeyEditor,
    ) -> (Key, KeyModifiers) {
        let mut code = Key::Null;
        let mut modifier = KeyModifiers::CONTROL;
        self.update_key_input_by_modifier(id_select.clone(), id_input.clone());
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
        (code, modifier)
    }
    fn update_key_input_by_modifier(&mut self, id_select: IdKeyEditor, id_input: IdKeyEditor) {
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
