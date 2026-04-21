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
    #[expect(clippy::similar_names)]
    pub fn new(config: &TuiOverlay) -> Self {
        let style_fg = Style::new()
            .bold()
            .fg(config.settings.theme.fallback_foreground());
        let style_hg = Style::new()
            .bold()
            .fg(config.settings.theme.fallback_highlight());

        Self {
            component: (LabelSpan::new(
                config,
                &[
                    SpanStatic::styled(" Help: ", style_fg),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.select_view_keys.open_help),
                        style_hg,
                    ),
                    SpanStatic::styled(" Config: ", style_fg),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.select_view_keys.open_config),
                        style_hg,
                    ),
                    SpanStatic::styled(" Library: ", style_fg),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.select_view_keys.view_library),
                        style_hg,
                    ),
                    SpanStatic::styled(" Database: ", style_fg),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.select_view_keys.view_database),
                        style_hg,
                    ),
                    SpanStatic::styled(" Podcasts: ", style_fg),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.select_view_keys.view_podcasts),
                        style_hg,
                    ),
                    SpanStatic::styled(" Version: ", style_fg),
                    // maybe consider moving version into Help or Config or its own popup (like a About)
                    SpanStatic::styled(env!("TERMUSIC_VERSION"), style_hg),
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
