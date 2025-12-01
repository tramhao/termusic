//! Functions to walk a given path and generate data for the component

use std::path::{Path, PathBuf};

use termusiclib::{config::v2::server::ScanDepth, utils::get_pin_yin};
use tuirealm_orx_tree::types::{NodeIdx, Tree};

use crate::ui::{
    components::orx_music_library::music_library::MusicLibData,
    model::{DownloadTracker, TxToMain},
    msg::{LIMsg, LINodeReady, Msg, RecVec},
};

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
            download_tracker.decrease_one(&path.to_string_lossy());
        })
        .expect("Failed to spawn thread");
}

/// Execute a library scan on a different thread.
///
/// Executes [`library_dir_tree`] on a different thread and send a [`LIMsg::TreeNodeReady`] on finish
pub fn library_scan<P: Into<PathBuf>>(
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
#[inline]
pub fn library_dir_tree(path: &Path, depth: ScanDepth) -> RecVec {
    library_dir_tree_inner(path, depth, None)
}

/// Scan the given `path` for up to `depth`, and return a [`Node`] tree.
///
/// Note: consider using [`library_scan`] instead of this directly for running in a different thread.
fn library_dir_tree_inner(path: &Path, depth: ScanDepth, is_dir: Option<bool>) -> RecVec {
    let is_dir = is_dir.unwrap_or_else(|| path.is_dir());
    let mut node = RecVec {
        path: path.to_path_buf(),
        is_dir,
        children: Vec::new(),
    };

    let depth = match depth {
        ScanDepth::Limited(v) => v,
        // put some kind of limit on it, thought the stack will likely overflow before this
        ScanDepth::Unlimited => u32::MAX,
    };

    if depth > 0
        && path.is_dir()
        && let Ok(paths) = std::fs::read_dir(path)
    {
        let mut paths: Vec<(String, (PathBuf, bool))> = paths
            .filter_map(std::result::Result::ok)
            // filter out hidden files
            .filter(|p| !p.file_name().to_string_lossy().starts_with('.'))
            .map(|v| {
                let sort_str = get_pin_yin(&v.file_name().to_string_lossy());
                let is_dir = v.file_type().is_ok_and(|v| v.is_dir());
                let path = v.path();
                (sort_str, (path, is_dir))
            })
            .collect();

        paths.sort_by(|a, b| alphanumeric_sort::compare_str(&a.0, &b.0));

        for (_sort_str, (path, is_dir)) in paths {
            node.children.push(library_dir_tree_inner(
                &path,
                ScanDepth::Limited(depth - 1),
                Some(is_dir),
            ));
        }
    }
    node
}

/// Convert a [`RecVec`] to a [`Node`].
///
/// Returns the root nodeidx.
pub fn recvec_to_tree(vec: RecVec) -> (NodeIdx<MusicLibData>, Tree<MusicLibData>) {
    let mut tree = Tree::default();

    (recvec_to_node_rec(vec, None, &mut tree), tree)
}

/// Convert the given `vec` to be child on `parent_node`.
///
/// If `parent_node` is `None` the new node will be pushed as the root.
pub fn recvec_to_node_rec(
    vec: RecVec,
    parent_node: Option<&tuirealm_orx_tree::types::NodeIdx<MusicLibData>>,
    tree: &mut tuirealm_orx_tree::types::Tree<MusicLibData>,
) -> NodeIdx<MusicLibData> {
    let is_dir = vec.path.is_dir();
    let nodeidx = if let Some(idx) = parent_node {
        tree.get_node_mut(idx)
            .unwrap()
            .push_child(MusicLibData::new(vec.path, is_dir))
    } else {
        tree.push_root(MusicLibData::new(vec.path, is_dir))
    };

    for val in vec.children {
        recvec_to_node_rec(val, Some(&nodeidx), tree);
    }

    nodeidx
}
