mod v1;
pub mod v2;
mod yaml_theme;

mod server_overlay;
mod tui_overlay;

use parking_lot::RwLock;
use std::sync::Arc;

pub use server_overlay::ServerOverlay;
pub use tui_overlay::TuiOverlay;

/// The Server-Settings Object, but shared across many places
// Note that this (at least currently) unused in lib itself, but used in many of the other dependant crates (playback, server, tui)
pub type SharedServerSettings = Arc<RwLock<ServerOverlay>>;

/// The Server-Settings Object, but shared across many places
// Note that this (at least currently) unused in lib itself, but used in many of the other dependant crates (playback, server, tui)
pub type SharedTuiSettings = Arc<RwLock<TuiOverlay>>;

/// Create a new [`SharedServerSettings`] without having to also depend on [`parking_lot`]
#[inline]
#[must_use]
pub fn new_shared_server_settings(settings: ServerOverlay) -> SharedServerSettings {
    Arc::new(RwLock::new(settings))
}

/// Create a new [`SharedTuiSettings`] without having to also depend on [`parking_lot`]
#[inline]
#[must_use]
pub fn new_shared_tui_settings(settings: TuiOverlay) -> SharedTuiSettings {
    Arc::new(RwLock::new(settings))
}
