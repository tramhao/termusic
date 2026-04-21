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
    pub fn new(config: &TuiOverlay) -> Self {
        let style_text = Style::new()
            .bold()
            .fg(config.settings.theme.library_foreground());
        let style_key = Style::new()
            .bold()
            .fg(config.settings.theme.library_highlight());

        Self {
            component: (LabelSpan::new(
                config,
                &[
                    SpanStatic::styled(" Save tag: ", style_text),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.config_keys.save),
                        style_key,
                    ),
                    SpanStatic::styled(" Exit: ", style_text),
                    SpanStatic::styled(format!("<{}>", config.settings.keys.escape), style_key),
                    SpanStatic::styled(" Change field: ", style_text),
                    SpanStatic::styled("<Tab/ShiftTab>", style_key),
                    SpanStatic::styled(" Search/Embed tag: ", style_text),
                    SpanStatic::styled("<ENTER>", style_key),
                    SpanStatic::styled(" Download: ", style_text),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.library_keys.youtube_search),
                        style_key,
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
