use std::path::{Path, PathBuf};

use super::v2::server::ScanDepth;

/// The Server Settings to use, with possible overwrite (like from CLI)
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct ServerOverlay {
    /// The saved Server-Settings
    pub settings: super::v2::server::ServerSettings,

    /// Overwrite what music directory should be opened first
    ///
    /// This music dir will not be saved to the config
    // Note that this is basically unused in the server currently, but used in the TUI
    // but it is here because the Server has all the music-dirs and in the future it should handle the roots
    pub music_dir_overwrite: Option<PathBuf>,
    /// Overwrite disabling the discord status setting
    pub disable_discord_status: bool,
    /// Overwrite the Metadata scan depth
    pub metadata_scan_depth: Option<ScanDepth>,
}

impl ServerOverlay {
    /// Get the Library scan depth, either the overwrite if present, otherwise the config itself
    #[must_use]
    pub fn get_metadata_scan_depth(&self) -> ScanDepth {
        if let Some(v) = self.metadata_scan_depth {
            v
        } else {
            self.settings.metadata.directory_scan_depth
        }
    }

    /// Get whether to enable the discord status
    #[must_use]
    pub fn get_discord_status_enable(&self) -> bool {
        if self.disable_discord_status {
            false
        } else {
            self.settings.player.set_discord_status
        }
    }

    /// Get the first music dir to use, either the overwrite if present, otherwise the config's first music music (if any)
    pub fn get_first_music_dir(&self) -> Option<&Path> {
        if let Some(ref overwrite) = self.music_dir_overwrite {
            Some(overwrite)
        } else {
            self.settings
                .player
                .music_dirs
                .first()
                .map(PathBuf::as_path)
        }
    }
}
