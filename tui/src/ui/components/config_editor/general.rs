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
use anyhow::Result;
use termusiclib::config::v2::server::{Backend, ComProtocol, default_uds_socket_path};
use termusiclib::config::v2::tui::theme::styles::ColorTermusic;
use termusiclib::config::v2::tui::{Alignment as XywhAlign, keys::Keys};
use termusiclib::config::{SharedTuiSettings, TuiOverlay};
use tui_realm_stdlib::Radio;
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType, Style};
use tuirealm::{
    Component, Event, MockComponent,
    command::{Cmd, Direction, Position},
    event::{Key, KeyEvent},
};

use crate::CombinedSettings;
use crate::ui::components::vendored::tui_realm_stdlib_input::Input;
use crate::ui::ids::{Id, IdCEGeneral, IdConfigEditor};
use crate::ui::model::{Model, UserEvent};
use crate::ui::msg::{ConfigEditorMsg, KFMsg, Msg};

/// Get a [`Input`] component with the common style applied.
#[inline]
fn common_input_comp(config: &TuiOverlay, title: &str) -> Input {
    Input::default()
        .borders(
            Borders::default()
                .color(config.settings.theme.library_border())
                .modifiers(BorderType::Rounded),
        )
        .foreground(config.settings.theme.library_foreground())
        .background(config.settings.theme.library_background())
        .inactive(Style::new().bg(config.settings.theme.library_background()))
        .invalid_style(
            Style::default().fg(config
                .settings
                .theme
                .get_color_from_theme(ColorTermusic::Red)),
        )
        .title(title, Alignment::Left)
}

/// Get a [`Radio`] component with the common style applied.
#[inline]
fn common_radio_comp(config: &TuiOverlay, title: &str) -> Radio {
    Radio::default()
        .borders(
            Borders::default()
                .color(config.settings.theme.library_border())
                .modifiers(BorderType::Rounded),
        )
        .foreground(config.settings.theme.library_foreground())
        .background(config.settings.theme.library_background())
        .inactive(Style::new().bg(config.settings.theme.library_background()))
        .title(title, Alignment::Left)
}

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
        let component =
            common_input_comp(&config_tui, " Root Music Directory:(use ; to separate) ")
                .input_type(InputType::Text)
                .placeholder("~/Music", Style::default().fg(Color::Rgb(128, 128, 128)))
                .value(music_dir_input);

        drop(config_tui);
        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, UserEvent> for MusicDir {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
        )
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_input_ev(
    component: &mut dyn MockComponent,
    ev: Event<UserEvent>,
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
        let component = common_radio_comp(&config_r, " Show exit confirmation? ")
            .choices(["Yes", "No"])
            .rewind(true)
            .value(usize::from(!enabled));

        drop(config_r);
        Self { component, config }
    }
}

impl Component<Msg, UserEvent> for ExitConfirmation {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
        )
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_radio_ev(
    component: &mut dyn MockComponent,
    ev: Event<UserEvent>,
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
        let component = common_radio_comp(&config_r, " Use symbols for playlist loop mode? ")
            .choices(["Yes", "No"])
            .rewind(true)
            .value(usize::from(!enabled));

        drop(config_r);
        Self { component, config }
    }
}

impl Component<Msg, UserEvent> for PlaylistDisplaySymbol {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
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
            common_input_comp(&config_tui, " Playlist Select Random Track Quantity: ")
                .input_type(InputType::UnsignedInteger)
                .placeholder("20", Style::default().fg(Color::Rgb(128, 128, 128)))
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

impl Component<Msg, UserEvent> for PlaylistRandomTrack {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
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
            common_input_comp(
                &config_tui,
                " Playlist Select Random Album with tracks no less than: ",
            )
            .input_type(InputType::UnsignedInteger)
            .placeholder("1", Style::default().fg(Color::Rgb(128, 128, 128)))
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

impl Component<Msg, UserEvent> for PlaylistRandomAlbum {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
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
            common_input_comp(&config_tui, " Podcast Download Directory: ")
                .input_type(InputType::Text)
                .placeholder(
                    "~/Music/podcast",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
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

impl Component<Msg, UserEvent> for PodcastDir {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
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
            common_input_comp(&config_tui, " Podcast Simultaneous Download: ")
                .input_type(InputType::UnsignedInteger)
                .placeholder(
                    "between 1 ~ 5 suggested",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
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

impl Component<Msg, UserEvent> for PodcastSimulDownload {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
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
            common_input_comp(&config_tui, " Podcast Download Max Retries: ")
                .input_type(InputType::UnsignedInteger)
                .placeholder(
                    "between 1 ~ 5 suggested",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
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

impl Component<Msg, UserEvent> for PodcastMaxRetries {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
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
        let component = common_radio_comp(&config_r, " Coverart Align: ")
            .choices(["BottomRight", "BottomLeft", "TopRight", "TopLeft"])
            .rewind(true)
            .value(align);

        drop(config_r);
        Self { component, config }
    }
}

impl Component<Msg, UserEvent> for AlbumPhotoAlign {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
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
        let component = common_radio_comp(&config_tui, " Remember last played position: ")
            .choices(["Unsupported", "No", "Yes"])
            .rewind(true)
            .value(save_last_position);

        drop(config_tui);
        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, UserEvent> for SaveLastPosition {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
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
        let component = common_radio_comp(&config_tui, " Seek step in seconds: ")
            .choices(["Unsupported"])
            .rewind(true)
            .value(seek_step);

        drop(config_tui);
        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, UserEvent> for ConfigSeekStep {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
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
        let component = common_radio_comp(&config_r, " Stop Server on TUI exit? ")
            .choices(["Yes", "No"])
            .rewind(true)
            .value(usize::from(!enabled));

        drop(config_r);
        Self { component, config }
    }
}

impl Component<Msg, UserEvent> for KillDaemon {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
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
        let component = common_radio_comp(&config_tui, " Support Media Controls? ")
            .choices(["Yes", "No"])
            .rewind(true)
            .value(usize::from(!enabled));

        drop(config_tui);
        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, UserEvent> for PlayerUseMpris {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
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
        let component = common_radio_comp(&config_tui, " Update discord rpc? ")
            .choices(["Yes", "No"])
            .rewind(true)
            .value(usize::from(!enabled));

        drop(config_tui);
        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, UserEvent> for PlayerUseDiscord {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
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
            common_input_comp(&config_tui, " Player Port: ")
                .input_type(InputType::UnsignedInteger)
                .placeholder(
                    "between 1000 ~ 60000 suggested",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
                .value(config.server.read().settings.com.port.to_string())
        };

        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, UserEvent> for PlayerPort {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
        )
    }
}

#[derive(MockComponent)]
pub struct PlayerAddress {
    component: Input,
    config: SharedTuiSettings,
}

impl PlayerAddress {
    pub fn new(config: CombinedSettings) -> Self {
        // TODO: this should likely also cover the MaybeCom settings from the TUI
        let component = {
            let config_tui = config.tui.read();
            common_input_comp(&config_tui, " Player Address: ")
                .input_type(InputType::Text) // we likely could make a custom matcher
                .placeholder(
                    "::1 or 127.0.0.1 recommended",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
                .value(config.server.read().settings.com.address.to_string())
        };

        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, UserEvent> for PlayerAddress {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
        )
    }
}

#[derive(MockComponent)]
pub struct PlayerProtocol {
    component: Radio,
    config: SharedTuiSettings,
}

impl PlayerProtocol {
    pub fn new(config: CombinedSettings) -> Self {
        let config_tui = config.tui.read();
        let value = match config.server.read().settings.com.protocol {
            ComProtocol::HTTP => 0,
            ComProtocol::UDS => 1,
        };
        let component = common_radio_comp(&config_tui, " Communication Protocol: ")
            .choices(["HTTP", "UDS"])
            .rewind(true)
            .value(value);

        drop(config_tui);
        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, UserEvent> for PlayerProtocol {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
        )
    }
}

#[derive(MockComponent)]
pub struct PlayerUDSPath {
    component: Input,
    config: SharedTuiSettings,
}

impl PlayerUDSPath {
    pub fn new(config: CombinedSettings) -> Self {
        let component = {
            let config_tui = config.tui.read();
            common_input_comp(&config_tui, " Player UDS Socket Path: ")
                .input_type(InputType::Text)
                .placeholder(
                    default_uds_socket_path().display().to_string(),
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
                .value(
                    config
                        .server
                        .read()
                        .settings
                        .com
                        .socket_path
                        .to_string_lossy(),
                )
        };

        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, UserEvent> for PlayerUDSPath {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
        )
    }
}

#[derive(MockComponent)]
pub struct PlayerBackend {
    component: Radio,
    config: SharedTuiSettings,
}

impl PlayerBackend {
    pub fn new(config: CombinedSettings) -> Self {
        let config_tui = config.tui.read();
        let value = match config.server.read().settings.player.backend {
            Backend::Rusty => 0,
            Backend::Mpv => 1,
            Backend::Gstreamer => 2,
        };
        let component = common_radio_comp(&config_tui, " Playback Backend: ")
            .choices(["Rusty", "MPV", "Gstreamer"])
            .rewind(true)
            .value(value);

        drop(config_tui);
        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, UserEvent> for PlayerBackend {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_radio_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
        )
    }
}

#[derive(MockComponent)]
pub struct ExtraYtdlpArgs {
    component: Input,
    config: SharedTuiSettings,
}

impl ExtraYtdlpArgs {
    pub fn new(config: CombinedSettings) -> Self {
        let component = {
            let config_tui = config.tui.read();
            common_input_comp(&config_tui, " Extra Args for yt-dlp: ")
                .input_type(InputType::Text)
                .placeholder(
                    r#"--cookies-from-browser brave+gnomekeyring or --cookies "d:\src\cookies.txt""#,
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
                .value(&config_tui.settings.ytdlp.extra_args)
        };

        Self {
            component,
            config: config.tui,
        }
    }
}

impl Component<Msg, UserEvent> for ExtraYtdlpArgs {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        handle_input_ev(
            &mut self.component,
            ev,
            &self.config.read().settings.keys,
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Next)),
            Msg::ConfigEditor(ConfigEditorMsg::General(KFMsg::Previous)),
        )
    }
}

impl Model {
    /// Mount / Remount the Config-Editor's First Page, the General Options
    #[allow(clippy::too_many_lines)]
    pub(super) fn remount_config_general(&mut self) -> Result<()> {
        // Mount general page
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::MusicDir)),
            Box::new(MusicDir::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::ExitConfirmation)),
            Box::new(ExitConfirmation::new(self.config_tui.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::PlaylistDisplaySymbol)),
            Box::new(PlaylistDisplaySymbol::new(self.config_tui.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::PlaylistRandomTrack)),
            Box::new(PlaylistRandomTrack::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::PlaylistRandomAlbum)),
            Box::new(PlaylistRandomAlbum::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::PodcastDir)),
            Box::new(PodcastDir::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::PodcastSimulDownload)),
            Box::new(PodcastSimulDownload::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::PodcastMaxRetries)),
            Box::new(PodcastMaxRetries::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::AlbumPhotoAlign)),
            Box::new(AlbumPhotoAlign::new(self.config_tui.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::SaveLastPosition)),
            Box::new(SaveLastPosition::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::SeekStep)),
            Box::new(ConfigSeekStep::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::KillDamon)),
            Box::new(KillDaemon::new(self.config_tui.clone())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::PlayerUseMpris)),
            Box::new(PlayerUseMpris::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::PlayerUseDiscord)),
            Box::new(PlayerUseDiscord::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::PlayerPort)),
            Box::new(PlayerPort::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::PlayerAddress)),
            Box::new(PlayerAddress::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::PlayerProtocol)),
            Box::new(PlayerProtocol::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::PlayerUDSPath)),
            Box::new(PlayerUDSPath::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::PlayerBackend)),
            Box::new(PlayerBackend::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::General(IdCEGeneral::ExtraYtdlpArgs)),
            Box::new(ExtraYtdlpArgs::new(self.get_combined_settings())),
            Vec::new(),
        )?;

        Ok(())
    }

    /// Unmount the Config-Editor's First Page, the General Options
    pub(super) fn umount_config_general(&mut self) -> Result<()> {
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::MusicDir,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::ExitConfirmation,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::PlaylistDisplaySymbol,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::PlaylistRandomAlbum,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::PlaylistRandomTrack,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::PodcastDir,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::PodcastSimulDownload,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::PodcastMaxRetries,
        )))?;
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::AlbumPhotoAlign,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::SaveLastPosition,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::SeekStep,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::KillDamon,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::PlayerUseMpris,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::PlayerUseDiscord,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::PlayerPort,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::PlayerAddress,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::PlayerProtocol,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::PlayerUDSPath,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::PlayerBackend,
        )))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::General(
            IdCEGeneral::ExtraYtdlpArgs,
        )))?;

        Ok(())
    }
}
