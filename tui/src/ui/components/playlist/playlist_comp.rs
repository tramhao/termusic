use std::borrow::Cow;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context as _, Result, anyhow};
use parking_lot::RwLockReadGuard;
use rand::seq::IndexedRandom;
use termusiclib::common::const_unknown::{UNKNOWN_ALBUM, UNKNOWN_ARTIST};
use termusiclib::config::v2::server::{LoopMode, ScanDepth};
use termusiclib::config::v2::tui::theme::styles::ColorTermusic;
use termusiclib::config::{SharedTuiSettings, TuiOverlay};
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
use termusiclib::track::DurationFmtShort;
use termusiclib::track::Track;
use termusiclib::utils::{filetype_supported, is_playlist, playlist_get_vec};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::Event;
use tuirealm::event::{Key, KeyEvent};
use tuirealm::props::{
    AttrValue, AttrValueRef, Attribute, BorderType, HorizontalAlignment, PropPayload, PropValue,
    QueryResult, Title,
};
use tuirealm::props::{Borders, Style};
use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::style::Color;
use tuirealm::ratatui::text::Span;
use tuirealm::ratatui::widgets::Widget;
use tuirealm::{
    command::{Cmd, CmdResult, Direction, Position},
    event::KeyModifiers,
};

use crate::ui::Model;
use crate::ui::components::orx_music_library::scanner::library_dir_tree;
use crate::ui::components::playlist::playlist_mock::{self, ListAcquire};
use crate::ui::components::playlist::playlist_mock::{
    Column, ListValue, ListValueRenderReturn, PlaylistTable,
};
use crate::ui::ids::Id;
use crate::ui::model::{SharedPlaylist, TUIPlaylist, TermusicLayout, UserEvent};
use crate::ui::msg::{GSMsg, Msg, PLMsg, SearchCriteria};
use crate::ui::tui_cmd::{PlaylistCmd, TuiCmd};

/// Holds the playlist reference.
///
/// Actual draw impl is in [`PlaylistDataBorrow`] to not have to acquire the lock for each iteration / item.
pub struct PlaylistData {
    list: SharedPlaylist,
    config: SharedTuiSettings,
}

impl<'a> ListAcquire<'a> for PlaylistData {
    type Value = PlaylistDataBorrow<'a>;

    fn acquire(&'a mut self) -> Self::Value {
        PlaylistDataBorrow {
            list: self.list.read(),
            config: self.config.read(),
        }
    }
}

/// The version of [`PlaylistData`] with all the locks acquired.
pub struct PlaylistDataBorrow<'a> {
    list: RwLockReadGuard<'a, TUIPlaylist>,
    config: RwLockReadGuard<'a, TuiOverlay>,
}

impl ListValue for PlaylistDataBorrow<'_> {
    fn render(
        &self,
        buf: &mut tuirealm::ratatui::prelude::Buffer,
        ctx: &super::playlist_mock::PlaylistTableContext<'_>,
        mut style: Style,
    ) -> ListValueRenderReturn {
        let Some(track) = self.list.tracks().get(ctx.item_offset) else {
            return ListValueRenderReturn::EMPTY;
        };

        // When in specific loop modes, change all previous entries to be grey (default color), to make it more obvious where it currently
        // is and what has been played.
        // If more modes like reverse play or other LoopModes are implemented, this should be updated too.
        if self
            .list
            .current_track_index()
            .is_some_and(|v| ctx.item_offset < v)
            && matches!(
                self.list.loop_mode(),
                LoopMode::Playlist | LoopMode::PlaylistOnce
            )
        {
            style = style.fg(self
                .config
                .settings
                .theme
                .get_color_from_theme(ColorTermusic::LightBlack));
        }

        let duration_str = if let Some(dur) = track.duration_str_short() {
            format!("[{dur:^7.7}]")
        } else {
            "[--:--]".to_string()
        };
        let title: Cow<'_, str> = track.title().map_or_else(|| track.id_str(), Into::into);

        for area in ctx.areas {
            // we only draw with Spans here, which can only be 1 height.
            let rect = Rect { height: 1, ..*area };
            buf.set_style(rect, style);
        }

        // only render the current track symbol for the selected item
        if !ctx.is_selected
            && self
                .list
                .current_track_index()
                .is_some_and(|v| v == ctx.item_offset)
        {
            Span::styled(
                &self
                    .config
                    .settings
                    .theme
                    .style
                    .playlist
                    .current_track_symbol,
                style,
            )
            .render(ctx.areas[0], buf);
        }

        // only render the highlight symbol for the selected item
        // this overwrites & takes precendence over the "current track" symbol
        if ctx.is_selected {
            Span::styled(
                &self.config.settings.theme.style.playlist.highlight_symbol,
                style,
            )
            .render(ctx.areas[0], buf);
        }

        // always display:
        // 1. Duration
        // 2. Title
        Span::styled(duration_str, style).render(ctx.areas[1], buf);
        Span::styled(title, style.bold()).render(ctx.areas[2], buf);

        // normal display, display some extra music data
        if ctx.areas.len() > 2 {
            // 3. Artist
            // 4. Album
            let artist = track.artist().unwrap_or(UNKNOWN_ARTIST);
            let album = track
                .as_track()
                .and_then(|v| v.album())
                .unwrap_or(UNKNOWN_ALBUM);

            Span::styled(artist, style).render(ctx.areas[3], buf);
            Span::styled(album, style).render(ctx.areas[4], buf);
        }

        // draw highlight all across the line
        for spacer in ctx.areas_spacer {
            let rect = Rect {
                height: 1,
                ..*spacer
            };
            buf.set_style(rect, style);
        }

        ListValueRenderReturn {
            consumed_vertical_size: 1,
            done: false,
        }
    }

    fn len(&self) -> Option<usize> {
        Some(self.list.len())
    }

    fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    fn fallback_select(&self) -> usize {
        // If there previously was no selection, start the selection at the currently playing track index
        self.list.current_track_index().unwrap_or_default()
    }
}

#[derive(Component)]
pub struct Playlist {
    component: PlaylistTable<PlaylistData>,
    config: SharedTuiSettings,
}

impl Playlist {
    pub fn new(config: SharedTuiSettings, playlist: SharedPlaylist) -> Self {
        let component = {
            let data = PlaylistData {
                list: playlist,
                config: config.clone(),
            };
            let config = config.read();
            let duration_width = DurationFmtShort::fmt_empty().len() + 2;
            let duration_width = u16::try_from(duration_width)
                .expect("This operation is static and always below u16::MAX");
            PlaylistTable::new(data)
                .border(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(config.settings.theme.playlist_border()),
                )
                .style(
                    Style::new()
                        .fg(config.settings.theme.playlist_foreground())
                        .bg(config.settings.theme.playlist_background()),
                )
                .inactive_style(Style::new().bg(config.settings.theme.playlist_background()))
                .title(Title::from(" Playlist ").alignment(HorizontalAlignment::Left))
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(config.settings.theme.playlist_highlight()),
                )
                .highlight_symbol(
                    config
                        .settings
                        .theme
                        .style
                        .playlist
                        .highlight_symbol
                        .clone(),
                )
                .vertical_scroll_step(const { NonZeroUsize::new(4).unwrap() })
                .columns(vec![
                    // symbols like "highlight" and "currently playing"; no title necessary (not that there would be enough space for it anyway)
                    Column::new("", 2, 2),
                    Column::new("Duration", duration_width, duration_width),
                    Column::new("Title", 10, 0),
                    Column::new("Artist", 10, 0),
                    Column::new("Album", 10, 0),
                ])
        };

        Self { component, config }
    }
}

impl AppComponent<Msg, UserEvent> for Playlist {
    #[expect(clippy::too_many_lines)]
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
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
            }) => self.perform(Cmd::Custom(playlist_mock::cmd::PG_DOWN)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Custom(playlist_mock::cmd::PG_UP)),
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
                if let Some(idx) = self.component.selected() {
                    return Some(Msg::Playlist(PLMsg::Delete(idx)));
                }

                CmdResult::NoChange
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
                if let Some(idx) = self.component.selected() {
                    return Some(Msg::Playlist(PLMsg::PlaySelected(idx)));
                }

                CmdResult::NoChange
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let Some(idx) = self.component.selected() {
                    return Some(Msg::Playlist(PLMsg::PlaySelected(idx)));
                }

                CmdResult::NoChange
            }
            Event::Keyboard(key) if key == keys.playlist_keys.search.get() => {
                return Some(Msg::GeneralSearch(GSMsg::PopupShowPlaylist));
            }
            Event::Keyboard(key) if key == keys.playlist_keys.swap_down.get() => {
                if let Some(idx) = self.component.selected() {
                    self.perform(Cmd::Move(Direction::Down));
                    return Some(Msg::Playlist(PLMsg::SwapDown(idx)));
                }

                CmdResult::NoChange
            }
            Event::Keyboard(key) if key == keys.playlist_keys.swap_up.get() => {
                if let Some(idx) = self.component.selected() {
                    self.perform(Cmd::Move(Direction::Up));
                    return Some(Msg::Playlist(PLMsg::SwapUp(idx)));
                }

                CmdResult::NoChange
            }
            Event::Keyboard(key) if key == keys.playlist_keys.add_random_album.get() => {
                return Some(Msg::Playlist(PLMsg::AddRandomAlbum));
            }
            Event::Keyboard(key) if key == keys.playlist_keys.add_random_songs.get() => {
                return Some(Msg::Playlist(PLMsg::AddRandomTracks));
            }
            _ => CmdResult::NoChange,
        };
        match cmd_result {
            CmdResult::NoChange => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

impl Model {
    pub fn playlist_reload(&mut self) {
        let _ = self.app.remount(
            Id::Playlist,
            Box::new(Playlist::new(
                self.config_tui.clone(),
                self.playback.playlist.clone(),
            )),
            Vec::new(),
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

    /// Get the mounted [`Playlist`] component for direct modification.
    fn playlist_comp_mut(&mut self) -> &mut Playlist {
        self.app
            .get_component_mut(&Id::Playlist)
            .expect("Expected Playlist to always be mounted")
            .as_any_mut()
            .downcast_mut::<Playlist>()
            .expect("Expected Playlist to always be Playlist")
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

        let playlist_len = self.playback.playlist.read().len();
        self.command(TuiCmd::Playlist(PlaylistCmd::AddTrack(
            PlaylistAddTrack::new_vec(u64::try_from(playlist_len).unwrap(), sources),
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
        let playlist_len = self.playback.playlist.read().len();
        self.command(TuiCmd::Playlist(PlaylistCmd::AddTrack(
            PlaylistAddTrack::new_single(u64::try_from(playlist_len).unwrap(), source),
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

            let playlist_len = self.playback.playlist.read().len();
            self.command(TuiCmd::Playlist(PlaylistCmd::AddTrack(
                PlaylistAddTrack::new_vec(u64::try_from(playlist_len).unwrap(), sources),
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

        let playlist_len = self.playback.playlist.read().len();
        self.command(TuiCmd::Playlist(PlaylistCmd::AddTrack(
            PlaylistAddTrack::new_single(u64::try_from(playlist_len).unwrap(), source),
        )));

        Ok(())
    }

    /// Add [`TrackDB`] to the playlist
    pub fn playlist_add_all_from_db(&mut self, vec: &[TrackRead]) {
        let sources = vec
            .iter()
            .map(|f| PlaylistTrackSource::Path(f.as_pathbuf().to_string_lossy().to_string()))
            .collect();

        let playlist_len = self.playback.playlist.read().len();
        self.command(TuiCmd::Playlist(PlaylistCmd::AddTrack(
            PlaylistAddTrack::new_vec(u64::try_from(playlist_len).unwrap(), sources),
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
        self.playback.playlist.write().add_tracks(
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
        self.playback.playlist.write().handle_grpc_remove(items)?;

        self.playlist_sync();

        Ok(())
    }

    /// Handle when a playlist was cleared
    pub fn handle_playlist_clear(&mut self) {
        self.playback.playlist.write().clear();
        self.playlist_comp_mut().component.reset_state();

        self.playlist_sync();
    }

    /// Handle when the playlist loop-mode was changed
    pub fn handle_playlist_loopmode(&mut self, loop_mode: &PlaylistLoopModeInfo) -> Result<()> {
        let as_u8 = u8::try_from(loop_mode.mode).context("Failed to convert u32 to u8")?;
        let loop_mode =
            LoopMode::tryfrom_discriminant(as_u8).context("Failed to get LoopMode from u8")?;
        self.playback.playlist.write().set_loop_mode(loop_mode);
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

        self.playback.playlist.write().swap(index_a, index_b)?;

        self.playlist_sync();

        Ok(())
    }

    /// Handle when the playlist has been shuffled and so has new order of tracks
    pub fn handle_playlist_shuffled(&mut self, shuffled: PlaylistShuffledInfo) -> Result<()> {
        let playlist_comp_selected_index = self.playlist_get_selected_index();
        let playlist = self.playback.playlist.read();
        // this might be fragile if there are multiple of the same track in the playlist as there is no unique identifier currently
        let playlist_track_at_old_file = playlist_comp_selected_index
            .and_then(|idx| playlist.tracks().get(idx))
            .map(Track::as_track_source);
        drop(playlist);

        self.playback
            .load_from_grpc(shuffled.tracks, &self.podcast.db_podcast)?;
        self.playlist_sync();

        if let Some(old_id) = playlist_track_at_old_file {
            let playlist = self.playback.playlist.read();
            let found_new_index = playlist
                .tracks()
                .iter()
                .enumerate()
                .find(|(_, track)| *track == old_id)
                .map(|(idx, _)| idx);
            drop(playlist);
            if let Some(new_index) = found_new_index {
                self.playlist_locate(new_index);
            }
        }

        Ok(())
    }

    /// Handle setting the current track index in the TUI playlist and selecting the proper list item
    pub fn handle_current_track_index(&mut self, current_track_index: usize, force_relocate: bool) {
        let mut playlist = self.playback.playlist.write();
        let tui_old_current_index = playlist.current_track_index();
        info!(
            "index from player is: {current_track_index:?}, index in tui is: {tui_old_current_index:?}"
        );
        let _ = playlist.set_current_track_index(current_track_index);
        drop(playlist);
        self.playback.clear_current_track();
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

    /// Update the `Playlist` Component title and force a re-draw due to updates to the [`TUIPlaylist`].
    pub fn playlist_sync(&mut self) {
        self.playlist_update_title();
    }

    /// Delete a track at `index` from the playlist
    pub fn playlist_delete_item(&mut self, index: usize) {
        let playlist = self.playback.playlist.read();
        if playlist.is_empty() || index >= playlist.len() {
            return;
        }

        let Some(track) = playlist.tracks().get(index) else {
            return;
        };

        let track_source = track.as_track_source();
        drop(playlist);

        self.command(TuiCmd::Playlist(PlaylistCmd::RemoveTrack(
            PlaylistRemoveTrackIndexed::new_single(u64::try_from(index).unwrap(), track_source),
        )));
    }

    /// Clear a entire playlist
    pub fn playlist_clear(&mut self) {
        if self.playback.playlist.read().is_empty() {
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
        let len = self.playback.playlist.read().tracks().len();
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
        if index >= self.playback.playlist.read().len().saturating_sub(1) {
            return;
        }

        self.playlist_swap(index, index.saturating_add(1));
    }

    pub fn playlist_update_library_delete(&mut self) {
        self.command(TuiCmd::Playlist(PlaylistCmd::RemoveDeletedItems));
    }

    pub fn playlist_update_title(&mut self) {
        let playlist = self.playback.playlist.read();
        let duration = playlist.tracks().iter().filter_map(Track::duration).sum();
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
            playlist.len(),
            DurationFmtShort(duration),
            loop_mode.display(display_symbol),
        );
        drop(playlist);
        self.app
            .attr(
                &Id::Playlist,
                Attribute::Title,
                AttrValue::Title(Title::from(title).alignment(HorizontalAlignment::Left)),
            )
            .ok();
    }

    /// Play the currently selected item in the playlist list
    pub fn playlist_play_selected(&mut self, index: usize) {
        let playlist = self.playback.playlist.read();
        let Some(track) = playlist.tracks().get(index) else {
            error!("Track {index} not in playlist!");
            return;
        };

        let track_source = track.as_track_source();
        drop(playlist);

        self.command(TuiCmd::Playlist(PlaylistCmd::PlaySpecific(
            PlaylistPlaySpecific {
                track_index: u64::try_from(index).unwrap(),
                id: track_source,
            },
        )));
    }

    pub fn playlist_update_search(&mut self, input: &str) {
        let playlist = self.playback.playlist.read();
        let filtered_music = Model::update_search(playlist.tracks(), input);
        let table = Model::build_table(filtered_music, &self.config_tui);
        drop(playlist);
        self.general_search_update_show(table);
    }

    /// Select the given index in the playlist list component
    pub fn playlist_locate(&mut self, index: usize) {
        let _ = self
            .app
            .attr(&Id::Playlist, Attribute::Value, AttrValue::Length(index));
    }

    /// Get the current selected index in the playlist list component
    pub fn playlist_get_selected_index(&self) -> Option<usize> {
        self.app
            .query(&Id::Playlist, Attribute::Value)
            .ok()
            .flatten()
            .as_ref()
            .map(QueryResult::as_ref)
            .and_then(AttrValueRef::as_length)
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

    /// Save the current playlist as m3u to the given path
    pub fn playlist_save_m3u_before(&mut self, path: PathBuf) -> Result<()> {
        if path.exists() {
            self.mount_save_playlist_confirm(path)
                .expect("Expected SavePlaylistConfirm to mount correctly");
            return Ok(());
        }

        self.playlist_save_m3u(path)
    }

    /// Save the current playlist as m3u in the given full path.
    pub fn playlist_save_m3u(&mut self, path: PathBuf) -> Result<()> {
        // TODO: move this to server?
        self.playback.playlist.read().save_m3u(&path)?;

        self.new_library_reload_and_focus(path);

        // only reload database results, if the criteria is for playlists
        if self.dw.criteria == SearchCriteria::Playlist {
            self.database_update_search_results();
        }

        Ok(())
    }
}
