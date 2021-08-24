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

// use spinners::{Spinner, Spinners};
use anyhow::{bail, Result};
use std::fs::{remove_dir_all, remove_file, rename};
use std::path::{Path, PathBuf};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use tui_realm_treeview::{Node, Tree, TreeViewPropsBuilder};
use tuirealm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tuirealm::{Payload, PropsBuilder, Value};
use ytd_rs::{Arg, ResultType, YoutubeDL};

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
            if let Ok(e) = std::fs::read_dir(p) {
                e.flatten()
                    .for_each(|x| node.add_child(Self::dir_tree(x.path().as_path(), depth - 1)));
            }
        }
        node
    }

    pub fn dir_children(p: &Path) -> Vec<String> {
        let mut children: Vec<String> = vec![];
        if p.is_dir() {
            if let Ok(e) = std::fs::read_dir(p) {
                e.flatten().for_each(|x| {
                    if x.path().is_dir() {
                    } else {
                        children.push(String::from(x.path().to_string_lossy()));
                    }
                });
            }
        }
        children
    }

    pub fn refresh_playlist(&mut self, node: Option<&str>) {
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

    pub fn youtube_dl(&mut self, link: &str) {
        let mut path: PathBuf = PathBuf::new();
        if let Some(Payload::One(Value::Str(node_id))) = self.view.get_state(COMPONENT_TREEVIEW) {
            let p: &Path = Path::new(node_id.as_str());
            if p.is_dir() {
                path = PathBuf::from(p);
            } else if let Some(p) = p.parent() {
                path = p.to_path_buf();
            }
        }

        let args = vec![
            // Arg::new("--quiet"),
            Arg::new("--extract-audio"),
            Arg::new_with_arg("--audio-format", "mp3"),
            Arg::new("--add-metadata"),
            Arg::new("--embed-thumbnail"),
            Arg::new_with_arg("--metadata-from-title", "%(artist) - %(title)s"),
            Arg::new("--write-sub"),
            Arg::new("--all-subs"),
            Arg::new_with_arg("--convert-subs", "lrc"),
            Arg::new_with_arg("--output", "%(title).90s.%(ext)s"),
        ];
        if let Ok(ytd) = YoutubeDL::new(&path, args, link) {
            let tx = self.sender.clone();

            thread::spawn(move || {
                if tx.send(super::TransferState::Running).is_ok() {}
                // start download
                let download = ytd.download();

                // check what the result is and print out the path to the download or the error
                match download.result_type() {
                    ResultType::SUCCESS => {
                        let _ = tx.send(super::TransferState::Success);
                        sleep(Duration::from_secs(5));
                        let _ = tx.send(super::TransferState::Completed(None));
                    }
                    ResultType::IOERROR | ResultType::FAILURE => {
                        let _ = tx.send(super::TransferState::ErrDownload);
                        sleep(Duration::from_secs(5));
                        let _ = tx.send(super::TransferState::Completed(None));
                    }
                }
            });
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

            self.refresh_playlist(None);
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

            self.refresh_playlist(None);
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
        if let Some(Payload::One(Value::Str(node_id))) = self.view.get_state(COMPONENT_TREEVIEW) {
            match self.yanked_node_id.as_ref() {
                Some(id) => {
                    let p: &Path = Path::new(node_id.as_str());
                    let pold: &Path = Path::new(id.as_str());
                    if let Some(p_parent) = p.parent() {
                        if let Some(pold_filename) = pold.file_name() {
                            let mut new_node_id = p_parent.join(pold_filename);
                            if p.is_dir() {
                                new_node_id = p.join(pold_filename);
                            }
                            rename(pold, new_node_id)?;
                        }
                    }
                }
                None => bail!("No file yanked yet."),
            }
        }
        self.yanked_node_id = None;
        self.refresh_playlist(None);
        self.update_item_delete();
        Ok(())
    }
}
