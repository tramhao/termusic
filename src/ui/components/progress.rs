use crate::config::Settings;
use crate::track::Track;
use crate::ui::{Id, Model, Msg};

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
        // self.progress_update_title();
    }

    pub fn progress_update_title(&mut self) {
        let gapless = if self.config.gapless { "True" } else { "False" };
        let progress_title = format!(
            " Status: {} | Volume: {} | Speed: {:^.1} | Gapless: {} ",
            self.player.status(),
            self.config.volume,
            self.config.speed as f32 / 10.0,
            gapless,
        );
        self.app
            .attr(
                &Id::Progress,
                Attribute::Title,
                AttrValue::Title((progress_title, Alignment::Center)),
            )
            .ok();
    }

    // #[cfg(any(feature = "mpv", feature = "gst"))]
    pub fn progress_update(&mut self, time_pos: i64, duration: i64) {
        // for unsupported file format, don't update progress
        if duration == 0 {
            return;
        }

        self.time_pos = time_pos;

        let progress = (time_pos * 100).checked_div(duration).unwrap() as f64;

        let new_prog = Self::progress_safeguard(progress);

        // About to finish signal is a simulation of gstreamer, and used for gapless
        #[cfg(not(feature = "gst"))]
        if !self.player.playlist.is_empty()
            && !self.player.has_next_track()
            && new_prog >= 0.5
            && duration - time_pos < 2
        {
            self.player
                .message_tx
                .send(crate::player::PlayerMsg::AboutToFinish)
                .unwrap();
        }

        self.progress_set(new_prog, duration);
    }

    // #[cfg(not(any(feature = "mpv", feature = "gst")))]
    // pub fn progress_update(&mut self) {
    //     if let Ok((progress, time_pos, duration)) = self.player.get_progress() {
    //         // for unsupported file format, don't update progress
    //         if duration == 0 {
    //             return;
    //         }

    //         self.time_pos = time_pos;

    //         let new_prog = Self::progress_safeguard(progress);

    //         // About to finish signal is a simulation of gstreamer, and used for gapless
    //         #[cfg(not(feature = "gst"))]
    //         if !self.player.playlist.is_empty()
    //             && !self.player.has_next_track()
    //             && new_prog >= 0.5
    //             && duration - time_pos < 2
    //         {
    //             self.player
    //                 .message_tx
    //                 .send(crate::player::PlayerMsg::AboutToFinish)
    //                 .unwrap();
    //         }

    //         self.progress_set(new_prog, duration);
    //     }
    // }

    fn progress_safeguard(progress: f64) -> f64 {
        let mut new_prog = progress / 100.0;
        if new_prog > 1.0 {
            new_prog = 1.0;
        }
        if new_prog < 0.0 {
            new_prog = 0.0;
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
