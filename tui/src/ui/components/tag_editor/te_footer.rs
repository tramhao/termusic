use termusiclib::{config::TuiOverlay, types::Msg};
use tuirealm::{props::TextSpan, Component, Event, MockComponent};

use crate::ui::{components::LabelSpan, model::UserEvent};

#[derive(MockComponent)]
pub struct TEFooter {
    component: LabelSpan,
}

impl TEFooter {
    pub fn new(config: &TuiOverlay) -> Self {
        Self {
            component: (LabelSpan::new(
                config,
                &[
                    TextSpan::new(" Save tag: ").fg(config.settings.theme.library_foreground()),
                    TextSpan::new(format!("<{}>", config.settings.keys.config_keys.save))
                        .bold()
                        .fg(config.settings.theme.library_highlight()),
                    TextSpan::new(" Exit: ").fg(config.settings.theme.library_foreground()),
                    TextSpan::new(format!("<{}>", config.settings.keys.escape))
                        .bold()
                        .fg(config.settings.theme.library_highlight()),
                    TextSpan::new(" Change field: ").fg(config.settings.theme.library_foreground()),
                    TextSpan::new("<Tab/ShiftTab>")
                        .bold()
                        .fg(config.settings.theme.library_highlight()),
                    TextSpan::new(" Search/Embed tag: ")
                        .fg(config.settings.theme.library_foreground()),
                    TextSpan::new("<ENTER>")
                        .bold()
                        .fg(config.settings.theme.library_highlight()),
                    TextSpan::new(" Download: ").fg(config.settings.theme.library_foreground()),
                    TextSpan::new(format!(
                        "<{}>",
                        config.settings.keys.library_keys.youtube_search
                    ))
                    .bold()
                    .fg(config.settings.theme.library_highlight()),
                ],
            )),
        }
    }
}

impl Component<Msg, UserEvent> for TEFooter {
    fn on(&mut self, _ev: Event<UserEvent>) -> Option<Msg> {
        None
    }
}
