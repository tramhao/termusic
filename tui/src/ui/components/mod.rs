/**
 * MIT License
 *
 * tui-realm - Copyright (C) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
// -- modules
mod config_editor;
mod database;
mod footer;
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
mod global_listener;

// -- export
pub use config_editor::*;
pub use database::{DBListCriteria, DBListSearchResult, DBListSearchTracks};
pub use footer::Footer;
pub use labels::{DownloadSpinner, LabelGeneric, LabelSpan};
pub use lyric::Lyric;
pub use music_library::MusicLibrary;
pub use playlist::Playlist;
pub use podcast::{EpisodeList, FeedsList};
pub use popups::general_search::{GSInputPopup, GSTablePopup, Source};
pub use progress::Progress;
pub use tag_editor::*;
pub use global_listener::GlobalListener;
