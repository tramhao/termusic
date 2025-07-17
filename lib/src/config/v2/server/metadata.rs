use serde::{Deserialize, Serialize};

use crate::config::v2::server::ScanDepth;

/// Settings specific for the metadata scanner & parser.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)] // allow missing fields and fill them with the `..Self::default()` in this struct
pub struct MetadataSettings {
    /// How deep the metadata scanner should go from the root of a given directory.
    ///
    /// It is recommended to keep this relatively high, or at least as how deep a given music directory root can be.
    ///
    /// Note that the Metadata Scanner is in the background and should never block any event processing (like the TUI)
    pub directory_scan_depth: ScanDepth,
    /// Separators to use to split a given Artist and Album Artist into multiple.
    ///
    /// This is used for artists if there is no `TXX:ARTISTS`(or equivalent) tag.
    /// This is used for album artists if there is no `TXX:ALBUMARTISTS`(or equivalent) tag.
    ///
    /// Note that values can contain spaces for example for `ArtistA x ArtistB`.
    ///
    /// After split, the Artist values are trimmed.
    pub artist_separators: Vec<String>,
}

/// The default and most common separators used for artists.
pub const DEFAULT_ARTIST_SEPARATORS: &[&str] =
    &[",", ";", "&", "ft.", "feat.", "/", "|", "×", "、", " x "];

impl Default for MetadataSettings {
    fn default() -> Self {
        Self {
            directory_scan_depth: ScanDepth::Limited(10),
            artist_separators: DEFAULT_ARTIST_SEPARATORS
                .iter()
                .map(ToString::to_string)
                .collect(),
        }
    }
}
