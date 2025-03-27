/// The TUI Settings to use, with possible overwrite (like from CLI)
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct TuiOverlay {
    /// The saved TUI-Settings
    pub settings: super::v2::tui::TuiSettings,

    /// Disable TUI images (like cover-art) from being displayed in the terminal
    ///
    /// This option will not be saved to the config and prevent saving to the config value
    ///
    /// (disables ueberzug, sixel, iterm, kitty image displaying)
    pub coverart_hidden_overwrite: Option<bool>,

    /// Enable/Disable checking for cover support.
    ///
    /// If `false`, will treat as if no cover features are compiled-in.
    pub cover_features: bool,
}

impl TuiOverlay {
    /// Get whether the coverart should be hidden or not, either the overwrite if present, otherwise the config itself
    ///
    /// true => hidden
    #[must_use]
    pub fn get_coverart_hidden(&self) -> bool {
        if let Some(overwrite) = self.coverart_hidden_overwrite {
            overwrite
        } else {
            self.settings.coverart.hidden
        }
    }

    /// Get whether cover features should be enabled or not, regardless if they are compiled-in or not.
    #[must_use]
    pub fn cover_features_enabled(&self) -> bool {
        self.cover_features
    }
}
