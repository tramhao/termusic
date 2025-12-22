use termusiclib::config::{SharedTuiSettings, v2::tui::theme::styles::ColorTermusic};
use tui_realm_stdlib::Paragraph;
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent,
    props::{Alignment, BorderType, Borders, Color, PropPayload, TextModifiers, TextSpan},
};

use crate::ui::ids::Id;
use crate::ui::model::{Model, UserEvent};
use crate::ui::msg::Msg;

#[derive(MockComponent)]
pub struct MessagePopup {
    component: Paragraph,
}

impl MessagePopup {
    pub fn new<S: Into<String>>(config: &SharedTuiSettings, title: S, msg: S) -> Self {
        let config_tui = config.read_recursive();
        Self {
            component: Paragraph::default()
                .borders(
                    Borders::default()
                        .color(Color::Cyan)
                        .modifiers(BorderType::Rounded),
                )
                .foreground(
                    config_tui
                        .settings
                        .theme
                        .get_color_from_theme(ColorTermusic::Green),
                )
                .background(config_tui.settings.theme.library_background())
                .modifiers(TextModifiers::BOLD)
                .alignment(Alignment::Center)
                .title(title.into(), Alignment::Center)
                .text(vec![TextSpan::from(msg)]),
        }
    }
}

impl Component<Msg, UserEvent> for MessagePopup {
    fn on(&mut self, _ev: Event<UserEvent>) -> Option<Msg> {
        None
    }
}

impl Model {
    pub fn mount_message(&mut self, title: &str, text: &str) {
        assert!(
            self.app
                .remount(
                    Id::MessagePopup,
                    Box::new(MessagePopup::new(&self.config_tui, title, text)),
                    vec![]
                )
                .is_ok()
        );
    }

    /// ### `umount_message`
    ///
    /// Umount error message
    pub fn umount_message(&mut self, _title: &str, text: &str) {
        if let Ok(Some(AttrValue::Payload(PropPayload::Vec(spans)))) =
            self.app.query(&Id::MessagePopup, Attribute::Text)
            && let Some(display_text) = spans.into_iter().next()
        {
            let d = display_text.unwrap_text_span().content;
            if text.eq(&d) {
                self.app.umount(&Id::MessagePopup).ok();
            }
        }
    }
}
