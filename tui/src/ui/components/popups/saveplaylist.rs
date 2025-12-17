use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use anyhow::Result;
use termusiclib::{
    config::{SharedTuiSettings, TuiOverlay, v2::tui::theme::styles::ColorTermusic},
    utils::get_parent_folder,
};
use tui_realm_stdlib::Span;
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, State, StateValue, Sub, SubClause,
    SubEventClause,
    command::{Cmd, CmdResult, Direction, Position},
    event::{Key, KeyEvent, KeyModifiers},
    props::{Alignment, BorderType, Borders, InputType, PropPayload, PropValue, TextSpan},
};

use super::{YNConfirm, YNConfirmStyle};
use crate::ui::model::{Model, UserEvent};
use crate::ui::msg::{Msg, SavePlaylistMsg};
use crate::ui::{components::vendored::tui_realm_stdlib_input::Input, model::TxToMain};
use crate::ui::{ids::Id, msg::SPUpdateData};

#[derive(MockComponent)]
pub struct SavePlaylistPopup {
    component: Input,
    tx_to_main: TxToMain,

    directory: PathBuf,
}

impl SavePlaylistPopup {
    pub fn new(config: &TuiOverlay, tx_to_main: TxToMain, directory: PathBuf) -> Self {
        let settings = &config.settings;
        Self {
            component: Input::default()
                .foreground(settings.theme.fallback_foreground())
                .background(settings.theme.fallback_background())
                .borders(
                    Borders::default()
                        .color(settings.theme.fallback_border())
                        .modifiers(BorderType::Rounded),
                )
                // .invalid_style(Style::default().fg(Color::Red))
                .input_type(InputType::Text)
                .title(" Save Playlist as: (Enter to confirm) ", Alignment::Left),
            tx_to_main,
            directory,
        }
    }
}

impl Component<Msg, UserEvent> for SavePlaylistPopup {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => {
                self.perform(Cmd::Delete);
                self.perform(Cmd::Submit)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE,
            }) => {
                self.perform(Cmd::Type(ch));
                self.perform(Cmd::Submit)
            }
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::SavePlaylist(SavePlaylistMsg::CloseCancel));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => match self.component.state() {
                State::One(StateValue::String(mut input_string)) => {
                    input_string.push_str(".m3u");
                    let joined = self.directory.join(input_string);

                    return Some(Msg::SavePlaylist(SavePlaylistMsg::CloseOk(joined)));
                }
                _ => CmdResult::None,
            },
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::String(input_string))) => {
                // We cannot just pass the message as a return from this, as that will just provide the message to "Model::update"
                // and not to any component subscriptions, but it *is* when we send it to a port.
                let _ = self
                    .tx_to_main
                    .send(Msg::SavePlaylist(SavePlaylistMsg::Update(SPUpdateData {
                        path: OsString::from(input_string),
                    })));

                None
            }
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

#[derive(MockComponent)]
pub struct SavePlaylistConfirmPopup {
    component: YNConfirm,
    full_path: PathBuf,
}

impl SavePlaylistConfirmPopup {
    pub fn new(config: SharedTuiSettings, filename: PathBuf) -> Self {
        let component = YNConfirm::new_with_cb(config, " Playlist exists. Overwrite? ", |config| {
            YNConfirmStyle {
                foreground_color: config.settings.theme.important_popup_foreground(),
                background_color: config.settings.theme.important_popup_background(),
                border_color: config.settings.theme.important_popup_border(),
                title_alignment: Alignment::Center,
            }
        });

        Self {
            component,
            full_path: filename,
        }
    }
}

impl Component<Msg, UserEvent> for SavePlaylistConfirmPopup {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::SavePlaylist(SavePlaylistMsg::OverwriteOk(self.full_path.clone())),
            Msg::SavePlaylist(SavePlaylistMsg::OverwriteCancel),
        )
    }
}

#[derive(MockComponent)]
pub struct SavePlaylistFullpath {
    // Cannot use "tui_realm_stdlib::Paragraph" as it will draw each span *as a new line*, but we want *a single line*
    component: Span,

    directory: PathBuf,
    filename: OsString,

    config: SharedTuiSettings,
}

impl SavePlaylistFullpath {
    /// Consistently get the text-spans to display
    pub fn get_text_spans(
        config: &TuiOverlay,
        directory: &Path,
        filename: &OsStr,
    ) -> [TextSpan; 4] {
        let mut path_string = directory.to_string_lossy().to_string();
        // push extra "/" as "Path::to_string()" does not end with a "/"
        path_string.push('/');

        [
            TextSpan::new("Full name: ")
                .fg(config.settings.theme.fallback_highlight())
                .bold(),
            TextSpan::new(path_string).bold(),
            TextSpan::new(filename.to_string_lossy())
                .fg(config
                    .settings
                    .theme
                    .get_color_from_theme(ColorTermusic::Cyan))
                .bold(),
            TextSpan::new(".m3u").bold(),
        ]
    }

    /// Create a new label component.
    pub fn new(config: SharedTuiSettings, directory: PathBuf) -> Self {
        let component = {
            let config = config.read_recursive();

            Span::default()
                .foreground(config.settings.theme.fallback_foreground())
                .background(config.settings.theme.fallback_background())
                .spans(Self::get_text_spans(&config, &directory, OsStr::new("")))
        };

        Self {
            component,
            directory,
            filename: OsString::new(),
            config,
        }
    }

    /// Get the subscriptions for this component.
    fn subs() -> Vec<Sub<Id, UserEvent>> {
        vec![Sub::new(
            SubEventClause::User(UserEvent::Forward(Msg::SavePlaylist(
                SavePlaylistMsg::Update(SPUpdateData::default()),
            ))),
            SubClause::Always,
        )]
    }
}

impl Component<Msg, UserEvent> for SavePlaylistFullpath {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        if let Event::User(UserEvent::Forward(Msg::SavePlaylist(SavePlaylistMsg::Update(update)))) =
            ev
        {
            self.filename = update.path;

            let values = Self::get_text_spans(
                &self.config.read_recursive(),
                &self.directory,
                &self.filename,
            );

            self.attr(
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(
                    values.into_iter().map(PropValue::TextSpan).collect(),
                )),
            );

            return Some(Msg::ForceRedraw);
        }

        None
    }
}

impl Model {
    /// Mount/Remount the [`SavePlaylistPopup`] component.
    pub fn mount_save_playlist(&mut self, raw_path: &Path) -> Result<()> {
        let directory = get_parent_folder(raw_path).to_path_buf();

        self.app.remount(
            Id::SavePlaylistPopup,
            Box::new(SavePlaylistPopup::new(
                &self.config_tui.read(),
                self.tx_to_main.clone(),
                directory.clone(),
            )),
            Vec::new(),
        )?;

        self.app.remount(
            Id::SavePlaylistLabel,
            Box::new(SavePlaylistFullpath::new(
                self.config_tui.clone(),
                directory,
            )),
            SavePlaylistFullpath::subs(),
        )?;
        self.app.active(&Id::SavePlaylistPopup)?;
        Ok(())
    }

    /// Unount the [`SavePlaylistPopup`] component.
    pub fn umount_save_playlist(&mut self) -> Result<()> {
        if self.app.mounted(&Id::SavePlaylistPopup) {
            self.app.umount(&Id::SavePlaylistPopup)?;
            self.app.umount(&Id::SavePlaylistLabel)?;
        }

        Ok(())
    }

    /// Mount the overwrite confirmation dialog.
    pub fn mount_save_playlist_confirm(&mut self, path: PathBuf) -> Result<()> {
        self.app.remount(
            Id::SavePlaylistConfirm,
            Box::new(SavePlaylistConfirmPopup::new(self.config_tui.clone(), path)),
            Vec::new(),
        )?;
        self.app.active(&Id::SavePlaylistConfirm)?;

        Ok(())
    }

    /// Unmount the overwrite confirmation dialog.
    pub fn umount_save_playlist_confirm(&mut self) -> Result<()> {
        if self.app.mounted(&Id::SavePlaylistConfirm) {
            self.app.umount(&Id::SavePlaylistConfirm)?;
        }

        Ok(())
    }
}
