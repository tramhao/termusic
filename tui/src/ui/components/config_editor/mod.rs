/**
 * MIT License
 *
 * tuifeed - Copyright (c) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use anyhow::Result;
use termusiclib::config::{SharedTuiSettings, TuiOverlay};
use tui_realm_stdlib::components::{Radio, Span};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::Event;
use tuirealm::props::{
    AttrValue, Attribute, BorderSides, BorderType, Borders, HorizontalAlignment, SpanStatic, Style,
};

use super::popups::{YNConfirm, YNConfirmStyle};
use crate::ui::ids::{Id, IdConfigEditor};
use crate::ui::model::{Model, UserEvent};
use crate::ui::msg::{ConfigEditorLayout, ConfigEditorMsg, Msg};

mod color;
mod general;
mod key_combo;
mod update;
mod view;

#[derive(Component)]
pub struct CEHeader {
    component: Radio,
}

impl CEHeader {
    pub fn new(layout: ConfigEditorLayout, config: &TuiOverlay) -> Self {
        let mut this = Self {
            component: Radio::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Plain)
                        .sides(BorderSides::BOTTOM)
                        .color(config.settings.theme.library_highlight()),
                )
                .choices(ConfigEditorLayout::choice_array())
                .foreground(config.settings.theme.library_highlight())
                .background(config.settings.theme.library_background())
                // .inactive(Style::default().fg(config.settings.theme.library_highlight()))
                .value(layout.to_array_idx()),
        };

        // trick the component into using the "focused" paths; this should be fixed upstream
        // re https://github.com/veeso/tui-realm-stdlib/issues/61
        this.attr(Attribute::Focus, AttrValue::Flag(true));

        this
    }
}

impl AppComponent<Msg, UserEvent> for CEHeader {
    fn on(&mut self, _ev: &Event<UserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(Component)]
pub struct CEFooter {
    component: Span,
}

impl CEFooter {
    pub fn new(config: &TuiOverlay) -> Self {
        let style_text = Style::new()
            .bold()
            .fg(config.settings.theme.library_foreground());
        let style_key = Style::new()
            .bold()
            .fg(config.settings.theme.library_highlight());

        Self {
            component: Span::default()
                .background(config.settings.theme.library_background())
                .style(Style::new().bold())
                .spans([
                    SpanStatic::styled(" Save parameters: ", style_text),
                    SpanStatic::styled(
                        format!("<{}>", config.settings.keys.config_keys.save),
                        style_key,
                    ),
                    SpanStatic::styled(" Exit: ", style_text),
                    SpanStatic::styled(format!("<{}>", config.settings.keys.escape), style_key),
                    SpanStatic::styled(" Change panel: ", style_text),
                    SpanStatic::styled("<TAB>", style_key),
                    SpanStatic::styled(" Change field: ", style_text),
                    SpanStatic::styled("<UP/DOWN>", style_key),
                    SpanStatic::styled(" Select theme/Preview symbol: ", style_text),
                    SpanStatic::styled("<ENTER>", style_key),
                ]),
        }
    }
}

impl AppComponent<Msg, UserEvent> for CEFooter {
    fn on(&mut self, _ev: &Event<UserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(Component)]
pub struct ConfigSavePopup {
    component: YNConfirm,
}

impl ConfigSavePopup {
    pub fn new(config: SharedTuiSettings) -> Self {
        let component =
            YNConfirm::new_with_cb(config, " Config changed. Do you want to save? ", |config| {
                YNConfirmStyle {
                    foreground_color: config.settings.theme.important_popup_foreground(),
                    background_color: config.settings.theme.important_popup_background(),
                    border_color: config.settings.theme.important_popup_border(),
                    title_alignment: HorizontalAlignment::Center,
                }
            });
        Self { component }
    }
}

impl AppComponent<Msg, UserEvent> for ConfigSavePopup {
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::ConfigEditor(ConfigEditorMsg::ConfigSaveOk),
            Msg::ConfigEditor(ConfigEditorMsg::ConfigSaveCancel),
        )
    }
}

impl Model {
    /// Mount / Remount the Config-Editor's Header & Footer.
    fn remount_config_header_footer(&mut self) -> Result<()> {
        let layout = self.config_editor.last_layout;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Header),
            Box::new(CEHeader::new(layout, &self.config_tui.read())),
            Vec::new(),
        )?;
        self.app.remount(
            Id::ConfigEditor(IdConfigEditor::Footer),
            Box::new(CEFooter::new(&self.config_tui.read())),
            Vec::new(),
        )?;

        Ok(())
    }

    /// Unmount the Config-Editor's Header & Footer
    fn umount_config_header_footer(&mut self) -> Result<()> {
        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Header))?;

        self.app.umount(&Id::ConfigEditor(IdConfigEditor::Footer))?;

        Ok(())
    }
}
