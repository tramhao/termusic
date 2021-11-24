// use crate::song::Song;
use crate::ui::{Id, Model, Msg, Status};

use humantime::format_duration;
use if_chain::if_chain;
use std::time::Duration;
use tui_realm_stdlib::ProgressBar;
use tuirealm::command::CmdResult;
use tuirealm::props::{Alignment, BorderType, Borders, Color, PropPayload, PropValue};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent};

#[derive(MockComponent)]
pub struct Progress {
    component: ProgressBar,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            component: ProgressBar::default()
                .borders(
                    Borders::default()
                        .color(Color::LightMagenta)
                        .modifiers(BorderType::Rounded),
                )
                .foreground(Color::LightYellow)
                .label("Song Name")
                .title("Playing", Alignment::Center)
                .progress(0.0),
        }
    }
}

impl Component<Msg, NoUserEvent> for Progress {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _drop = match ev {
            // Event::User(UserEvent::Loaded(prog)) => {
            //     // Update
            //     let label = format!("{:02}%", (prog * 100.0) as usize);
            //     self.attr(
            //         Attribute::Value,
            //         AttrValue::Payload(PropPayload::One(PropValue::F64(prog))),
            //     );
            //     self.attr(Attribute::Text, AttrValue::String(label));
            //     CmdResult::None
            // }
            // Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => return Some(Msg::GaugeAlfaBlur),
            // Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

impl Model {
    pub fn update_progress_title(&mut self) {
        if_chain! {
            if let Some(song) = &self.current_song;
            let artist = song.artist().unwrap_or("Unknown Artist");
            let title = song.title().unwrap_or("Unknown Title");
            let progress_title = format!(
                "Playing: {:^.20} - {:^.20} | Volume: {}",
                            artist, title, self.config.volume
                );
            then {
                self.app.attr( &Id::Progress,
                    Attribute::Title,
                    AttrValue::Title((progress_title,Alignment::Center)),
                    ).ok();

            }
        }
    }

    pub fn update_progress(&mut self) {
        let (new_prog, time_pos, duration) = self.player.get_progress();
        if (new_prog, time_pos, duration) == (0.9, 0, 100) {
            return;
        }

        if time_pos >= duration {
            self.status = Some(Status::Stopped);
            return;
        }

        // let song = match self.current_song.clone() {
        //     Some(s) => s,
        //     None => return,
        // };

        // if time_pos > self.time_pos || time_pos < 2 {
        if time_pos - self.time_pos < 1 {
            return;
        }
        self.time_pos = time_pos;
        self.app
            .attr(
                &Id::Progress,
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::F64(new_prog))),
            )
            .ok();

        self.app
            .attr(
                &Id::Progress,
                Attribute::Text,
                AttrValue::String(format!(
                    "{}     :     {} ",
                    format_duration(Duration::from_secs(time_pos as u64)),
                    format_duration(Duration::from_secs(duration as u64))
                )),
            )
            .ok();

        // if let Some(props) = self.view.get_props(COMPONENT_PROGRESS) {
        //     let props = ProgressBarPropsBuilder::from(props)
        //         .with_progress(new_prog)
        //         .with_label(format!(
        //             "{}     :     {} ",
        //             format_duration(Duration::from_secs(time_pos as u64)),
        //             format_duration(Duration::from_secs(duration as u64))
        //         ))
        //         .build();
        //     let msg = self.view.update(COMPONENT_PROGRESS, props);
        //     self.redraw = true;
        //     self.update(&msg);
        // }
        // }

        // Update lyrics
        // if self.playlist_items.is_empty() {
        //     return;
        // }

        // if song.lyric_frames_is_empty() {
        //     if let Some(props) = self.view.get_props(COMPONENT_PARAGRAPH_LYRIC) {
        //         let props = ParagraphPropsBuilder::from(props)
        //             .with_texts(vec![TextSpan::new("No lyrics available.")])
        //             .build();
        //         self.view.update(COMPONENT_PARAGRAPH_LYRIC, props);
        //         return;
        //     }
        // }

        // let mut line = String::new();
        // if let Some(l) = song.parsed_lyric() {
        //     if l.unsynced_captions.is_empty() {
        //         return;
        //     }
        //     if let Some(l) = l.get_text(time_pos) {
        //         line = l;
        //     }
        // }

        // if let Some(props) = self.view.get_props(COMPONENT_PARAGRAPH_LYRIC) {
        //     let props = ParagraphPropsBuilder::from(props)
        //         .with_texts(vec![TextSpan::new(line)])
        //         .build();
        //     self.view.update(COMPONENT_PARAGRAPH_LYRIC, props);
        // }
    }
}
