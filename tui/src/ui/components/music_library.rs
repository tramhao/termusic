use std::fs::{remove_dir_all, remove_file, rename};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use termusiclib::config::SharedTuiSettings;
use termusiclib::config::v2::server::ScanDepth;
use termusiclib::config::v2::server::config_extra::ServerConfigVersionedDefaulted;
use tui_realm_treeview::{Node, TREE_CMD_CLOSE, TREE_CMD_OPEN, Tree, TreeView};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, TableBuilder, TextSpan};
use tuirealm::{
    Attribute, Component, Event, MockComponent, State, StateValue, Sub, SubClause, SubEventClause,
};

use crate::ui::ids::Id;
use crate::ui::model::{DownloadTracker, Model, TxToMain, UserEvent};
use crate::ui::msg::{
    DeleteConfirmMsg, GSMsg, LIMsg, LINodeReady, LINodeReadySub, LIReloadData, LIReloadPathData,
    Msg, PLMsg, RecVec, TEMsg, YSMsg,
};
use crate::ui::tui_cmd::TuiCmd;
use crate::utils::get_pin_yin;

#[derive(MockComponent)]
pub struct MusicLibrary {
    component: TreeView<String>,
    config: SharedTuiSettings,
    tx_to_main: TxToMain,
    download_tracker: DownloadTracker,

    /// The path of the last yanked node.
    yanked_path: Option<PathBuf>,
}

impl MusicLibrary {
    pub fn new(
        tree: &Tree<String>,
        initial_node: Option<String>,
        config: SharedTuiSettings,
        tx_to_main: TxToMain,
        download_tracker: DownloadTracker,
    ) -> Self {
        // Preserve initial node if exists
        let initial_node = match initial_node {
            Some(id) if tree.root().query(&id).is_some() => id,
            _ => tree.root().id().clone(),
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

        let mut ret = Self {
            component,
            config,
            tx_to_main,
            download_tracker,
            yanked_path: None,
        };

        ret.open_root_node();

        ret
    }

    /// Also known as going up in the tree
    fn handle_left_key(&mut self) -> (CmdResult, Option<Msg>) {
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
                        return (CmdResult::None, Some(Msg::ForceRedraw));
                    }
                    self.perform(Cmd::Custom(TREE_CMD_CLOSE));
                }

                return (CmdResult::None, Some(Msg::ForceRedraw));
            }
        }
        (CmdResult::None, None)
    }

    /// Also known as going down the tree / adding file to playlist
    fn handle_right_key(&mut self) -> (CmdResult, Option<Msg>) {
        let current_node = self.component.tree_state().selected().unwrap();
        let path: &Path = Path::new(current_node);
        if path.is_dir() {
            // string required due to orange-trees weirdness
            let current_node = current_node.to_string();
            if self
                .component
                .tree()
                .root()
                .query(&current_node)
                .is_some_and(Node::is_leaf)
            {
                self.handle_reload_at(LIReloadPathData {
                    path: path.to_path_buf(),
                    change_focus: true,
                });
                (CmdResult::None, None)
            } else {
                // "ForceRedraw" as "TreeView" will always return "CmdResult::None"
                (
                    self.perform(Cmd::Custom(TREE_CMD_OPEN)),
                    Some(Msg::ForceRedraw),
                )
            }
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

    /// Trigger a load with a message to change the tree root to the given path.
    ///
    /// This will make the given path(which will be the root node) the focused node.
    ///
    /// This will send a [`LIMsg::TreeNodeReady`] and change the root to `path`.
    fn trigger_load_stepinto<P: Into<PathBuf>>(&self, path: P) {
        library_scan(
            self.download_tracker.clone(),
            path,
            ScanDepth::Limited(2),
            self.tx_to_main.clone(),
            None,
        );
    }

    /// Trigger a load with a message to change the tree root to the given path.
    ///
    /// This will make the current tree root be the new focused node.
    ///
    /// This will send a [`LIMsg::TreeNodeReady`] and change the root to `path`.
    fn trigger_load_with_focus<P: Into<PathBuf>>(&self, scan_path: P, focus_node: Option<String>) {
        let path = scan_path.into();
        library_scan(
            self.download_tracker.clone(),
            path,
            ScanDepth::Limited(2),
            self.tx_to_main.clone(),
            focus_node,
        );
    }

    /// Trigger a load for the given path, with the given depth.
    ///
    /// This will send a [`LIMsg::TreeNodeReadySub`] and does not change the root, unless the
    /// given path *is* the root.
    fn trigger_subload_with_focus(
        &self,
        path: PathBuf,
        depth: ScanDepth,
        focus_node: Option<PathBuf>,
    ) {
        let tx = self.tx_to_main.clone();
        library_scan_cb(self.download_tracker.clone(), path, depth, move |vec| {
            let _ = tx.send(Msg::Library(LIMsg::TreeNodeReadySub(LINodeReadySub {
                vec,
                focus_node,
            })));
        });
    }

    /// Store the currently selected node as yanked (for pasting with [`Self::paste`]).
    fn yank(&mut self) {
        if let State::One(StateValue::String(node_id)) = self.state() {
            self.yanked_path = Some(PathBuf::from(node_id));
        }
    }

    /// Paste the previously yanked node in the currently selected node if it is a directory, otherwise in its parent.
    fn paste(&mut self) -> Result<Option<LIMsg>> {
        if let State::One(StateValue::String(new_id)) = self.state() {
            let Some(old_path) = self.yanked_path.take() else {
                return Ok(None);
            };
            let selected_node_path = Path::new(&new_id);

            let pold_filename = old_path.file_name().context("no file name found")?;
            let old_parent = old_path.parent().context("old path had no parent")?;
            let selected_parent = selected_node_path
                .parent()
                .context("No Parent for currently selected node")?;

            let new_path = if selected_node_path.is_dir() {
                selected_node_path.join(pold_filename)
            } else {
                selected_parent.join(pold_filename)
            };

            rename(&old_path, &new_path)?;

            if new_path.starts_with(old_parent) {
                // new path is contained within old path's parent directory
                self.handle_reload_at(LIReloadPathData {
                    path: new_path,
                    change_focus: true,
                });
                self.handle_reload_at(LIReloadPathData {
                    path: old_path,
                    change_focus: false,
                });
            } else if old_parent.starts_with(selected_parent) {
                // new path is contained within old path's parent directory
                // We cannot use the sub-load functions here as they replace the given path's node
                // and does not clear open/closed status, which treeview cannot handle and panics on going up.
                // See <https://github.com/veeso/tui-realm-treeview/issues/15>
                // self.handle_reload_at(LIReloadPathData { path: new_path.to_path_buf(), change_focus: true });
                // self.handle_reload_at(LIReloadPathData { path: old_path, change_focus: false });
                self.trigger_load_with_focus(
                    selected_parent,
                    Some(new_path.to_string_lossy().to_string()),
                );
            } else {
                // new path is not contained within old path's parent directory, so need to load both
                self.handle_reload_at(LIReloadPathData {
                    path: new_path,
                    change_focus: true,
                });
                self.handle_reload_at(LIReloadPathData {
                    path: old_parent.to_path_buf(),
                    change_focus: false,
                });
            }
        }
        Ok(Some(LIMsg::PlaylistRunDelete))
    }

    /// Handle sending a request to delete the currently selected node.
    fn handle_delete(&mut self) -> Msg {
        let current_node = self.component.tree_state().selected().unwrap();
        let path = PathBuf::from(current_node);

        let focus_node_after = {
            let current_node = current_node.to_string();
            let mut route = self
                .component
                .tree()
                .root()
                .route_by_node(&current_node)
                .unwrap();

            let tree = self.component.tree();
            // increase index as we dont modify the tree, to use that node if available
            if let Some(v) = route.last_mut() {
                *v += 1;
            }

            let mut focus_node = None;
            // case 1: the route exists due to having a sibling beyond the index
            if let Some(node) = tree.root().node_by_route(&route) {
                focus_node = Some(node.id());
            } else if !route.is_empty() {
                let _ = route.pop();
                // case 2: the route does not exist anymore, but there is a parent in the route
                if let Some(parent) = tree.root().node_by_route(&route) {
                    // case 2.1: the parent has children, select the last of them
                    if let Some(last_child) = parent.children().last() {
                        focus_node = Some(last_child.id());
                    } else {
                        // case 2.2: the parent exists, but has no children
                        focus_node = Some(parent.id());
                    }
                }
            }

            focus_node.cloned()
        };

        Msg::DeleteConfirm(DeleteConfirmMsg::Show(path, focus_node_after))
    }

    /// Handle a full reload / potential change of the current tree root.
    ///
    /// Also changes focus, if requested.
    fn handle_full_reload(&mut self, data: LIReloadData) -> Msg {
        let path = data
            .change_root_path
            .unwrap_or_else(|| PathBuf::from(self.component.tree().root().id()));
        let focus_node = data
            .focus_node
            .unwrap_or_else(|| self.component.tree_state().selected().unwrap().to_string());

        *self.component.tree_mut() = Model::loading_tree();

        self.trigger_load_with_focus(path, Some(focus_node));

        Msg::ForceRedraw
    }

    /// Truncate `node`'s path to `root_node`'s path, then split `node`'s path by the separator, iterate over the non-empty components.
    ///
    /// This assumes `node` contains `root_node`!
    fn split_components_root<'a>(root_node: &str, node: &'a str) -> impl Iterator<Item = &'a str> {
        node[root_node.len()..]
            .split(std::path::MAIN_SEPARATOR)
            .filter(|v| !v.is_empty())
    }

    /// Handle reloading of the given path, potentially without changing root, but also change focus.
    ///
    /// If necessary, load all paths in-between.
    fn handle_reload_at(&mut self, data: LIReloadPathData) {
        let path = data.path;
        let path_str = path.to_string_lossy();
        let root_node = self.component.tree().root();

        if !path_str.starts_with(root_node.id()) {
            debug!("Given path is outside of tree root, not loading!");
            return;
        }

        // because of the if above, we know the node is at least within the tree
        // so it is safe to use the root as the initial starting node.

        // this contains one of 3:
        // - the node itself
        // - the root node
        // - the nearest directory node
        let mut nearest_node = root_node;
        let mut nearest_match = 0;

        let components_between_root_and_path: Vec<&str> =
            Self::split_components_root(root_node.id(), &path_str).collect();

        for node in RecursiveNodeIter::new(root_node) {
            // exact match found, no need to further iterate
            if node.id().as_str() == path_str {
                nearest_node = node;
                break;
            }

            // The parent directory node will always contain the wanted path partially
            // skip everything else.
            // Otherwise it might decend into "root/to_delete/another" instead of wanted "root/dir/another".
            if !path_str.starts_with(node.id()) {
                continue;
            }

            for (idx, comp) in Self::split_components_root(root_node.id(), node.id()).enumerate() {
                let Some(gotten) = components_between_root_and_path.get(idx) else {
                    break;
                };

                if *gotten == comp && idx > nearest_match {
                    nearest_match = idx;
                    nearest_node = node;
                }
            }
        }

        trace!(
            "found nearest match: {:#?}",
            (&path_str, nearest_match, nearest_node.id())
        );

        let depth = components_between_root_and_path
            .len()
            .saturating_sub(nearest_match);
        let depth = u32::try_from(depth).unwrap_or_default();

        let focus_node = if data.change_focus { Some(path) } else { None };

        self.trigger_subload_with_focus(
            PathBuf::from(nearest_node.id()),
            ScanDepth::Limited(depth),
            focus_node,
        );
    }

    /// Apply the given data as the root of the tree, resetting the state of the tree.
    ///
    /// This will always replace the root of the tree.
    fn handle_ready(&mut self, data: LINodeReady) -> Msg {
        let vec = data.vec;
        let initial_node = data.focus_node;

        let initial_node =
            initial_node.or_else(|| self.component.tree_state().selected().map(String::from));

        let tree = Tree::new(recvec_to_node(vec));

        let focus = self.component.query(Attribute::Focus);

        // There is no "clear" method for state in Treeview currently, so we have to
        // entirely replace the Treeview component. The simplest way is to just replace via a new instance.
        *self = Self::new(
            &tree,
            initial_node,
            self.config.clone(),
            self.tx_to_main.clone(),
            self.download_tracker.clone(),
        );

        if let Some(focus) = focus {
            self.attr(Attribute::Focus, focus);
        }

        Msg::ForceRedraw
    }

    /// Apply the given data at the path the data is, potentially without changing root.
    ///
    /// This will replace the root if the given data is starting at the root path.
    fn handle_ready_sub(&mut self, data: LINodeReadySub) -> Option<Msg> {
        let vec = data.vec;
        let path_str = vec.id.to_string_lossy().to_string();

        let tree_mut = self.component.tree_mut().root_mut();

        if tree_mut.id() == &path_str {
            // the given data *is* the root, so we have to replace the whole tree
            self.component.set_tree(Tree::new(recvec_to_node(vec)));
        } else {
            let Some(parent_node) = self.component.tree_mut().root_mut().parent_mut(&path_str)
            else {
                warn!(
                    "Ready node ({}) not found in tree ({})!",
                    vec.id.display(),
                    self.component.tree().root().id()
                );
                return None;
            };

            let mut new_node = Some(recvec_to_node(vec));

            let children: Vec<Node<String>> = parent_node
                .iter()
                .map(|node| {
                    if node.id() == &path_str {
                        // we can gurantee the node only exists once in the tree
                        // if it somehow is not the case, a panic is good
                        new_node.take().unwrap()
                    } else {
                        node.clone()
                    }
                })
                .collect();

            // there is no function in "orange-trees" to replace a node, only a node's value (but nots its children)
            parent_node.clear();
            for child in children {
                parent_node.add_child(child);
            }

            // try to set a initially selected node
            if self.component.tree_state().selected().is_none() {
                self.component.perform(Cmd::GoTo(Position::Begin));
            }

            if let Some(focus_node) = data.focus_node {
                let focus_node_str = focus_node.to_string_lossy().to_string();
                self.move_focus_on_current_tree(&focus_node_str);
            }

            // Treeview does not allow calling "tree_changed" without supplying a new tree
            // but gives us a direct mutable reference to the tree, so we can just
            // swap in a bogus temporary one, and call "set_tree" with the old one.
            let mut tree = Model::loading_tree();
            std::mem::swap(self.component.tree_mut(), &mut tree);
            self.component.set_tree(tree);
        }

        Some(Msg::ForceRedraw)
    }

    /// Move focus in the current tree to the `wanted_path`.
    ///
    /// This function is necessary as [`TreeView`] only provides immutable access to the state,
    /// so we have to move by [`Cmd`]s, one-by-one.
    fn move_focus_on_current_tree(&mut self, wanted_path: &String) {
        // unwrap is safe due to use literally just having added it
        let Some(route_to_node) = self.component.tree().root().route_by_node(wanted_path) else {
            // wanted path is not part of the tree
            return;
        };

        // this is just a fallback as i dont fully trust the loop below
        let mut max_iters: usize = u16::MAX.into();

        loop {
            if max_iters == 0 {
                error!(
                    "Focus change logic has consumed too many steps. This is a BUG, please report it with the steps that were taken!"
                );
                break;
            }

            max_iters = max_iters.saturating_sub(1);

            let Some(selected) = self.component.tree_state().selected() else {
                // we cant really do anything, if it does not even respond to "Begin"
                break;
            };

            // check if we are done
            if selected == wanted_path {
                // always open the node, if it can be opened
                self.component.perform(Cmd::Custom(TREE_CMD_OPEN));
                break;
            }

            // route_by_node requires "&String", does not accept "&str"
            let as_string = selected.to_string();
            let root_node = self.component.tree().root();

            let Some(selected_route) = root_node.route_by_node(&as_string) else {
                // logic error in treeview, dont handle that
                debug!("logic break");
                break;
            };

            // handle being too deep
            if selected_route.len() > route_to_node.len() {
                self.component.perform(Cmd::Move(Direction::Up));
                continue;
            }

            // fetch which direction we need to move in
            let mut remaining_route = route_to_node.as_slice();
            let mut selected_route_remaining = selected_route.as_slice();
            for sibling_idx in &selected_route {
                let Some(route_idx) = remaining_route.first() else {
                    break;
                };

                if route_idx == sibling_idx {
                    remaining_route = &remaining_route[1..];
                    selected_route_remaining = &selected_route_remaining[1..];
                }
            }

            // move in that direction
            if let (Some(wanted_idx), Some(current_idx)) =
                (remaining_route.first(), selected_route_remaining.first())
            {
                if wanted_idx > current_idx {
                    self.component.perform(Cmd::Move(Direction::Down));
                    // if the node we got down to is the one we want, open it
                    if *wanted_idx == current_idx + 1 {
                        self.component.perform(Cmd::Custom(TREE_CMD_OPEN));
                    }
                } else if wanted_idx <= current_idx {
                    // If we are below the wanted index, move up.
                    // Also if we are *on* the wanted index, but somehow it didnt catch it above to be the same node
                    // then that means we are on different parents than wanted. Move up
                    self.component.perform(Cmd::Move(Direction::Up));
                    // if the node we got up to is the one we want, open it
                    if *wanted_idx == current_idx.saturating_sub(1) {
                        self.component.perform(Cmd::Custom(TREE_CMD_OPEN));
                    }
                }
                continue;
            }

            // we are on the wanted parent already, so just move down
            if let Some(_wanted_idx) = remaining_route.first() {
                self.component.perform(Cmd::Move(Direction::Down));
            }

            // repeat
        }
    }

    /// Handle all custom messages.
    fn handle_user_events(&mut self, ev: LIMsg) -> Option<Msg> {
        // handle subscriptions
        match ev {
            LIMsg::Reload(data) => Some(self.handle_full_reload(data)),
            LIMsg::ReloadPath(data) => {
                self.handle_reload_at(data);
                None
            }
            LIMsg::TreeNodeReady(data) => Some(self.handle_ready(data)),
            LIMsg::TreeNodeReadySub(data) => self.handle_ready_sub(data),
            _ => None,
        }
    }
}

impl Component<Msg, UserEvent> for MusicLibrary {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        if let Event::User(UserEvent::Forward(Msg::Library(ev))) = ev {
            return self.handle_user_events(ev);
        }

        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let result = match ev {
            // selection
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.left.get() => {
                match self.handle_left_key() {
                    (_, Some(msg)) => return Some(msg),
                    (cmdresult, None) => cmdresult,
                }
            }
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
                code: Key::Left,
                modifiers: KeyModifiers::NONE,
            }) => match self.handle_left_key() {
                (_, Some(msg)) => return Some(msg),
                (cmdresult, None) => cmdresult,
            },
            Event::Keyboard(KeyEvent {
                code: Key::Right,
                modifiers: KeyModifiers::NONE,
            }) => match self.handle_right_key() {
                (_, Some(msg)) => return Some(msg),
                (cmdresult, None) => cmdresult,
            },
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),

            // quick selection movement
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
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::End)),

            // file modifying
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.delete.get() => {
                return Some(self.handle_delete());
            }
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.yank.get() => {
                self.yank();
                CmdResult::None
            }
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.paste.get() => {
                match self.paste() {
                    Ok(None) => CmdResult::None,
                    Ok(Some(msg)) => return Some(Msg::Library(msg)),
                    Err(err) => return Some(Msg::Library(LIMsg::PasteError(err.to_string()))),
                }
            }

            // music root modification
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.cycle_root.get() => {
                let root_node = self.component.tree().root().id();
                let path = PathBuf::from(root_node);
                return Some(Msg::Library(LIMsg::SwitchRoot(path)));
            }

            Event::Keyboard(keyevent) if keyevent == keys.library_keys.add_root.get() => {
                let root_node = self.component.tree().root().id();
                let path = PathBuf::from(root_node);
                return Some(Msg::Library(LIMsg::AddRoot(path)));
            }
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.remove_root.get() => {
                let root_node = self.component.tree().root().id();
                let path = PathBuf::from(root_node);
                return Some(Msg::Library(LIMsg::RemoveRoot(path)));
            }

            // load more tree
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => {
                let current_root = Path::new(self.component.tree().root().id());
                let parent = current_root.parent().unwrap_or(current_root);

                // only trigger a load if we are not at the root of the filesystem already
                if current_root != parent {
                    let focus_node = Some(self.component.tree().root().id().clone());
                    self.trigger_load_with_focus(parent, focus_node);
                }
                // there is no special indicator or message; the download_tracker should force a draw once active
                CmdResult::None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Submit),

            // search
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.search.get() => {
                let root_node = self.component.tree().root().id();
                let path = PathBuf::from(root_node);
                return Some(Msg::GeneralSearch(GSMsg::PopupShowLibrary(path)));
            }

            Event::Keyboard(keyevent) if keyevent == keys.library_keys.youtube_search.get() => {
                return Some(Msg::YoutubeSearch(YSMsg::InputPopupShow));
            }

            // load into playlist
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.load_dir.get() => {
                let current_node = self.component.tree_state().selected().unwrap();
                let path = Path::new(current_node);
                if path.is_dir() {
                    return Some(Msg::Playlist(PLMsg::Add(path.to_path_buf())));
                }
                CmdResult::None
            }
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.load_track.get() => {
                let current_node = self.component.tree_state().selected().unwrap();
                let path = Path::new(current_node);
                if !path.is_dir() {
                    return Some(Msg::Playlist(PLMsg::Add(path.to_path_buf())));
                }
                CmdResult::None
            }

            // other
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
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.open_tag_editor.get() => {
                let current_node = self.component.tree_state().selected().unwrap();
                let path = Path::new(current_node);
                if !path.is_dir() {
                    return Some(Msg::TagEditor(TEMsg::Open(path.to_path_buf())));
                }
                CmdResult::None
            }

            _ => CmdResult::None,
        };
        match result {
            CmdResult::Submit(State::One(StateValue::String(node))) => {
                let path = Path::new(&node);
                if path.is_dir() {
                    self.trigger_load_stepinto(path);
                    // there is no special indicator or message; the download_tracker should force a draw once active
                }
                None
            }
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

/// Execute a library scan on a different thread.
///
/// Executes [`library_dir_tree`] on a different thread and calls `cb` on finish.
pub fn library_scan_cb<P: Into<PathBuf>, F>(
    download_tracker: DownloadTracker,
    path: P,
    depth: ScanDepth,
    cb: F,
) where
    F: FnOnce(RecVec) + Send + 'static,
{
    let path = path.into();
    std::thread::Builder::new()
        .name("library tree scan".to_string())
        .spawn(move || {
            download_tracker.increase_one(path.to_string_lossy());
            let vec = library_dir_tree(&path, depth);

            cb(vec);
            // let _ = tx.send(Msg::Library(LIMsg::TreeNodeReady(root_node, focus_node)));
            download_tracker.decrease_one(&path.to_string_lossy());
        })
        .expect("Failed to spawn thread");
}

/// Execute a library scan on a different thread.
///
/// Executes [`library_dir_tree`] on a different thread and send a [`LIMsg::TreeNodeReady`] on finish
fn library_scan<P: Into<PathBuf>>(
    download_tracker: DownloadTracker,
    path: P,
    depth: ScanDepth,
    tx: TxToMain,
    focus_node: Option<String>,
) {
    library_scan_cb(download_tracker, path, depth, move |vec| {
        let _ = tx.send(Msg::Library(LIMsg::TreeNodeReady(LINodeReady {
            vec,
            focus_node,
        })));
    });
}

/// Scan the given `path` for up to `depth`, and return a [`Node`] tree.
///
/// Note: consider using [`library_scan`] instead of this directly for running in a different thread.
pub fn library_dir_tree(path: &Path, depth: ScanDepth) -> RecVec {
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
                    .push(library_dir_tree(&p.1, ScanDepth::Limited(depth - 1)));
            }
        }
    }
    node
}

/// Convert a [`RecVec`] to a [`Node`].
fn recvec_to_node(vec: RecVec) -> Node<String> {
    let mut node = Node::new(vec.id.to_string_lossy().to_string(), vec.value);

    for val in vec.children {
        node.add_child(recvec_to_node(val));
    }

    node
}

impl Model {
    /// Mount the Music library
    pub fn mount_library(&mut self) -> Result<()> {
        self.app.mount(
            Id::Library,
            Box::new(MusicLibrary::new(
                &Self::loading_tree(),
                None,
                self.config_tui.clone(),
                self.tx_to_main.clone(),
                self.download_tracker.clone(),
            )),
            Self::library_subs(),
        )?;

        Ok(())
    }

    /// Get all subscriptions for the [`MusicLibrary`] Component.
    fn library_subs() -> Vec<Sub<Id, UserEvent>> {
        vec![
            Sub::new(
                SubEventClause::User(UserEvent::Forward(Msg::Library(LIMsg::Reload(
                    LIReloadData::default(),
                )))),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::User(UserEvent::Forward(Msg::Library(LIMsg::ReloadPath(
                    LIReloadPathData::default(),
                )))),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::User(UserEvent::Forward(Msg::Library(LIMsg::TreeNodeReady(
                    LINodeReady::default(),
                )))),
                SubClause::Always,
            ),
            Sub::new(
                SubEventClause::User(UserEvent::Forward(Msg::Library(LIMsg::TreeNodeReadySub(
                    LINodeReadySub::default(),
                )))),
                SubClause::Always,
            ),
        ]
    }

    /// Execute [`library_scan`] from a `&self` instance.
    ///
    /// Executes [`library_dir_tree`] on a different thread and send a [`LIMsg::TreeNodeReady`] on finish
    #[inline]
    pub fn library_scan_dir<P: Into<PathBuf>>(&self, path: P, focus_node: Option<String>) {
        library_scan(
            self.download_tracker.clone(),
            path,
            ScanDepth::Limited(2),
            self.tx_to_main.clone(),
            focus_node,
        );
    }

    /// Get a new tree with the root node showing "Loading...".
    pub fn loading_tree() -> Tree<String> {
        Tree::new(Node::new("/dev/null".to_string(), "Loading...".to_string()))
    }

    /// Reload the given path in the library and focus that node.
    ///
    /// Also re-indexes the path for the database if it is part of a music root.
    ///
    /// The input path is expected to be absolute.
    #[expect(clippy::unnecessary_debug_formatting)]
    pub fn library_reload_and_focus<P: Into<PathBuf>>(&mut self, path: P) {
        let path = path.into();
        if !path.is_absolute() {
            debug!("library reload, given path is not absolute! {path:#?}");
        }

        let config_read = self.config_server.read_recursive();
        for dir in &self.config_server.read().settings.player.music_dirs {
            let absolute_dir = shellexpand::path::tilde(dir);

            if path.starts_with(absolute_dir) {
                if let Err(err) = self.db.scan_path(&path, &config_read, false) {
                    error!("Error scanning path {:#?}: {err:#?}", path.display());
                }
            }
        }

        let _ = self
            .tx_to_main
            .send(Msg::Library(LIMsg::ReloadPath(LIReloadPathData {
                path,
                change_focus: true,
            })));
    }

    /// Show a deletion confirmation for the currently selected node.
    pub fn library_show_delete_confirm(&mut self, path: PathBuf, focus_node: Option<String>) {
        if path.is_file() {
            self.mount_confirm_radio(path, focus_node);
        } else {
            self.mount_confirm_input(
                path,
                focus_node,
                "You're about to delete the whole directory.",
            );
        }
    }

    /// Delete the currently selected node from the filesystem and reload the tree and remove the deleted paths from the playlist.
    pub fn library_delete_node(&mut self, path: &Path, focus_node: Option<String>) -> Result<()> {
        if path.is_file() {
            remove_file(path)?;
        } else {
            path.canonicalize()?;
            remove_dir_all(path)?;
        }

        // always scan the parent, as otherwise, if the deleted "path" is the root
        // we end up never actually loading something correct and still have the stale tree
        let parent = path.parent().expect("Path to have a parent");

        self.library_scan_dir(parent, focus_node);

        // this line remove the deleted songs from playlist
        self.playlist_update_library_delete();
        Ok(())
    }

    /// Generate the result table for search `input`, recursively from the tree's root node's path.
    pub fn library_update_search(&mut self, input: &str, path: &Path) {
        let mut table: TableBuilder = TableBuilder::default();
        let all_items = walkdir::WalkDir::new(path).follow_links(true);
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

    /// Switch the current tree root to the next one in the stored list, if available.
    pub fn library_switch_root(&mut self, old_path: &Path) {
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
        for (idx, dir) in vec.iter().enumerate() {
            if old_path == dir {
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
        }
    }

    /// Add the given path as a new library root for quick switching & metadata(database) scraping.
    pub fn library_add_root<P: Into<PathBuf>>(&mut self, path: P) -> Result<()> {
        let path = path.into();
        let mut config_server = self.config_server.write();

        for dir in &config_server.settings.player.music_dirs {
            let absolute_dir = shellexpand::path::tilde(dir);
            if absolute_dir == path {
                bail!("Same root already exists");
            }
        }
        config_server.settings.player.music_dirs.push(path);
        let res = ServerConfigVersionedDefaulted::save_config_path(&config_server.settings);
        drop(config_server);

        res.context("Error while saving config")?;
        self.command(TuiCmd::ReloadConfig);
        Ok(())
    }

    /// Remove the given path as a library root.
    pub fn library_remove_root<P: Into<PathBuf>>(&mut self, path: P) -> Result<()> {
        let path = path.into();
        let mut config_server = self.config_server.write();

        let mut vec = Vec::with_capacity(config_server.settings.player.music_dirs.len());
        for dir in &config_server.settings.player.music_dirs {
            let absolute_dir = shellexpand::path::tilde(dir);
            if absolute_dir == path {
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

        self.library_switch_root(&path);

        res.context("Error while saving config")?;

        Ok(())
    }
}

/// This exists as `orange-trees` does not have a recursive iter.
///
/// This is a depth-first iterator.
struct RecursiveNodeIter<'a> {
    stack: Vec<&'a Node<String>>,
}

impl<'a> RecursiveNodeIter<'a> {
    /// Create a new iterator with the children from the given node.
    ///
    /// Does *not* iterate over the node itself
    fn new(start_node: &'a Node<String>) -> Self {
        Self {
            stack: start_node.iter().collect(),
        }
    }
}

impl<'a> Iterator for RecursiveNodeIter<'a> {
    type Item = &'a Node<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.stack.pop()?;

        self.stack.extend(next.iter());

        Some(next)
    }
}
