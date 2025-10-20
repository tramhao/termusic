use std::time::Duration;

use termusiclib::config::TuiOverlay;
use termusiclib::player::RunningStatus;
use termusiclib::track::DurationFmtShort;
use termusiclib::track::MediaTypesSimple;
use tuirealm::props::{Alignment, BorderType, Borders, PropPayload, PropValue};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent};

use crate::ui::Model;
use crate::ui::components::vendored::tui_realm_stdlib_progressbar::ProgressBar;
use crate::ui::ids::Id;
use crate::ui::model::UserEvent;
use crate::ui::msg::Msg;

#[derive(MockComponent)]
pub struct Progress {
    component: ProgressBar,
}

impl Progress {
    #[allow(clippy::cast_precision_loss)]
    pub fn new(config: &TuiOverlay) -> Self {
        Self {
            component: ProgressBar::default()
                .borders(
                    Borders::default()
                        .color(config.settings.theme.progress_border())
                        .modifiers(BorderType::Rounded),
                )
                .background(config.settings.theme.progress_background())
                .foreground(config.settings.theme.progress_foreground())
                .label("Progress")
                .title(
                    " Status: Stopped | Volume: ?? | Speed: ??.? ",
                    Alignment::Center,
                )
                .progress(0.0),
        }
    }
}

impl Component<Msg, UserEvent> for Progress {
    fn on(&mut self, _ev: Event<UserEvent>) -> Option<Msg> {
        None
    }
}

#[allow(clippy::cast_precision_loss)] // speed is never realisitcally expected to be above i16::MAX
fn title_format(
    status: RunningStatus,
    title: Option<&str>,
    volume: u16,
    speed: i32,
    gapless: bool,
) -> String {
    let gapless = if gapless { "True" } else { "False" };

    if let Some(title) = title {
        format!(
            " Status: {} {:^.20} | Volume: {} | Speed: {:^.1} | Gapless: {} ",
            status,
            title,
            volume,
            speed as f32 / 10.0,
            gapless,
        )
    } else {
        format!(
            " Status: {} | Volume: {} | Speed: {:^.1} | Gapless: {} ",
            status,
            volume,
            speed as f32 / 10.0,
            gapless,
        )
    }
}

impl Model {
    pub fn progress_reload(&mut self) {
        assert!(
            self.app
                .remount(
                    Id::Progress,
                    Box::new(Progress::new(&self.config_tui.read())),
                    Vec::new()
                )
                .is_ok()
        );
        self.progress_update_title();
    }

    /// Update the [`Progress`] component's title.
    ///
    /// This needs to be run if one of the following changes:
    /// - volume
    /// - speed
    /// - gapless
    /// - running status
    /// - moving onto / off a podcast track
    pub fn progress_update_title(&mut self) {
        let config_server = self.config_server.read();
        let player = &config_server.settings.player;

        let progress_title = if let Some(track) = self.playback.current_track() {
            match track.media_type() {
                MediaTypesSimple::Music | MediaTypesSimple::LiveRadio => title_format(
                    self.playback.status(),
                    None,
                    player.volume,
                    player.speed,
                    player.gapless,
                ),
                MediaTypesSimple::Podcast => title_format(
                    self.playback.status(),
                    Some(track.title().unwrap_or("Unknown title")),
                    player.volume,
                    player.speed,
                    player.gapless,
                ),
            }
        } else {
            title_format(
                self.playback.status(),
                None,
                player.volume,
                player.speed,
                player.gapless,
            )
        };

        drop(config_server);
        self.app
            .attr(
                &Id::Progress,
                Attribute::Title,
                AttrValue::Title((progress_title, Alignment::Center)),
            )
            .ok();
        self.force_redraw();
    }

    /// Handle progress updates.
    ///
    /// Updates all places where progress updates need to be populated to.
    #[allow(clippy::cast_precision_loss)]
    pub fn progress_update(&mut self, time_pos: Option<Duration>, total_duration: Duration) {
        let time_pos = time_pos.unwrap_or_default();

        self.playback.set_current_track_pos(time_pos);

        let progress = if time_pos.as_secs() > 0 && total_duration.as_secs() > 0 {
            (time_pos.as_secs() * 100)
                .checked_div(total_duration.as_secs())
                .unwrap() as f64
        } else {
            0.0
        };

        let new_prog = Self::progress_safeguard(progress);

        self.progress_set(new_prog, total_duration);
        self.lyric_update();
    }

    /// Convert the input to a scale of `0.0` to `1.0` (clamped).
    fn progress_safeguard(progress: f64) -> f64 {
        let new_prog = progress / 100.0;
        new_prog.clamp(0.0, 1.0)
    }

    /// Set the progress bar text.
    fn progress_set(&mut self, progress: f64, total_duration: Duration) {
        self.app
            .attr(
                &Id::Progress,
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::F64(progress))),
            )
            .ok();

        let text = if self.playback.is_stopped() {
            DurationFmtShort::fmt_empty().to_string()
        } else if total_duration.is_zero() {
            format!("{}", DurationFmtShort(self.playback.current_track_pos()),)
        } else {
            format!(
                "{}    -    {}",
                DurationFmtShort(self.playback.current_track_pos()),
                DurationFmtShort(total_duration),
            )
        };

        let _ = self
            .app
            .attr(&Id::Progress, Attribute::Text, AttrValue::String(text));
    }
}
