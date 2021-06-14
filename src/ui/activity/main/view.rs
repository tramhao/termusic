//! ## SetupActivity
//!
//! `setup_activity` is the module which implements the Setup activity, which is the activity to
//! work on termscp configuration

use super::scrolltable;
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
    Context, MainActivity, COMPONENT_LABEL_HELP, COMPONENT_PARAGRAPH_LYRIC, COMPONENT_PROGRESS,
    COMPONENT_SCROLLTABLE, COMPONENT_TREEVIEW,
};
// Ext
use tuirealm::components::{label, paragraph, progress_bar};
use tuirealm::props::borders::{BorderType, Borders};
use tuirealm::props::{TableBuilder, TextSpan, TextSpanBuilder};
use tuirealm::{PropsBuilder, View};
// tui
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Color;
use tui_realm_treeview::{TreeView, TreeViewPropsBuilder};

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
                    .with_progbar_color(Color::LightCyan)
                    .with_texts(Some(String::from("Playing")), String::from("Song Name"))
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
                    .with_borders(Borders::ALL,BorderType::Rounded,Color::Blue)
                    .with_texts(Some(String::from("Lyrics")),vec![
                TextSpanBuilder::new("Lorem ipsum dolor sit amet").underlined().with_foreground(Color::Green).build(),
                TextSpan::from(", consectetur adipiscing elit. Praesent mauris est, vehicula et imperdiet sed, tincidunt sed est. Sed sed dui odio. Etiam nunc neque, sodales ut ex nec, tincidunt malesuada eros. Sed quis eros non felis sodales accumsan in ac risus"),
                TextSpan::from("Duis augue diam, tempor vitae posuere et, tempus mattis ligula.")
            ])

                    .build(),
            )),
        );

        // Scrolltable
        self.view.mount(
            COMPONENT_SCROLLTABLE,
            Box::new(scrolltable::Scrolltable::new(
                scrolltable::ScrollTablePropsBuilder::default()
                    // .with_background(Color::Black)
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

        self.view.mount(
            COMPONENT_TREEVIEW,
            Box::new(TreeView::new(
                TreeViewPropsBuilder::default()
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightYellow)
                    .with_foreground(Color::LightYellow)
                    .with_background(Color::Black)
                    .with_title(Some(String::from("Playlist")))
                    .with_tree_and_depth(self.tree.root(), 3)
                    .with_highlighted_str("🚀")
                    .build(),
            )),
        );

        // self.view.mount(
        //     COMPONENT_SCROLLTABLE,
        //     Box::new(TreeView::new(
        //         TreeViewPropsBuilder::default()
        //             .with_borders(Borders::ALL, BorderType::Rounded, Color::LightYellow)
        //             .with_foreground(Color::LightYellow)
        //             .with_background(Color::Black)
        //             .with_title(Some(String::from("Playlist")))
        //             .with_tree_and_depth(self.tree.root(), 3)
        //             .with_highlighted_str("🚀")
        //             .build(),
        //     )),
        // );

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
        });
        self.context = Some(ctx);
    }

    // -- mount

    // ### mount_error
    //
    // Mount error box
    // pub(super) fn mount_error(&mut self, text: &str) {
    //     // Mount
    //     self.view.mount(
    //         super::COMPONENT_TEXT_ERROR,
    //         Box::new(MsgBox::new(
    //             MsgBoxPropsBuilder::default()
    //                 .with_foreground(Color::Red)
    //                 .bold()
    //                 .with_borders(Borders::ALL, BorderType::Rounded, Color::Red)
    //                 .with_texts(None, vec![TextSpan::from(text)])
    //                 .build(),
    //         )),
    //     );
    //     // Give focus to error
    //     self.view.active(super::COMPONENT_TEXT_ERROR);
    // }

    // /// ### umount_error
    // ///
    // /// Umount error message
    // pub(super) fn umount_error(&mut self) {
    //     self.view.umount(super::COMPONENT_TEXT_ERROR);
    // }

    // /// ### mount_del_ssh_key
    // ///
    // /// Mount delete ssh key component
    // pub(super) fn mount_del_ssh_key(&mut self) {
    //     self.view.mount(
    //         super::COMPONENT_RADIO_DEL_SSH_KEY,
    //         Box::new(Radio::new(
    //             RadioPropsBuilder::default()
    //                 .with_color(Color::LightRed)
    //                 .with_inverted_color(Color::Black)
    //                 .with_borders(Borders::ALL, BorderType::Rounded, Color::LightRed)
    //                 .with_options(
    //                     Some(String::from("Delete key?")),
    //                     vec![String::from("Yes"), String::from("No")],
    //                 )
    //                 .with_value(1) // Default: No
    //                 .build(),
    //         )),
    //     );
    //     // Active
    //     self.view.active(super::COMPONENT_RADIO_DEL_SSH_KEY);
    // }

    // /// ### umount_del_ssh_key
    // ///
    // /// Umount delete ssh key
    // pub(super) fn umount_del_ssh_key(&mut self) {
    //     self.view.umount(super::COMPONENT_RADIO_DEL_SSH_KEY);
    // }

    // /// ### mount_new_ssh_key
    // ///
    // /// Mount new ssh key prompt
    // pub(super) fn mount_new_ssh_key(&mut self) {
    //     self.view.mount(
    //         super::COMPONENT_INPUT_SSH_HOST,
    //         Box::new(Input::new(
    //             InputPropsBuilder::default()
    //                 .with_label(String::from("Hostname or address"))
    //                 .with_borders(
    //                     Borders::TOP | Borders::RIGHT | Borders::LEFT,
    //                     BorderType::Plain,
    //                     Color::Reset,
    //                 )
    //                 .build(),
    //         )),
    //     );
    //     self.view.mount(
    //         super::COMPONENT_INPUT_SSH_USERNAME,
    //         Box::new(Input::new(
    //             InputPropsBuilder::default()
    //                 .with_label(String::from("Username"))
    //                 .with_borders(
    //                     Borders::BOTTOM | Borders::RIGHT | Borders::LEFT,
    //                     BorderType::Plain,
    //                     Color::Reset,
    //                 )
    //                 .build(),
    //         )),
    //     );
    //     self.view.active(super::COMPONENT_INPUT_SSH_HOST);
    // }

    // /// ### umount_new_ssh_key
    // ///
    // /// Umount new ssh key prompt
    // pub(super) fn umount_new_ssh_key(&mut self) {
    //     self.view.umount(super::COMPONENT_INPUT_SSH_HOST);
    //     self.view.umount(super::COMPONENT_INPUT_SSH_USERNAME);
    // }

    // /// ### mount_quit
    // ///
    // /// Mount quit popup
    // pub(super) fn mount_quit(&mut self) {
    //     self.view.mount(
    //         super::COMPONENT_RADIO_QUIT,
    //         Box::new(Radio::new(
    //             RadioPropsBuilder::default()
    //                 .with_color(Color::LightRed)
    //                 .with_inverted_color(Color::Black)
    //                 .with_borders(Borders::ALL, BorderType::Rounded, Color::LightRed)
    //                 .with_options(
    //                     Some(String::from("Exit setup?")),
    //                     vec![
    //                         String::from("Save"),
    //                         String::from("Don't save"),
    //                         String::from("Cancel"),
    //                     ],
    //                 )
    //                 .build(),
    //         )),
    //     );
    //     // Active
    //     self.view.active(super::COMPONENT_RADIO_QUIT);
    // }

    // /// ### umount_quit
    // ///
    // /// Umount quit
    // pub(super) fn umount_quit(&mut self) {
    //     self.view.umount(super::COMPONENT_RADIO_QUIT);
    // }

    // /// ### mount_save_popup
    // ///
    // /// Mount save popup
    // pub(super) fn mount_save_popup(&mut self) {
    //     self.view.mount(
    //         super::COMPONENT_RADIO_SAVE,
    //         Box::new(Radio::new(
    //             RadioPropsBuilder::default()
    //                 .with_color(Color::LightYellow)
    //                 .with_inverted_color(Color::Black)
    //                 .with_borders(Borders::ALL, BorderType::Rounded, Color::LightYellow)
    //                 .with_options(
    //                     Some(String::from("Save changes?")),
    //                     vec![String::from("Yes"), String::from("No")],
    //                 )
    //                 .build(),
    //         )),
    //     );
    //     // Active
    //     self.view.active(super::COMPONENT_RADIO_SAVE);
    // }

    // /// ### umount_quit
    // ///
    // /// Umount quit
    // pub(super) fn umount_save_popup(&mut self) {
    //     self.view.umount(super::COMPONENT_RADIO_SAVE);
    // }

    // /// ### mount_help
    // ///
    // /// Mount help
    // pub(super) fn mount_help(&mut self) {
    //     self.view.mount(
    //         super::COMPONENT_TEXT_HELP,
    //         Box::new(Table::new(
    //             TablePropsBuilder::default()
    //                 .with_borders(Borders::ALL, BorderType::Rounded, Color::White)
    //                 .with_table(
    //                     Some(String::from("Help")),
    //                     TableBuilder::default()
    //                         .add_col(
    //                             TextSpanBuilder::new("<ESC>")
    //                                 .bold()
    //                                 .with_foreground(Color::Cyan)
    //                                 .build(),
    //                         )
    //                         .add_col(TextSpan::from("           Exit setup"))
    //                         .add_row()
    //                         .add_col(
    //                             TextSpanBuilder::new("<TAB>")
    //                                 .bold()
    //                                 .with_foreground(Color::Cyan)
    //                                 .build(),
    //                         )
    //                         .add_col(TextSpan::from("           Change setup page"))
    //                         .add_row()
    //                         .add_col(
    //                             TextSpanBuilder::new("<RIGHT/LEFT>")
    //                                 .bold()
    //                                 .with_foreground(Color::Cyan)
    //                                 .build(),
    //                         )
    //                         .add_col(TextSpan::from("    Change cursor"))
    //                         .add_row()
    //                         .add_col(
    //                             TextSpanBuilder::new("<UP/DOWN>")
    //                                 .bold()
    //                                 .with_foreground(Color::Cyan)
    //                                 .build(),
    //                         )
    //                         .add_col(TextSpan::from("       Change input field"))
    //                         .add_row()
    //                         .add_col(
    //                             TextSpanBuilder::new("<ENTER>")
    //                                 .bold()
    //                                 .with_foreground(Color::Cyan)
    //                                 .build(),
    //                         )
    //                         .add_col(TextSpan::from("         Select / Dismiss popup"))
    //                         .add_row()
    //                         .add_col(
    //                             TextSpanBuilder::new("<DEL|E>")
    //                                 .bold()
    //                                 .with_foreground(Color::Cyan)
    //                                 .build(),
    //                         )
    //                         .add_col(TextSpan::from("         Delete SSH key"))
    //                         .add_row()
    //                         .add_col(
    //                             TextSpanBuilder::new("<CTRL+N>")
    //                                 .bold()
    //                                 .with_foreground(Color::Cyan)
    //                                 .build(),
    //                         )
    //                         .add_col(TextSpan::from("        New SSH key"))
    //                         .add_row()
    //                         .add_col(
    //                             TextSpanBuilder::new("<CTRL+R>")
    //                                 .bold()
    //                                 .with_foreground(Color::Cyan)
    //                                 .build(),
    //                         )
    //                         .add_col(TextSpan::from("        Revert changes"))
    //                         .add_row()
    //                         .add_col(
    //                             TextSpanBuilder::new("<CTRL+S>")
    //                                 .bold()
    //                                 .with_foreground(Color::Cyan)
    //                                 .build(),
    //                         )
    //                         .add_col(TextSpan::from("        Save configuration"))
    //                         .build(),
    //                 )
    //                 .build(),
    //         )),
    //     );
    //     // Active help
    //     self.view.active(super::COMPONENT_TEXT_HELP);
    // }

    // /// ### umount_help
    // ///
    // /// Umount help
    // pub(super) fn umount_help(&mut self) {
    //     self.view.umount(super::COMPONENT_TEXT_HELP);
    // }

    // /// ### load_input_values
    // ///
    // /// Load values from configuration into input fields
    // pub(super) fn load_input_values(&mut self) {
    //     if let Some(cli) = self.context.as_mut().unwrap().config_client.as_mut() {
    //         // Text editor
    //         if let Some(props) = self.view.get_props(super::COMPONENT_INPUT_TEXT_EDITOR) {
    //             let text_editor: String =
    //                 String::from(cli.get_text_editor().as_path().to_string_lossy());
    //             let props = InputPropsBuilder::from(props)
    //                 .with_value(text_editor)
    //                 .build();
    //             let _ = self.view.update(super::COMPONENT_INPUT_TEXT_EDITOR, props);
    //         }
    //         // Protocol
    //         if let Some(props) = self.view.get_props(super::COMPONENT_RADIO_DEFAULT_PROTOCOL) {
    //             let protocol: usize = match cli.get_default_protocol() {
    //                 FileTransferProtocol::Sftp => 0,
    //                 FileTransferProtocol::Scp => 1,
    //                 FileTransferProtocol::Ftp(false) => 2,
    //                 FileTransferProtocol::Ftp(true) => 3,
    //             };
    //             let props = RadioPropsBuilder::from(props).with_value(protocol).build();
    //             let _ = self
    //                 .view
    //                 .update(super::COMPONENT_RADIO_DEFAULT_PROTOCOL, props);
    //         }
    //         // Hidden files
    //         if let Some(props) = self.view.get_props(super::COMPONENT_RADIO_HIDDEN_FILES) {
    //             let hidden: usize = match cli.get_show_hidden_files() {
    //                 true => 0,
    //                 false => 1,
    //             };
    //             let props = RadioPropsBuilder::from(props).with_value(hidden).build();
    //             let _ = self.view.update(super::COMPONENT_RADIO_HIDDEN_FILES, props);
    //         }
    //         // Updates
    //         if let Some(props) = self.view.get_props(super::COMPONENT_RADIO_UPDATES) {
    //             let updates: usize = match cli.get_check_for_updates() {
    //                 true => 0,
    //                 false => 1,
    //             };
    //             let props = RadioPropsBuilder::from(props).with_value(updates).build();
    //             let _ = self.view.update(super::COMPONENT_RADIO_UPDATES, props);
    //         }
    //         // Group dirs
    //         if let Some(props) = self.view.get_props(super::COMPONENT_RADIO_GROUP_DIRS) {
    //             let dirs: usize = match cli.get_group_dirs() {
    //                 Some(GroupDirs::First) => 0,
    //                 Some(GroupDirs::Last) => 1,
    //                 None => 2,
    //             };
    //             let props = RadioPropsBuilder::from(props).with_value(dirs).build();
    //             let _ = self.view.update(super::COMPONENT_RADIO_GROUP_DIRS, props);
    //         }
    //         // Local File Fmt
    //         if let Some(props) = self.view.get_props(super::COMPONENT_INPUT_LOCAL_FILE_FMT) {
    //             let file_fmt: String = cli.get_local_file_fmt().unwrap_or_default();
    //             let props = InputPropsBuilder::from(props).with_value(file_fmt).build();
    //             let _ = self
    //                 .view
    //                 .update(super::COMPONENT_INPUT_LOCAL_FILE_FMT, props);
    //         }
    //         // Remote File Fmt
    //         if let Some(props) = self.view.get_props(super::COMPONENT_INPUT_REMOTE_FILE_FMT) {
    //             let file_fmt: String = cli.get_remote_file_fmt().unwrap_or_default();
    //             let props = InputPropsBuilder::from(props).with_value(file_fmt).build();
    //             let _ = self
    //                 .view
    //                 .update(super::COMPONENT_INPUT_REMOTE_FILE_FMT, props);
    //         }
    //     }
    // }

    // /// ### collect_input_values
    // ///
    // /// Collect values from input and put them into the configuration
    // pub(super) fn collect_input_values(&mut self) {
    //     if let Some(cli) = self.context.as_mut().unwrap().config_client.as_mut() {
    //         if let Some(Payload::One(Value::Str(editor))) =
    //             self.view.get_state(super::COMPONENT_INPUT_TEXT_EDITOR)
    //         {
    //             cli.set_text_editor(PathBuf::from(editor.as_str()));
    //         }
    //         if let Some(Payload::One(Value::Usize(protocol))) =
    //             self.view.get_state(super::COMPONENT_RADIO_DEFAULT_PROTOCOL)
    //         {
    //             let protocol: FileTransferProtocol = match protocol {
    //                 1 => FileTransferProtocol::Scp,
    //                 2 => FileTransferProtocol::Ftp(false),
    //                 3 => FileTransferProtocol::Ftp(true),
    //                 _ => FileTransferProtocol::Sftp,
    //             };
    //             cli.set_default_protocol(protocol);
    //         }
    //         if let Some(Payload::One(Value::Usize(opt))) =
    //             self.view.get_state(super::COMPONENT_RADIO_HIDDEN_FILES)
    //         {
    //             let show: bool = matches!(opt, 0);
    //             cli.set_show_hidden_files(show);
    //         }
    //         if let Some(Payload::One(Value::Usize(opt))) =
    //             self.view.get_state(super::COMPONENT_RADIO_UPDATES)
    //         {
    //             let check: bool = matches!(opt, 0);
    //             cli.set_check_for_updates(check);
    //         }
    //         if let Some(Payload::One(Value::Str(fmt))) =
    //             self.view.get_state(super::COMPONENT_INPUT_LOCAL_FILE_FMT)
    //         {
    //             cli.set_local_file_fmt(fmt);
    //         }
    //         if let Some(Payload::One(Value::Str(fmt))) =
    //             self.view.get_state(super::COMPONENT_INPUT_REMOTE_FILE_FMT)
    //         {
    //             cli.set_remote_file_fmt(fmt);
    //         }
    //         if let Some(Payload::One(Value::Usize(opt))) =
    //             self.view.get_state(super::COMPONENT_RADIO_GROUP_DIRS)
    //         {
    //             let dirs: Option<GroupDirs> = match opt {
    //                 0 => Some(GroupDirs::First),
    //                 1 => Some(GroupDirs::Last),
    //                 _ => None,
    //             };
    //             cli.set_group_dirs(dirs);
    //         }
    //     }
    // }

    // /// ### reload_ssh_keys
    // ///
    // /// Reload ssh keys
    // pub(super) fn reload_ssh_keys(&mut self) {
    //     if let Some(cli) = self.context.as_ref().unwrap().config_client.as_ref() {
    //         // get props
    //         if let Some(props) = self.view.get_props(super::COMPONENT_LIST_SSH_KEYS) {
    //             // Create texts
    //             let keys: Vec<String> = cli
    //                 .iter_ssh_keys()
    //                 .map(|x| {
    //                     let (addr, username, _) = cli.get_ssh_key(x).ok().unwrap().unwrap();
    //                     format!("{} at {}", addr, username)
    //                 })
    //                 .collect();
    //             let props = BookmarkListPropsBuilder::from(props)
    //                 .with_bookmarks(Some(String::from("SSH Keys")), keys)
    //                 .build();
    //             self.view.update(super::COMPONENT_LIST_SSH_KEYS, props);
    //         }
    //     }
    // }
}
