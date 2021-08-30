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
use super::{MainActivity, COMPONENT_TREEVIEW};
use anyhow::{bail, Result};
use pinyin::ToPinyin;
use std::fs::{remove_dir_all, remove_file, rename};
use std::path::Path;
use tui_realm_treeview::{Node, Tree, TreeViewPropsBuilder};
use tuirealm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tuirealm::{Payload, PropsBuilder, Value};

impl MainActivity {
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
                let mut paths: Vec<_> = paths.filter_map(|r| r.ok()).collect();

                paths.sort_by_cached_key(|k| {
                    get_pin_yin(&k.file_name().to_string_lossy().to_string())
                });
                //     .sort_by(|a, b| {
                //     get_pin_yin(&a.file_name().to_string_lossy().to_string())
                //         .cmp(&get_pin_yin(&b.file_name().to_string_lossy().to_string()))
                // });
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
                let mut paths: Vec<_> = paths.filter_map(|r| r.ok()).collect();

                paths.sort_by_cached_key(|k| {
                    get_pin_yin(&k.file_name().to_string_lossy().to_string())
                });
                // paths.sort_by(|a, b| {
                //     get_pin_yin(&a.file_name().to_string_lossy().to_string())
                //         .cmp(&get_pin_yin(&b.file_name().to_string_lossy().to_string()))
                // });
                for p in paths {
                    if !p.path().is_dir() {
                        children.push(String::from(p.path().to_string_lossy()));
                    }
                }
            }
        }
        children
    }

    pub fn sync_playlist(&mut self, node: Option<&str>) {
        self.tree = Tree::new(Self::dir_tree(self.path.as_ref(), 3));

        if let Some(props) = self.view.get_props(COMPONENT_TREEVIEW) {
            let props = TreeViewPropsBuilder::from(props)
                .with_tree(self.tree.root())
                .with_title(
                    self.path.to_string_lossy(),
                    tuirealm::tui::layout::Alignment::Left,
                )
                .keep_state(true)
                .with_node(node)
                .build();

            let msg = self.view.update(COMPONENT_TREEVIEW, props);
            self.update(msg);
        }
    }

    pub fn delete_song(&mut self) -> Result<()> {
        if let Some(Payload::One(Value::Str(node_id))) = self.view.get_state(COMPONENT_TREEVIEW) {
            let p: &Path = Path::new(node_id.as_str());
            remove_file(p)?;
            // this is to keep the state of playlist
            let event: Event = Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
            });

            self.view.on(event);

            self.sync_playlist(None);
        }

        // this line remove the deleted songs from queue
        self.update_item_delete();
        Ok(())
    }

    pub fn delete_songs(&mut self) -> Result<()> {
        if let Some(Payload::One(Value::Str(node_id))) = self.view.get_state(COMPONENT_TREEVIEW) {
            let p: &Path = Path::new(node_id.as_str());
            p.canonicalize()?;
            remove_dir_all(p)?;

            // this is to keep the state of playlist
            let event: Event = Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
            });
            self.view.on(event);

            self.sync_playlist(None);
        }

        // this line remove the deleted songs from queue
        self.update_item_delete();
        Ok(())
    }

    pub fn yank(&mut self) {
        if let Some(Payload::One(Value::Str(node_id))) = self.view.get_state(COMPONENT_TREEVIEW) {
            self.yanked_node_id = Some(node_id);
        }
    }

    pub fn paste(&mut self) -> Result<()> {
        if let Some(Payload::One(Value::Str(new_id))) = self.view.get_state(COMPONENT_TREEVIEW) {
            match self.yanked_node_id.as_ref() {
                Some(old_id) => {
                    let p: &Path = Path::new(new_id.as_str());
                    let pold: &Path = Path::new(old_id.as_str());
                    if let Some(p_parent) = p.parent() {
                        if let Some(pold_filename) = pold.file_name() {
                            let mut new_node_id = p_parent.join(pold_filename);
                            if p.is_dir() {
                                new_node_id = p.join(pold_filename);
                            }
                            rename(pold, new_node_id.as_path())?;
                            self.sync_playlist(new_node_id.to_str());
                        }
                    }
                }
                None => bail!("No file yanked yet."),
            }
        }
        self.yanked_node_id = None;
        self.update_item_delete();
        Ok(())
    }
}

fn get_pin_yin(input: &str) -> String {
    let mut b = String::new();
    for (index, f) in input.to_pinyin().enumerate() {
        match f {
            Some(p) => {
                b.push_str(p.plain());
            }
            None => {
                if let Some(c) = input.to_uppercase().chars().nth(index) {
                    b.push(c);
                }
            }
        }
    }
    b
}

#[cfg(test)]
mod tests {

    use crate::ui::activity::main::playlist::get_pin_yin;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pin_yin() {
        assert_eq!(get_pin_yin("陈一发儿"), "chenyifaer".to_string());
        assert_eq!(get_pin_yin("Gala乐队"), "GALAledui".to_string());
        assert_eq!(get_pin_yin("乐队Gala乐队"), "leduiGALAledui".to_string());
        assert_eq!(get_pin_yin("Annett Louisan"), "ANNETT LOUISAN".to_string());
    }
}
