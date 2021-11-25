use crate::ui::model::MAX_DEPTH;
use crate::ui::{Id, Model, Msg};
use crate::utils::get_pin_yin;
use anyhow::Result;
use std::fs::{remove_dir_all, remove_file, rename};
use std::path::{Path, PathBuf};
use tui_realm_treeview::{Node, Tree, TreeView, TREE_CMD_CLOSE, TREE_CMD_OPEN};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders};
use tuirealm::tui::style::{Color, Style};
use tuirealm::{Component, Event, MockComponent, NoUserEvent, State, StateValue};

#[derive(MockComponent)]
pub struct MusicLibrary {
    component: TreeView,
}

impl MusicLibrary {
    pub fn new(tree: &Tree, initial_node: Option<String>) -> Self {
        // Preserve initial node if exists
        let initial_node = match initial_node {
            Some(id) if tree.root().query(&id).is_some() => id,
            _ => tree.root().id().to_string(),
        };
        Self {
            component: TreeView::default()
                .foreground(Color::Reset)
                .borders(
                    Borders::default()
                        .color(Color::LightYellow)
                        .modifiers(BorderType::Rounded),
                )
                .inactive(Style::default().fg(Color::Gray))
                .indent_size(2)
                .scroll_step(6)
                .title("Library", Alignment::Left)
                .highlighted_color(Color::LightYellow)
                .highlight_symbol("\u{1f984}")
                // .highlight_symbol("🦄")
                .with_tree(tree.clone())
                .initial_node(initial_node),
        }
    }
}

impl Component<Msg, NoUserEvent> for MusicLibrary {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left | Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Custom(TREE_CMD_CLOSE)),
            Event::Keyboard(KeyEvent {
                code: Key::Right | Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            }) => {
                let current_node = self.component.tree_state().selected().unwrap();
                let p: &Path = Path::new(current_node);
                if p.is_dir() {
                    self.perform(Cmd::Custom(TREE_CMD_OPEN))
                } else {
                    return Some(Msg::PlaylistAdd(current_node.to_string()));
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::Down | Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up | Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::Home | Key::Char('g'),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent {
                code: Key::End | Key::Char('G'),
                modifiers: KeyModifiers::SHIFT,
            }) => self.perform(Cmd::GoTo(Position::End)),
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Submit),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::LibraryTreeGoToUpperDir),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::LibraryTreeBlur),
            Event::Keyboard(KeyEvent {
                code: Key::Char('d'),
                ..
            }) => return Some(Msg::DeleteConfirmShow),

            _ => return None,
        };
        match result {
            CmdResult::Submit(State::One(StateValue::String(node))) => {
                Some(Msg::LibraryTreeExtendDir(node))
            }
            _ => Some(Msg::None),
        }
    }
}

impl Model {
    pub fn scan_dir(&mut self, p: &Path) {
        self.path = p.to_path_buf();
        self.tree = Tree::new(Self::dir_tree(p, MAX_DEPTH));
    }

    pub fn upper_dir(&self) -> Option<PathBuf> {
        self.path.parent().map(std::path::Path::to_path_buf)
    }

    // pub fn extend_dir(&mut self, id: &str, p: &Path, depth: usize) {
    //     if let Some(node) = self.tree.root_mut().query_mut(&String::from(id)) {
    //         if depth > 0 && p.is_dir() {
    //             // Clear node
    //             node.clear();
    //             // Scan dir
    //             if let Ok(paths) = std::fs::read_dir(p) {
    //                 let mut paths: Vec<_> = paths.filter_map(std::result::Result::ok).collect();

    //                 paths.sort_by_cached_key(|k| {
    //                     get_pin_yin(&k.file_name().to_string_lossy().to_string())
    //                 });
    //                 for p in paths {
    //                     if !p.path().is_dir() {
    //                         node.add_child(Self::dir_tree(p.path().as_path(), depth - 1));
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    pub fn dir_tree(p: &Path, depth: usize) -> Node {
        let name: String = match p.file_name() {
            None => "/".to_string(),
            Some(n) => n.to_string_lossy().into_owned(),
        };
        let mut node: Node = Node::new(p.to_string_lossy().into_owned(), name);
        if depth > 0 && p.is_dir() {
            if let Ok(paths) = std::fs::read_dir(p) {
                let mut paths: Vec<_> = paths.filter_map(std::result::Result::ok).collect();

                paths.sort_by_cached_key(|k| {
                    get_pin_yin(&k.file_name().to_string_lossy().to_string())
                });
                for p in paths {
                    node.add_child(Self::dir_tree(p.path().as_path(), depth - 1));
                }
            }
        }
        node
    }
    pub fn reload_tree(&mut self) {
        self.tree = Tree::new(Self::dir_tree(self.path.as_ref(), MAX_DEPTH));
        let current_node = match self.app.state(&Id::Library).ok().unwrap() {
            State::One(StateValue::String(id)) => Some(id),
            _ => None,
        };
        // Remount tree
        assert!(self.app.umount(&Id::Library).is_ok());
        assert!(self
            .app
            .mount(
                Id::Library,
                Box::new(MusicLibrary::new(&self.tree.clone(), current_node)),
                Vec::new()
            )
            .is_ok());
        assert!(self.app.active(&Id::Library).is_ok());
    }

    pub fn library_stepinto(&mut self, node_id: &str) {
        self.scan_dir(PathBuf::from(node_id).as_path());
        self.config.music_dir = node_id.to_string();
        self.reload_tree();
    }

    pub fn library_stepout(&mut self) {
        if let Some(p) = self.upper_dir() {
            let p: PathBuf = p.to_path_buf();
            self.scan_dir(p.as_path());
            self.reload_tree();
        }
    }

    pub fn update_library_delete(&mut self) {
        if let Ok(State::One(StateValue::String(node_id))) = self.app.state(&Id::Library) {
            let p: &Path = Path::new(node_id.as_str());
            if p.is_file() {
                self.mount_confirm_radio();
            } else {
                self.mount_confirm_input();
            }
        }
    }

    pub fn library_delete_song(&mut self) -> Result<()> {
        if let Ok(State::One(StateValue::String(node_id))) = self.app.state(&Id::Library) {
            let p: &Path = Path::new(node_id.as_str());
            if p.is_file() {
                remove_file(p)?;
            } else {
                p.canonicalize()?;
                remove_dir_all(p)?;
            }
            // this is to keep the state of playlist
            // let event: Event = Event::Key(KeyEvent {
            //     code: KeyCode::Down,
            //     modifiers: KeyModifiers::NONE,
            // });
            // self.view.on(event);
            // let event: Event<NoUserEvent> = Event::Keyboard(KeyEvent {
            //     code: Key::Down,
            //     modifiers: KeyModifiers::NONE,
            // });
            self.reload_tree();
            // this line remove the deleted songs from playlist
            self.update_item_delete();
        }
        Ok(())
    }

    // pub fn update_delete_songs(&mut self) {
    //     if let Some(Payload::One(Value::Str(p))) = self.view.get_state(COMPONENT_CONFIRMATION_INPUT)
    //     {
    //         self.umount_confirmation_input();
    //         if p == "DELETE" {
    //             if let Err(e) = self.delete_songs() {
    //                 self.mount_error(format!("delete song error: {}", e).as_str());
    //             };
    //         }
    //     }
    // }

    // pub fn update_delete_song(&mut self) {
    //     if let Some(Payload::One(Value::Usize(index))) =
    //         self.view.get_state(COMPONENT_CONFIRMATION_RADIO)
    //     {
    //         self.umount_confirmation_radio();

    //         if index != 0 {
    //             return;
    //         }
    //         if let Err(e) = self.delete_song() {
    //             self.mount_error(format!("delete song error: {}", e).as_str());
    //         };
    //     }
    // }

    // pub fn yank(&mut self) {
    //     if let Some(Payload::One(Value::Str(node_id))) =
    //         self.view.get_state(COMPONENT_TREEVIEW_LIBRARY)
    //     {
    //         self.yanked_node_id = Some(node_id);
    //     }
    // }

    // pub fn paste(&mut self) -> Result<()> {
    //     if_chain! {
    //         if let Some(Payload::One(Value::Str(new_id))) = self.view.get_state(COMPONENT_TREEVIEW_LIBRARY);
    //         if let Some(old_id) = self.yanked_node_id.as_ref();
    //         let p: &Path = Path::new(new_id.as_str());
    //         let pold: &Path = Path::new(old_id.as_str());
    //         if let Some(p_parent) = p.parent();
    //         if let Some(pold_filename) = pold.file_name();
    //         let new_node_id = if p.is_dir() {
    //                 p.join(pold_filename)
    //             } else {
    //                 p_parent.join(pold_filename)
    //             };
    //         then {
    //             rename(pold, new_node_id.as_path())?;
    //             self.sync_library(new_node_id.to_str());
    //         } else {
    //             bail!("paste error. No file yanked?");
    //         }
    //     }
    //     self.yanked_node_id = None;
    //     self.update_item_delete();
    //     Ok(())
    // }

    // pub fn update_search_library(&mut self, input: &str) {
    //     let mut table: TableBuilder = TableBuilder::default();
    //     let root = self.tree.root();
    //     let p: &Path = Path::new(root.id());
    //     let all_items = walkdir::WalkDir::new(p).follow_links(true);
    //     let mut idx = 0;
    //     let mut search = "*".to_string();
    //     search.push_str(input);
    //     search.push('*');
    //     for record in all_items.into_iter().filter_map(std::result::Result::ok) {
    //         let file_name = record.path();
    //         if wildmatch::WildMatch::new(&search).matches(file_name.to_string_lossy().as_ref()) {
    //             if idx > 0 {
    //                 table.add_row();
    //             }
    //             idx += 1;
    //             table
    //                 .add_col(TextSpan::new(idx.to_string()))
    //                 .add_col(TextSpan::new(file_name.to_string_lossy()));
    //         }
    //     }

    //     let table = table.build();
    //     if let Some(props) = self.view.get_props(COMPONENT_TABLE_SEARCH_LIBRARY) {
    //         let props = TablePropsBuilder::from(props).with_table(table).build();
    //         let msg = self.view.update(COMPONENT_TABLE_SEARCH_LIBRARY, props);
    //         self.update(&msg);
    //     }
    // }

    // pub fn select_after_search_library(&mut self, node_id: usize) {
    //     if_chain! {
    //         if let Some(props) = self.view.get_props(COMPONENT_TABLE_SEARCH_LIBRARY);
    //         if let Some(PropPayload::One(PropValue::Table(table))) = props.own.get("table");
    //         if let Some(line) = table.get(node_id);
    //         if let Some(text_span) = line.get(1);
    //         let text = text_span.content.clone();
    //         if let Some(props) = self.view.get_props(COMPONENT_TREEVIEW_LIBRARY);
    //         then {
    //             let props = TreeViewPropsBuilder::from(props)
    //                 .with_node(Some(&text))
    //                 .build();

    //             let msg = self.view.update(COMPONENT_TREEVIEW_LIBRARY, props);
    //             self.update(&msg);
    //         }
    //     }
    // }

    // pub fn add_playlist_after_search_library(&mut self, node_id: usize) {
    //     if_chain! {
    //         if let Some(props) = self.view.get_props(COMPONENT_TABLE_SEARCH_LIBRARY);
    //         if let Some(PropPayload::One(PropValue::Table(table))) = props.own.get("table");
    //         if let Some(line) = table.get(node_id);
    //         if let Some(text_span) = line.get(1);
    //         let text = text_span.content.clone();
    //         let p: &Path = Path::new(&text);
    //         if p.exists();
    //         then {
    //             if p.is_dir() {
    //                 let new_items = Self::dir_children(p);
    //                 for i in new_items.iter().rev() {
    //                     match Song::from_str(i) {
    //                         Ok(s) => self.add_playlist(s),
    //                         Err(e) => {
    //                             self.mount_error(
    //                                 format!("add playlist error: {}", e).as_str(),
    //                             );
    //                         }
    //                     };
    //                 }
    //             } else  {
    //                 match Song::from_str(&text) {
    //                     Ok(s) => self.add_playlist(s),
    //                     Err(e) => {
    //                         self.mount_error(format!("add playlist error: {}", e).as_str());
    //                     }
    //                 };
    //             }
    //         }
    //     }
    // }
}
