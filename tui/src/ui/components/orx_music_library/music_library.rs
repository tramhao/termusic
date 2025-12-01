//! The actual Music Library Component Implementation

use std::{
    cell::{OnceCell, RefCell},
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

use anyhow::Result;
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
    types::{NodeValue, Tree},
    widget::{CHILD_INDICATOR_LENGTH, RenderIndicator, calc_area_for_value},
};

use crate::ui::{
    components::orx_music_library::scanner::library_scan,
    model::{DownloadTracker, TxToMain, UserEvent},
    msg::{
        GSMsg, LIMsg, LINodeReady, LINodeReadySub, LIReloadData, LIReloadPathData, Msg, PLMsg,
        TEMsg, YSMsg,
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
    is_loading: RefCell<bool>,
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
            is_loading: RefCell::new(false),
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
        } else if !(*self.is_loading.borrow()) {
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
pub struct NewMusicLibraryComponent {
    component: TreeView<MusicLibData>,
    config: SharedTuiSettings,
}

impl NewMusicLibraryComponent {
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
        // .on_open(|event, idx, tree| {
        //     debug!("on_open {:#?}", (event, idx /* , tree */));

        //     if tree.get_root().is_some_and(|v| v.idx() == idx) {
        //         return;
        //     }

        //     let mut node = tree.get_node_mut(&idx).unwrap();
        //     let mut traverser = Dfs::<OverData>::new();
        //     // inital tree walker
        //     let walker = node.walk_mut_with(&mut traverser);
        //     // filter only for leafs
        //     let walker = walker.skip(1);
        //     // filter only directories & not loading
        //     let walker = walker.filter(|v| v.is_dir && !v.is_loading);
        //     for node in walker {
        //         debug!("on_open found: {:#?}", node.path.file_name().unwrap());
        //         node.is_loading = true;
        //         scan_path_parent(tx, download_tracker, path, depth, parent);
        //     }
        // })
    }

    /// Get a new empty instance, which shows "Loading..." while empty.
    pub fn new_loading(config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();

            Self::get_inner_comp(&config)
        };

        Self { component, config }
    }

    /// Create a new instance, with a tree already set.
    pub fn new(
        tree: Tree<MusicLibData>,
        config: SharedTuiSettings,
        tx: TxToMain,
        tracker: DownloadTracker,
    ) -> Self {
        let mut this = Self::new_loading(config);

        this.component = this.component.tree(tree).on_open(move |event, idx, tree| {
            debug!("on_open {:#?}", (event, idx /* , tree */));

            if tree.get_root().is_some_and(|v| v.idx() == idx) {
                return;
            }

            let node = tree.get_node_mut(&idx).unwrap();
            let mut traverser = Dfs::<OverNode>::new();
            // inital tree walker
            let walker = node.walk_with(&mut traverser);
            // filter only for leafs
            let walker = walker.skip(1);
            // filter only directories & not loading
            let walker = walker.filter(|v| v.data().is_dir && !(*v.data().is_loading.borrow()));
            for node in walker {
                let idx = node.idx();
                let data = node.data();
                debug!("on_open found: {:#?}", data.path.file_name().unwrap());
                *data.is_loading.borrow_mut() = true;
                library_scan(
                    tracker.clone(),
                    &data.path,
                    ScanDepth::Limited(2),
                    tx.clone(),
                    Some(idx),
                );
            }
        });

        debug!("what {this:#?}");

        this
    }

    /// Trigger a load with a message to change the tree root to the given path.
    ///
    /// This will make the given path(which will be the root node) the focused node.
    ///
    /// This will send a [`LIMsg::TreeNodeReady`] and change the root to `path`.
    fn trigger_load_stepinto<P: Into<PathBuf>>(&self, path: P) {
        todo!();
        // library_scan(
        //     self.download_tracker.clone(),
        //     path,
        //     ScanDepth::Limited(2),
        //     self.tx_to_main.clone(),
        //     None,
        // );
    }

    /// Trigger a load with a message to change the tree root to the given path.
    ///
    /// This will make the current tree root be the new focused node.
    ///
    /// This will send a [`LIMsg::TreeNodeReady`] and change the root to `path`.
    fn trigger_load_with_focus<P: Into<PathBuf>>(&self, scan_path: P, focus_node: Option<PathBuf>) {
        todo!();
        // let path = scan_path.into();
        // library_scan(
        //     self.download_tracker.clone(),
        //     path,
        //     ScanDepth::Limited(2),
        //     self.tx_to_main.clone(),
        //     focus_node,
        // );
    }

    /// Store the currently selected node as yanked (for pasting with [`Self::paste`]).
    fn yank(&mut self) {
        todo!();
    }

    /// Paste the previously yanked node in the currently selected node if it is a directory, otherwise in its parent.
    fn paste(&mut self) -> Result<Option<LIMsg>> {
        todo!();
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
        todo!();
    }

    /// Also known as going down the tree / adding file to playlist
    fn handle_right_key(&mut self) -> (CmdResult, Option<Msg>) {
        todo!();
    }

    /// Handle sending a request to delete the currently selected node.
    fn handle_delete(&mut self) -> Msg {
        todo!();
    }

    /// Handle a full reload / potential change of the current tree root.
    ///
    /// Also changes focus, if requested.
    fn handle_full_reload(&mut self, data: LIReloadData) -> Msg {
        todo!();
    }

    /// Handle reloading of the given path, potentially without changing root, but also change focus.
    ///
    /// If necessary, load all paths in-between.
    fn handle_reload_at(&mut self, data: LIReloadPathData) {
        todo!();
    }

    /// Apply the given data as the root of the tree, resetting the state of the tree.
    ///
    /// This will always replace the root of the tree.
    fn handle_ready(&mut self, data: LINodeReady) -> Msg {
        todo!();
    }

    /// Apply the given data at the path the data is, potentially without changing root.
    ///
    /// This will replace the root if the given data is starting at the root path.
    fn handle_ready_sub(&mut self, data: LINodeReadySub) -> Option<Msg> {
        todo!();
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

impl Component<Msg, UserEvent> for NewMusicLibraryComponent {
    #[allow(clippy::too_many_lines)]
    #[allow(unsafe_code)]
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
                let Some(current_root) = self.get_root_path() else {
                    return None;
                };
                let parent = current_root.parent().unwrap_or(current_root);

                // only trigger a load if we are not at the root of the filesystem already
                if current_root != parent {
                    self.trigger_load_with_focus(parent, Some(current_root.to_path_buf()));
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
