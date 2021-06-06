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
use super::{
    MainActivity, COMPONENT_INPUT, COMPONENT_LABEL, COMPONENT_SCROLLTABLE, COMPONENT_TREEVIEW,
};
use crate::ui::keymap::*;
// ext
use std::path::PathBuf;
use tuirealm::components::label;
use tuirealm::PropsBuilder;
use tuirealm::{Msg, Payload, Value};

use tui_realm_treeview::TreeViewPropsBuilder;

impl MainActivity {
    /// ### update
    ///
    /// Update auth activity model based on msg
    /// The function exits when returns None
    pub(super) fn update(&mut self, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
        let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
        match ref_msg {
            None => None, // Exit after None
            Some(msg) => match msg {
                (COMPONENT_INPUT, Msg::OnChange(Payload::One(Value::Str(input)))) => {
                    // Update span
                    let props = label::LabelPropsBuilder::from(
                        self.view.get_props(COMPONENT_LABEL).unwrap(),
                    )
                    .with_text(format!("You typed: '{}'", input))
                    .build();
                    // Report submit
                    let msg = self.view.update(COMPONENT_LABEL, props);
                    self.update(msg)
                }
                (COMPONENT_INPUT, &MSG_KEY_TAB) => {
                    self.view.active(COMPONENT_SCROLLTABLE);
                    None
                }
                (COMPONENT_SCROLLTABLE, &MSG_KEY_TAB) => {
                    self.view.active(COMPONENT_INPUT);
                    None
                }
                (COMPONENT_TREEVIEW, Msg::OnChange(Payload::One(Value::Str(node_id)))) => {
                    // Update span
                    let props = label::LabelPropsBuilder::from(
                        self.view.get_props(COMPONENT_LABEL).unwrap(),
                    )
                    .with_text(format!("Selected: '{}'", node_id))
                    .build();
                    // Report submit
                    let msg = self.view.update(COMPONENT_LABEL, props);
                    self.update(msg)
                }
                (COMPONENT_TREEVIEW, Msg::OnSubmit(Payload::One(Value::Str(node_id)))) => {
                    // Update tree
                    self.scan_dir(PathBuf::from(node_id.as_str()).as_path());
                    // Update
                    let props = TreeViewPropsBuilder::from(
                        self.view.get_props(COMPONENT_TREEVIEW).unwrap(),
                    )
                    .with_tree(self.tree.root())
                    .with_title(Some(String::from(self.path.to_string_lossy())))
                    .build();
                    let msg = self.view.update(COMPONENT_TREEVIEW, props);
                    self.update(msg)
                }
                (COMPONENT_TREEVIEW, &MSG_KEY_BACKSPACE) => {
                    // Update tree
                    match self.upper_dir() {
                        None => None,
                        Some(p) => {
                            let p: PathBuf = p.to_path_buf();
                            self.scan_dir(p.as_path());
                            // Update
                            let props = TreeViewPropsBuilder::from(
                                self.view.get_props(COMPONENT_TREEVIEW).unwrap(),
                            )
                            .with_tree(self.tree.root())
                            .with_title(Some(String::from(self.path.to_string_lossy())))
                            .build();
                            let msg = self.view.update(COMPONENT_TREEVIEW, props);
                            self.update(msg)
                        }
                    }
                }
                (_, &MSG_KEY_ESC) => {
                    // Quit on esc
                    self.exit_reason = Some(super::ExitReason::Quit);
                    None
                }
                _ => None,
            },
        }
    }
}
