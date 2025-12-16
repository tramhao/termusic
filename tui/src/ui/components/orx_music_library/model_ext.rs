//! Music Library extensions on [`Model`]

use std::{
    fs::{remove_dir_all, remove_file},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use termusiclib::config::v2::server::{ScanDepth, config_extra::ServerConfigVersionedDefaulted};
use tuirealm::{
    Sub, SubClause, SubEventClause,
    props::{TableBuilder, TextSpan},
};

use crate::ui::{
    components::orx_music_library::{
        music_library::OrxMusicLibraryComponent, scanner::library_scan,
    },
    ids::Id,
    model::{Model, UserEvent},
    msg::{LIMsg, LINodeReady, LINodeReadySub, LIReloadData, LIReloadPathData, Msg},
    tui_cmd::TuiCmd,
};

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

impl Model {
    /// Mount the Orx Music library
    pub fn mount_new_library(&mut self) -> Result<()> {
        self.app.mount(
            Id::Library,
            Box::new(OrxMusicLibraryComponent::new_loading(
                self.config_tui.clone(),
                self.tx_to_main.clone(),
                self.download_tracker.clone(),
            )),
            library_subs(),
        )?;

        Ok(())
    }

    /// Execute [`library_scan`] from a `&self` instance.
    ///
    /// Executes [`library_dir_tree`](super::scanner::library_dir_tree) on a different thread and send a [`LIMsg::TreeNodeReady`] on finish.
    #[inline]
    pub fn new_library_scan_dir<P: Into<PathBuf>>(&self, path: P, focus_node: Option<String>) {
        library_scan(
            self.download_tracker.clone(),
            path,
            ScanDepth::Limited(2),
            self.tx_to_main.clone(),
            focus_node,
        );
    }

    /// Reload the given path in the library and focus that node.
    ///
    /// Also re-indexes the path for the database if it is part of a music root.
    ///
    /// The input path is expected to be absolute.
    #[expect(clippy::unnecessary_debug_formatting)]
    pub fn new_library_reload_and_focus<P: Into<PathBuf>>(&mut self, path: P) {
        let path = path.into();
        if !path.is_absolute() {
            debug!("library reload, given path is not absolute! {path:#?}");
        }

        let config_read = self.config_server.read_recursive();
        for dir in &self.config_server.read().settings.player.music_dirs {
            let absolute_dir = shellexpand::path::tilde(dir);

            if path.starts_with(absolute_dir)
                && let Err(err) = self.db.scan_path(&path, &config_read, false)
            {
                error!("Error scanning path {:#?}: {err:#?}", path.display());
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
    pub fn new_library_show_delete_confirm(&mut self, path: PathBuf, focus_node: Option<String>) {
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
    pub fn new_library_delete_node(
        &mut self,
        path: &Path,
        focus_node: Option<String>,
    ) -> Result<()> {
        if path.is_file() {
            remove_file(path)?;
        } else {
            path.canonicalize()?;
            remove_dir_all(path)?;
        }

        // always scan the parent, as otherwise, if the deleted "path" is the root
        // we end up never actually loading something correct and still have the stale tree
        let parent = path.parent().expect("Path to have a parent");

        self.new_library_scan_dir(parent, focus_node);

        // this line remove the deleted songs from playlist
        self.playlist_update_library_delete();
        Ok(())
    }

    /// Generate the result table for search `input`, recursively from the tree's root node's path.
    pub fn new_library_update_search(&mut self, input: &str, path: &Path) {
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
    pub fn new_library_switch_root(&mut self, old_path: &Path) {
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
            self.new_library_scan_dir(pathbuf, None);
        }
    }

    /// Add the given path as a new library root for quick switching & metadata(database) scraping.
    pub fn new_library_add_root<P: Into<PathBuf>>(&mut self, path: P) -> Result<()> {
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
    pub fn new_library_remove_root<P: Into<PathBuf>>(&mut self, path: P) -> Result<()> {
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

        self.new_library_switch_root(&path);

        res.context("Error while saving config")?;

        Ok(())
    }
}
