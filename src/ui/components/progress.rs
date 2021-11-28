// use crate::song::Song;
use crate::ui::{Id, Model, Msg, Status};

use humantime::format_duration;
use if_chain::if_chain;
use std::time::Duration;
use tui_realm_stdlib::ProgressBar;
// use tuirealm::command::CmdResult;
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
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        // let _drop = match ev {
        //     _ => CmdResult::None,
        // };
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

        if time_pos > self.time_pos && time_pos - self.time_pos < 1 {
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
                    format_duration(Duration::from_secs(self.time_pos)),
                    format_duration(Duration::from_secs(duration))
                )),
            )
            .ok();
    }
}
