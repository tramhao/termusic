use crate::config::Settings;
use crate::ui::{model::TermusicLayout, Id, Model, Msg};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use tui_realm_stdlib::Paragraph;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{
    Alignment, AttrValue, Attribute, BorderType, Borders, Color, PropPayload, PropValue, TextSpan,
};
use tuirealm::{Component, Event, MockComponent, State, StateValue};

lazy_static! {
    /// Regex for finding <br/> tags -- also captures any surrounding
    /// line breaks
    static ref RE_BR_TAGS: Regex = Regex::new(r"((\r\n)|\r|\n)*<br */?>((\r\n)|\r|\n)*").expect("Regex error");

    /// Regex for finding HTML tags
    static ref RE_HTML_TAGS: Regex = Regex::new(r"<[^<>]*>").expect("Regex error");

    /// Regex for finding more than two line breaks
    static ref RE_MULT_LINE_BREAKS: Regex = Regex::new(r"((\r\n)|\r|\n){3,}").expect("Regex error");
}

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

    pub fn update_lyric_for_podcast(&mut self) -> Result<()> {
        self.app
            .attr(
                &Id::Lyric,
                Attribute::Title,
                AttrValue::Title((" Description: ".to_string(), Alignment::Left)),
            )
            .ok();

        if let Ok(State::One(StateValue::Usize(podcast_index))) = self.app.state(&Id::Podcast) {
            if let Ok(State::One(StateValue::Usize(episode_index))) = self.app.state(&Id::Episode) {
                let podcast_selected = self
                    .podcasts
                    .get(podcast_index)
                    .ok_or_else(|| anyhow!("get podcast selected failed."))?;
                let episode_selected = podcast_selected
                    .episodes
                    .get(episode_index)
                    .ok_or_else(|| anyhow!("get episode selected failed."))?;

                // convert <br/> tags to a single line break
                let br_to_lb = RE_BR_TAGS.replace_all(&episode_selected.description, "\n");

                // strip all HTML tags
                let stripped_tags = RE_HTML_TAGS.replace_all(&br_to_lb, "");

                // convert HTML entities (e.g., &amp;)
                let decoded = match escaper::decode_html(&stripped_tags) {
                    Err(_) => stripped_tags.to_string(),
                    Ok(s) => s,
                };

                // remove anything more than two line breaks (i.e., one blank line)
                let no_line_breaks = RE_MULT_LINE_BREAKS.replace_all(&decoded, "\n\n");

                self.app
                    .attr(
                        &Id::Lyric,
                        Attribute::Text,
                        AttrValue::Payload(PropPayload::Vec(vec![PropValue::TextSpan(
                            TextSpan::from(no_line_breaks),
                        )])),
                    )
                    .ok();
            }
        }
        Ok(())
    }
    pub fn update_lyric(&mut self) {
        if self.layout == TermusicLayout::Podcast {
            if let Err(e) = self.update_lyric_for_podcast() {
                self.mount_error_popup(format!("update episode description error: {e}"));
            }
            return;
        }
        if self.player.playlist.is_stopped() {
            self.lyric_set_lyric("Stopped.");
            return;
        }
        if let Some(song) = self.player.playlist.current_track() {
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
        if let Some(mut song) = self.player.playlist.current_track_as_mut() {
            if let Ok(f) = song.cycle_lyrics() {
                let lang_ext = f.description.clone();
                self.player.playlist.set_current_track(Some(&song));
                self.show_message_timeout(
                    "Lyric switch successful",
                    format!("{lang_ext} lyric is showing").as_str(),
                    None,
                );
            }
        }
    }
    pub fn lyric_adjust_delay(&mut self, offset: i64) {
        if let Some(mut song) = self.player.playlist.current_track_as_mut() {
            if let Err(e) = song.adjust_lyric_delay(self.time_pos, offset) {
                self.mount_error_popup(format!("adjust lyric delay error: {e}"));
            };
        }
    }

    pub fn lyric_update_title(&mut self) {
        let mut lyric_title = " No track is playing ".to_string();
        if let Some(song) = self.player.playlist.current_track() {
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
