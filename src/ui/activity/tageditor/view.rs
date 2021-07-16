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
use super::{Context, TagEditorActivity};
use crate::song::Song;
use crate::ui::components::msgbox::{MsgBox, MsgBoxPropsBuilder};
use crate::ui::components::scrolltable;
use crate::ui::draw_area_in;
// Ext
use tuirealm::components::{
    input, label, radio,
    table::{Table, TablePropsBuilder},
    textarea,
};
use tuirealm::props::borders::{BorderType, Borders};
use tuirealm::props::{TableBuilder, TextSpan, TextSpanBuilder};
use tuirealm::{PropsBuilder, View};

// tui
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Color;
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
            super::COMPONENT_TE_LABEL_HELP,
            Box::new(label::Label::new(
                label::LabelPropsBuilder::default()
                    .with_foreground(Color::Cyan)
                    .with_text(String::from("Press \"?\" for help."))
                    .build(),
            )),
        );

        self.view.mount(
            super::COMPONENT_TE_RADIO_TAG,
            Box::new(radio::Radio::new(
                radio::RadioPropsBuilder::default()
                    .with_color(Color::Magenta)
                    .with_borders(
                        Borders::BOTTOM | Borders::TOP,
                        BorderType::Double,
                        Color::Magenta,
                    )
                    .with_inverted_color(Color::Black)
                    .with_value(0)
                    .with_options(
                        Some(String::from("Tag operation:")),
                        vec![String::from("Rename file by Tag")],
                    )
                    .build(),
            )),
        );

        self.view.mount(
            super::COMPONENT_TE_INPUT_ARTIST,
            Box::new(input::Input::new(
                input::InputPropsBuilder::default()
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightYellow)
                    .with_foreground(Color::Cyan)
                    .with_label(String::from("Search Artist"))
                    .build(),
            )),
        );
        self.view.mount(
            super::COMPONENT_TE_INPUT_SONGNAME,
            Box::new(input::Input::new(
                input::InputPropsBuilder::default()
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightYellow)
                    .with_foreground(Color::Cyan)
                    .with_label(String::from("Search Song"))
                    .build(),
            )),
        );
        // Scrolltable
        self.view.mount(
            super::COMPONENT_TE_SCROLLTABLE_OPTIONS,
            Box::new(scrolltable::Scrolltable::new(
                scrolltable::ScrollTablePropsBuilder::default()
                    .with_background(Color::Black)
                    .with_highlighted_str(Some("🚀"))
                    .with_highlighted_color(Color::LightBlue)
                    .with_max_scroll_step(4)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::Blue)
                    .with_table(
                        Some(String::from(
                            "─ Artist ───┼──── Title ─────┼──── Album ─────┤  api  ├ Copyright Info ┤",
                        )),
                        TableBuilder::default()
                            .add_col(TextSpan::from("0"))
                            .add_col(TextSpan::from(" "))
                            .add_col(TextSpan::from("No Results."))
                            .build(),
                    )
                    .build(),
            )),
        );
        // Textarea
        self.view.mount(
            super::COMPONENT_TE_TEXTAREA_LYRIC,
            Box::new(textarea::Textarea::new(
                textarea::TextareaPropsBuilder::default()
                    .with_foreground(Color::Green)
                    .with_highlighted_str(Some("🚀"))
                    .with_max_scroll_step(4)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightMagenta)
                    .with_texts(
                        Some(String::from("Lyrics")),
                        vec![TextSpanBuilder::new("No Lyrics.")
                            .bold()
                            .underlined()
                            .with_foreground(Color::Yellow)
                            .build()],
                    )
                    .build(),
            )),
        );

        // We need to initialize the focus
        self.view.active(super::COMPONENT_TE_RADIO_TAG);
    }

    /// View gui
    pub(super) fn view(&mut self) {
        let mut ctx: Context = self.context.take().unwrap();
        let _ = ctx.context.draw(|f| {
            // Prepare chunks
            let chunks_main = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints(
                    [
                        Constraint::Length(4),
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let chunks_middle1 = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints(
                    [
                        Constraint::Ratio(1, 4),
                        Constraint::Ratio(2, 4),
                        Constraint::Ratio(1, 4),
                    ]
                    .as_ref(),
                )
                .split(chunks_main[1]);
            let chunks_middle2 = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Ratio(3, 5), Constraint::Ratio(2, 5)].as_ref())
                .split(chunks_main[2]);

            self.view
                .render(super::COMPONENT_TE_RADIO_TAG, f, chunks_main[0]);
            self.view
                .render(super::COMPONENT_TE_INPUT_ARTIST, f, chunks_middle1[0]);
            self.view
                .render(super::COMPONENT_TE_INPUT_SONGNAME, f, chunks_middle1[1]);
            self.view.render(
                super::COMPONENT_TE_SCROLLTABLE_OPTIONS,
                f,
                chunks_middle2[0],
            );
            self.view
                .render(super::COMPONENT_TE_LABEL_HELP, f, chunks_main[3]);

            self.view
                .render(super::COMPONENT_TE_TEXTAREA_LYRIC, f, chunks_middle2[1]);
            if let Some(props) = self.view.get_props(super::COMPONENT_TE_TEXT_ERROR) {
                if props.visible {
                    let popup = draw_area_in(f.size(), 50, 10);
                    f.render_widget(Clear, popup);
                    // make popup
                    self.view.render(super::COMPONENT_TE_TEXT_ERROR, f, popup);
                }
            }

            if let Some(props) = self.view.get_props(super::COMPONENT_TE_TEXT_HELP) {
                if props.visible {
                    // make popup
                    let popup = draw_area_in(f.size(), 50, 70);
                    f.render_widget(Clear, popup);
                    self.view.render(super::COMPONENT_TE_TEXT_HELP, f, popup);
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
            super::COMPONENT_TE_TEXT_ERROR,
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
        self.view.active(super::COMPONENT_TE_TEXT_ERROR);
    }

    /// ### umount_error
    ///
    /// Umount error message
    pub(super) fn umount_error(&mut self) {
        self.view.umount(super::COMPONENT_TE_TEXT_ERROR);
    }

    // initialize the value in tageditor based on info from Song
    pub fn init_by_song(&mut self, s: Song) {
        self.song = Some(s.clone());
        let props = input::InputPropsBuilder::from(
            self.view
                .get_props(super::COMPONENT_TE_INPUT_ARTIST)
                .unwrap(),
        )
        .with_value(s.artist.unwrap_or_else(|| String::from("")))
        .build();
        self.view.update(super::COMPONENT_TE_INPUT_ARTIST, props);

        let props = input::InputPropsBuilder::from(
            self.view
                .get_props(super::COMPONENT_TE_INPUT_SONGNAME)
                .unwrap(),
        )
        .with_value(s.title.unwrap_or_else(|| String::from("")))
        .build();
        self.view.update(super::COMPONENT_TE_INPUT_SONGNAME, props);

        if !s.lyric_frames.is_empty() {
            let mut vec_lang: Vec<String> = vec![];
            for l in s.lyric_frames.iter() {
                vec_lang.push(l.lang.clone());
            }

            let mut vec_lyric: Vec<TextSpan> = vec![];
            for line in s.lyric_frames[0].text.split('\n') {
                vec_lyric.push(TextSpan::from(line));
            }
            let props = textarea::TextareaPropsBuilder::from(
                self.view
                    .get_props(super::COMPONENT_TE_TEXTAREA_LYRIC)
                    .unwrap(),
            )
            .with_texts(
                Some(format!("{} Lyrics:", s.lyric_frames[0].lang)),
                vec_lyric,
            )
            .build();
            let msg = self.view.update(super::COMPONENT_TE_TEXTAREA_LYRIC, props);
            self.update(msg);
        }
    }

    pub(super) fn mount_help(&mut self) {
        self.view.mount(
            super::COMPONENT_TE_TEXT_HELP,
            Box::new(Table::new(
                TablePropsBuilder::default()
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::Green)
                    .with_table(
                        Some(String::from("Help")),
                        TableBuilder::default()
                            .add_col(
                                TextSpanBuilder::new("<ESC> or <Q>")
                                    .bold()
                                    .with_foreground(Color::Cyan)
                                    .build(),
                            )
                            .add_col(TextSpan::from("     Exit"))
                            .add_row()
                            .add_col(
                                TextSpanBuilder::new("<TAB>")
                                    .bold()
                                    .with_foreground(Color::Cyan)
                                    .build(),
                            )
                            .add_col(TextSpan::from("            Switch focus"))
                            .add_row()
                            .add_col(
                                TextSpanBuilder::new("<h,j,k,l,g,G>")
                                    .bold()
                                    .with_foreground(Color::Cyan)
                                    .build(),
                            )
                            .add_col(TextSpan::from("    Move cursor(vim style)"))
                            .add_row()
                            .add_col(
                                TextSpanBuilder::new("<enter>")
                                    .bold()
                                    .with_foreground(Color::Cyan)
                                    .build(),
                            )
                            .add_col(TextSpan::from("          in editor start search"))
                            .add_row()
                            .add_col(
                                TextSpanBuilder::new("<enter/l>")
                                    .bold()
                                    .with_foreground(Color::Cyan)
                                    .build(),
                            )
                            .add_col(TextSpan::from("        Embed Lyrics"))
                            .add_row()
                            .add_col(
                                TextSpanBuilder::new("<s>")
                                    .bold()
                                    .with_foreground(Color::Cyan)
                                    .build(),
                            )
                            .add_col(TextSpan::from("              Download selected song"))
                            .build(),
                    )
                    .build(),
            )),
        );
        // Active help
        self.view.active(super::COMPONENT_TE_TEXT_HELP);
    }

    /// ### umount_help
    ///
    /// Umount help
    pub(super) fn umount_help(&mut self) {
        self.view.umount(super::COMPONENT_TE_TEXT_HELP);
    }
}
