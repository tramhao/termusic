//! The actual Music Library Component Implementation

use std::{
    cell::{OnceCell, RefCell},
    path::PathBuf,
};

use tuirealm::{
    props::Style,
    ratatui::{
        buffer::Buffer,
        layout::Rect,
        widgets::{Clear, Widget},
    },
};
use tuirealm_orx_tree::{
    types::NodeValue,
    widget::{CHILD_INDICATOR_LENGTH, RenderIndicator, calc_area_for_value},
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
