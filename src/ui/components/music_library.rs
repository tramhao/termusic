use crate::config::{Keys, Settings};
use crate::ui::{Id, LIMsg, Model, Msg, TEMsg, YSMsg};
use crate::utils::get_pin_yin;
use anyhow::{bail, Context, Result};
use std::fs::{remove_dir_all, remove_file, rename};
use std::path::{Path, PathBuf};
use tui_realm_treeview::{Node, Tree, TreeView, TREE_CMD_CLOSE, TREE_CMD_OPEN, TREE_INITIAL_NODE};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{Alignment, BorderType, Borders, TableBuilder, TextSpan};
use tuirealm::tui::style::Color;
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

#[derive(MockComponent)]
pub struct MusicLibrary {
    component: TreeView,
    keys: Keys,
}

impl MusicLibrary {
    pub fn new(tree: &Tree, initial_node: Option<String>, config: &Settings) -> Self {
        // Preserve initial node if exists
        let initial_node = match initial_node {
            Some(id) if tree.root().query(&id).is_some() => id,
            _ => tree.root().id().to_string(),
        };
        Self {
            component: TreeView::default()
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Magenta),
                )
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::Magenta),
                        )
                        .modifiers(BorderType::Rounded),
                )
                // .inactive(Style::default().fg(Color::Gray))
                .indent_size(2)
                .scroll_step(6)
                .title(" Library ", Alignment::Left)
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::Yellow),
                )
                .highlight_symbol(&config.style_color_symbol.library_highlight_symbol)
                .preserve_state(true)
                // .highlight_symbol("🦄")
                .with_tree(tree.clone())
                .initial_node(initial_node),
            keys: config.keys.clone(),
        }
    }

    fn handle_left_key(&mut self) -> CmdResult {
        if let State::One(StateValue::String(node_id)) = self.state() {
            if let Some(node) = self.component.tree().root().query(&node_id) {
                if node.is_leaf() {
                    // When the selected node is a file, move focus to upper directory
                    self.perform(Cmd::GoTo(Position::Begin));
                    self.perform(Cmd::Move(Direction::Up));
                } else {
                    // When the selected node is a directory
                    if self.component.tree_state().is_closed(node) {
                        self.perform(Cmd::GoTo(Position::Begin));
                        self.perform(Cmd::Move(Direction::Up));
                        return CmdResult::None;
                    }
                    self.perform(Cmd::Custom(TREE_CMD_CLOSE));
                }
            }
        }
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for MusicLibrary {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let result = match ev {
            Event::Keyboard(keyevent) if keyevent == self.keys.global_left.key_event() => {
                self.handle_left_key()
            }
            Event::Keyboard(KeyEvent {
                code: Key::Left,
                modifiers: KeyModifiers::NONE,
            }) => self.handle_left_key(),
            Event::Keyboard(KeyEvent {
                code: Key::Right,
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
            Event::Keyboard(keyevent) if keyevent == self.keys.global_right.key_event() => {
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
            Event::Keyboard(keyevent) if keyevent == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),

            Event::Keyboard(keyevent) if keyevent == self.keys.library_load_dir.key_event() => {
                let current_node = self.component.tree_state().selected().unwrap();
                let p: &Path = Path::new(current_node);
                if p.is_dir() {
                    return Some(Msg::Playlist(crate::ui::PLMsg::Add(
                        current_node.to_string(),
                    )));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(keyevent) if keyevent == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
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
            Event::Keyboard(keyevent) if keyevent == self.keys.library_delete.key_event() => {
                return Some(Msg::DeleteConfirmShow)
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.library_yank.key_event() => {
                return Some(Msg::Library(LIMsg::Yank))
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.library_paste.key_event() => {
                return Some(Msg::Library(LIMsg::Paste))
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.library_switch_root.key_event() => {
                return Some(Msg::Library(LIMsg::SwitchRoot))
            }

            Event::Keyboard(keyevent) if keyevent == self.keys.library_add_root.key_event() => {
                return Some(Msg::Library(LIMsg::AddRoot))
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.library_remove_root.key_event() => {
                return Some(Msg::Library(LIMsg::RemoveRoot))
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.library_search.key_event() => {
                return Some(Msg::GeneralSearch(crate::ui::GSMsg::PopupShowLibrary))
            }

            Event::Keyboard(keyevent)
                if keyevent == self.keys.library_search_youtube.key_event() =>
            {
                return Some(Msg::YoutubeSearch(YSMsg::InputPopupShow))
            }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.library_tag_editor_open.key_event() =>
            {
                let current_node = self.component.tree_state().selected().unwrap();
                return Some(Msg::TagEditor(TEMsg::TagEditorRun(
                    current_node.to_string(),
                )));
            }

            _ => CmdResult::None,
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
        self.tree = Tree::new(Self::library_dir_tree(p, self.config.max_depth_cli));
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
                let mut paths: Vec<_> = paths
                    .filter_map(std::result::Result::ok)
                    .filter(|p| !p.file_name().to_string_lossy().to_string().starts_with('.'))
                    .collect();

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

    pub fn library_reload_with_node_focus(&mut self, node: Option<&str>) {
        self.db.sync_database(self.path.as_path());
        self.database_reload();
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
        self.tree = Tree::new(Self::library_dir_tree(
            self.path.as_ref(),
            self.config.max_depth_cli,
        ));
        let current_node = match self.app.state(&Id::Library).ok().unwrap() {
            State::One(StateValue::String(id)) => Some(id),
            _ => None,
        };
        let mut focus = false;

        if let Ok(f) = self.app.query(&Id::Library, Attribute::Focus) {
            if Some(AttrValue::Flag(true)) == f {
                focus = true;
            }
        }

        if focus {
            self.app.umount(&Id::Library).ok();
            assert!(self
                .app
                .mount(
                    Id::Library,
                    Box::new(MusicLibrary::new(
                        &self.tree.clone(),
                        current_node,
                        &self.config,
                    ),),
                    Vec::new()
                )
                .is_ok());
            self.app.active(&Id::Library).ok();
            return;
        }

        assert!(self
            .app
            .remount(
                Id::Library,
                Box::new(MusicLibrary::new(
                    &self.tree.clone(),
                    current_node,
                    &self.config,
                ),),
                Vec::new()
            )
            .is_ok());
    }

    // if let Ok(f) = self.app.query(&Id::Library, Attribute::Focus) {
    //     if Some(AttrValue::Flag(true)) == f {
    //         eprintln!("focus after remount: true");
    //     } else {
    //         eprintln!("focus after remount: false");
    //     }
    // }
    pub fn library_stepinto(&mut self, node_id: &str) {
        self.library_scan_dir(PathBuf::from(node_id).as_path());
        self.library_reload_tree();
    }

    pub fn library_stepout(&mut self) {
        if let Some(p) = self.library_upper_dir() {
            self.library_scan_dir(p.as_path());
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
                    self.library_reload_with_node_focus(Some(new_node.id()));
                } else {
                    //special case 1: old route not available but have siblings
                    if let Some(last) = route.last_mut() {
                        if last > &mut 0 {
                            *last -= 1;
                        }
                    }
                    if let Some(new_node) = tree.root().node_by_route(&route) {
                        self.library_reload_with_node_focus(Some(new_node.id()));
                    } else {
                        //special case 2: old route not available and no siblings
                        route.truncate(route.len() - 1);
                        if let Some(new_node) = tree.root().node_by_route(&route) {
                            self.library_reload_with_node_focus(Some(new_node.id()));
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
        if let Ok(State::One(StateValue::String(new_id))) = self.app.state(&Id::Library) {
            let old_id = self.yanked_node_id.as_ref().context("no id yanked")?;
            let p: &Path = Path::new(new_id.as_str());
            let pold: &Path = Path::new(old_id.as_str());
            let p_parent = p.parent().context("no parent folder found")?;
            let pold_filename = pold.file_name().context("no file name found")?;
            let new_node_id = if p.is_dir() {
                p.join(pold_filename)
            } else {
                p_parent.join(pold_filename)
            };
            rename(pold, new_node_id.as_path())?;
            self.library_reload_with_node_focus(new_node_id.to_str());
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

    pub fn library_switch_root(&mut self) {
        let mut vec = Vec::new();
        for dir in &self.config.music_dir {
            let absolute_dir = shellexpand::tilde(dir).to_string();
            vec.push(absolute_dir);
        }
        if let Some(dir) = &self.config.music_dir_from_cli {
            let absolute_dir = shellexpand::tilde(&dir).to_string();
            vec.push(absolute_dir);
        }
        if vec.is_empty() {
            return;
        }

        let mut index = 0;
        let current_path_str = self.path.to_string_lossy().to_string();
        for (idx, dir) in vec.iter().enumerate() {
            if current_path_str == *dir {
                index = idx + 1;
                break;
            }
        }
        if index > vec.len() - 1 {
            index = 0;
        }
        if let Some(dir) = vec.get(index) {
            let pathbuf = PathBuf::from(dir);
            self.library_scan_dir(pathbuf.as_path());
            self.library_reload_with_node_focus(None);
        }
    }

    pub fn library_add_root(&mut self) {
        let current_path_string = self.path.to_string_lossy().to_string();

        for dir in &self.config.music_dir {
            let absolute_dir = shellexpand::tilde(dir).to_string();
            if absolute_dir == current_path_string {
                return;
            }
        }
        self.config.music_dir.push(current_path_string);
    }
    pub fn library_remove_root(&mut self) -> Result<()> {
        let current_path_string = self.path.to_string_lossy().to_string();

        let mut vec = Vec::new();
        for dir in &self.config.music_dir {
            let absolute_dir = shellexpand::tilde(dir).to_string();
            if absolute_dir == current_path_string {
                continue;
            }
            vec.push(dir.clone());
        }
        if vec.is_empty() {
            bail!("At least 1 root music directory should be kept");
        }

        self.config.music_dir = vec;
        self.library_switch_root();
        Ok(())
    }
}
