// -- modules
mod config_editor;
mod database;
mod footer;
mod global_listener;
mod labels;
mod lyric;
mod music_library;
mod playlist;
mod podcast;
mod popups;
mod progress;
#[allow(
    clippy::match_bool,
    clippy::redundant_closure_for_method_calls,
    clippy::doc_markdown,
    clippy::module_name_repetitions
)]
/// Tag Editor Controls
mod tag_editor;
mod xywh;

// -- export
pub use config_editor::*;
pub use database::{DBListCriteria, DBListSearchResult, DBListSearchTracks};
pub use footer::Footer;
pub use global_listener::GlobalListener;
pub use labels::{DownloadSpinner, LabelGeneric, LabelSpan};
pub use lyric::Lyric;
pub use music_library::MusicLibrary;
pub use playlist::Playlist;
pub use podcast::{EpisodeList, FeedsList};
pub use popups::general_search::{GSInputPopup, GSTablePopup, Source};
pub use progress::Progress;
pub use tag_editor::*;
