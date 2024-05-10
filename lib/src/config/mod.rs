mod v1;
pub mod v2;
mod yaml_theme;

mod server_overlay;
mod tui_overlay;

use parking_lot::RwLock;
use std::sync::Arc;

pub use v1::{
    Alacritty, Alignment, BindingForEvent, ColorTermusic, Keys, LastPosition, Loop, SeekStep,
    Settings, StyleColorSymbol, Xywh,
};

pub use server_overlay::ServerOverlay;
pub use tui_overlay::TuiOverlay;

/// The Settings Object, but shared across many places
// Note that this (at least currently) unused in lib itself, but used in many of the other dependant crates (playback, server, tui)
pub type SharedSettings = Arc<RwLock<Settings>>;

/// Create a new [`SharedSettings`] from just [`Settings`] without having to also depend on [`parking_lot`]
#[inline]
pub fn new_shared_settings(settings: Settings) -> SharedSettings {
    Arc::new(RwLock::new(settings))
}
