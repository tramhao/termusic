use crate::config::Termusic;
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
    pub fn new(config: &Termusic) -> Self {
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
                .title("Lyrics", Alignment::Left)
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
        // let _drop = match ev {
        //     _ => CmdResult::None,
        // };
        Some(Msg::None)
    }
}

impl Model {
    pub fn lyric_reload(&mut self) {
        // assert!(self.app.umount(&Id::Lyric).is_ok());
        assert!(self
            .app
            .remount(Id::Lyric, Box::new(Lyric::new(&self.config)), Vec::new())
            .is_ok());
        self.lyric_update_title();
        self.update_lyric();
    }

    pub fn update_lyric(&mut self) {
        if self.player.is_stopped() {
            self.app
                .attr(
                    &Id::Lyric,
                    Attribute::Text,
                    AttrValue::Payload(PropPayload::Vec(vec![PropValue::TextSpan(
                        TextSpan::from("Stopped."),
                    )])),
                )
                .ok();
            return;
        }
        let song = match self.player.playlist.current_track.clone() {
            Some(s) => s,
            None => return,
        };

        if song.lyric_frames_is_empty() {
            self.app
                .attr(
                    &Id::Lyric,
                    Attribute::Text,
                    AttrValue::Payload(PropPayload::Vec(vec![PropValue::TextSpan(
                        TextSpan::from("No lyrics available."),
                    )])),
                )
                .ok();
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
        self.lyric_line = line.clone();
        self.app
            .attr(
                &Id::Lyric,
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(vec![PropValue::TextSpan(TextSpan::from(
                    line,
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
                    format!("{} lyric is showing", lang_ext).as_str(),
                    None,
                );
            }
        }
    }
    pub fn lyric_adjust_delay(&mut self, offset: i64) {
        if let Some(song) = self.player.playlist.current_track.as_mut() {
            if let Err(e) = song.adjust_lyric_delay(self.time_pos, offset) {
                self.mount_error_popup(format!("adjust lyric delay error: {}", e).as_str());
            };
        }
    }

    pub fn lyric_update_title(&mut self) {
        if let Some(song) = &self.player.playlist.current_track {
            let artist = song.artist().unwrap_or("Unknown Artist");
            let title = song.title().unwrap_or("Unknown Title");
            let lyric_title = format!(" Lyrics of {:^.20} - {:^.20} ", artist, title,);
            {
                self.app
                    .attr(
                        &Id::Lyric,
                        Attribute::Title,
                        AttrValue::Title((lyric_title, Alignment::Center)),
                    )
                    .ok();
            }
        }
    }
}
