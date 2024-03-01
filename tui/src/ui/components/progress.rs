use crate::ui::Model;
use std::time::Duration;
use termusiclib::config::Settings;
use termusiclib::track::{MediaType, Track};
use termusiclib::types::{Id, Msg};
use tui_realm_stdlib::ProgressBar;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{Alignment, BorderType, Borders, Color, PropPayload, PropValue};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent};

#[derive(MockComponent)]
pub struct Progress {
    component: ProgressBar,
}

impl Progress {
    #[allow(clippy::cast_precision_loss)]
    pub fn new(config: &Settings) -> Self {
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
                        " Status: Stopped | Volume: {} | Speed: {:^.1} ",
                        config.player_volume,
                        config.player_speed as f32 / 10.0,
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
        assert!(self
            .app
            .remount(
                Id::Progress,
                Box::new(Progress::new(&self.config)),
                Vec::new()
            )
            .is_ok());
        self.progress_update_title();
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn progress_update_title(&mut self) {
        let gapless = if self.config.player_gapless {
            "True"
        } else {
            "False"
        };
        let mut progress_title = String::new();
        if let Some(track) = &self.current_song {
            match track.media_type {
                Some(MediaType::Music | MediaType::LiveRadio) => {
                    progress_title = format!(
                        " Status: {} | Volume: {} | Speed: {:^.1} | Gapless: {} ",
                        self.playlist.status(),
                        self.config.player_volume,
                        self.config.player_speed as f32 / 10.0,
                        gapless,
                    );
                }
                Some(MediaType::Podcast) => {
                    progress_title = format!(
                        " Status: {} {:^.20} | Volume: {} | Speed: {:^.1} | Gapless: {} ",
                        self.playlist.status(),
                        track.title().unwrap_or("Unknown title"),
                        self.config.player_volume,
                        self.config.player_speed as f32 / 10.0,
                        gapless,
                    );
                }
                None => {}
            }
        }

        self.app
            .attr(
                &Id::Progress,
                Attribute::Title,
                AttrValue::Title((progress_title, Alignment::Center)),
            )
            .ok();
        self.force_redraw();
    }

    // TODO: refactor to have "duration" optional
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
    pub fn progress_update(&mut self, time_pos: Duration, total_duration: Duration) {
        // for unsupported file format, don't update progress
        if total_duration.is_zero() {
            return;
        }

        self.time_pos = time_pos;

        let progress = (time_pos.as_secs() * 100)
            .checked_div(total_duration.as_secs())
            .unwrap() as f64;

        let new_prog = Self::progress_safeguard(progress);

        self.progress_set(new_prog, total_duration);
    }

    fn progress_safeguard(progress: f64) -> f64 {
        let new_prog = progress / 100.0;
        new_prog.clamp(0.0, 1.0)
    }

    fn progress_set(&mut self, progress: f64, total_duration: Duration) {
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
                    Track::duration_formatted_short(&self.time_pos),
                    Track::duration_formatted_short(&total_duration)
                )),
            )
            .ok();
        // self.force_redraw();
    }
}
