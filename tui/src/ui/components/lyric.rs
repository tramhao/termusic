use crate::ui::{model::TermusicLayout, Model};
use termusiclib::podcast::Episode;
use termusiclib::track::MediaType;
use termusiclib::types::{Id, LyricMsg, Msg};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use termusicplayback::SharedSettings;
use tui_realm_stdlib::Textarea;
// use tui_realm_textarea::TextArea;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
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
    component: Textarea,
    config: SharedSettings,
}

impl Lyric {
    pub fn new(config: SharedSettings) -> Self {
        let component = {
            let config = config.read();
            Textarea::default()
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
                // .wrap(true)
                .step(4)
                .highlighted_str(&config.style_color_symbol.playlist_highlight_symbol)
                .text_rows(&[TextSpan::new(format!(
                    "{}.",
                    termusicplayback::Status::Stopped
                ))])
        };

        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for Lyric {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().keys;
        let _drop = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::End)),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::LyricMessage(LyricMsg::LyricTextAreaBlurDown)),
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::LyricMessage(LyricMsg::LyricTextAreaBlurUp)),

            Event::Keyboard(key) if key == keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }

            Event::Keyboard(key) if key == keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

impl Model {
    pub fn lyric_reload(&mut self) {
        assert!(self
            .app
            .remount(
                Id::Lyric,
                Box::new(Lyric::new(self.config.clone())),
                Vec::new()
            )
            .is_ok());
        self.lyric_update_title();
        let lyric_line = self.lyric_line.clone();
        self.lyric_set_lyric(&lyric_line);
    }

    pub fn lyric_update_for_podcast_by_current_track(&mut self) {
        let mut need_update = false;
        let mut pod_title = String::new();
        let mut ep_for_lyric = Episode::default();
        if let Some(track) = self.playlist.current_track().cloned() {
            if MediaType::Podcast == track.media_type {
                if let Some(file) = track.file() {
                    'outer: for pod in &self.podcasts {
                        for ep in &pod.episodes {
                            if ep.url == file {
                                pod_title.clone_from(&pod.title);
                                ep_for_lyric = ep.clone();
                                need_update = true;
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }

        if need_update {
            self.lyric_update_for_episode_after(&pod_title, &ep_for_lyric);
        }

        self.lyric_update_title();
    }

    pub fn lyric_update_for_podcast(&mut self) -> Result<()> {
        if self.podcasts.is_empty() {
            return Ok(());
        }
        if let Ok(State::One(StateValue::Usize(episode_index))) = self.app.state(&Id::Episode) {
            let podcast_selected = self
                .podcasts
                .get(self.podcasts_index)
                .ok_or_else(|| anyhow!("get podcast selected failed."))?
                .clone();
            let episode_selected = podcast_selected
                .episodes
                .get(episode_index)
                .ok_or_else(|| anyhow!("get episode selected failed."))?;

            self.lyric_update_for_episode_after(&podcast_selected.title, episode_selected);
        }

        self.lyric_update_title();
        Ok(())
    }

    pub fn lyric_update_for_episode_after(&mut self, po_title: &str, ep: &Episode) {
        // convert <br/> tags to a single line break
        let br_to_lb = RE_BR_TAGS.replace_all(&ep.description, "\n");

        // strip all HTML tags
        let stripped_tags = RE_HTML_TAGS.replace_all(&br_to_lb, "");

        // convert HTML entities (e.g., &amp;)
        let decoded = match escaper::decode_html(&stripped_tags) {
            Err(_) => stripped_tags.to_string(),
            Ok(s) => s,
        };

        // remove anything more than two line breaks (i.e., one blank line)
        let no_line_breaks = RE_MULT_LINE_BREAKS.replace_all(&decoded, "\n\n");

        let (term_width, _) = viuer::terminal_size();
        let term_width = usize::from(term_width);
        let lyric_width = term_width * 3 / 5;
        let lines_vec: Vec<_> = no_line_breaks.split('\n').collect();
        let mut short_string_vec: Vec<_> = Vec::new();
        for l in lines_vec {
            let unicode_width = unicode_width::UnicodeWidthStr::width(l);
            if unicode_width > lyric_width {
                let mut string_tmp = textwrap::wrap(l, lyric_width);
                short_string_vec.append(&mut string_tmp);
            } else {
                short_string_vec.push(std::borrow::Cow::Borrowed(l));
            }
        }

        let mut lines_textspan: Vec<_> = short_string_vec
            .into_iter()
            .map(|l| PropValue::TextSpan(TextSpan::from(l)))
            .collect();

        let mut final_vec: Vec<_> = Vec::new();
        final_vec.push(PropValue::TextSpan(TextSpan::from(po_title).bold()));
        final_vec.push(PropValue::TextSpan(TextSpan::from(&ep.title).bold()));
        final_vec.push(PropValue::TextSpan(TextSpan::from("   ")));

        if let Some(date) = ep.pubdate {
            final_vec.push(PropValue::TextSpan(
                TextSpan::from(format!("Published: {}", date.format("%B %-d, %Y"))).italic(),
            ));
        }

        final_vec.push(PropValue::TextSpan(
            TextSpan::from(format!("Duration: {}", ep.format_duration())).italic(),
        ));

        final_vec.push(PropValue::TextSpan(TextSpan::from("   ")));
        final_vec.push(PropValue::TextSpan(TextSpan::from("Description:").bold()));
        final_vec.append(&mut lines_textspan);

        self.app
            .attr(
                &Id::Lyric,
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(final_vec)),
            )
            .ok();
    }

    pub fn lyric_update(&mut self) {
        if self.layout == TermusicLayout::Podcast {
            if let Err(e) = self.lyric_update_for_podcast() {
                self.mount_error_popup(e.context("lyric update for podcast"));
            }
            return;
        }
        if self.playlist.is_stopped() {
            self.lyric_set_lyric("Stopped.");
            return;
        }
        if let Some(song) = &self.current_song {
            if MediaType::LiveRadio == song.media_type {
                return;
            }

            let mut line = String::new();
            if song.lyric_frames_is_empty() {
                self.lyric_set_lyric("No lyrics available.");
                return;
            }

            if let Some(l) = song.parsed_lyric() {
                if l.unsynced_captions.is_empty() {
                    self.lyric_set_lyric("No lyrics available.");
                    return;
                }
                if let Some(l) = l.get_text(self.time_pos) {
                    line = l;
                }
            }
            self.lyric_set_lyric(&line);
        }
    }

    pub fn lyric_update_for_radio<T: AsRef<str>>(&mut self, radio_title: T) {
        if let Some(song) = self.playlist.current_track() {
            if MediaType::LiveRadio == song.media_type {
                let radio_title = radio_title.as_ref();
                if radio_title.is_empty() {
                    return;
                }
                self.lyric_set_lyric(format!("Currently Playing: {radio_title}"));
            }
        }
    }

    fn lyric_set_lyric<T: Into<String>>(&mut self, text: T) {
        let text = text.into();
        if self.lyric_line == *text {
            return;
        }
        self.app
            .attr(
                &Id::Lyric,
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(vec![PropValue::TextSpan(TextSpan::from(
                    &text,
                ))])),
            )
            .ok();
        self.lyric_line = text;
    }

    pub fn lyric_cycle(&mut self) {
        if let Some(track) = self.playlist.current_track_as_mut() {
            if let Ok(f) = track.cycle_lyrics() {
                let lang_ext = f.description.clone();
                self.update_show_message_timeout(
                    "Lyric switch successful",
                    format!("{lang_ext} lyric is showing").as_str(),
                    None,
                );
            }
        }
    }
    pub fn lyric_adjust_delay(&mut self, offset: i64) {
        if let Some(track) = self.playlist.current_track_as_mut() {
            if let Err(e) = track.adjust_lyric_delay(self.time_pos, offset) {
                self.mount_error_popup(e.context("adjust lyric delay"));
            };
        }
    }

    pub fn lyric_update_title(&mut self) {
        let mut lyric_title = " No track is playing ".to_string();
        // error!("current track is: {:?}", self.playlist.get_current_track());

        if self.playlist.is_stopped() {
            self.lyric_title_set(&lyric_title);
            return;
        }

        if let Some(track) = &self.current_song {
            match track.media_type {
                MediaType::Music => {
                    let artist = track.artist().unwrap_or("Unknown Artist");
                    let title = track.title().unwrap_or("Unknown Title");
                    lyric_title = format!(" Lyrics of {artist:^.20} - {title:^.20} ");
                }
                MediaType::Podcast => {
                    lyric_title = " Details: ".to_string();
                }
                MediaType::LiveRadio => {
                    lyric_title = " Live Radio ".to_string();
                }
            }
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
