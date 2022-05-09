use crate::config::Termusic;
use crate::player::GeneralP;
use crate::track::Track;
// use crate::ui::Status;
use crate::ui::{Id, Model, Msg};
// use std::thread::{self, sleep};
// use std::thread::sleep;

use std::time::Duration;
use tui_realm_stdlib::ProgressBar;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{Alignment, BorderType, Borders, Color, PropPayload, PropValue};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent};

#[derive(MockComponent)]
pub struct Progress {
    component: ProgressBar,
}

impl Progress {
    pub fn new(config: &Termusic) -> Self {
        Self {
            component: ProgressBar::default()
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .progress_border()
                                .unwrap_or(Color::LightMagenta),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .background(
                    config
                        .style_color_symbol
                        .progress_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .progress_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .label("Progress")
                .title(
                    format!(
                        "Status: Stopped | Volume: {} | Speed: {:^.1} ",
                        config.volume, config.speed,
                    ),
                    Alignment::Center,
                )
                .progress(0.0),
        }
    }
}

impl Component<Msg, NoUserEvent> for Progress {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        Some(Msg::None)
    }
}

impl Model {
    pub fn progress_reload(&mut self) {
        assert!(self.app.umount(&Id::Progress).is_ok());
        assert!(self
            .app
            .mount(
                Id::Progress,
                Box::new(Progress::new(&self.config)),
                Vec::new()
            )
            .is_ok());
        self.progress_update_title();
    }

    pub fn progress_update_title(&mut self) {
        let progress_title = format!(
            "Status: {} | Volume: {} | Speed: {:^.1} ",
            self.status, self.config.volume, self.config.speed,
        );
        self.app
            .attr(
                &Id::Progress,
                Attribute::Title,
                AttrValue::Title((progress_title, Alignment::Center)),
            )
            .ok();
    }

    pub fn progress_update(&mut self) {
        if let Ok((progress, time_pos, duration)) = self.player.get_progress() {
            // for unsupported file format, don't update progress
            if duration == 0 {
                return;
            }

            if time_pos >= duration {
                self.player_next();
                return;
            }

            self.time_pos = time_pos;

            let new_prog = Self::progress_safeguard(progress);
            self.progress_set(new_prog, duration);
        }
    }

    fn progress_safeguard(progress: f64) -> f64 {
        let mut new_prog = progress / 100.0;
        if new_prog > 1.0 {
            new_prog = 1.0;
        }
        new_prog
    }

    fn progress_set(&mut self, progress: f64, duration: i64) {
        self.app
            .attr(
                &Id::Progress,
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::F64(progress))),
            )
            .ok();

        self.app
            .attr(
                &Id::Progress,
                Attribute::Text,
                AttrValue::String(format!(
                    "{}    -    {}",
                    Track::duration_formatted_short(&Duration::from_secs(
                        self.time_pos.try_into().unwrap_or(0)
                    )),
                    Track::duration_formatted_short(&Duration::from_secs(
                        duration.try_into().unwrap_or(0)
                    ))
                )),
            )
            .ok();
    }
}
