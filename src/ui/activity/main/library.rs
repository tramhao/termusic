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
use super::{TermusicActivity, COMPONENT_TABLE_SEARCH_LIBRARY, COMPONENT_TREEVIEW_LIBRARY};
use crate::song::Song;
use crate::utils::get_pin_yin;
use anyhow::{bail, Result};
use if_chain::if_chain;
use std::fs::{remove_dir_all, remove_file, rename};
use std::path::Path;
use std::str::FromStr;
use tui_realm_stdlib::TablePropsBuilder;
use tui_realm_treeview::{Node, Tree, TreeViewPropsBuilder};
use tuirealm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tuirealm::props::{TableBuilder, TextSpan};
use tuirealm::{Payload, PropPayload, PropValue, PropsBuilder, Value};

impl TermusicActivity {
    pub fn scan_dir(&mut self, p: &Path) {
        self.path = p.to_path_buf();
        self.tree = Tree::new(Self::dir_tree(p, 3));
    }

    pub fn upper_dir(&self) -> Option<&Path> {
        self.path.parent()
    }

    pub fn dir_tree(p: &Path, depth: usize) -> Node {
        let name: String = match p.file_name() {
            None => "/".to_string(),
            Some(n) => n.to_string_lossy().into_owned(),
        };
        let mut node: Node = Node::new(p.to_string_lossy().into_owned(), name);
        if depth > 0 && p.is_dir() {
            if let Ok(paths) = std::fs::read_dir(p) {
                let mut paths: Vec<_> = paths.filter_map(std::result::Result::ok).collect();

                paths.sort_by_cached_key(|k| get_pin_yin(&k.file_name().to_string_lossy().to_string()));
                for p in paths {
                    node.add_child(Self::dir_tree(p.path().as_path(), depth - 1));
                }
            }
        }
        node
    }

    pub fn dir_children(p: &Path) -> Vec<String> {
        let mut children: Vec<String> = vec![];
        if p.is_dir() {
            if let Ok(paths) = std::fs::read_dir(p) {
                let mut paths: Vec<_> = paths.filter_map(std::result::Result::ok).collect();

                paths.sort_by_cached_key(|k| get_pin_yin(&k.file_name().to_string_lossy().to_string()));
                for p in paths {
                    if !p.path().is_dir() {
                        children.push(String::from(p.path().to_string_lossy()));
                    }
                }
            }
        }
        children
    }

    pub fn sync_library(&mut self, node: Option<&str>) {
        self.tree = Tree::new(Self::dir_tree(self.path.as_ref(), 3));

        if let Some(props) = self.view.get_props(COMPONENT_TREEVIEW_LIBRARY) {
            let props = TreeViewPropsBuilder::from(props)
                .with_tree(self.tree.root())
                .with_title(self.path.to_string_lossy(), tuirealm::tui::layout::Alignment::Left)
                .keep_state(true)
                .with_node(node)
                .build();

            let msg = self.view.update(COMPONENT_TREEVIEW_LIBRARY, props);
            self.update(&msg);
        }
    }

    pub fn delete_song(&mut self) -> Result<()> {
        if let Some(Payload::One(Value::Str(node_id))) = self.view.get_state(COMPONENT_TREEVIEW_LIBRARY) {
            let p: &Path = Path::new(node_id.as_str());
            remove_file(p)?;
            // this is to keep the state of playlist
            let event: Event = Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
            });

            self.view.on(event);

            self.sync_library(None);
        }

        // this line remove the deleted songs from playlist
        self.update_item_delete();
        Ok(())
    }

    pub fn delete_songs(&mut self) -> Result<()> {
        if let Some(Payload::One(Value::Str(node_id))) = self.view.get_state(COMPONENT_TREEVIEW_LIBRARY) {
            let p: &Path = Path::new(node_id.as_str());
            p.canonicalize()?;
            remove_dir_all(p)?;

            // this is to keep the state of playlist
            let event: Event = Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
            });
            self.view.on(event);

            self.sync_library(None);
        }

        // this line remove the deleted songs from playlist
        self.update_item_delete();
        Ok(())
    }

    pub fn yank(&mut self) {
        if let Some(Payload::One(Value::Str(node_id))) = self.view.get_state(COMPONENT_TREEVIEW_LIBRARY) {
            self.yanked_node_id = Some(node_id);
        }
    }

    pub fn paste(&mut self) -> Result<()> {
        if_chain! {
            if let Some(Payload::One(Value::Str(new_id))) = self.view.get_state(COMPONENT_TREEVIEW_LIBRARY);
            if let Some(old_id) = self.yanked_node_id.as_ref();
            let p: &Path = Path::new(new_id.as_str());
            let pold: &Path = Path::new(old_id.as_str());
            if let Some(p_parent) = p.parent();
            if let Some(pold_filename) = pold.file_name();
            let new_node_id = if p.is_dir() {
                    p.join(pold_filename)
                } else {
                    p_parent.join(pold_filename)
                };
            then {
                rename(pold, new_node_id.as_path())?;
                self.sync_library(new_node_id.to_str());
            } else {
                bail!("paste error. No file yanked?");
            }
        }
        self.yanked_node_id = None;
        self.update_item_delete();
        Ok(())
    }

    pub fn update_search_library(&mut self, input: &str) {
        let mut table: TableBuilder = TableBuilder::default();
        let root = self.tree.root();
        let p: &Path = Path::new(root.id());
        let all_items = walkdir::WalkDir::new(p).follow_links(true);
        let mut idx = 0;
        let mut search = "*".to_string();
        search.push_str(input);
        search.push('*');
        for record in all_items.into_iter().filter_map(std::result::Result::ok) {
            let file_name = record.path();
            if wildmatch::WildMatch::new(&search).matches(file_name.to_string_lossy().as_ref()) {
                if idx > 0 {
                    table.add_row();
                }
                idx += 1;
                table
                    .add_col(TextSpan::new(idx.to_string()))
                    .add_col(TextSpan::new(file_name.to_string_lossy()));
            }
        }

        let table = table.build();
        if let Some(props) = self.view.get_props(COMPONENT_TABLE_SEARCH_LIBRARY) {
            let props = TablePropsBuilder::from(props).with_table(table).build();
            let msg = self.view.update(COMPONENT_TABLE_SEARCH_LIBRARY, props);
            self.update(&msg);
        }
    }

    pub fn select_after_search_library(&mut self, node_id: usize) {
        if_chain! {
            if let Some(props) = self.view.get_props(COMPONENT_TABLE_SEARCH_LIBRARY);
            if let Some(PropPayload::One(PropValue::Table(table))) = props.own.get("table");
            if let Some(line) = table.get(node_id);
            if let Some(text_span) = line.get(1);
            let text = text_span.content.clone();
            if let Some(props) = self.view.get_props(COMPONENT_TREEVIEW_LIBRARY);
            then {
                let props = TreeViewPropsBuilder::from(props)
                    .with_node(Some(&text))
                    .build();

                let msg = self.view.update(COMPONENT_TREEVIEW_LIBRARY, props);
                self.update(&msg);
            }
        }
    }

    pub fn add_playlist_after_search_library(&mut self, node_id: usize) {
        if_chain! {
            if let Some(props) = self.view.get_props(COMPONENT_TABLE_SEARCH_LIBRARY);
            if let Some(PropPayload::One(PropValue::Table(table))) = props.own.get("table");
            if let Some(line) = table.get(node_id);
            if let Some(text_span) = line.get(1);
            let text = text_span.content.clone();
            let p: &Path = Path::new(&text);
            if p.exists();
            then {
                if p.is_dir() {
                    let new_items = Self::dir_children(p);
                    for i in new_items.iter().rev() {
                        match Song::from_str(i) {
                            Ok(s) => self.add_playlist(s),
                            Err(e) => {
                                self.mount_error(
                                    format!("add playlist error: {}", e).as_str(),
                                );
                            }
                        };
                    }
                } else  {
                    match Song::from_str(&text) {
                        Ok(s) => self.add_playlist(s),
                        Err(e) => {
                            self.mount_error(format!("add playlist error: {}", e).as_str());
                        }
                    };
                }
            }
        }
    }
}
