use crate::config::Settings;
use crate::ui::{ConfigEditorMsg, Msg};

use tui_realm_stdlib::{Input, Radio};
// use tuirealm::props::{Alignment, BorderSides, BorderType, Borders, Color, TableBuilder, TextSpan};
use crate::ui::components::Alignment as XywhAlign;
use tuirealm::props::{Alignment, BorderType, Borders, Color, InputType, Style};
use tuirealm::{
    command::{Cmd, Direction, Position},
    event::{Key, KeyEvent, NoUserEvent},
    Component, Event, MockComponent,
};

#[derive(MockComponent)]
pub struct MusicDir {
    component: Input,
    config: Settings,
}

impl MusicDir {
    pub fn new(config: &Settings) -> Self {
        let mut music_dir = String::new();
        for m in &config.music_dir {
            music_dir.push_str(m.as_str());
            music_dir.push(';');
        }
        let _ = music_dir.remove(music_dir.len() - 1);
        Self {
            component: Input::default()
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::LightGreen),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightGreen),
                )
                .input_type(InputType::Text)
                .placeholder("~/Music", Style::default().fg(Color::Rgb(128, 128, 128)))
                .title(" Root Music Directory ", Alignment::Left)
                .value(music_dir),
            config: config.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for MusicDir {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        handle_input_ev(
            self,
            ev,
            &config,
            Msg::ConfigEditor(ConfigEditorMsg::MusicDirBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::MusicDirBlurUp),
        )
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_input_ev(
    component: &mut dyn Component<Msg, NoUserEvent>,
    ev: Event<NoUserEvent>,
    config: &Settings,
    on_key_down: Msg,
    on_key_up: Msg,
) -> Option<Msg> {
    match ev {
        // Global Hotkeys
        Event::Keyboard(keyevent) if keyevent == config.keys.global_config_save.key_event() => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk))
        }
        Event::Keyboard(KeyEvent {
            code: Key::Down, ..
        }) => Some(on_key_down),
        Event::Keyboard(KeyEvent { code: Key::Up, .. }) => Some(on_key_up),
        Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout))
        }
        Event::Keyboard(keyevent) if keyevent == config.keys.global_esc.key_event() => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel))
        }

        // Local Hotkeys
        Event::Keyboard(KeyEvent {
            code: Key::Left, ..
        }) => {
            component.perform(Cmd::Move(Direction::Left));
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Right, ..
        }) => {
            component.perform(Cmd::Move(Direction::Right));
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Home, ..
        }) => {
            component.perform(Cmd::GoTo(Position::Begin));
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
            component.perform(Cmd::GoTo(Position::End));
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Delete, ..
        }) => {
            component.perform(Cmd::Cancel);
            Some(Msg::None)
        }
        Event::Keyboard(KeyEvent {
            code: Key::Backspace,
            ..
        }) => {
            component.perform(Cmd::Delete);
            Some(Msg::None)
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
    config: Settings,
}

impl ExitConfirmation {
    pub fn new(config: &Settings) -> Self {
        let enabled = config.enable_exit_confirmation;
        Self {
            component: Radio::default()
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::LightRed),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .choices(&["Yes", "No"])
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightRed),
                )
                .rewind(true)
                .title(" Show exit confirmation? ", Alignment::Left)
                .value(if enabled { 0 } else { 1 }),
            config: config.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for ExitConfirmation {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        handle_radio_ev(
            self,
            ev,
            &config,
            Msg::ConfigEditor(ConfigEditorMsg::ExitConfirmationBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::ExitConfirmationBlurUp),
        )
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_radio_ev(
    component: &mut dyn Component<Msg, NoUserEvent>,
    ev: Event<NoUserEvent>,
    config: &Settings,
    on_key_down: Msg,
    on_key_up: Msg,
) -> Option<Msg> {
    match ev {
        // Global Hotkeys
        Event::Keyboard(keyevent) if keyevent == config.keys.global_config_save.key_event() => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk))
        }
        Event::Keyboard(KeyEvent {
            code: Key::Down, ..
        }) => Some(on_key_down),
        Event::Keyboard(KeyEvent { code: Key::Up, .. }) => Some(on_key_up),
        Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout))
        }
        Event::Keyboard(keyevent) if keyevent == config.keys.global_down.key_event() => {
            Some(on_key_down)
        }
        Event::Keyboard(keyevent) if keyevent == config.keys.global_up.key_event() => {
            Some(on_key_up)
        }
        Event::Keyboard(keyevent) if keyevent == config.keys.global_quit.key_event() => {
            Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel))
        }
        Event::Keyboard(keyevent) if keyevent == config.keys.global_esc.key_event() => {
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
    config: Settings,
}

impl PlaylistDisplaySymbol {
    pub fn new(config: &Settings) -> Self {
        let enabled = config.enable_exit_confirmation;
        Self {
            component: Radio::default()
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::LightRed),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .choices(&["Yes", "No"])
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightRed),
                )
                .rewind(true)
                .title(" Display symbol in playlist title? ", Alignment::Left)
                .value(if enabled { 0 } else { 1 }),
            config: config.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for PlaylistDisplaySymbol {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        handle_radio_ev(
            self,
            ev,
            &config,
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistDisplaySymbolBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistDisplaySymbolBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct PlaylistRandomTrack {
    component: Input,
    config: Settings,
}

impl PlaylistRandomTrack {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Input::default()
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::LightRed),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightRed),
                )
                .input_type(InputType::Number)
                .placeholder("20", Style::default().fg(Color::Rgb(128, 128, 128)))
                .title(" Playlist Select Random Track Quantity: ", Alignment::Left)
                .value(config.playlist_select_random_track_quantity.to_string()),
            config: config.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for PlaylistRandomTrack {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        handle_input_ev(
            self,
            ev,
            &config,
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistRandomTrackBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistRandomTrackBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct PlaylistRandomAlbum {
    component: Input,
    config: Settings,
}

impl PlaylistRandomAlbum {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Input::default()
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::LightRed),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightRed),
                )
                .input_type(InputType::Number)
                .placeholder("1", Style::default().fg(Color::Rgb(128, 128, 128)))
                .title(" Playlist Select Random Track Quantity: ", Alignment::Left)
                .value(config.playlist_select_random_album_quantity.to_string()),
            config: config.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for PlaylistRandomAlbum {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        handle_input_ev(
            self,
            ev,
            &config,
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistRandomAlbumBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::PlaylistRandomAlbumBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct AlbumPhotoX {
    component: Input,
    config: Settings,
}

impl AlbumPhotoX {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Input::default()
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::LightRed),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightRed),
                )
                .input_type(InputType::Number)
                .placeholder(
                    "between 1 ~ 100",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
                .title(" Album photo x position(relative): ", Alignment::Left)
                .value(config.album_photo_xywh.x_between_1_100.to_string()),
            config: config.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for AlbumPhotoX {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        handle_input_ev(
            self,
            ev,
            &config,
            Msg::ConfigEditor(ConfigEditorMsg::AlbumPhotoXBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::AlbumPhotoXBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct AlbumPhotoY {
    component: Input,
    config: Settings,
}

impl AlbumPhotoY {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Input::default()
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::LightRed),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightRed),
                )
                .input_type(InputType::Number)
                .placeholder(
                    "between 1 ~ 100",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
                .title(" Album photo y position(relative): ", Alignment::Left)
                .value(config.album_photo_xywh.y_between_1_100.to_string()),
            config: config.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for AlbumPhotoY {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        handle_input_ev(
            self,
            ev,
            &config,
            Msg::ConfigEditor(ConfigEditorMsg::AlbumPhotoYBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::AlbumPhotoYBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct AlbumPhotoWidth {
    component: Input,
    config: Settings,
}

impl AlbumPhotoWidth {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Input::default()
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::LightRed),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightRed),
                )
                .input_type(InputType::Number)
                .placeholder(
                    "between 1 ~ 100",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
                .title(" Album photo width position(relative): ", Alignment::Left)
                .value(format!("{}", config.album_photo_xywh.width_between_1_100)),
            config: config.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for AlbumPhotoWidth {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        handle_input_ev(
            self,
            ev,
            &config,
            Msg::ConfigEditor(ConfigEditorMsg::AlbumPhotoWidthBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::AlbumPhotoWidthBlurUp),
        )
    }
}

#[derive(MockComponent)]
pub struct AlbumPhotoAlign {
    component: Radio,
    config: Settings,
}

impl AlbumPhotoAlign {
    pub fn new(config: &Settings) -> Self {
        let align = match config.album_photo_xywh.align {
            XywhAlign::BottomRight => 0,
            XywhAlign::BottomLeft => 1,
            XywhAlign::TopRight => 2,
            XywhAlign::TopLeft => 3,
        };
        Self {
            component: Radio::default()
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::LightRed),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .choices(&["BottomRight", "BottomLeft", "TopRight", "TopLeft"])
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightRed),
                )
                .rewind(true)
                .title(" Album Photo Align: ", Alignment::Left)
                .value(align),
            config: config.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for AlbumPhotoAlign {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        handle_radio_ev(
            self,
            ev,
            &config,
            Msg::ConfigEditor(ConfigEditorMsg::AlbumPhotoAlignBlurDown),
            Msg::ConfigEditor(ConfigEditorMsg::AlbumPhotoAlignBlurUp),
        )
    }
}
