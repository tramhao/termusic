//! ## SetupActivity
//!
//! `setup_activity` is the module which implements the Setup activity, which is the activity to
//! work on termscp configuration

/**
 * MIT License
 *
 * termscp - Copyright (c) 2021 Christian Visintin
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
// Locals
use super::{
    Context, TagEditorActivity, COMPONENT_LABEL_HELP, COMPONENT_PARAGRAPH_LYRIC,
    COMPONENT_PROGRESS, COMPONENT_SCROLLTABLE, COMPONENT_TREEVIEW,
};
use crate::ui::components::msgbox::{MsgBox, MsgBoxPropsBuilder};
use crate::ui::components::scrolltable;
use crate::ui::draw_area_in;
// Ext
use tuirealm::components::{
    input, label, paragraph, progress_bar, radio,
    table::{Table, TablePropsBuilder},
};
use tuirealm::props::borders::{BorderType, Borders};
use tuirealm::props::{TableBuilder, TextSpan, TextSpanBuilder};
use tuirealm::{PropsBuilder, View};
// tui
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Color;
use tui_realm_treeview::{TreeView, TreeViewPropsBuilder};
use tuirealm::tui::widgets::Clear;

impl TagEditorActivity {
    // -- view

    /// ### init_setup
    ///
    /// Initialize setup view
    pub(super) fn init_setup(&mut self) {
        // Init view
        self.view = View::init();
        // Let's mount the component we need
        self.view.mount(
            COMPONENT_PROGRESS,
            Box::new(progress_bar::ProgressBar::new(
                progress_bar::ProgressBarPropsBuilder::default()
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightMagenta)
                    .with_progbar_color(Color::LightYellow)
                    .with_texts(Some(String::from("Playing")), String::from("Song Name"))
                    .with_background(Color::Black)
                    .with_progress(0.0)
                    .build(),
            )),
        );
        self.view.mount(
            COMPONENT_LABEL_HELP,
            Box::new(label::Label::new(
                label::LabelPropsBuilder::default()
                    .with_foreground(Color::Cyan)
                    .with_text(String::from("Press \"?\" for help."))
                    .build(),
            )),
        );
        self.view.mount(
            COMPONENT_PARAGRAPH_LYRIC,
            Box::new(paragraph::Paragraph::new(
                paragraph::ParagraphPropsBuilder::default()
                    .with_foreground(Color::Cyan)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::Green)
                    .with_texts(
                        Some(String::from("Lyrics")),
                        vec![
                            TextSpanBuilder::new("No Lyrics available.")
                                .underlined()
                                .with_foreground(Color::Green)
                                .build(), // ,
                        ],
                    )
                    .build(),
            )),
        );

        // Scrolltable
        self.view.mount(
            COMPONENT_SCROLLTABLE,
            Box::new(scrolltable::Scrolltable::new(
                scrolltable::ScrollTablePropsBuilder::default()
                    .with_background(Color::Black)
                    .with_highlighted_str(Some("🚀"))
                    .with_highlighted_color(Color::LightBlue)
                    .with_max_scroll_step(4)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::Blue)
                    .with_table(
                        Some(String::from("Queue")),
                        TableBuilder::default()
                            .add_col(TextSpan::from("0"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("andreas"))
                            .build(),
                    )
                    .build(),
            )),
        );

        // We need to initialize the focus
        self.view.active(COMPONENT_TREEVIEW);
    }

    /// View gui
    pub(super) fn view(&mut self) {
        let mut ctx: Context = self.context.take().unwrap();
        let _ = ctx.context.draw(|f| {
            // Prepare chunks
            let chunks_main = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
                .split(f.size());
            let chunks_left = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)].as_ref())
                .split(chunks_main[0]);
            let chunks_right = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints(
                    [
                        Constraint::Min(2),
                        Constraint::Length(3),
                        Constraint::Length(4),
                    ]
                    .as_ref(),
                )
                .split(chunks_left[1]);

            self.view.render(COMPONENT_TREEVIEW, f, chunks_left[0]);
            self.view.render(COMPONENT_LABEL_HELP, f, chunks_main[1]);
            self.view.render(COMPONENT_SCROLLTABLE, f, chunks_right[0]);
            self.view.render(COMPONENT_PROGRESS, f, chunks_right[1]);
            self.view
                .render(COMPONENT_PARAGRAPH_LYRIC, f, chunks_right[2]);
            if let Some(props) = self.view.get_props(super::COMPONENT_TEXT_ERROR) {
                if props.visible {
                    let popup = draw_area_in(f.size(), 50, 10);
                    f.render_widget(Clear, popup);
                    // make popup
                    self.view.render(super::COMPONENT_TEXT_ERROR, f, popup);
                }
            }
        });
        self.context = Some(ctx);
    }

    // -- mount

    // ### mount_error
    //
    // Mount error box
    pub(super) fn mount_error(&mut self, text: &str) {
        // Mount
        self.view.mount(
            super::COMPONENT_TEXT_ERROR,
            Box::new(MsgBox::new(
                MsgBoxPropsBuilder::default()
                    .with_foreground(Color::Red)
                    .bold()
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::Red)
                    .with_texts(None, vec![TextSpan::from(text)])
                    .build(),
            )),
        );
        // Give focus to error
        self.view.active(super::COMPONENT_TEXT_ERROR);
    }

    /// ### umount_error
    ///
    /// Umount error message
    pub(super) fn umount_error(&mut self) {
        self.view.umount(super::COMPONENT_TEXT_ERROR);
    }
}
