use termusiclib::config::TuiOverlay;
use tuirealm::{
    component::{AppComponent, Component},
    event::Event,
    props::{SpanStatic, Style},
};

use crate::ui::{components::LabelSpan, model::UserEvent, msg::Msg};

#[derive(Component)]
pub struct TEFooter {
    component: LabelSpan,
}

impl TEFooter {
    #[expect(clippy::similar_names)]
    pub fn new(config: &TuiOverlay) -> Self {
        let style_fg = Style::new().fg(config.settings.theme.library_foreground());
        let style_hg = Style::new()
            .bold()
            .fg(config.settings.theme.library_highlight());

        Self {
            component: (LabelSpan::new(
                config,
                &[
                    SpanStatic::styled(" Save tag: ", style_fg),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.config_keys.save),
                        style_hg,
                    ),
                    SpanStatic::styled(" Exit: ", style_fg),
                    SpanStatic::styled(format!("<{}>", config.settings.keys.escape), style_hg),
                    SpanStatic::styled(" Change field: ", style_fg),
                    SpanStatic::styled("<Tab/ShiftTab>", style_hg),
                    SpanStatic::styled(" Search/Embed tag: ", style_fg),
                    SpanStatic::styled("<ENTER>", style_hg),
                    SpanStatic::styled(" Download: ", style_fg),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.library_keys.youtube_search),
                        style_hg,
                    ),
                ],
            )),
        }
    }
}

impl AppComponent<Msg, UserEvent> for TEFooter {
    fn on(&mut self, _ev: &Event<UserEvent>) -> Option<Msg> {
        None
    }
}
