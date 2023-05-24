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
                        config.volume,
                        config.speed as f32 / 10.0,
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
        let gapless = if self.config.gapless { "True" } else { "False" };
        let mut progress_title = String::new();
        if let Some(track) = self.playlist.current_track() {
            match track.media_type {
                Some(MediaType::Music) => {
                    progress_title = format!(
                        " Status: {} | Volume: {} | Speed: {:^.1} | Gapless: {} ",
                        self.playlist.status(),
                        self.config.volume,
                        self.config.speed as f32 / 10.0,
                        gapless,
                    );
                }
                Some(MediaType::Podcast) => {
                    progress_title = format!(
                        " Status: {} {:^.20} | Volume: {} | Speed: {:^.1} | Gapless: {} ",
                        self.playlist.status(),
                        track.title().unwrap_or("Unknown title"),
                        self.config.volume,
                        self.config.speed as f32 / 10.0,
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
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn progress_update(&mut self, time_pos: i64, duration: i64) {
        // for unsupported file format, don't update progress
        if duration == 0 {
            return;
        }

        self.time_pos = time_pos;

        let progress = (time_pos * 100).checked_div(duration).unwrap() as f64;

        let new_prog = Self::progress_safeguard(progress);

        // About to finish signal is a simulation of gstreamer, and used for gapless
        // #[cfg(any(not(feature = "gst"), feature = "mpv"))]
        // if !self.player.playlist.is_empty()
        //     && !self.player.playlist.has_next_track()
        //     && new_prog >= 0.5
        //     && duration - time_pos < 2
        //     && self.config.gapless
        // {
        //     // eprintln!("about to finish sent");
        //     self.player
        //         .message_tx
        //         .send(termusicplayback::PlayerMsg::AboutToFinish)
        //         .ok();
        // }

        self.progress_set(new_prog, duration);
    }

    fn progress_safeguard(progress: f64) -> f64 {
        let new_prog = progress / 100.0;
        new_prog.clamp(0.0, 1.0)
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
        self.force_redraw();
    }
}
