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
// locals
use super::TagEditorActivity;
use crate::lyric;
use crate::ui::keymap::*;
// use lrc::{Lyrics, TimeTag};
use anyhow::Error;
use tuirealm::{Msg, Payload, Value};

impl TagEditorActivity {
    /// ### update
    ///
    /// Update auth activity model based on msg
    /// The function exits when returns None
    pub(super) fn update(&mut self, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
        let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
        match ref_msg {
            None => None, // Exit after None
            Some(msg) => match msg {
                (super::COMPONENT_TE_RADIO_TAG, &MSG_KEY_TAB) => {
                    self.view.active(super::COMPONENT_TE_INPUT_ARTIST);
                    None
                }
                (super::COMPONENT_TE_INPUT_ARTIST, &MSG_KEY_TAB) => {
                    self.view.active(super::COMPONENT_TE_INPUT_SONGNAME);
                    None
                }
                (super::COMPONENT_TE_INPUT_SONGNAME, &MSG_KEY_TAB) => {
                    self.view.active(super::COMPONENT_TE_INPUT_ALBUM);
                    None
                }
                (super::COMPONENT_TE_INPUT_ALBUM, &MSG_KEY_TAB) => {
                    self.view.active(super::COMPONENT_TE_CHECKBOX_LANG);
                    None
                }

                (super::COMPONENT_TE_CHECKBOX_LANG, &MSG_KEY_TAB) => {
                    self.view.active(super::COMPONENT_TE_SCROLLTABLE_OPTIONS);
                    None
                }
                (super::COMPONENT_TE_SCROLLTABLE_OPTIONS, &MSG_KEY_TAB) => {
                    self.view.active(super::COMPONENT_TE_TEXTAREA_LYRIC);
                    None
                }
                (super::COMPONENT_TE_TEXTAREA_LYRIC, &MSG_KEY_TAB) => {
                    self.view.active(super::COMPONENT_TE_RADIO_TAG);
                    None
                }
                (
                    super::COMPONENT_TE_RADIO_TAG,
                    Msg::OnSubmit(Payload::One(Value::Usize(choice))),
                ) => {
                    match *choice {
                        0 => {
                            // Get Tag
                            let song = self.song.as_ref().unwrap();
                            match lyric::lyric_options(&song.name) {
                                Ok(l) => self.add_lyric_options(l),
                                Err(e) => self.mount_error(&e.to_string()),
                            };
                        }
                        1 => {
                            // Save Tag
                        }
                        _ => {}
                    }
                    // let props = TreeViewPropsBuilder::from(
                    //     self.view.get_props(COMPONENT_TREEVIEW).unwrap(),
                    // )
                    // .with_tree(self.tree.root())
                    // .with_title(Some(String::from(self.path.to_string_lossy())))
                    // .build();
                    // let msg = self.view.update(COMPONENT_TREEVIEW, props);
                    // self.update(msg)
                    None
                }

                // (COMPONENT_SCROLLTABLE, &MSG_KEY_TAB) => {
                //     self.view.active(COMPONENT_TREEVIEW);
                //     None
                // }
                // (COMPONENT_TREEVIEW, Msg::OnChange(Payload::One(Value::Str(node_id)))) => {
                //     // Update span
                //     let props = label::LabelPropsBuilder::from(
                //         self.view.get_props(COMPONENT_LABEL_HELP).unwrap(),
                //     )
                //     .with_text(format!("Selected: '{}'", node_id))
                //     .build();
                //     // Report submit
                //     let msg = self.view.update(COMPONENT_LABEL_HELP, props);
                //     self.update(msg)
                // }
                // // -- error
                (super::COMPONENT_TE_TEXT_ERROR, &MSG_KEY_ESC)
                | (super::COMPONENT_TE_TEXT_ERROR, &MSG_KEY_ENTER) => {
                    self.umount_error();
                    None
                }

                (_, &MSG_KEY_ESC) | (_, &MSG_KEY_CHAR_CAPITAL_Q) => {
                    // Quit on esc
                    self.exit_reason = Some(super::ExitReason::Quit);
                    None
                }
                _ => None,
            },
        }
    }
}
