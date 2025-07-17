use termusiclib::{config::TuiOverlay, types::Msg};
use tuirealm::{Component, Event, MockComponent, props::TextSpan};

use crate::ui::{components::LabelSpan, model::UserEvent};

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
                    )),
                    TextSpan::new(" DLNA server: ")
                        .fg(config.settings.theme.fallback_foreground())
                        .bold(),
                    TextSpan::new(format!(
                        "<{}>",
                        config.settings.keys.select_view_keys.view_dlnaserver
                    ))
                        .fg(config.settings.theme.fallback_highlight())
                        .bold(),
                    TextSpan::new(" DLNA: ")
                        .fg(config.settings.theme.fallback_foreground())
                        .bold(),
                    TextSpan::new(format!(
                        "<{}>",
                        config.settings.keys.select_view_keys.view_dlnaserver
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

impl Component<Msg, UserEvent> for Footer {
    fn on(&mut self, _ev: Event<UserEvent>) -> Option<Msg> {
        None
    }
}
