//! The actual Music Library Component Implementation

use std::{
    cell::OnceCell,
    fs::rename,
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use termusiclib::config::{SharedTuiSettings, TuiOverlay, v2::server::ScanDepth};
use tuirealm::{
    Component, Event, MockComponent, State, StateValue,
    command::{Cmd, CmdResult, Direction, Position},
    event::{Key, KeyEvent, KeyModifiers},
    props::{Alignment, BorderType, Borders, Style},
    ratatui::{
        buffer::Buffer,
        layout::Rect,
        widgets::{Clear, Widget},
    },
};
use tuirealm_orx_tree::{
    NodeRef,
    component::TreeView,
    traversal::{Dfs, OverNode, Traverser},
    types::{MotionDirection, NodeIdx, NodeValue, Tree},
    widget::{CHILD_INDICATOR_LENGTH, RenderIndicator, calc_area_for_value},
};

use crate::ui::{
    components::orx_music_library::scanner::{library_scan, library_scan_cb, recvec_to_tree},
    model::{DownloadTracker, TxToMain, UserEvent},
    msg::{
        DeleteConfirmMsg, GSMsg, LIMsg, LINodeReady, LINodeReadySub, LIReloadData,
        LIReloadPathData, Msg, PLMsg, TEMsg, YSMsg,
    },
};

/// Data stored in a node in the [`NewMusicLibraryComponent`]'s tree.
#[derive(Debug)]
pub struct MusicLibData {
    /// The actual path of the node.
    path: PathBuf,
    // TODO: refactor bools to be bitflags to save on storage?
    /// Store whether that path is a dir to show indicators & use for prefetching
    is_dir: bool,
    /// Indicator if the we already send a request to fetch this directory
    is_loading: bool,
    /// Indicator that loading information about this (file EACCESS) or directory loading has failed.
    is_error: bool,
    /// The `path.file_name`'s string representation.
    ///
    /// Lazily evaluated from `path`, only when it becomes necessary.
    // TODO: evaluate if it would be more performant to only cache if `path.file_name().to_str_lossy()` returns `Cow::Owned`.
    as_str: OnceCell<String>,
}

impl MusicLibData {
    /// Create new data.
    pub fn new(path: PathBuf, is_dir: bool) -> Self {
        assert!(path.is_absolute());
        let cell = OnceCell::new();
        // Due to our expectation of the path not ending in `..`, we can assume
        // that there is always a file_name, EXCEPT on linux on the root ("/").
        // We *could* call `canonicalize` here again, but it is more likely the caller already has done that.
        if path.file_name().is_none() {
            let _ = cell.set("/".to_string());
        }

        Self {
            path,
            is_dir,
            is_loading: false,
            is_error: false,
            as_str: OnceCell::default(),
        }
    }
}

/// Indicator when for directories when we already issued a load for it (and not have gotten a response back yet).
///
/// It should look like "⟳".
const LOADING_SYMBOL: &str = "\u{27F3}";
/// Indicator for when directory loading had failed.
///
/// It should look like "✕" (Multiplication) chosen for being 1 draw width.
const ERROR_SYMBOL: &str = "\u{2715}";

impl NodeValue for MusicLibData {
    fn render(&self, buf: &mut Buffer, area: Rect, offset: usize, style: Style) {
        // Unwrap should never panic here as we already check the case of there not being a file_name on instance creation.
        // The *only* possible way to currently get this panic is when using the default instance (which shouldnt be used).
        let res = self
            .as_str
            .get_or_init(|| self.path.file_name().unwrap().to_string_lossy().to_string());

        NodeValue::render(res, buf, area, offset, style);
    }

    fn render_with_indicators(
        &self,
        buf: &mut Buffer,
        mut area: Rect,
        mut offset: usize,
        style: Style,
        _is_leaf: bool,
        is_opened: impl FnOnce() -> bool,
    ) {
        if self.is_error {
            // indicator error loading that directory / file
            RenderIndicator::new(ERROR_SYMBOL, "", 2).render(&mut offset, &mut area, buf, true);
        } else if !self.is_dir {
            // not a directory

            // indent leaf nodes by what is taken up on the parent by the indicators, otherwise children and the parent would have the same visible indent
            let leaf_indent = CHILD_INDICATOR_LENGTH;
            let indent_area = calc_area_for_value(&mut offset, &mut area, usize::from(leaf_indent));
            Clear.render(indent_area, buf);
        } else if !self.is_loading {
            // directory that is not loading
            RenderIndicator::default().render(&mut offset, &mut area, buf, is_opened());
        } else {
            // directory that is loading
            RenderIndicator::new(LOADING_SYMBOL, "", 2).render(&mut offset, &mut area, buf, true);
        }

        self.render(buf, area, offset, style);
    }
}

const LOADING_TREE_TEXT: &str = "Loading...";

#[derive(Debug, MockComponent)]
pub struct OrxMusicLibraryComponent {
    component: TreeView<MusicLibData>,
    config: SharedTuiSettings,

    tx_to_main: TxToMain,
    download_tracker: DownloadTracker,
    /// The path of the last yanked node.
    yanked_path: Option<PathBuf>,
}

impl OrxMusicLibraryComponent {
    fn get_inner_comp(config: &TuiOverlay) -> TreeView<MusicLibData> {
        TreeView::<MusicLibData>::default()
            .background(config.settings.theme.library_background())
            .foreground(config.settings.theme.library_foreground())
            .border(
                Borders::default()
                    .color(config.settings.theme.library_border())
                    .modifiers(BorderType::Rounded),
            )
            .indent_size(2)
            .scroll_step_horizontal(NonZeroUsize::new(2).unwrap())
            .title(" Library ", Alignment::Left)
            .highlight_color(config.settings.theme.library_highlight())
            .highlight_symbol(&config.settings.theme.style.library.highlight_symbol)
            .empty_tree_text(LOADING_TREE_TEXT)
    }

    /// Get a new empty instance, which shows "Loading..." while empty.
    pub fn new_loading(
        config: SharedTuiSettings,
        tx_to_main: TxToMain,
        download_tracker: DownloadTracker,
    ) -> Self {
        let component = {
            let config = config.read();

            Self::get_inner_comp(&config)
        };

        Self {
            component,
            config,
            yanked_path: None,
            download_tracker,
            tx_to_main,
        }
    }

    /// Create a new instance, with a tree already set.
    #[expect(unused)]
    pub fn new(
        tree: Tree<MusicLibData>,
        config: SharedTuiSettings,
        tx_to_main: TxToMain,
        download_tracker: DownloadTracker,
    ) -> Self {
        let mut this = Self::new_loading(config, tx_to_main, download_tracker);

        this.component = this.component.tree(tree);

        this
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
    fn trigger_load_with_focus<P: Into<PathBuf>>(&self, scan_path: P, focus_node: Option<PathBuf>) {
        let path = scan_path.into();
        library_scan(
            self.download_tracker.clone(),
            path,
            ScanDepth::Limited(2),
            self.tx_to_main.clone(),
            focus_node.map(|v| v.to_string_lossy().to_string()),
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
        if let Some(path) = self.get_selected_path() {
            self.yanked_path = Some(path.to_path_buf());
        }
    }

    /// Paste the previously yanked node in the currently selected node if it is a directory, otherwise in its parent.
    fn paste(&mut self) -> Result<Option<LIMsg>> {
        // This should happen before "yanked_path.take" so that we dont take, if we cannot apply it.
        // And "get_selected_path" cannot be put before here as that uses a immutable self reference, but ".take" requires mutable.
        if self.component.get_current_selected_node().is_none() {
            return Ok(None);
        }
        let Some(old_path) = self.yanked_path.take() else {
            return Ok(None);
        };
        let Some(selected_node_path) = self.get_selected_path() else {
            return Ok(None);
        };

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
            // // new path is contained within old path's parent directory
            // // We cannot use the sub-load functions here as they replace the given path's node
            // // and does not clear open/closed status, which treeview cannot handle and panics on going up.
            // // See <https://github.com/veeso/tui-realm-treeview/issues/15>
            self.handle_reload_at(LIReloadPathData {
                path: new_path.clone(),
                change_focus: true,
            });
            self.handle_reload_at(LIReloadPathData {
                path: old_path,
                change_focus: false,
            });
            // self.trigger_load_with_focus(
            //     selected_parent,
            //     Some(new_path),
            // );
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

        Ok(Some(LIMsg::PlaylistRunDelete))
    }

    /// Get the current root node's path, if there is one.
    fn get_root_path(&self) -> Option<&Path> {
        self.component
            .get_tree()
            .get_root()
            .map(|v| v.data().path.as_path())
    }

    /// Get the current selected node's path, if there is one.
    fn get_selected_path(&self) -> Option<&Path> {
        self.component
            .get_current_selected_node()
            .map(|v| v.data().path.as_path())
    }

    /// Also known as going up in the tree
    fn handle_left_key(&mut self) -> (CmdResult, Option<Msg>) {
        let Some(selected_node) = self.component.get_current_selected_node() else {
            return (CmdResult::None, None);
        };

        if !selected_node.data().is_dir
            || !self.component.get_state().is_opened(&selected_node.idx())
        {
            // When the selected node is a file or a closed directory, move focus to upper directory
            self.perform(Cmd::Custom(
                tuirealm_orx_tree::component::cmd::SELECT_PARENT,
            ));
        } else {
            // Directory is selected, but still open, close it
            // "Direction::Left" closes the current node
            self.component.perform(Cmd::Move(Direction::Left));
            return (CmdResult::None, Some(Msg::ForceRedraw));
        }

        (CmdResult::None, Some(Msg::ForceRedraw))
    }

    /// Also known as going down the tree / adding file to playlist
    fn handle_right_key(&mut self) -> (CmdResult, Option<Msg>) {
        let Some(selected_node) = self.component.get_current_selected_node() else {
            return (CmdResult::None, None);
        };

        if selected_node.data().is_dir {
            if selected_node.num_children() > 0 {
                // Current node has children loaded, just open it.

                // "Direction::Right" opens the current node
                self.perform(Cmd::Move(Direction::Right));

                (CmdResult::None, Some(Msg::ForceRedraw))
            } else if !selected_node.data().is_loading {
                // Current node does not have any children and is not loading, trigger a load for it
                self.handle_reload_at(LIReloadPathData {
                    path: selected_node.data().path.clone(),
                    change_focus: true,
                });
                (CmdResult::None, Some(Msg::ForceRedraw))
            } else {
                // Current node does not have any children is is loading, dont do anything
                (CmdResult::None, None)
            }
        } else {
            // Node is a file, try to add it to the playlist
            (
                CmdResult::None,
                Some(Msg::Playlist(PLMsg::Add(selected_node.data().path.clone()))),
            )
        }
    }

    /// Handle sending a request to delete the currently selected node.
    fn handle_delete(&mut self) -> Option<Msg> {
        let current_node = self.component.get_current_selected_node()?;
        let path = current_node.data().path.clone();

        let focus_node_after = {
            let num_siblings = current_node.num_siblings();
            // number returned includes the current node
            if num_siblings == 1 {
                // if this returns "None", the to-be deleted path is the root
                current_node
                    .parent()
                    .map(|parent| parent.data().path.clone())
            } else {
                // use the next closest sibling after delete of current node
                let next_child = current_node
                    .sibling_idx()
                    .min(num_siblings.saturating_sub(1));
                // if we have more than one siblings, it is guranteed to have a parent
                Some(
                    current_node
                        .parent()
                        .unwrap()
                        .child(next_child)
                        .data()
                        .path
                        .clone(),
                )
            }
        };

        Some(Msg::DeleteConfirm(DeleteConfirmMsg::Show(
            path,
            focus_node_after.map(|v| v.to_string_lossy().to_string()),
        )))
    }

    /// Handle a full reload / potential change of the current tree root.
    ///
    /// Also changes focus, if requested.
    fn handle_full_reload(&mut self, data: LIReloadData) -> Option<Msg> {
        let Some(path) = data
            .change_root_path
            .or_else(|| self.get_root_path().map(Path::to_path_buf))
        else {
            debug!("No \"change_root_path\" and no current root, not reloading!");
            return None;
        };
        let focus_node = data
            .focus_node
            .map(PathBuf::from)
            .or_else(|| self.get_selected_path().map(Path::to_path_buf));

        self.component.clear_tree();

        self.trigger_load_with_focus(path, focus_node);

        Some(Msg::ForceRedraw)
    }

    /// Truncate `node`'s path to `root_node`'s path, then split `node`'s path by the separator, iterate over the non-empty components.
    ///
    /// This assumes `node` contains `root_node`!
    fn split_components_root<'a>(
        root_node: &Path,
        node: &'a Path,
    ) -> impl Iterator<Item = std::path::Component<'a>> {
        node.components().skip(root_node.components().count())
    }

    /// Handle reloading of the given path, potentially without changing root, but also change focus.
    ///
    /// If necessary, load all paths in-between.
    fn handle_reload_at(&mut self, data: LIReloadPathData) {
        let path = data.path;
        let Some(root_node) = self.component.get_tree().get_root() else {
            debug!("No root node, not reloading!");
            return;
        };

        if !path.starts_with(&root_node.data().path) {
            debug!("Given path is outside of tree root, not loading!");
            return;
        }

        // because of the if above, we know the node is at least within the tree
        // so it is safe to use the root as the initial starting node.

        // this contains one of 3:
        // - the path of the node itself
        // - the root node's path
        // - the nearest directory node's path
        let mut nearest_path = &root_node.data().path;
        let mut nearest_idx = root_node.idx();
        let mut nearest_match = 0;

        let components_between_root_and_path: Vec<std::path::Component<'_>> =
            Self::split_components_root(&root_node.data().path, &path).collect();

        let mut traverser = Dfs::<OverNode>::new();
        // inital tree walker
        let walker = root_node.walk_with(&mut traverser);

        for node in walker {
            // exact match found, no need to further iterate
            if node.data().path == path {
                nearest_path = &node.data().path;
                nearest_idx = node.idx();
                break;
            }

            // The parent directory node will always contain the wanted path partially
            // skip everything else.
            // Otherwise it might decend into "root/to_delete/another" instead of wanted "root/dir/another".
            if !path.starts_with(&node.data().path) {
                continue;
            }

            for (idx, comp) in
                Self::split_components_root(&root_node.data().path, &node.data().path).enumerate()
            {
                let Some(gotten) = components_between_root_and_path.get(idx) else {
                    break;
                };

                if *gotten == comp && idx > nearest_match {
                    nearest_match = idx;
                    nearest_path = &node.data().path;
                    nearest_idx = node.idx();
                }
            }
        }

        let nearest_path = nearest_path.clone();

        trace!(
            "found nearest match: {:#?}",
            (&path, nearest_match, &nearest_path)
        );

        let depth = components_between_root_and_path
            .len()
            .saturating_sub(nearest_match);
        let depth = u32::try_from(depth).unwrap_or_default();

        let focus_node = if data.change_focus { Some(path) } else { None };

        // unwrap is safe as we literally just gotten the idx from the tree
        // set current node to loading, to indicate such to the user
        self.component
            .get_node_mut(&nearest_idx)
            .unwrap()
            .data_mut()
            .is_loading = true;

        self.trigger_subload_with_focus(nearest_path, ScanDepth::Limited(depth), focus_node);
    }

    /// Get the [`NodeIdx`] of a given [`Path`], searches from current tree root.
    fn get_idx_of_path(&self, path: &Path) -> Option<NodeIdx<MusicLibData>> {
        let root_node = self.component.get_tree().get_root()?;

        let mut traverser = Dfs::<OverNode>::new();
        // inital tree walker
        let mut walker = root_node.walk_with(&mut traverser);

        // TODO: ask orx why "traverser" needs to live as long as "tree"
        walker.find(|v| v.data().path == path).map(|v| v.idx())
    }

    /// Apply the given data as the root of the tree, resetting the state of the tree.
    ///
    /// This will always replace the root of the tree.
    #[expect(unsafe_code)]
    fn handle_ready(&mut self, data: LINodeReady) -> Msg {
        let vec = data.vec;
        let initial_node = data.focus_node;

        let initial_node = initial_node
            .map(PathBuf::from)
            .or_else(|| self.get_selected_path().map(Path::to_path_buf));

        let (_, tree) = recvec_to_tree(vec);

        self.component.clear_tree();
        // SAFETY: everything is already invalidated and cleared.
        *unsafe { self.component.get_tree_mut() } = tree;

        if let Some(initial_node) = initial_node {
            let idx = self.get_idx_of_path(&initial_node);
            if let Some(idx) = idx {
                self.component.select(MotionDirection::Upwards, idx);
            }
        } else {
            // always select the root node
            self.component.perform(Cmd::Move(Direction::Down));
            // always open the root node
            self.component.perform(Cmd::Move(Direction::Right));
        }

        Msg::ForceRedraw
    }

    /// Apply the given data at the path the data is, potentially without changing root.
    ///
    /// This will replace the root if the given data is starting at the root path.
    #[expect(unsafe_code)]
    fn handle_ready_sub(&mut self, data: LINodeReadySub) -> Option<Msg> {
        let vec = data.vec;

        // let tree_mut = self.component.tree_mut().root_mut();
        let Some(root_path) = self.get_root_path() else {
            // TODO: should we apply it?
            debug!("No root path, not applying");
            return None;
        };

        if root_path == vec.path {
            // the given data *is* the root, so we have to replace the whole tree
            self.component.clear_tree();
            // SAFETY: everything is already invalidated and cleared.
            *unsafe { self.component.get_tree_mut() } = recvec_to_tree(vec).1;
        } else {
            let vec_path = &vec.path;
            let mut traverser = Dfs::<OverNode>::new();
            // Unwrap is safe due to this path only being possible if there is a root path
            let root_node = self.component.get_tree().get_root().unwrap();
            // inital tree walker
            let mut walker = root_node.walk_with(&mut traverser);
            let Some(found_node) = walker.find(|v| &v.data().path == vec_path) else {
                warn!(
                    "Ready node ({}) not found in tree ({})!",
                    vec.path.display(),
                    self.component.get_tree().root().data().path.display()
                );
                return None;
            };
            // explicitly drop the walker, as otherwise it stays around for the entire scope for some reason
            drop(walker);

            let found_node_idx = found_node.idx();

            // Unwrap is safe, as we literally just searched the tree for this node
            let mut node_mut = self.component.get_node_mut(&found_node_idx).unwrap();

            // TODO: ask orx-tree for replacement function
            node_mut.push_sibling_tree(tuirealm_orx_tree::Side::Left, recvec_to_tree(vec).1);
            node_mut.prune();
            // NOTE: we dont need to re-set "is_loading" as the full node gets overwritten with new data, which defaults to "false"

            // try to set a initially selected node
            if self.component.get_current_selected_node().is_none() {
                self.component.perform(Cmd::GoTo(Position::Begin));
            }

            if let Some(focus_node) = data.focus_node {
                let idx = self.get_idx_of_path(&focus_node);
                if let Some(idx) = idx {
                    self.component.select(MotionDirection::Upwards, idx);
                    // always open the newly selected node
                    self.component.perform(Cmd::Move(Direction::Right));
                }
            }

            // TODO: call tree changed?
        }

        Some(Msg::ForceRedraw)
    }

    /// Handle all custom messages.
    fn handle_user_events(&mut self, ev: LIMsg) -> Option<Msg> {
        // handle subscriptions
        match ev {
            LIMsg::Reload(data) => self.handle_full_reload(data),
            LIMsg::ReloadPath(data) => {
                self.handle_reload_at(data);
                Some(Msg::ForceRedraw)
            }
            LIMsg::TreeNodeReady(data) => Some(self.handle_ready(data)),
            LIMsg::TreeNodeReadySub(data) => self.handle_ready_sub(data),
            _ => None,
        }
    }
}

impl Component<Msg, UserEvent> for OrxMusicLibraryComponent {
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
                return self.handle_delete();
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
                if let Some(path) = self.get_root_path() {
                    return Some(Msg::Library(LIMsg::SwitchRoot(path.to_path_buf())));
                }
                CmdResult::None
            }

            Event::Keyboard(keyevent) if keyevent == keys.library_keys.add_root.get() => {
                if let Some(path) = self.get_root_path() {
                    return Some(Msg::Library(LIMsg::AddRoot(path.to_path_buf())));
                }
                CmdResult::None
            }
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.remove_root.get() => {
                if let Some(path) = self.get_root_path() {
                    return Some(Msg::Library(LIMsg::RemoveRoot(path.to_path_buf())));
                }
                CmdResult::None
            }

            // load more tree
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let Some(current_root) = self.get_root_path() {
                    let parent = current_root.parent().unwrap_or(current_root);

                    // only trigger a load if we are not at the root of the filesystem already
                    if current_root != parent {
                        self.trigger_load_with_focus(parent, Some(current_root.to_path_buf()));
                    }
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
                if let Some(path) = self.get_root_path() {
                    return Some(Msg::GeneralSearch(GSMsg::PopupShowLibrary(
                        path.to_path_buf(),
                    )));
                }
                CmdResult::None
            }

            Event::Keyboard(keyevent) if keyevent == keys.library_keys.youtube_search.get() => {
                return Some(Msg::YoutubeSearch(YSMsg::InputPopupShow));
            }

            // load into playlist
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.load_dir.get() => {
                if let Some(path) = self.get_selected_path()
                    && path.is_dir()
                {
                    return Some(Msg::Playlist(PLMsg::Add(path.to_path_buf())));
                }
                CmdResult::None
            }
            Event::Keyboard(keyevent) if keyevent == keys.library_keys.load_track.get() => {
                if let Some(path) = self.get_selected_path()
                    && !path.is_dir()
                {
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
                if let Some(path) = self.get_selected_path()
                    && !path.is_dir()
                {
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
