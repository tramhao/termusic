/// The TUI Settings to use, with possible overwrite (like from CLI)
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct TuiOverlay {
    /// The saved TUI-Settings
    pub settings: super::v2::tui::TuiSettings,

    /// Disable TUI images (like cover-art) from being displayed in the terminal
    ///
    /// (disables ueberzug, sixel, iterm, kitty image displaying)
    pub disable_tui_images: bool,
}
