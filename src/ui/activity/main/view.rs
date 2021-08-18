/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
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
    MainActivity, COMPONENT_LABEL_HELP, COMPONENT_PARAGRAPH_LYRIC, COMPONENT_PROGRESS,
    COMPONENT_TABLE, COMPONENT_TREEVIEW,
};
use crate::ui::components::msgbox::{MsgBox, MsgBoxPropsBuilder};
use crate::ui::components::table;
use crate::ui::draw_area_in;
// Ext
use tui_realm_stdlib::{
    input, label, paragraph, progress_bar, radio,
    table::{Table, TablePropsBuilder},
};

use tuirealm::props::borders::{BorderType, Borders};
use tuirealm::props::{TableBuilder, TextSpan};
use tuirealm::{PropsBuilder, View};
// tui
use tui_realm_treeview::{TreeView, TreeViewPropsBuilder};
use tuirealm::tui::layout::{Alignment, Constraint, Direction, Layout};
use tuirealm::tui::style::Color;
use tuirealm::tui::widgets::Clear;

impl MainActivity {
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
                    .with_title("Playing", Alignment::Center)
                    .with_label("Song Name")
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
                    .with_title("Lyrics", Alignment::Left)
                    .with_texts(vec![TextSpan::new("No Lyrics available.")
                        .underlined()
                        .fg(Color::Green)])
                    .build(),
            )),
        );

        // Scrolltable
        self.view.mount(
            COMPONENT_TABLE,
            Box::new(table::Table::new(
                table::TablePropsBuilder::default()
                    .with_background(Color::Black)
                    .with_highlighted_str(Some("🚀"))
                    .with_highlighted_color(Color::LightBlue)
                    .with_max_scroll_step(4)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::Blue)
                    .scrollable(true)
                    // .with_title(" Duration ┼─── Artist ───┼────── Title ───────┼────── Album ──────┼────── Queue─")
                    .with_title("Queue", Alignment::Left)
                    .with_header(&["Duration", "Artist", "Title", "Album"])
                    .with_widths(&[10, 20, 25, 45])
                    .with_table(
                        TableBuilder::default()
                            .add_col(TextSpan::from("empty queue"))
                            .add_col(TextSpan::from(""))
                            .add_col(TextSpan::from(""))
                            .add_col(TextSpan::from(""))
                            .build(),
                    )
                    .build(),
            )),
        );

        self.view.mount(
            COMPONENT_TREEVIEW,
            Box::new(TreeView::new(
                TreeViewPropsBuilder::default()
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightYellow)
                    .with_foreground(Color::LightYellow)
                    .with_background(Color::Black)
                    .with_title("Playlist", Alignment::Left)
                    .with_tree_and_depth(self.tree.root(), 3)
                    .with_highlighted_str("🚀")
                    .build(),
            )),
        );

        // We need to initialize the focus
        self.view.active(COMPONENT_TREEVIEW);
    }

    /// View gui
    pub(super) fn view(&mut self) {
        if let Some(mut ctx) = self.context.take() {
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
                self.view.render(COMPONENT_TABLE, f, chunks_right[0]);
                self.view.render(COMPONENT_PROGRESS, f, chunks_right[1]);
                self.view
                    .render(COMPONENT_PARAGRAPH_LYRIC, f, chunks_right[2]);

                if let Some(props) = self.view.get_props(super::COMPONENT_TEXT_HELP) {
                    if props.visible {
                        // make popup
                        let popup = draw_area_in(f.size(), 50, 70);
                        f.render_widget(Clear, popup);
                        self.view.render(super::COMPONENT_TEXT_HELP, f, popup);
                    }
                }

                if let Some(props) = self.view.get_props(super::COMPONENT_INPUT_URL) {
                    if props.visible {
                        // make popup
                        let popup = draw_area_in(f.size(), 50, 10);
                        f.render_widget(Clear, popup);
                        self.view.render(super::COMPONENT_INPUT_URL, f, popup);
                    }
                }

                if let Some(props) = self.view.get_props(super::COMPONENT_TEXT_ERROR) {
                    if props.visible {
                        let popup = draw_area_in(f.size(), 50, 10);
                        f.render_widget(Clear, popup);
                        // make popup
                        self.view.render(super::COMPONENT_TEXT_ERROR, f, popup);
                    }
                }
                if let Some(props) = self.view.get_props(super::COMPONENT_CONFIRMATION_RADIO) {
                    if props.visible {
                        let popup = draw_area_in(f.size(), 20, 10);
                        f.render_widget(Clear, popup);
                        // make popup
                        self.view
                            .render(super::COMPONENT_CONFIRMATION_RADIO, f, popup);
                    }
                }

                if let Some(props) = self.view.get_props(super::COMPONENT_CONFIRMATION_INPUT) {
                    if props.visible {
                        let popup = draw_area_in(f.size(), 20, 10);
                        f.render_widget(Clear, popup);
                        // make popup
                        self.view
                            .render(super::COMPONENT_CONFIRMATION_INPUT, f, popup);
                    }
                }

                if let Some(props) = self.view.get_props(super::COMPONENT_SCROLLTABLE_YOUTUBE) {
                    if props.visible {
                        let popup = draw_area_in(f.size(), 66, 60);
                        f.render_widget(Clear, popup);
                        // make popup
                        self.view
                            .render(super::COMPONENT_SCROLLTABLE_YOUTUBE, f, popup);
                    }
                }
            });
            self.context = Some(ctx);
        }
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

    /// ### mount_del_ssh_key
    ///
    /// Mount delete ssh key component
    pub(super) fn mount_confirmation_radio(&mut self) {
        self.view.mount(
            super::COMPONENT_CONFIRMATION_RADIO,
            Box::new(radio::Radio::new(
                radio::RadioPropsBuilder::default()
                    .with_color(Color::LightRed)
                    .with_inverted_color(Color::Black)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightRed)
                    .with_title("Delete song?", Alignment::Left)
                    .with_options(&["Yes", "No"])
                    .with_value(1) // Default: No
                    .build(),
            )),
        );
        // Active
        self.view.active(super::COMPONENT_CONFIRMATION_RADIO);
    }

    /// ### umount_del_ssh_key
    ///
    /// Umount delete ssh key
    pub(super) fn umount_confirmation_radio(&mut self) {
        self.view.umount(super::COMPONENT_CONFIRMATION_RADIO);
    }

    pub(super) fn mount_confirmation_input(&mut self) {
        self.view.mount(
            super::COMPONENT_CONFIRMATION_INPUT,
            Box::new(input::Input::new(
                input::InputPropsBuilder::default()
                    .with_label(String::from("Type DELETE to confirm:"), Alignment::Left)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::Green)
                    .build(),
            )),
        );
        self.view.active(super::COMPONENT_CONFIRMATION_INPUT);
    }

    /// ### umount_new_ssh_key
    ///
    /// Umount new ssh key prompt
    pub(super) fn umount_confirmation_input(&mut self) {
        self.view.umount(super::COMPONENT_CONFIRMATION_INPUT);
    }

    /// ### mount_new_ssh_key
    ///
    /// Mount new ssh key prompt
    pub(super) fn mount_youtube_url(&mut self) {
        self.view.mount(
            super::COMPONENT_INPUT_URL,
            Box::new(input::Input::new(
                input::InputPropsBuilder::default()
                    .with_label(String::from("Download url or search:"), Alignment::Left)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::Green)
                    .build(),
            )),
        );
        self.view.active(super::COMPONENT_INPUT_URL);
    }

    /// ### umount_new_ssh_key
    ///
    /// Umount new ssh key prompt
    pub(super) fn umount_youtube_url(&mut self) {
        self.view.umount(super::COMPONENT_INPUT_URL);
    }

    // /// ### mount_help
    // ///
    // /// Mount help
    pub(super) fn mount_help(&mut self) {
        self.view.mount(
            super::COMPONENT_TEXT_HELP,
            Box::new(Table::new(
                TablePropsBuilder::default()
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::Green)
                    .with_title("Help", Alignment::Center)
                    .with_header(&["Key", "Function"])
                    .with_widths(&[30, 70])
                    .with_table(
                        TableBuilder::default()
                            .add_col(TextSpan::new("<ESC> or <Q>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Exit"))
                            .add_row()
                            .add_col(TextSpan::new("<TAB>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Switch focus"))
                            .add_row()
                            .add_col(TextSpan::new("<h,j,k,l,g,G>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Move cursor(vim style)"))
                            .add_row()
                            .add_col(TextSpan::new("<f/b>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Seek forward/backward 5 seconds"))
                            .add_row()
                            .add_col(TextSpan::new("<F/B>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Seek forward/backward 1 second for lyrics"))
                            .add_row()
                            .add_col(TextSpan::new("<n/space>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Next/Pause current song"))
                            .add_row()
                            .add_col(TextSpan::new("<+,=/-,_>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Increase/Decrease volume"))
                            .add_row()
                            .add_col(TextSpan::new("Playlist").bold().fg(Color::LightYellow))
                            .add_row()
                            .add_col(TextSpan::new("<l/L>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Add one/all songs to queue"))
                            .add_row()
                            .add_col(TextSpan::new("<d>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Delete song or folder"))
                            .add_row()
                            .add_col(TextSpan::new("<s>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Download or search song from youtube"))
                            .add_row()
                            .add_col(TextSpan::new("<t>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Open tag editor for tag and lyric download"))
                            .add_row()
                            .add_col(TextSpan::new("<y/p>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Yank and Paste files"))
                            .add_row()
                            .add_col(TextSpan::new("Queue").bold().fg(Color::LightYellow))
                            .add_row()
                            .add_col(TextSpan::new("<d/D>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Delete one/all songs from queue"))
                            .add_row()
                            .add_col(TextSpan::new("<l>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Play selected"))
                            .add_row()
                            .add_col(TextSpan::new("<s>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("Shuffle queue"))
                            .build(),
                    )
                    .build(),
            )),
        );
        // Active help
        self.view.active(super::COMPONENT_TEXT_HELP);
    }

    /// ### umount_help
    ///
    /// Umount help
    pub(super) fn umount_help(&mut self) {
        self.view.umount(super::COMPONENT_TEXT_HELP);
    }

    /// ### mount_youtube_options
    ///
    /// Mount youtube options
    pub(super) fn mount_youtube_options(&mut self) {
        self.view.mount(
            super::COMPONENT_SCROLLTABLE_YOUTUBE,
            Box::new(table::Table::new(
                table::TablePropsBuilder::default()
                    .with_background(Color::Black)
                    .with_highlighted_str(Some("🚀"))
                    .with_highlighted_color(Color::LightBlue)
                    .with_max_scroll_step(4)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::Blue)
                    .with_title("Tab/Shift+Tab for next and previous page", Alignment::Left)
                    .scrollable(true)
                    .with_table(
                        TableBuilder::default()
                            .add_col(TextSpan::from(
                                "empty queue.Probably the invidious server is down.",
                            ))
                            .build(),
                    )
                    .build(),
            )),
        );
        self.view.active(super::COMPONENT_SCROLLTABLE_YOUTUBE);
    }

    /// ### umount_youtube_options
    ///
    /// Umount youtube options
    pub(super) fn umount_youtube_options(&mut self) {
        self.view.umount(super::COMPONENT_SCROLLTABLE_YOUTUBE);
    }
}
