use termusiclib::config::TuiOverlay;
use tuirealm::{
    component::{AppComponent, Component},
    event::Event,
    props::{SpanStatic, Style},
};

use crate::ui::{components::LabelSpan, model::UserEvent, msg::Msg};

#[derive(Component)]
pub struct Footer {
    component: LabelSpan,
}

impl Footer {
    pub fn new(config: &TuiOverlay) -> Self {
        let style_text = Style::new()
            .bold()
            .fg(config.settings.theme.fallback_foreground());
        let style_key = Style::new()
            .bold()
            .fg(config.settings.theme.fallback_highlight());

        Self {
            component: (LabelSpan::new(
                config,
                &[
                    SpanStatic::styled(" Help: ", style_text),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.select_view_keys.open_help),
                        style_key,
                    ),
                    SpanStatic::styled(" Config: ", style_text),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.select_view_keys.open_config),
                        style_key,
                    ),
                    SpanStatic::styled(" Library: ", style_text),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.select_view_keys.view_library),
                        style_key,
                    ),
                    SpanStatic::styled(" Database: ", style_text),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.select_view_keys.view_database),
                        style_key,
                    ),
                    SpanStatic::styled(" Podcasts: ", style_text),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.select_view_keys.view_podcasts),
                        style_key,
                    ),
                    SpanStatic::styled(" Version: ", style_text),
                    // maybe consider moving version into Help or Config or its own popup (like a About)
                    SpanStatic::styled(env!("TERMUSIC_VERSION"), style_key),
                ],
            )),
        }
    }
}

impl AppComponent<Msg, UserEvent> for Footer {
    fn on(&mut self, _ev: &Event<UserEvent>) -> Option<Msg> {
        None
    }
}
