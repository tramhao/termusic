use crate::ui::model::Model;
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
use crate::ui::{ConfigEditorMsg, Msg};
use crate::CombinedSettings;

use anyhow::Result;
use termusiclib::config::v2::tui::{keys::Keys, Alignment as XywhAlign};
use termusiclib::config::SharedTuiSettings;
use termusiclib::types::{Id, IdConfigEditor};
use tui_realm_stdlib::{Input, Radio};
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType, Style};
use tuirealm::{
    command::{Cmd, Direction, Position},
    event::{Key, KeyEvent, NoUserEvent},
    Component, Event, MockComponent,
};

#[derive(MockComponent)]
pub struct MusicDir {
    component: Input,
    config: SharedTuiSettings,
}

impl MusicDir {
    pub fn new(config: CombinedSettings) -> Self {
        let config_tui = config.tui.read();
        let mut music_dir_input = String::new();
        for music_dir in &config.server.read().settings.player.music_dirs {
            music_dir_input.push_str(&music_dir.to_string_lossy());
            music_dir_input.push(';');
        }
        // remove the last ";"
        if !music_dir_input.is_empty() {
            music_dir_input.remove(music_dir_input.len() - 1);
        }
        let component = Input::default()
            .borders(
                Borders::default()
                    .color(config_tui.settings.theme.library_border())
                    .modifiers(BorderType::Rounded),
            )
            .foreground(config_tui.settings.theme.library_highlight())
            .input_type(InputType::Text)
            .placeholder("~/Music", Style::default().fg(Color::Rgb(128, 128, 128)))
            .title(
                " Root Music Directory:(use ; to separate) ",
                Alignment::Left,
            )
            .value(music_dir_input);

        drop(config_tui);
        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, NoUserEvent> for MusicDir {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::MusicDirBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::MusicDirBlurUp),
        )
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_input_ev(
    component: &mut dyn MockComponent,
    ev: Event<NoUserEvent>,
    keys: &Keys,
    on_key_down: Msg,
    on_key_up: Msg,
) -> Option<Msg> {
    match ev {
        // Global Hotkeys
        Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk))
        }
        Event::Keyboard(KeyEvent {
            code: Key::Down, ..
        }) => Some(on_key_down),
        Event::Keyboard(KeyEvent { code: Key::Up, .. }) => Some(on_key_up),
        Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout))
        }
        Event::Keyboard(keyevent) if keyevent == keys.escape.get() => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel))
        }

        // Local Hotkeys
        Event::Keyboard(KeyEvent {
            code: Key::Left, ..
        }) => {
            component.perform(Cmd::Move(Direction::Left));
            Some(Msg::ForceRedraw)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Right, ..
        }) => {
            component.perform(Cmd::Move(Direction::Right));
            Some(Msg::ForceRedraw)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Home, ..
        }) => {
            component.perform(Cmd::GoTo(Position::Begin));
            Some(Msg::ForceRedraw)
        }
        Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
            component.perform(Cmd::GoTo(Position::End));
            Some(Msg::ForceRedraw)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Delete, ..
        }) => {
            component.perform(Cmd::Cancel);
            Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigChanged))
        }
        Event::Keyboard(KeyEvent {
            code: Key::Backspace,
            ..
        }) => {
            component.perform(Cmd::Delete);
            Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigChanged))
        }

        Event::Keyboard(KeyEvent {
            code: Key::Char(ch),
            ..
        }) => {
            component.perform(Cmd::Type(ch));
            Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigChanged))
        }

        _ => None,
    }
}

#[derive(MockComponent)]
pub struct ExitConfirmation {
    component: Radio,
    config: SharedTuiSettings,
}

impl ExitConfirmation {
    pub fn new(config: SharedTuiSettings) -> Self {
        let config_r = config.read();
        let enabled = config_r.settings.behavior.confirm_quit;
        let component = Radio::default()
            .borders(
                Borders::default()
                    .color(config_r.settings.theme.library_border())
                    .modifiers(BorderType::Rounded),
            )
            .choices(&["Yes", "No"])
            .foreground(config_r.settings.theme.library_highlight())
            .rewind(true)
            .title(" Show exit confirmation? ", Alignment::Left)
            .value(usize::from(!enabled));

        drop(config_r);
        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for ExitConfirmation {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::ExitConfirmationBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::ExitConfirmationBlurUp),
        )
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_radio_ev(
    component: &mut dyn MockComponent,
    ev: Event<NoUserEvent>,
    keys: &Keys,
    on_key_down: Msg,
    on_key_up: Msg,
) -> Option<Msg> {
    match ev {
        // Global Hotkeys
        Event::Keyboard(keyevent) if keyevent == keys.config_keys.save.get() => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk))
        }
        Event::Keyboard(KeyEvent {
            code: Key::Down, ..
        }) => Some(on_key_down),
        Event::Keyboard(KeyEvent { code: Key::Up, .. }) => Some(on_key_up),
        Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout))
        }
        Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.down.get() => {
            Some(on_key_down)
        }
        Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.up.get() => Some(on_key_up),
        Event::Keyboard(keyevent) if keyevent == keys.quit.get() => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel))
        }
        Event::Keyboard(keyevent) if keyevent == keys.escape.get() => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel))
        }

        // Local Hotkeys
        Event::Keyboard(KeyEvent {
            code: Key::Left, ..
        }) => {
            component.perform(Cmd::Move(Direction::Left));
            Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigChanged))
        }
        Event::Keyboard(KeyEvent {
            code: Key::Right, ..
        }) => {
            component.perform(Cmd::Move(Direction::Right));
            Some(Msg::ConfigEditor(ConfigEditorMsg::ConfigChanged))
        }

        _ => None,
    }
}

#[derive(MockComponent)]
pub struct PlaylistDisplaySymbol {
    component: Radio,
    config: SharedTuiSettings,
}

impl PlaylistDisplaySymbol {
    pub fn new(config: SharedTuiSettings) -> Self {
        let config_r = config.read();
        let enabled = config_r.settings.theme.style.playlist.use_loop_mode_symbol;
        let component = Radio::default()
            .borders(
                Borders::default()
                    .color(config_r.settings.theme.library_border())
                    .modifiers(BorderType::Rounded),
            )
            .choices(&["Yes", "No"])
            .foreground(config_r.settings.theme.library_highlight())
            .rewind(true)
            .title(" Display symbol in playlist title? ", Alignment::Left)
            .value(usize::from(!enabled));

        drop(config_r);
        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for PlaylistDisplaySymbol {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistDisplaySymbolBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistDisplaySymbolBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct PlaylistRandomTrack {
    component: Input,
    config: SharedTuiSettings,
}

impl PlaylistRandomTrack {
    pub fn new(config: CombinedSettings) -> Self {
        let component = {
            let config_tui = config.tui.read();
            Input::default()
                .borders(
                    Borders::default()
                        .color(config_tui.settings.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                .foreground(config_tui.settings.theme.library_highlight())
                .input_type(InputType::UnsignedInteger)
                .invalid_style(Style::default().fg(Color::Red))
                .placeholder("20", Style::default().fg(Color::Rgb(128, 128, 128)))
                .title(" Playlist Select Random Track Quantity: ", Alignment::Left)
                .value(
                    config
                        .server
                        .read()
                        .settings
                        .player
                        .random_track_quantity
                        .to_string(),
                )
        };

        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, NoUserEvent> for PlaylistRandomTrack {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistRandomTrackBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistRandomTrackBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct PlaylistRandomAlbum {
    component: Input,
    config: SharedTuiSettings,
}

impl PlaylistRandomAlbum {
    pub fn new(config: CombinedSettings) -> Self {
        let component = {
            let config_tui = config.tui.read();
            Input::default()
                .borders(
                    Borders::default()
                        .color(config_tui.settings.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                .foreground(config_tui.settings.theme.library_highlight())
                .input_type(InputType::UnsignedInteger)
                .invalid_style(Style::default().fg(Color::Red))
                .placeholder("1", Style::default().fg(Color::Rgb(128, 128, 128)))
                .title(
                    " Playlist Select Random Album with tracks no less than: ",
                    Alignment::Left,
                )
                .value(
                    config
                        .server
                        .read()
                        .settings
                        .player
                        .random_album_min_quantity
                        .to_string(),
                )
        };

        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, NoUserEvent> for PlaylistRandomAlbum {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistRandomAlbumBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistRandomAlbumBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct PodcastDir {
    component: Input,
    config: SharedTuiSettings,
}

impl PodcastDir {
    pub fn new(config: CombinedSettings) -> Self {
        let component = {
            let config_tui = config.tui.read();
            Input::default()
                .borders(
                    Borders::default()
                        .color(config_tui.settings.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                .foreground(config_tui.settings.theme.library_highlight())
                .input_type(InputType::Text)
                .invalid_style(Style::default().fg(Color::Red))
                .placeholder(
                    "~/Music/podcast",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
                .title(" Podcast Download Directory: ", Alignment::Left)
                .value(
                    config
                        .server
                        .read()
                        .settings
                        .podcast
                        .download_dir
                        .to_string_lossy(),
                )
        };

        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, NoUserEvent> for PodcastDir {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::PodcastDirBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::PodcastDirBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct PodcastSimulDownload {
    component: Input,
    config: SharedTuiSettings,
}

impl PodcastSimulDownload {
    pub fn new(config: CombinedSettings) -> Self {
        let component = {
            let config_tui = config.tui.read();
            Input::default()
                .borders(
                    Borders::default()
                        .color(config_tui.settings.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                .foreground(config_tui.settings.theme.library_highlight())
                .input_type(InputType::UnsignedInteger)
                .invalid_style(Style::default().fg(Color::Red))
                .placeholder(
                    "between 1 ~ 5 suggested",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
                .title(" Podcast Simultanious Download: ", Alignment::Left)
                .value(
                    config
                        .server
                        .read()
                        .settings
                        .podcast
                        .concurrent_downloads_max
                        .to_string(),
                )
        };

        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, NoUserEvent> for PodcastSimulDownload {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::PodcastSimulDownloadBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::PodcastSimulDownloadBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct PodcastMaxRetries {
    component: Input,
    config: SharedTuiSettings,
}

impl PodcastMaxRetries {
    pub fn new(config: CombinedSettings) -> Self {
        let component = {
            let config_tui = config.tui.read();
            Input::default()
                .borders(
                    Borders::default()
                        .color(config_tui.settings.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                .foreground(config_tui.settings.theme.library_highlight())
                .input_type(InputType::UnsignedInteger)
                .invalid_style(Style::default().fg(Color::Red))
                .placeholder(
                    "between 1 ~ 5 suggested",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
                .title(" Podcast Download Max Retries: ", Alignment::Left)
                .value(
                    config
                        .server
                        .read()
                        .settings
                        .podcast
                        .max_download_retries
                        .to_string(),
                )
        };

        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, NoUserEvent> for PodcastMaxRetries {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::PodcastMaxRetriesBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::PodcastMaxRetriesBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct AlbumPhotoAlign {
    component: Radio,
    config: SharedTuiSettings,
}

impl AlbumPhotoAlign {
    pub fn new(config: SharedTuiSettings) -> Self {
        let config_r = config.read();
        let align = match config_r.settings.coverart.align {
            XywhAlign::BottomRight => 0,
            XywhAlign::BottomLeft => 1,
            XywhAlign::TopRight => 2,
            XywhAlign::TopLeft => 3,
        };
        let component = Radio::default()
            .borders(
                Borders::default()
                    .color(config_r.settings.theme.library_border())
                    .modifiers(BorderType::Rounded),
            )
            .choices(&["BottomRight", "BottomLeft", "TopRight", "TopLeft"])
            .foreground(config_r.settings.theme.library_highlight())
            .rewind(true)
            .title(" Album Photo Align: ", Alignment::Left)
            .value(align);

        drop(config_r);
        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for AlbumPhotoAlign {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::AlbumPhotoAlignBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::AlbumPhotoAlignBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct SaveLastPosition {
    component: Radio,
    config: SharedTuiSettings,
}

impl SaveLastPosition {
    pub fn new(config: CombinedSettings) -> Self {
        let config_tui = config.tui.read();
        // 0 == unsupported, dont save value
        let save_last_position = match config.server.read().settings.player.remember_position {
            termusiclib::config::v2::server::RememberLastPosition::All(
                termusiclib::config::v2::server::PositionYesNo::Simple(ref v),
            ) => match v {
                termusiclib::config::v2::server::PositionYesNoLower::Yes => 2,
                termusiclib::config::v2::server::PositionYesNoLower::No => 1,
            },
            _ => 0,
            // LastPosition::Auto => 0,
            // LastPosition::No => 1,
            // LastPosition::Yes => 2,
        };
        let component = Radio::default()
            .borders(
                Borders::default()
                    .color(config_tui.settings.theme.library_border())
                    .modifiers(BorderType::Rounded),
            )
            .choices(&["Unsupported", "No", "Yes"])
            .foreground(config_tui.settings.theme.library_highlight())
            .rewind(true)
            .title(" Remember last played position: ", Alignment::Left)
            .value(save_last_position);

        drop(config_tui);
        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, NoUserEvent> for SaveLastPosition {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::SaveLastPositionBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::SaveLastPosotionBlurUp),
        )
    }
}
#[derive(MockComponent)]
pub struct ConfigSeekStep {
    component: Radio,
    config: SharedTuiSettings,
}

impl ConfigSeekStep {
    pub fn new(config: CombinedSettings) -> Self {
        let config_tui = config.tui.read();
        // let seek_step = match config.server.read().settings.player.seek_step {
        //     SeekStep::Auto => 0,
        //     SeekStep::Short => 1,
        //     SeekStep::Long => 2,
        // };
        let seek_step = 0;
        let component = Radio::default()
            .borders(
                Borders::default()
                    .color(config_tui.settings.theme.library_border())
                    .modifiers(BorderType::Rounded),
            )
            .choices(&["Unsupported"])
            .foreground(config_tui.settings.theme.library_highlight())
            .rewind(true)
            .title(" Seek step in seconds: ", Alignment::Left)
            .value(seek_step);

        drop(config_tui);
        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigSeekStep {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::SeekStepBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::SeekStepBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct KillDaemon {
    component: Radio,
    config: SharedTuiSettings,
}

impl KillDaemon {
    pub fn new(config: SharedTuiSettings) -> Self {
        let config_r = config.read();
        let enabled = config_r.settings.behavior.quit_server_on_exit;
        let component = Radio::default()
            .borders(
                Borders::default()
                    .color(config_r.settings.theme.library_border())
                    .modifiers(BorderType::Rounded),
            )
            .choices(&["Yes", "No"])
            .foreground(config_r.settings.theme.library_highlight())
            .rewind(true)
            .title(" Kill daemon when quit termusic? ", Alignment::Left)
            .value(usize::from(!enabled));

        drop(config_r);
        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for KillDaemon {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::KillDaemonBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::KillDaemonBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct PlayerUseMpris {
    component: Radio,
    config: SharedTuiSettings,
}

impl PlayerUseMpris {
    pub fn new(config: CombinedSettings) -> Self {
        let config_tui = config.tui.read();
        let enabled = config.server.read().settings.player.use_mediacontrols;
        let component = Radio::default()
            .borders(
                Borders::default()
                    .color(config_tui.settings.theme.library_border())
                    .modifiers(BorderType::Rounded),
            )
            .choices(&["Yes", "No"])
            .foreground(config_tui.settings.theme.library_highlight())
            .rewind(true)
            .title(" Support Mpris? ", Alignment::Left)
            .value(usize::from(!enabled));

        drop(config_tui);
        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, NoUserEvent> for PlayerUseMpris {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::PlayerUseMprisBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::PlayerUseMprisBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct PlayerUseDiscord {
    component: Radio,
    config: SharedTuiSettings,
}

impl PlayerUseDiscord {
    pub fn new(config: CombinedSettings) -> Self {
        let config_tui = config.tui.read();
        let enabled = config.server.read().settings.player.set_discord_status;
        let component = Radio::default()
            .borders(
                Borders::default()
                    .color(config_tui.settings.theme.library_border())
                    .modifiers(BorderType::Rounded),
            )
            .choices(&["Yes", "No"])
            .foreground(config_tui.settings.theme.library_highlight())
            .rewind(true)
            .title(" Update discord rpc? ", Alignment::Left)
            .value(usize::from(!enabled));

        drop(config_tui);
        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, NoUserEvent> for PlayerUseDiscord {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::PlayerUseDiscordBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::PlayerUseDiscordBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct PlayerPort {
    component: Input,
    config: SharedTuiSettings,
}

impl PlayerPort {
    pub fn new(config: CombinedSettings) -> Self {
        // TODO: this should likely also cover the MaybeCom settings from the TUI
        let component = {
            let config_tui = config.tui.read();
            Input::default()
                .borders(
                    Borders::default()
                        .color(config_tui.settings.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                .foreground(config_tui.settings.theme.library_highlight())
                .input_type(InputType::UnsignedInteger)
                .invalid_style(Style::default().fg(Color::Red))
                .placeholder(
                    "between 1000 ~ 60000 suggested",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
                .title(" Player Port: ", Alignment::Left)
                .value(config.server.read().settings.com.port.to_string())
        };

        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, NoUserEvent> for PlayerPort {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::PlayerPortBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::PlayerPortBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct UseNativeRadio {
    component: Radio,
    config: SharedTuiSettings,
}

impl UseNativeRadio {
    pub fn new(config: SharedTuiSettings) -> Self {
        let config_r = config.read();
        let enabled = config_r.settings.theme.use_native;
        let component = Radio::default()
            .borders(
                Borders::default()
                    .color(config_r.settings.theme.library_border())
                    .modifiers(BorderType::Rounded),
            )
            .choices(&["Yes", "No"])
            .foreground(config_r.settings.theme.library_highlight())
            .rewind(true)
            .title(" Use Native Color Theme(Pywal Support)? ", Alignment::Left)
            .value(usize::from(!enabled));

        drop(config_r);
        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for UseNativeRadio {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::UseNativeBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::UseNativeBlurUp),
        )
    }
}

impl Model {
    /// Mount / Remount the Config-Editor's First Page, the General Options
    pub(super) fn remount_config_general(&mut self) -> Result<()> {
        // Mount general page
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::MusicDir),
            Box::new(MusicDir::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::ExitConfirmation),
            Box::new(ExitConfirmation::new(self.config_tui.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PlaylistDisplaySymbol),
            Box::new(PlaylistDisplaySymbol::new(self.config_tui.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PlaylistRandomTrack),
            Box::new(PlaylistRandomTrack::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PlaylistRandomAlbum),
            Box::new(PlaylistRandomAlbum::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PodcastDir),
            Box::new(PodcastDir::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PodcastSimulDownload),
            Box::new(PodcastSimulDownload::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PodcastMaxRetries),
            Box::new(PodcastMaxRetries::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::AlbumPhotoAlign),
            Box::new(AlbumPhotoAlign::new(self.config_tui.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::SaveLastPosition),
            Box::new(SaveLastPosition::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::SeekStep),
            Box::new(ConfigSeekStep::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::KillDamon),
            Box::new(KillDaemon::new(self.config_tui.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PlayerUseMpris),
            Box::new(PlayerUseMpris::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PlayerUseDiscord),
            Box::new(PlayerUseDiscord::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::PlayerPort),
            Box::new(PlayerPort::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::UseNative),
            Box::new(UseNativeRadio::new(self.config_tui.clone())),
            Vec::new(),
        )?;

        Ok(())
    }

    /// Unmount the Config-Editor's First Page, the General Options
    pub(super) fn umount_config_general(&mut self) -> Result<()> {
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::MusicDir))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::ExitConfirmation))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistDisplaySymbol))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistRandomAlbum))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlaylistRandomTrack))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PodcastDir))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PodcastSimulDownload))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PodcastMaxRetries))?;
        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::AlbumPhotoAlign))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::SaveLastPosition))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::SeekStep))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::KillDamon))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlayerUseMpris))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlayerUseDiscord))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::PlayerPort))?;

        self.app
            .umount(&Id::ConfigEditor(IdConfigEditor::UseNative))?;

        Ok(())
    }
}
