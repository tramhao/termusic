use crate::config::Settings;
use crate::ui::{Id, Model, Msg};

use tui_realm_stdlib::Paragraph;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{
    Alignment, AttrValue, Attribute, BorderType, Borders, Color, PropPayload, PropValue, TextSpan,
};
use tuirealm::{Component, Event, MockComponent};

#[derive(MockComponent)]
pub struct Lyric {
    component: Paragraph,
}

impl Lyric {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Paragraph::default()
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .lyric_border()
                                .unwrap_or(Color::Green),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .background(
                    config
                        .style_color_symbol
                        .lyric_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .lyric_foreground()
                        .unwrap_or(Color::Cyan),
                )
                .title(" Lyrics ", Alignment::Left)
                .wrap(true)
                .text(&[TextSpan::new(format!(
                    "{}.",
                    crate::player::Status::Stopped
                ))]),
        }
    }
}

impl Component<Msg, NoUserEvent> for Lyric {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        Some(Msg::None)
    }
}

impl Model {
    pub fn lyric_reload(&mut self) {
        assert!(self
            .app
            .remount(Id::Lyric, Box::new(Lyric::new(&self.config)), Vec::new())
            .is_ok());
        self.lyric_update_title();
        let lyric_line = self.lyric_line.clone();
        self.lyric_set_lyric(&lyric_line);
    }

    pub fn update_lyric(&mut self) {
        if self.player.is_stopped() {
            self.lyric_set_lyric("Stopped.");
            return;
        }
        if let Some(song) = &self.player.playlist.current_track {
            if song.lyric_frames_is_empty() {
                self.lyric_set_lyric("No lyrics available.");
                return;
            }

            let mut line = String::new();
            if let Some(l) = song.parsed_lyric() {
                if l.unsynced_captions.is_empty() {
                    return;
                }
                if let Some(l) = l.get_text(self.time_pos) {
                    line = l;
                }
            }
            if self.lyric_line == line {
                return;
            }
            self.lyric_set_lyric(&line);
            self.lyric_line = line;
        }
    }

    fn lyric_set_lyric(&mut self, text: &str) {
        self.app
            .attr(
                &Id::Lyric,
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(vec![PropValue::TextSpan(TextSpan::from(
                    text,
                ))])),
            )
            .ok();
    }

    pub fn lyric_cycle(&mut self) {
        if let Some(mut song) = self.player.playlist.current_track.clone() {
            if let Ok(f) = song.cycle_lyrics() {
                let lang_ext = f.description.clone();
                self.player.playlist.current_track = Some(song);
                self.show_message_timeout(
                    "Lyric switch successful",
                    format!("{lang_ext} lyric is showing").as_str(),
                    None,
                );
            }
        }
    }
    pub fn lyric_adjust_delay(&mut self, offset: i64) {
        if let Some(song) = self.player.playlist.current_track.as_mut() {
            if let Err(e) = song.adjust_lyric_delay(self.time_pos, offset) {
                self.mount_error_popup(format!("adjust lyric delay error: {e}"));
            };
        }
    }

    pub fn lyric_update_title(&mut self) {
        let mut lyric_title = " No track is playing ".to_string();
        if let Some(song) = &self.player.playlist.current_track {
            let artist = song.artist().unwrap_or("Unknown Artist");
            let title = song.title().unwrap_or("Unknown Title");
            lyric_title = format!(" Lyrics of {artist:^.20} - {title:^.20} ");
        }
        self.lyric_title_set(&lyric_title);
    }

    fn lyric_title_set(&mut self, lyric_title: &str) {
        self.app
            .attr(
                &Id::Lyric,
                Attribute::Title,
                AttrValue::Title((lyric_title.to_string(), Alignment::Center)),
            )
            .ok();
    }
}
