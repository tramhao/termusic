use std::borrow::Cow;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context as _, Result, anyhow, bail};
use rand::seq::IndexedRandom;
use termusiclib::common::const_unknown::{UNKNOWN_ALBUM, UNKNOWN_ARTIST};
use termusiclib::config::SharedTuiSettings;
use termusiclib::config::v2::server::{LoopMode, ScanDepth};
use termusiclib::new_database::track_ops::TrackRead;
use termusiclib::new_database::{album_ops, track_ops};
use termusiclib::player::playlist_helpers::{
    PlaylistAddTrack, PlaylistPlaySpecific, PlaylistRemoveTrackIndexed, PlaylistSwapTrack,
    PlaylistTrackSource,
};
use termusiclib::player::{
    PlaylistAddTrackInfo, PlaylistLoopModeInfo, PlaylistRemoveTrackInfo, PlaylistShuffledInfo,
    PlaylistSwapInfo,
};
use termusiclib::track::Track;
use termusiclib::track::{DurationFmtShort, PodcastTrackData};
use termusiclib::utils::{filetype_supported, get_parent_folder, is_playlist, playlist_get_vec};
use tui_realm_stdlib::Table;
use tuirealm::props::Borders;
use tuirealm::props::{Alignment, BorderType, PropPayload, PropValue, TableBuilder, TextSpan};
use tuirealm::{
    AttrValue, Attribute, Component, Event, MockComponent, State, StateValue,
    event::{Key, KeyEvent},
};
use tuirealm::{
    command::{Cmd, CmdResult, Direction, Position},
    event::KeyModifiers,
};

use crate::ui::Model;
use crate::ui::components::orx_music_library::scanner::library_dir_tree;
use crate::ui::ids::Id;
use crate::ui::model::{TermusicLayout, UserEvent};
use crate::ui::msg::{GSMsg, Msg, PLMsg, SearchCriteria};
use crate::ui::tui_cmd::{PlaylistCmd, TuiCmd};

#[derive(MockComponent)]
pub struct Playlist {
    component: Table,
    config: SharedTuiSettings,
}

impl Playlist {
    pub fn new(config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();
            Table::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(config.settings.theme.playlist_border()),
                )
                .background(config.settings.theme.playlist_background())
                .foreground(config.settings.theme.playlist_foreground())
                .title(" Playlist ", Alignment::Left)
                .scroll(true)
                .highlighted_color(config.settings.theme.playlist_highlight())
                .highlighted_str(&config.settings.theme.style.playlist.highlight_symbol)
                .rewind(false)
                .step(4)
                .row_height(1)
                .headers(["Duration", "Artist", "Title", "Album"])
                .column_spacing(2)
                .widths(&[12, 20, 25, 43])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty"))
                        .add_col(TextSpan::from("Empty Queue"))
                        .add_col(TextSpan::from("Empty"))
                        .build(),
                )
        };

        Self { component, config }
    }
}

impl Component<Msg, UserEvent> for Playlist {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(key) if key == keys.navigation_keys.down.get() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.up.get() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == keys.navigation_keys.goto_top.get() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == keys.navigation_keys.goto_bottom.get() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::End)),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::Playlist(PLMsg::PlaylistTableBlurDown)),
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::Playlist(PLMsg::PlaylistTableBlurUp)),
            Event::Keyboard(key) if key == keys.playlist_keys.delete.get() => {
                match self.component.state() {
                    State::One(StateValue::Usize(index_selected)) => {
                        return Some(Msg::Playlist(PLMsg::Delete(index_selected)));
                    }
                    _ => CmdResult::None,
                }
            }
            Event::Keyboard(key) if key == keys.playlist_keys.delete_all.get() => {
                return Some(Msg::Playlist(PLMsg::DeleteAll));
            }
            Event::Keyboard(key) if key == keys.playlist_keys.shuffle.get() => {
                return Some(Msg::Playlist(PLMsg::Shuffle));
            }
            Event::Keyboard(key) if key == keys.playlist_keys.cycle_loop_mode.get() => {
                return Some(Msg::Playlist(PLMsg::LoopModeCycle));
            }
            Event::Keyboard(key) if key == keys.playlist_keys.play_selected.get() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Playlist(PLMsg::PlaySelected(index)));
                }
                CmdResult::None
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Playlist(PLMsg::PlaySelected(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(key) if key == keys.playlist_keys.search.get() => {
                return Some(Msg::GeneralSearch(GSMsg::PopupShowPlaylist));
            }
            Event::Keyboard(key) if key == keys.playlist_keys.swap_down.get() => {
                match self.component.state() {
                    State::One(StateValue::Usize(index_selected)) => {
                        self.perform(Cmd::Move(Direction::Down));
                        return Some(Msg::Playlist(PLMsg::SwapDown(index_selected)));
                    }
                    _ => CmdResult::None,
                }
            }
            Event::Keyboard(key) if key == keys.playlist_keys.swap_up.get() => {
                match self.component.state() {
                    State::One(StateValue::Usize(index_selected)) => {
                        self.perform(Cmd::Move(Direction::Up));
                        return Some(Msg::Playlist(PLMsg::SwapUp(index_selected)));
                    }
                    _ => CmdResult::None,
                }
            }
            Event::Keyboard(key) if key == keys.playlist_keys.add_random_album.get() => {
                return Some(Msg::Playlist(PLMsg::AddRandomAlbum));
            }
            Event::Keyboard(key) if key == keys.playlist_keys.add_random_songs.get() => {
                return Some(Msg::Playlist(PLMsg::AddRandomTracks));
            }
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

impl Model {
    pub fn playlist_reload(&mut self) {
        assert!(
            self.app
                .remount(
                    Id::Playlist,
                    Box::new(Playlist::new(self.config_tui.clone())),
                    Vec::new()
                )
                .is_ok()
        );
        self.playlist_switch_layout();
        self.playlist_sync();
    }

    pub fn playlist_switch_layout(&mut self) {
        if self.layout == TermusicLayout::Podcast {
            let headers = &["Duration", "Episodes"];
            self.app
                .attr(
                    &Id::Playlist,
                    Attribute::Text,
                    AttrValue::Payload(PropPayload::Vec(
                        headers
                            .iter()
                            .map(|x| PropValue::Str((*x).to_string()))
                            // .map(|x| PropValue::Str(x.as_ref().to_string()))
                            .collect(),
                    )),
                )
                .ok();

            let widths = &[12, 88];
            self.app
                .attr(
                    &Id::Playlist,
                    Attribute::Width,
                    AttrValue::Payload(PropPayload::Vec(
                        widths.iter().map(|x| PropValue::U16(*x)).collect(),
                    )),
                )
                .ok();
            self.playlist_sync();
            return;
        }

        let headers = &["Duration", "Artist", "Title", "Album"];
        self.app
            .attr(
                &Id::Playlist,
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(
                    headers
                        .iter()
                        .map(|x| PropValue::Str((*x).to_string()))
                        // .map(|x| PropValue::Str(x.as_ref().to_string()))
                        .collect(),
                )),
            )
            .ok();

        let widths = &[12, 20, 25, 43];
        self.app
            .attr(
                &Id::Playlist,
                Attribute::Width,
                AttrValue::Payload(PropPayload::Vec(
                    widths.iter().map(|x| PropValue::U16(*x)).collect(),
                )),
            )
            .ok();
        self.playlist_sync();
    }

    /// Add a playlist (like m3u) to the playlist.
    fn playlist_add_playlist(&mut self, playlist_path: &Path) -> Result<()> {
        let vec = playlist_get_vec(playlist_path)?;

        let sources = vec
            .into_iter()
            .map(|v| {
                if v.starts_with("http") {
                    PlaylistTrackSource::Url(v)
                } else {
                    PlaylistTrackSource::Path(v)
                }
            })
            .collect();

        self.command(TuiCmd::Playlist(PlaylistCmd::AddTrack(
            PlaylistAddTrack::new_vec(
                u64::try_from(self.playback.playlist.len()).unwrap(),
                sources,
            ),
        )));

        Ok(())
    }

    /// Add a podcast episode to the playlist.
    pub fn playlist_add_episode(&mut self, episode_index: usize) -> Result<()> {
        if self.podcast.podcasts.is_empty() {
            return Ok(());
        }
        let podcast_selected = self
            .podcast
            .podcasts
            .get(self.podcast.podcasts_index)
            .ok_or_else(|| anyhow!("get podcast selected failed."))?;
        let episode_selected = podcast_selected
            .episodes
            .get(episode_index)
            .ok_or_else(|| anyhow!("get episode selected failed."))?;

        let source = PlaylistTrackSource::PodcastUrl(episode_selected.url.clone());
        self.command(TuiCmd::Playlist(PlaylistCmd::AddTrack(
            PlaylistAddTrack::new_single(
                u64::try_from(self.playback.playlist.len()).unwrap(),
                source,
            ),
        )));

        Ok(())
    }

    fn playlist_get_dir_entries(path: &Path) -> Vec<String> {
        // use the same function as the tree order gets generated in, so that we add in a expected order
        let vec = library_dir_tree(path, ScanDepth::Limited(1));
        vec.children
            .into_iter()
            .map(|v| v.path.to_string_lossy().to_string())
            .collect()
    }

    /// Add the `current_node`, regardless if it is a Track, dir, playlist, etc.
    ///
    /// See [`Model::playlist_add_episode`] for podcast episode adding
    pub fn playlist_add(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        if path.is_dir() {
            let new_items_vec = Self::playlist_get_dir_entries(path);

            let sources = new_items_vec
                .into_iter()
                .map(PlaylistTrackSource::Path)
                .collect();

            self.command(TuiCmd::Playlist(PlaylistCmd::AddTrack(
                PlaylistAddTrack::new_vec(
                    u64::try_from(self.playback.playlist.len()).unwrap(),
                    sources,
                ),
            )));

            return Ok(());
        }
        self.playlist_add_item(path)?;
        self.playlist_sync();
        Ok(())
    }

    /// Add a Track or a Playlist to the playlist
    fn playlist_add_item(&mut self, path: &Path) -> Result<()> {
        if is_playlist(path) {
            self.playlist_add_playlist(path)?;
            return Ok(());
        }
        let source = if path.starts_with("http") {
            PlaylistTrackSource::Url(path.to_string_lossy().to_string())
        } else {
            PlaylistTrackSource::Path(path.to_string_lossy().to_string())
        };

        self.command(TuiCmd::Playlist(PlaylistCmd::AddTrack(
            PlaylistAddTrack::new_single(
                u64::try_from(self.playback.playlist.len()).unwrap(),
                source,
            ),
        )));

        Ok(())
    }

    /// Add [`TrackDB`] to the playlist
    pub fn playlist_add_all_from_db(&mut self, vec: &[TrackRead]) {
        let sources = vec
            .iter()
            .map(|f| PlaylistTrackSource::Path(f.as_pathbuf().to_string_lossy().to_string()))
            .collect();

        self.command(TuiCmd::Playlist(PlaylistCmd::AddTrack(
            PlaylistAddTrack::new_vec(
                u64::try_from(self.playback.playlist.len()).unwrap(),
                sources,
            ),
        )));
    }

    /// Add random album(s) from the database to the playlist
    pub fn playlist_add_random_album(&mut self) {
        let playlist_select_random_album_quantity = self
            .config_server
            .read()
            .settings
            .player
            .random_album_min_quantity
            .get();
        let vec = self.playlist_get_random_album_tracks(playlist_select_random_album_quantity);
        self.playlist_add_all_from_db(&vec);
    }

    /// Add random tracks from the database to the playlist
    pub fn playlist_add_random_tracks(&mut self) {
        let playlist_select_random_track_quantity = self
            .config_server
            .read()
            .settings
            .player
            .random_track_quantity
            .get();
        let vec = self.playlist_get_random_tracks(playlist_select_random_track_quantity);
        self.playlist_add_all_from_db(&vec);
    }

    /// Handle when a playlist has added a track
    pub fn handle_playlist_add(&mut self, items: PlaylistAddTrackInfo) -> Result<()> {
        // piggyback off-of the server side implementation for now by re-parsing everything.
        self.playback.playlist.add_tracks(
            PlaylistAddTrack {
                at_index: items.at_index,
                tracks: vec![items.trackid],
            },
            &self.podcast.db_podcast,
        )?;

        self.playlist_sync();

        Ok(())
    }

    /// Handle when a playlist has removed a track
    pub fn handle_playlist_remove(&mut self, items: &PlaylistRemoveTrackInfo) -> Result<()> {
        self.playback.playlist.handle_grpc_remove(items)?;

        self.playlist_sync();

        Ok(())
    }

    /// Handle when a playlist was cleared
    pub fn handle_playlist_clear(&mut self) {
        self.playback.playlist.clear();

        self.playlist_sync();
    }

    /// Handle when the playlist loop-mode was changed
    pub fn handle_playlist_loopmode(&mut self, loop_mode: &PlaylistLoopModeInfo) -> Result<()> {
        let as_u8 = u8::try_from(loop_mode.mode).context("Failed to convert u32 to u8")?;
        let loop_mode =
            LoopMode::tryfrom_discriminant(as_u8).context("Failed to get LoopMode from u8")?;
        self.playback.playlist.set_loop_mode(loop_mode);
        self.config_server.write().settings.player.loop_mode = loop_mode;
        self.playlist_update_title();
        // Force a redraw as stream updates are not part of the "tick" event and so cant send "Msg"
        // but need a redraw because ofthe title change
        self.force_redraw();

        Ok(())
    }

    /// Handle when the playlist had swapped some tracks
    pub fn handle_playlist_swap_tracks(&mut self, swapped_tracks: &PlaylistSwapInfo) -> Result<()> {
        let index_a = usize::try_from(swapped_tracks.index_a)
            .context("Failed to convert index_a to usize")?;
        let index_b = usize::try_from(swapped_tracks.index_b)
            .context("Failed to convert index_b to usize")?;

        self.playback.playlist.swap(index_a, index_b)?;

        self.playlist_sync();

        Ok(())
    }

    /// Handle when the playlist has been shuffled and so has new order of tracks
    pub fn handle_playlist_shuffled(&mut self, shuffled: PlaylistShuffledInfo) -> Result<()> {
        let playlist_comp_selected_index = self.playlist_get_selected_index();
        // this might be fragile if there are multiple of the same track in the playlist as there is no unique identifier currently
        let playlist_track_at_old_file = playlist_comp_selected_index
            .and_then(|idx| self.playback.playlist.tracks().get(idx))
            .map(Track::as_track_source);

        self.playback
            .load_from_grpc(shuffled.tracks, &self.podcast.db_podcast)?;
        self.playlist_sync();

        if let Some(old_id) = playlist_track_at_old_file {
            let found_new_index = self
                .playback
                .playlist
                .tracks()
                .iter()
                .enumerate()
                .find(|(_, track)| *track == old_id);
            if let Some((new_index, _)) = found_new_index {
                self.playlist_locate(new_index);
            }
        }

        Ok(())
    }

    /// Handle setting the current track index in the TUI playlist and selecting the proper list item
    ///
    /// Note: currently this function is called twice per track change, once for `UpdateEvents::TrackChanged` and once for `run_playback::GetProgress`
    pub fn handle_current_track_index(&mut self, current_track_index: usize, force_relocate: bool) {
        let tui_old_current_index = self.playback.playlist.current_track_index();
        info!(
            "index from player is: {current_track_index:?}, index in tui is: {tui_old_current_index:?}"
        );
        self.playback.clear_current_track();
        let _ = self
            .playback
            .playlist
            .set_current_track_index(current_track_index);
        self.playback.set_current_track_from_playlist();

        let playlist_comp_selected_index = self.playlist_get_selected_index();

        // only re-select the current-track if the old selection was the old-current-track
        if force_relocate
            || (tui_old_current_index.is_some()
                && playlist_comp_selected_index.is_none_or(|v| v == tui_old_current_index.unwrap()))
        {
            self.playlist_locate(current_track_index);
        }

        self.update_layout_for_current_track();
        self.playback.set_current_track_pos(Duration::ZERO);
        self.player_update_current_track_after();

        self.lyric_update_for_podcast_by_current_track();

        if let Err(e) = self.podcast_mark_current_track_played() {
            self.mount_error_popup(e.context("Marking podcast track as played"));
        }
    }

    fn playlist_sync_podcasts(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, track) in self.playback.playlist.tracks().iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let duration_str = if let Some(dur) = track.duration_str_short() {
                format!("[{dur:^7.7}]")
            } else {
                "[--:--]".to_string()
            };

            let mut title = track.title().unwrap_or("Unknown Title").to_string();
            if track
                .as_podcast()
                .is_some_and(PodcastTrackData::has_localfile)
            {
                title = format!("[D] {title}");
            }
            if Some(idx) == self.playback.playlist.current_track_index() {
                title = format!(
                    "{}{title}",
                    self.config_tui
                        .read()
                        .settings
                        .theme
                        .style
                        .playlist
                        .current_track_symbol
                );
            }
            table
                .add_col(TextSpan::new(duration_str.as_str()))
                .add_col(TextSpan::new(title).bold());
        }
        if self.playback.playlist.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty playlist"));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::Playlist,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();

        self.playlist_update_title();
    }

    pub fn playlist_sync(&mut self) {
        if self.layout == TermusicLayout::Podcast {
            self.playlist_sync_podcasts();
            return;
        }

        let mut table: TableBuilder = TableBuilder::default();

        for (idx, track) in self.playback.playlist.tracks().iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let duration_str = if let Some(dur) = track.duration_str_short() {
                format!("[{dur:^7.7}]")
            } else {
                "[--:--]".to_string()
            };

            let mut title: Cow<'_, str> = track.title().map_or_else(|| track.id_str(), Into::into);

            let artist = track.artist().unwrap_or(UNKNOWN_ARTIST);
            let album = track
                .as_track()
                .and_then(|v| v.album())
                .unwrap_or(UNKNOWN_ALBUM);

            // TODO: is there maybe a better option to do this on-demand instead of the whole playlist; like on draw-time?
            if Some(idx) == self.playback.playlist.current_track_index() {
                title = format!(
                    "{}{title}",
                    self.config_tui
                        .read()
                        .settings
                        .theme
                        .style
                        .playlist
                        .current_track_symbol
                )
                .into();
            }

            table
                .add_col(TextSpan::new(duration_str.as_str()))
                .add_col(TextSpan::new(artist).fg(tuirealm::ratatui::style::Color::LightYellow))
                .add_col(TextSpan::new(title).bold())
                .add_col(TextSpan::new(album));
        }
        if self.playback.playlist.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty playlist"));
            table.add_col(TextSpan::from(""));
            table.add_col(TextSpan::from(""));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::Playlist,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();

        self.playlist_update_title();
    }

    /// Delete a track at `index` from the playlist
    pub fn playlist_delete_item(&mut self, index: usize) {
        if self.playback.playlist.is_empty() || index >= self.playback.playlist.len() {
            return;
        }

        let Some(track) = self.playback.playlist.tracks().get(index) else {
            return;
        };

        let track_source = track.as_track_source();

        self.command(TuiCmd::Playlist(PlaylistCmd::RemoveTrack(
            PlaylistRemoveTrackIndexed::new_single(u64::try_from(index).unwrap(), track_source),
        )));
    }

    /// Clear a entire playlist
    pub fn playlist_clear(&mut self) {
        if self.playback.playlist.is_empty() {
            return;
        }

        self.command(TuiCmd::Playlist(PlaylistCmd::Clear));
    }

    /// Shuffle the whole playlist
    pub fn playlist_shuffle(&mut self) {
        self.command(TuiCmd::Playlist(PlaylistCmd::Shuffle));
    }

    /// Send command to swap 2 indexes. Does nothing if either index is out-of-bounds.
    ///
    /// # Panics
    ///
    /// if `usize` cannot be converted to `u64`
    fn playlist_swap(&mut self, index_a: usize, index_b: usize) {
        let len = self.playback.playlist.tracks().len();
        if index_a.max(index_b) >= len {
            error!(
                "Index out-of-bounds, not executing swap: {}",
                index_a.max(index_b)
            );
            return;
        }

        self.command(TuiCmd::Playlist(PlaylistCmd::SwapTrack(
            PlaylistSwapTrack {
                index_a: u64::try_from(index_a).unwrap(),
                index_b: u64::try_from(index_b).unwrap(),
            },
        )));
    }

    /// Swap the given index upwards, does nothing if out-of-bounds or would result in itself.
    pub fn playlist_swap_up(&mut self, index: usize) {
        if index == 0 {
            return;
        }

        // always guranteed to be above 0, no saturated necessary
        self.playlist_swap(index, index - 1);
    }

    /// Swap the given index downwards, does nothing if out-of-bounds.
    pub fn playlist_swap_down(&mut self, index: usize) {
        if index >= self.playback.playlist.len().saturating_sub(1) {
            return;
        }

        self.playlist_swap(index, index.saturating_add(1));
    }

    pub fn playlist_update_library_delete(&mut self) {
        self.command(TuiCmd::Playlist(PlaylistCmd::RemoveDeletedItems));
    }

    pub fn playlist_update_title(&mut self) {
        let duration = self
            .playback
            .playlist
            .tracks()
            .iter()
            .filter_map(Track::duration)
            .sum();
        let display_symbol = self
            .config_tui
            .read()
            .settings
            .theme
            .style
            .playlist
            .use_loop_mode_symbol;
        let loop_mode = self.config_server.read().settings.player.loop_mode;
        let title = format!(
            "\u{2500} Playlist \u{2500}\u{2500}\u{2524} Total {} tracks | {} | Mode: {} \u{251c}\u{2500}",
            self.playback.playlist.len(),
            DurationFmtShort(duration),
            loop_mode.display(display_symbol),
        );
        self.app
            .attr(
                &Id::Playlist,
                tuirealm::Attribute::Title,
                tuirealm::AttrValue::Title((title, Alignment::Left)),
            )
            .ok();
    }

    /// Play the currently selected item in the playlist list
    pub fn playlist_play_selected(&mut self, index: usize) {
        let Some(track) = self.playback.playlist.tracks().get(index) else {
            error!("Track {index} not in playlist!");
            return;
        };

        let track_source = track.as_track_source();

        self.command(TuiCmd::Playlist(PlaylistCmd::PlaySpecific(
            PlaylistPlaySpecific {
                track_index: u64::try_from(index).unwrap(),
                id: track_source,
            },
        )));
    }

    pub fn playlist_update_search(&mut self, input: &str) {
        let filtered_music = Model::update_search(self.playback.playlist.tracks(), input);
        self.general_search_update_show(Model::build_table(filtered_music));
    }

    /// Select the given index in the playlist list component
    pub fn playlist_locate(&mut self, index: usize) {
        assert!(
            self.app
                .attr(
                    &Id::Playlist,
                    Attribute::Value,
                    AttrValue::Payload(PropPayload::One(PropValue::Usize(index))),
                )
                .is_ok()
        );
    }

    /// Get the current selected index in the playlist list component
    pub fn playlist_get_selected_index(&self) -> Option<usize> {
        // the index on a "Table" can be set via "AttrValue::Payload(PropPayload::One(PropValue::Usize(val)))", but reading that is stale
        // as that value is only read in the "Table", not removed or updated, but "state" is
        let Ok(State::One(StateValue::Usize(val))) = self.app.state(&Id::Playlist) else {
            return None;
        };

        Some(val)
    }

    pub fn playlist_get_random_tracks(&mut self, quantity: u32) -> Vec<TrackRead> {
        let mut result = Vec::with_capacity(usize::try_from(quantity).unwrap_or_default());
        let all_tracks =
            track_ops::get_all_tracks(&self.db.get_connection(), track_ops::RowOrdering::IdAsc);
        if let Ok(vec) = all_tracks {
            let mut i = 0;
            loop {
                if let Some(record) = vec.choose(&mut rand::rng()) {
                    let path = record.as_pathbuf();
                    if filetype_supported(&path) {
                        result.push(record.clone());
                        i += 1;
                        if i > quantity - 1 {
                            break;
                        }
                    }
                }
            }
        }
        result
    }

    pub fn playlist_get_random_album_tracks(&mut self, quantity: u32) -> Vec<TrackRead> {
        let mut result = Vec::with_capacity(usize::try_from(quantity).unwrap_or_default());
        let all_albums =
            album_ops::get_all_albums(&self.db.get_connection(), album_ops::RowOrdering::IdAsc);
        if let Ok(vec) = all_albums {
            loop {
                if let Some(v) = vec.choose(&mut rand::rng()) {
                    let all_tracks_in_album = track_ops::get_tracks_from_album(
                        &self.db.get_connection(),
                        &v.title,
                        &v.artist_display,
                        track_ops::RowOrdering::IdAsc,
                    );
                    if let Ok(vec2) = all_tracks_in_album {
                        if vec2.len() < quantity as usize {
                            continue;
                        }

                        result.extend(vec2);

                        break;
                    }
                }
            }
        }
        result
    }

    /// Save the current playlist as m3u with the given `filename`
    pub fn playlist_save_m3u_before(&mut self, filename: PathBuf) -> Result<()> {
        let current_node: String = match self.app.state(&Id::Library).ok().unwrap() {
            State::One(StateValue::String(id)) => id,
            _ => bail!("Invalid node selected in library"),
        };

        let path_m3u = {
            let mut parent_folder = get_parent_folder(Path::new(&current_node)).to_path_buf();
            let mut filename = OsString::from(filename);
            filename.push(".m3u");
            parent_folder.push(filename);

            parent_folder
        };

        if path_m3u.exists() {
            self.mount_save_playlist_confirm(path_m3u)
                .expect("Expected SavePlaylistConfirm to mount correctly");
            return Ok(());
        }

        self.playlist_save_m3u(path_m3u)
    }

    /// Save the current playlist as m3u in the given full path.
    pub fn playlist_save_m3u(&mut self, path: PathBuf) -> Result<()> {
        // TODO: move this to server?
        self.playback.playlist.save_m3u(&path)?;

        self.new_library_reload_and_focus(path);

        // only reload database results, if the criteria is for playlists
        if self.dw.criteria == SearchCriteria::Playlist {
            self.database_update_search_results();
        }

        Ok(())
    }
}
