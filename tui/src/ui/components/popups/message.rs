use termusiclib::config::{SharedTuiSettings, v2::tui::theme::styles::ColorTermusic};
use tui_realm_stdlib::components::Paragraph;
use tuirealm::{
    component::{AppComponent, Component},
    event::Event,
    props::{
        AttrValueRef, Attribute, BorderType, Borders, HorizontalAlignment, PropPayloadRef,
        QueryResult, TextModifiers, TextStatic, Title,
    },
};

use crate::ui::ids::Id;
use crate::ui::model::{Model, UserEvent};
use crate::ui::msg::Msg;

#[derive(Component)]
pub struct MessagePopup {
    component: Paragraph,
}

impl MessagePopup {
    pub fn new<T: Into<Title>, V: Into<TextStatic>>(
        config: &SharedTuiSettings,
        title: T,
        msg: V,
    ) -> Self {
        let config_tui = config.read_recursive();
        Self {
            component: Paragraph::default()
                .borders(
                    Borders::default()
                        .color(
                            config_tui
                                .settings
                                .theme
                                .get_color_from_theme(ColorTermusic::Cyan),
                        )
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
                .alignment_horizontal(HorizontalAlignment::Center)
                .title(title.into().alignment(HorizontalAlignment::Center))
                .text(msg.into()),
        }
    }
}

impl AppComponent<Msg, UserEvent> for MessagePopup {
    fn on(&mut self, _ev: &Event<UserEvent>) -> Option<Msg> {
        None
    }
}

impl Model {
    pub fn mount_message<T: Into<Title>, V: Into<TextStatic>>(&mut self, title: T, text: V) {
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
        if let Some(spans) = self
            .app
            .query(&Id::MessagePopup, Attribute::Text)
            .ok()
            .flatten()
            .as_ref()
            .map(QueryResult::as_ref)
            .and_then(AttrValueRef::as_payload)
            .and_then(PropPayloadRef::as_vec)
            && let Some(display_text) = spans.iter().next()
        {
            let d = &display_text.as_textspan().unwrap().content;
            if text.eq(d) {
                self.app.umount(&Id::MessagePopup).ok();
            }
        }
    }
}
