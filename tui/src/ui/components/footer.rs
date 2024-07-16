use termusiclib::{config::TuiOverlay, types::Msg};
use tuirealm::{props::TextSpan, Component, Event, MockComponent, NoUserEvent};

use crate::ui::components::LabelSpan;

#[derive(MockComponent)]
pub struct Footer {
    component: LabelSpan,
}

impl Footer {
    pub fn new(config: &TuiOverlay) -> Self {
        Self {
            component: (LabelSpan::new(
                config,
                &[
                    TextSpan::new(" Help: ")
                        .fg(config.settings.theme.fallback_foreground())
                        .bold(),
                    TextSpan::new(format!(
                        "<{}>",
                        config.settings.keys.select_view_keys.open_help
                    ))
                    .fg(config.settings.theme.fallback_highlight())
                    .bold(),
                    TextSpan::new(" Config: ")
                        .fg(config.settings.theme.fallback_foreground())
                        .bold(),
                    TextSpan::new(format!(
                        "<{}>",
                        config.settings.keys.select_view_keys.open_config
                    ))
                    .fg(config.settings.theme.fallback_highlight())
                    .bold(),
                    TextSpan::new(" Library: ")
                        .fg(config.settings.theme.fallback_foreground())
                        .bold(),
                    TextSpan::new(format!(
                        "<{}>",
                        config.settings.keys.select_view_keys.view_library
                    ))
                    .fg(config.settings.theme.fallback_highlight())
                    .bold(),
                    TextSpan::new(" Database: ")
                        .fg(config.settings.theme.fallback_foreground())
                        .bold(),
                    TextSpan::new(format!(
                        "<{}>",
                        config.settings.keys.select_view_keys.view_database
                    ))
                    .fg(config.settings.theme.fallback_highlight())
                    .bold(),
                    TextSpan::new(" Podcasts: ")
                        .fg(config.settings.theme.fallback_foreground())
                        .bold(),
                    TextSpan::new(format!(
                        "<{}>",
                        config.settings.keys.select_view_keys.view_podcasts
                    ))
                    .fg(config.settings.theme.fallback_highlight())
                    .bold(),
                    TextSpan::new(" Version: ")
                        .fg(config.settings.theme.fallback_foreground())
                        .bold(),
                    // maybe consider moving version into Help or Config or its own popup (like a About)
                    TextSpan::new(env!("TERMUSIC_VERSION"))
                        .fg(config.settings.theme.fallback_highlight())
                        .bold(),
                ],
            )),
        }
    }
}

impl Component<Msg, NoUserEvent> for Footer {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}
