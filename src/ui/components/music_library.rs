use crate::ui::components::StyleColorSymbol;
use crate::ui::model::MAX_DEPTH;
use crate::ui::{Id, LIMsg, Model, Msg, TEMsg, YSMsg};
use anyhow::{bail, Result};
use if_chain::if_chain;
use std::fs::{remove_dir_all, remove_file, rename};
use std::path::{Path, PathBuf};
use tui_realm_treeview::{Node, Tree, TreeView, TREE_CMD_CLOSE, TREE_CMD_OPEN, TREE_INITIAL_NODE};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, TableBuilder, TextSpan};
// use tuirealm::tui::style::{Color, Style};
use tuirealm::tui::style::Color;
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

#[derive(MockComponent)]
pub struct MusicLibrary {
    component: TreeView,
}

impl MusicLibrary {
    pub fn new(
        tree: &Tree,
        initial_node: Option<String>,
        color_mapping: &StyleColorSymbol,
    ) -> Self {
        // Preserve initial node if exists
        let initial_node = match initial_node {
            Some(id) if tree.root().query(&id).is_some() => id,
            _ => tree.root().id().to_string(),
        };
        Self {
            component: TreeView::default()
                .background(color_mapping.library_background().unwrap_or(Color::Reset))
                .foreground(color_mapping.library_foreground().unwrap_or(Color::Magenta))
                .borders(
                    Borders::default()
                        .color(color_mapping.library_border().unwrap_or(Color::Magenta))
                        .modifiers(BorderType::Rounded),
                )
                // .inactive(Style::default().fg(Color::Gray))
                .indent_size(2)
                .scroll_step(6)
                .title("Library", Alignment::Left)
                .highlighted_color(color_mapping.library_highlight().unwrap_or(Color::Yellow))
                .highlight_symbol(&color_mapping.library_highlight_symbol)
                .preserve_state(true)
                // .highlight_symbol("🦄")
                .with_tree(tree.clone())
                .initial_node(initial_node),
        }
    }
}

impl Component<Msg, NoUserEvent> for MusicLibrary {
    #[allow(clippy::too_many_lines)]
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
                    return Some(Msg::Playlist(crate::ui::PLMsg::Add(
                        current_node.to_string(),
                    )));
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right | Key::Char('L'),
                modifiers: KeyModifiers::SHIFT,
            }) => {
                let current_node = self.component.tree_state().selected().unwrap();
                let p: &Path = Path::new(current_node);
                if p.is_dir() {
                    return Some(Msg::Playlist(crate::ui::PLMsg::Add(
                        current_node.to_string(),
                    )));
                }
                return None;
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
            }) => return Some(Msg::Library(LIMsg::TreeGoToUpperDir)),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::Library(LIMsg::TreeBlur)),
            Event::Keyboard(KeyEvent {
                code: Key::Char('d'),
                ..
            }) => return Some(Msg::DeleteConfirmShow),
            Event::Keyboard(KeyEvent {
                code: Key::Char('y'),
                ..
            }) => return Some(Msg::Library(LIMsg::Yank)),
            Event::Keyboard(KeyEvent {
                code: Key::Char('p'),
                ..
            }) => return Some(Msg::Library(LIMsg::Paste)),
            Event::Keyboard(KeyEvent {
                code: Key::Char('/'),
                ..
            }) => return Some(Msg::GeneralSearch(crate::ui::GSMsg::PopupShowLibrary)),
            Event::Keyboard(KeyEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::YoutubeSearch(YSMsg::InputPopupShow)),
            Event::Keyboard(KeyEvent {
                code: Key::Char('t'),
                modifiers: KeyModifiers::NONE,
            }) => {
                let current_node = self.component.tree_state().selected().unwrap();
                return Some(Msg::TagEditor(TEMsg::TagEditorRun(
                    current_node.to_string(),
                )));
            }

            _ => return None,
        };
        match result {
            CmdResult::Submit(State::One(StateValue::String(node))) => {
                Some(Msg::Library(LIMsg::TreeExtendDir(node)))
            }
            _ => Some(Msg::None),
        }
    }
}

impl Model {
    pub fn library_scan_dir(&mut self, p: &Path) {
        self.path = p.to_path_buf();
        self.tree = Tree::new(Self::library_dir_tree(p, MAX_DEPTH));
    }

    pub fn library_upper_dir(&self) -> Option<PathBuf> {
        self.path.parent().map(std::path::Path::to_path_buf)
    }

    pub fn library_dir_tree(p: &Path, depth: usize) -> Node {
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
                    node.add_child(Self::library_dir_tree(p.path().as_path(), depth - 1));
                }
            }
        }
        node
    }
    pub fn library_dir_children(p: &Path) -> Vec<String> {
        let mut children: Vec<String> = vec![];
        if p.is_dir() {
            if let Ok(paths) = std::fs::read_dir(p) {
                let mut paths: Vec<_> = paths.filter_map(std::result::Result::ok).collect();

                paths.sort_by_cached_key(|k| {
                    get_pin_yin(&k.file_name().to_string_lossy().to_string())
                });
                for p in paths {
                    if !p.path().is_dir() {
                        children.push(String::from(p.path().to_string_lossy()));
                    }
                }
            }
        }
        children
    }

    pub fn library_sync(&mut self, node: Option<&str>) {
        self.library_reload_tree();
        if let Some(n) = node {
            assert!(self
                .app
                .attr(
                    &Id::Library,
                    Attribute::Custom(TREE_INITIAL_NODE),
                    AttrValue::String(n.to_string()),
                )
                .is_ok());
        }
    }

    pub fn library_reload_tree(&mut self) {
        self.tree = Tree::new(Self::library_dir_tree(self.path.as_ref(), MAX_DEPTH));
        let current_node = match self.app.state(&Id::Library).ok().unwrap() {
            State::One(StateValue::String(id)) => Some(id),
            _ => None,
        };
        // Remount tree
        // keep focus
        let mut focus = false;
        if let Ok(f) = self.app.query(&Id::Library, Attribute::Focus) {
            if Some(AttrValue::Flag(true)) == f {
                focus = true;
            }
        }

        assert!(self.app.umount(&Id::Library).is_ok());
        assert!(self
            .app
            .mount(
                Id::Library,
                Box::new(MusicLibrary::new(
                    &self.tree.clone(),
                    current_node,
                    &self.config.style_color_symbol,
                ),),
                Vec::new()
            )
            .is_ok());
        if focus {
            assert!(self.app.active(&Id::Library).is_ok());
        }
    }

    pub fn library_stepinto(&mut self, node_id: &str) {
        self.library_scan_dir(PathBuf::from(node_id).as_path());
        self.config.music_dir = node_id.to_string();
        self.library_reload_tree();
    }

    pub fn library_stepout(&mut self) {
        if let Some(p) = self.library_upper_dir() {
            self.library_scan_dir(p.as_path());
            self.config.music_dir = p.to_string_lossy().to_string();
            self.library_reload_tree();
        }
    }

    pub fn library_before_delete(&mut self) {
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
            if let Some(mut route) = self.tree.root().route_by_node(&node_id) {
                let p: &Path = Path::new(node_id.as_str());
                if p.is_file() {
                    remove_file(p)?;
                } else {
                    p.canonicalize()?;
                    remove_dir_all(p)?;
                }

                // // this is to keep the state of playlist
                self.library_reload_tree();
                let tree = self.tree.clone();
                if let Some(new_node) = tree.root().node_by_route(&route) {
                    self.library_sync(Some(new_node.id()));
                } else {
                    //special case 1: old route not available but have siblings
                    if let Some(last) = route.last_mut() {
                        if last > &mut 0 {
                            *last -= 1;
                        }
                    }
                    if let Some(new_node) = tree.root().node_by_route(&route) {
                        self.library_sync(Some(new_node.id()));
                    } else {
                        //special case 2: old route not available and no siblings
                        route.truncate(route.len() - 1);
                        if let Some(new_node) = tree.root().node_by_route(&route) {
                            self.library_sync(Some(new_node.id()));
                        }
                    }
                }
            }
            // this line remove the deleted songs from playlist
            self.playlist_update_library_delete();
        }
        Ok(())
    }

    pub fn library_yank(&mut self) {
        if let Ok(State::One(StateValue::String(node_id))) = self.app.state(&Id::Library) {
            self.yanked_node_id = Some(node_id);
        }
    }

    pub fn library_paste(&mut self) -> Result<()> {
        if_chain! {
            if let  Ok(State::One(StateValue::String(new_id))) = self.app.state(&Id::Library);
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
                self.library_sync(new_node_id.to_str());
                // self.reload_tree();
            } else {
                bail!("paste error. No file yanked?");
            }
        }
        self.yanked_node_id = None;
        self.playlist_update_library_delete();
        Ok(())
    }

    pub fn library_update_search(&mut self, input: &str) {
        let mut table: TableBuilder = TableBuilder::default();
        let root = self.tree.root();
        let p: &Path = Path::new(root.id());
        let all_items = walkdir::WalkDir::new(p).follow_links(true);
        let mut idx = 0;
        let search = format!("*{}*", input.to_lowercase());
        for record in all_items.into_iter().filter_map(std::result::Result::ok) {
            let file_name = record.path();
            if wildmatch::WildMatch::new(&search)
                .matches(&file_name.to_string_lossy().to_lowercase())
            {
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

        self.general_search_update_show(table);
    }
}

use pinyin::ToPinyin;

pub fn get_pin_yin(input: &str) -> String {
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
#[allow(clippy::non_ascii_literal)]
mod tests {

    use crate::ui::components::music_library::get_pin_yin;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pin_yin() {
        assert_eq!(get_pin_yin("陈一发儿"), "chenyifaer".to_string());
        assert_eq!(get_pin_yin("Gala乐队"), "GALAledui".to_string());
        assert_eq!(get_pin_yin("乐队Gala乐队"), "leduiGALAledui".to_string());
        assert_eq!(get_pin_yin("Annett Louisan"), "ANNETT LOUISAN".to_string());
    }
}
