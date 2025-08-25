use std::fs::{DirEntry, remove_dir_all, remove_file, rename};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use termusiclib::config::SharedTuiSettings;
use termusiclib::config::v2::server::ScanDepth;
use termusiclib::config::v2::server::config_extra::ServerConfigVersionedDefaulted;
use tui_realm_treeview::{Node, TREE_CMD_CLOSE, TREE_CMD_OPEN, TREE_INITIAL_NODE, Tree, TreeView};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, TableBuilder, TextSpan};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

use crate::ui::ids::Id;
use crate::ui::model::{DownloadTracker, Model, TxToMain, UserEvent};
use crate::ui::msg::{DeleteConfirmMsg, GSMsg, LIMsg, Msg, PLMsg, RecVec, TEMsg, YSMsg};
use crate::ui::tui_cmd::TuiCmd;
use crate::utils::get_pin_yin;

#[derive(MockComponent)]
pub struct MusicLibrary {
    component: TreeView<String>,
    config: SharedTuiSettings,
}

impl MusicLibrary {
    pub fn new(
        tree: &Tree<String>,
        initial_node: Option<String>,
        config: SharedTuiSettings,
    ) -> Self {
        // Preserve initial node if exists
        let initial_node = match initial_node {
            Some(id) if tree.root().query(&id).is_some() => id,
            _ => tree.root().id().to_string(),
        };
        let component = {
            let config = config.read();
            TreeView::default()
                .background(config.settings.theme.library_background())
                .foreground(config.settings.theme.library_foreground())
                .borders(
                    Borders::default()
                        .color(config.settings.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                // .inactive(Style::default().fg(Color::Gray))
                .indent_size(2)
                .scroll_step(6)
                .title(" Library ", Alignment::Left)
                .highlighted_color(config.settings.theme.library_highlight())
                .highlight_symbol(&config.settings.theme.style.library.highlight_symbol)
                .preserve_state(true)
                // .highlight_symbol("🦄")
                .with_tree(tree.clone())
                .initial_node(initial_node)
        };

        let mut ret = Self { component, config };

        ret.open_root_node();

        ret
    }

    /// Also known as going up in the tree
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

    /// Also known as going down the tree / adding file to playlist
    fn handle_right_key(&mut self) -> (CmdResult, Option<Msg>) {
        let current_node = self.component.tree_state().selected().unwrap();
        let path: &Path = Path::new(current_node);
        if path.is_dir() {
            // TODO: try to load the directory if it is not loaded yet.
            (self.perform(Cmd::Custom(TREE_CMD_OPEN)), None)
        } else {
            (
                CmdResult::None,
                Some(Msg::Playlist(PLMsg::Add(path.to_path_buf()))),
            )
        }
    }

    /// [`TreeView`] does not start with the root node opened, this function does that.
    fn open_root_node(&mut self) {
        let root = self.component.tree().root();
        if self.component.tree_state().is_closed(root) {
            self.perform(Cmd::Custom(TREE_CMD_OPEN));
        }
    }
}

impl Component<Msg, UserEvent> for MusicLibrary {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let result = match ev {
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.left.get() => {
                self.handle_left_key()
            }
            Event::Keyboard(KeyEvent {
                code: Key::Left,
                modifiers: KeyModifiers::NONE,
            }) => self.handle_left_key(),
            Event::Keyboard(KeyEvent {
                code: Key::Right,
                modifiers: KeyModifiers::NONE,
            }) => match self.handle_right_key() {
                (_, Some(msg)) => return Some(msg),
                (cmdresult, None) => cmdresult,
            },
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.right.get() => {
                match self.handle_right_key() {
                    (_, Some(msg)) => return Some(msg),
                    (cmdresult, None) => cmdresult,
                }
            }
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.down.get() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.up.get() => {
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

            Event::Keyboard(keyevent) if keyevent == keys.library_keys.load_dir.get() => {
                let current_node = self.component.tree_state().selected().unwrap();
                let path: &Path = Path::new(current_node);
                if path.is_dir() {
                    return Some(Msg::Playlist(PLMsg::Add(path.to_path_buf())));
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
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.goto_top.get() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.goto_bottom.get() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Submit),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::Library(LIMsg::TreeStepOut)),
            Event::Keyboard(
                KeyEvent {
                    code: Key::Tab,
                    modifiers: KeyModifiers::NONE,
                }
                | KeyEvent {
                    code: Key::BackTab,
                    modifiers: KeyModifiers::SHIFT,
                },
            ) => return Some(Msg::Library(LIMsg::TreeBlur)),
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.delete.get() => {
                return Some(Msg::DeleteConfirm(DeleteConfirmMsg::Show));
            }
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.yank.get() => {
                return Some(Msg::Library(LIMsg::Yank));
            }
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.paste.get() => {
                return Some(Msg::Library(LIMsg::Paste));
            }
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.cycle_root.get() => {
                return Some(Msg::Library(LIMsg::SwitchRoot));
            }

            Event::Keyboard(keyevent) if keyevent == keys.library_keys.add_root.get() => {
                return Some(Msg::Library(LIMsg::AddRoot));
            }
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.remove_root.get() => {
                return Some(Msg::Library(LIMsg::RemoveRoot));
            }
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.search.get() => {
                return Some(Msg::GeneralSearch(GSMsg::PopupShowLibrary));
            }

            Event::Keyboard(keyevent) if keyevent == keys.library_keys.youtube_search.get() => {
                return Some(Msg::YoutubeSearch(YSMsg::InputPopupShow));
            }
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.open_tag_editor.get() => {
                let current_node = self.component.tree_state().selected().unwrap();
                return Some(Msg::TagEditor(TEMsg::Open(current_node.to_string())));
            }

            _ => CmdResult::None,
        };
        match result {
            CmdResult::Submit(State::One(StateValue::String(node))) => {
                Some(Msg::Library(LIMsg::TreeStepInto(node)))
            }
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

impl Model {
    pub fn library_upper_dir(&self) -> Option<PathBuf> {
        self.library
            .tree_path
            .parent()
            .map(std::path::Path::to_path_buf)
    }

    /// Execute [`Self::library_scan`] from a `&self` instance.
    #[inline]
    pub fn library_scan_dir<P: Into<PathBuf>>(&self, path: P, focus_node: Option<String>) {
        Self::library_scan(
            self.tx_to_main.clone(),
            self.download_tracker.clone(),
            path,
            ScanDepth::Limited(2),
            focus_node,
        );
    }

    pub fn loading_tree() -> Tree<String> {
        Tree::new(Node::new("/dev/null".to_string(), "Loading...".to_string()))
    }

    /// Execute a library scan on a different thread.
    ///
    /// Executes [`Self::library_dir_tree`] on a different thread and send a [`LIMsg::TreeNodeReady`] on finish
    pub fn library_scan<P: Into<PathBuf>>(
        tx: TxToMain,
        download_tracker: DownloadTracker,
        path: P,
        depth: ScanDepth,
        focus_node: Option<String>,
    ) {
        let path = path.into();
        std::thread::Builder::new()
            .name("library tree scan".to_string())
            .spawn(move || {
                download_tracker.increase_one(path.to_string_lossy());
                let root_node = Self::library_dir_tree(&path, depth);

                let _ = tx.send(Msg::Library(LIMsg::TreeNodeReady(root_node, focus_node)));
                download_tracker.decrease_one(&path.to_string_lossy());
            })
            .expect("Failed to spawn thread");
    }

    /// Scan the given `path` for up to `depth`, and return a [`Node`] tree.
    ///
    /// Note: consider using [`Self::library_scan`] instead of this directly.
    fn library_dir_tree(path: &Path, depth: ScanDepth) -> RecVec<PathBuf, String> {
        let name: String = match path.file_name() {
            None => "/".to_string(),
            Some(n) => n.to_string_lossy().into_owned(),
        };
        let mut node = RecVec {
            id: path.to_path_buf(),
            value: name,
            children: Vec::new(),
        };

        let depth = match depth {
            ScanDepth::Limited(v) => v,
            // put some kind of limit on it, thought the stack will likely overflow before this
            ScanDepth::Unlimited => u32::MAX,
        };

        if depth > 0 && path.is_dir() {
            if let Ok(paths) = std::fs::read_dir(path) {
                let mut paths: Vec<(String, PathBuf)> = paths
                    .filter_map(std::result::Result::ok)
                    .filter(|p| !p.file_name().to_string_lossy().starts_with('.'))
                    .map(|v| (get_pin_yin(&v.file_name().to_string_lossy()), v.path()))
                    .collect();

                paths.sort_by(|a, b| alphanumeric_sort::compare_str(&a.0, &b.0));

                for p in paths {
                    node.children
                        .push(Self::library_dir_tree(&p.1, ScanDepth::Limited(depth - 1)));
                }
            }
        }
        node
    }

    pub fn library_dir_children(p: &Path) -> Vec<String> {
        let mut children: Vec<String> = vec![];
        if p.is_dir() {
            if let Ok(paths) = std::fs::read_dir(p) {
                let mut paths: Vec<(String, DirEntry)> = paths
                    .filter_map(std::result::Result::ok)
                    .map(|v| (get_pin_yin(&v.file_name().to_string_lossy()), v))
                    .collect();

                paths.sort_by(|a, b| alphanumeric_sort::compare_str(&a.0, &b.0));

                for (_, p) in paths {
                    if !p.path().is_dir() {
                        children.push(String::from(p.path().to_string_lossy()));
                    }
                }
            }
        }
        children
    }

    /// Reload the library with the given `node` as a focus, also starts a new database sync worker for the current path.
    pub fn library_reload_with_node_focus(&mut self, node: Option<String>) {
        if let Err(err) = self.db.scan_path(
            self.library.tree_path.as_path(),
            &self.config_server.read_recursive(),
            false,
        ) {
            error!(
                "Error scanning path {:#?}: {err:#?}",
                self.library.tree_path.display()
            );
        }
        self.database_reload();
        self.library_scan_dir(&self.library.tree_path, node);
    }

    /// Convert a [`RecVec`] to a [`Node`].
    fn recvec_to_node(vec: RecVec<PathBuf, String>) -> Node<String> {
        let mut node = Node::new(vec.id.to_string_lossy().to_string(), vec.value);

        for val in vec.children {
            node.add_child(Self::recvec_to_node(val));
        }

        node
    }

    /// Apply the given [`RecVec`] as a tree
    pub fn library_apply_as_tree(
        &mut self,
        msg: RecVec<PathBuf, String>,
        focus_node: Option<String>,
    ) {
        let root_path = msg.id.clone();
        let root_node = Self::recvec_to_node(msg);

        let old_current_node = match self.app.state(&Id::Library).ok().unwrap() {
            State::One(StateValue::String(id)) => Some(id),
            _ => None,
        };

        self.library.tree_path = root_path;
        self.library.tree = Tree::new(root_node);

        // remount preserves focus
        let _ = self.app.remount(
            Id::Library,
            Box::new(MusicLibrary::new(
                &self.library.tree,
                old_current_node,
                self.config_tui.clone(),
            )),
            Vec::new(),
        );

        // focus the specified node
        if let Some(id) = focus_node {
            let _ = self.app.attr(
                &Id::Library,
                Attribute::Custom(TREE_INITIAL_NODE),
                AttrValue::String(id),
            );
        }
    }

    /// Handle stepping into a node on the tree
    pub fn library_stepinto(&mut self, node_id: &str) {
        self.library_scan_dir(PathBuf::from(node_id), None);
    }

    /// Handle stepping out of the current root node on the tree
    pub fn library_stepout(&mut self) {
        if let Some(p) = self.library_upper_dir() {
            let focus_node = Some(self.library.tree_path.to_string_lossy().to_string());
            self.library_scan_dir(p, focus_node);
        }
    }

    pub fn library_before_delete(&mut self) {
        if let Ok(State::One(StateValue::String(node_id))) = self.app.state(&Id::Library) {
            let p: &Path = Path::new(node_id.as_str());
            if p.is_file() {
                self.mount_confirm_radio();
            } else {
                self.mount_confirm_input("You're about to delete the whole directory.");
            }
        }
    }

    /// Delete the currently selected node from the filesystem and reload the tree and remove the deleted paths from the playlist.
    pub fn library_delete_node(&mut self) -> Result<()> {
        if let Ok(State::One(StateValue::String(node_id))) = self.app.state(&Id::Library) {
            if let Some(mut route) = self.library.tree.root().route_by_node(&node_id) {
                let p: &Path = Path::new(node_id.as_str());
                if p.is_file() {
                    remove_file(p)?;
                } else {
                    p.canonicalize()?;
                    remove_dir_all(p)?;
                }

                let mut tree = self.library.tree.clone();
                tree.root_mut().remove_child(&node_id);
                let mut focus_node: Option<String> = None;
                // case 1: the route still exists due to having a sibling beyond the index which now takes the same index
                if let Some(node) = tree.root().node_by_route(&route) {
                    focus_node = Some(node.id().to_string());
                } else if !route.is_empty() {
                    let _ = route.pop();
                    // case 2: the route does not exist anymore, but there is a parent in the route
                    if let Some(parent) = tree.root().node_by_route(&route) {
                        // case 2.1: the parent has children, select the last of them
                        if let Some(last_child) = parent.children().last() {
                            focus_node = Some(last_child.id().to_string());
                        } else {
                            // case 2.2: the parent exists, but has no children
                            focus_node = Some(parent.id().to_string());
                        }
                    }
                }

                self.library_scan_dir(&self.library.tree_path, focus_node);
            }
            // this line remove the deleted songs from playlist
            self.playlist_update_library_delete();
        }
        Ok(())
    }

    pub fn library_yank(&mut self) {
        if let Ok(State::One(StateValue::String(node_id))) = self.app.state(&Id::Library) {
            self.library.yanked_node_id = Some(node_id);
        }
    }

    pub fn library_paste(&mut self) -> Result<()> {
        if let Ok(State::One(StateValue::String(new_id))) = self.app.state(&Id::Library) {
            let old_id = self
                .library
                .yanked_node_id
                .as_ref()
                .context("no id yanked")?;
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
            self.library_reload_with_node_focus(Some(new_node_id.to_string_lossy().to_string()));
        }
        self.library.yanked_node_id = None;
        self.playlist_update_library_delete();
        Ok(())
    }

    pub fn library_update_search(&mut self, input: &str) {
        let mut table: TableBuilder = TableBuilder::default();
        let root = self.library.tree.root();
        let p: &Path = Path::new(root.id());
        let all_items = walkdir::WalkDir::new(p).follow_links(true);
        let mut idx: usize = 0;
        let search = format!("*{}*", input.to_lowercase());
        let search = wildmatch::WildMatch::new(&search);
        for record in all_items.into_iter().filter_map(std::result::Result::ok) {
            let file_name = record.path();
            if search.matches(&file_name.to_string_lossy().to_lowercase()) {
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
        let config_server = self.config_server.read();
        for dir in &config_server.settings.player.music_dirs {
            let absolute_dir = shellexpand::path::tilde(dir).into_owned();
            vec.push(absolute_dir);
        }
        if let Some(dir) = &config_server.music_dir_overwrite {
            let absolute_dir = shellexpand::path::tilde(dir).into_owned();
            vec.push(absolute_dir);
        }
        if vec.is_empty() {
            return;
        }
        drop(config_server);

        let mut index = 0;
        let current_path = &self.library.tree_path;
        for (idx, dir) in vec.iter().enumerate() {
            if current_path == dir {
                index = idx + 1;
                break;
            }
        }
        if index > vec.len() - 1 {
            index = 0;
        }
        if let Some(dir) = vec.get(index) {
            let pathbuf = PathBuf::from(dir);
            self.library_scan_dir(pathbuf, None);
            self.library_reload_with_node_focus(None);
        }
    }

    pub fn library_add_root(&mut self) -> Result<()> {
        let current_path = &self.library.tree_path;

        let mut config_server = self.config_server.write();

        for dir in &config_server.settings.player.music_dirs {
            let absolute_dir = shellexpand::path::tilde(dir);
            if &absolute_dir == current_path {
                bail!("Add root failed, same root already exists");
            }
        }
        config_server
            .settings
            .player
            .music_dirs
            .push(current_path.clone());
        let res = ServerConfigVersionedDefaulted::save_config_path(&config_server.settings);
        drop(config_server);

        res.context("Error while saving config")?;
        self.command(TuiCmd::ReloadConfig);
        Ok(())
    }

    pub fn library_remove_root(&mut self) -> Result<()> {
        let current_path = &self.library.tree_path;
        let mut config_server = self.config_server.write();

        let mut vec = Vec::new();
        for dir in &config_server.settings.player.music_dirs {
            let absolute_dir = shellexpand::path::tilde(dir);
            if &absolute_dir == current_path {
                continue;
            }
            vec.push(dir.clone());
        }
        if vec.is_empty() {
            bail!("At least 1 root music directory should be kept");
        }

        config_server.settings.player.music_dirs = vec;
        let res = ServerConfigVersionedDefaulted::save_config_path(&config_server.settings);
        drop(config_server);

        self.library_switch_root();

        res.context("Error while saving config")?;

        Ok(())
    }
}
