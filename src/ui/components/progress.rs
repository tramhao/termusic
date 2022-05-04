use crate::player::GeneralP;
use crate::song::Song;
// use crate::ui::Status;
use crate::ui::{Id, Model, Msg, Status};
// use std::thread::{self, sleep};
// use std::thread::sleep;

use crate::ui::components::StyleColorSymbol;
use if_chain::if_chain;
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
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: ProgressBar::default()
                .borders(
                    Borders::default()
                        .color(
                            style_color_symbol
                                .progress_border()
                                .unwrap_or(Color::LightMagenta),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .background(
                    style_color_symbol
                        .progress_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    style_color_symbol
                        .progress_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .label("Song Name")
                .title("Playing", Alignment::Center)
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
                Box::new(Progress::new(&self.config.style_color_symbol)),
                Vec::new()
            )
            .is_ok());
        self.progress_update_title();
    }

    pub fn progress_update_title(&mut self) {
        let stop_title = format!(
            "Stopped | Volume: {} | Speed: {:^.1} ",
            self.config.volume, self.config.speed,
        );
        if let Some(Status::Stopped) = self.status {
            self.app
                .attr(
                    &Id::Progress,
                    Attribute::Title,
                    AttrValue::Title((stop_title, Alignment::Center)),
                )
                .ok();
            return;
        }
        if_chain! {
        if let Some(song) = &self.current_song;
        let artist = song.artist().unwrap_or("Unknown Artist");
        let title = song.title().unwrap_or("Unknown Title");
        let progress_title = format!(
            "Playing: {:^.20} - {:^.20} | Volume: {} | Speed: {:^.1} ",
                        artist, title, self.config.volume, self.config.speed,
            );
        then {
            self.app.attr( &Id::Progress,
                Attribute::Title,
                AttrValue::Title((progress_title,Alignment::Center)),
                ).ok();

            }
        }
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn progress_update(&mut self) {
        if let Ok((progress, time_pos, duration)) = self.player.get_progress() {
            // for unsupported file format, don't update progress
            if duration == 0 {
                return;
            }

            if time_pos >= duration {
                // std::thread::sleep(Duration::from_millis(500));
                // println!("{}--{}", time_pos, duration);
                // self.status = Some(Status::Stopped);
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
                    Song::duration_formatted_short(&Duration::from_secs(
                        self.time_pos.try_into().unwrap_or(0)
                    )),
                    Song::duration_formatted_short(&Duration::from_secs(
                        duration.try_into().unwrap_or(0)
                    ))
                )),
            )
            .ok();
    }
}
