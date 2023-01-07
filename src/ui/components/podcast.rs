use crate::config::{Keys, Settings};
use crate::podcast::{download_list, EpData, PodcastFeed, PodcastNoId};
use crate::ui::{Id, Model, Msg, PCMsg};
use anyhow::{anyhow, bail, Result};
use sanitize_filename::{sanitize_with_options, Options};
use tui_realm_stdlib::List;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::{Alignment, BorderType, TableBuilder, TextSpan};
use tuirealm::props::{Borders, Color};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers, NoUserEvent},
    AttrValue, Attribute, Component, Event, MockComponent, State, StateValue,
};

#[derive(MockComponent)]
pub struct FeedsList {
    component: List,
    on_key_tab: Msg,
    on_key_backtab: Msg,
    keys: Keys,
}

impl FeedsList {
    pub fn new(config: &Settings, on_key_tab: Msg, on_key_backtab: Msg) -> Self {
        Self {
            component: List::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .title(" Podcasts: ", Alignment::Left)
                .scroll(true)
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
                .rewind(false)
                .step(4)
                .scroll(true)
                .rows(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty"))
                        .build(),
                ),
            on_key_tab,
            on_key_backtab,
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for FeedsList {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => {
                if let Some(AttrValue::Table(t)) = self.query(Attribute::Content) {
                    if let State::One(StateValue::Usize(index)) = self.state() {
                        if index >= t.len() - 1 {
                            return Some(self.on_key_tab.clone());
                        }
                    }
                }
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                if let Some(AttrValue::Table(t)) = self.query(Attribute::Content) {
                    if let State::One(StateValue::Usize(index)) = self.state() {
                        if index >= t.len() - 1 {
                            return Some(self.on_key_tab.clone());
                        }
                    }
                }
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::PodcastSelected(index)));
                }
                CmdResult::None
            }

            Event::Keyboard(key) if key == self.keys.global_right.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::PodcastSelected(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(self.on_key_tab.clone());
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backtab.clone()),

            Event::Keyboard(keyevent) if keyevent == self.keys.podcast_add_feed.key_event() => {
                return Some(Msg::Podcast(PCMsg::PodcastAddPopupShow));
            }

            Event::Keyboard(keyevent) if keyevent == self.keys.podcast_sync_pod.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::PodcastSyncOne(index)));
                }
                CmdResult::None
            }

            Event::Keyboard(keyevent)
                if keyevent == self.keys.podcast_sync_all_pods.key_event() =>
            {
                return Some(Msg::Podcast(PCMsg::PodcastSyncAll));
            }

            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct EpisodeList {
    component: List,
    on_key_tab: Msg,
    on_key_backtab: Msg,
    keys: Keys,
}

impl EpisodeList {
    pub fn new(config: &Settings, on_key_tab: Msg, on_key_backtab: Msg) -> Self {
        Self {
            component: List::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .title(" Episodes: ", Alignment::Left)
                .scroll(true)
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
                .rewind(false)
                .step(4)
                .scroll(true)
                .rows(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty"))
                        .build(),
                ),
            on_key_tab,
            on_key_backtab,
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for EpisodeList {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => {
                self.perform(Cmd::Move(Direction::Down));
                return Some(Msg::Podcast(PCMsg::DescriptionUpdate));
            }
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    if index == 0 {
                        return Some(self.on_key_backtab.clone());
                    }
                }
                self.perform(Cmd::Move(Direction::Up));
                return Some(Msg::Podcast(PCMsg::DescriptionUpdate));
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down));
                return Some(Msg::Podcast(PCMsg::DescriptionUpdate));
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    if index == 0 {
                        return Some(self.on_key_backtab.clone());
                    }
                }
                self.perform(Cmd::Move(Direction::Up));
                return Some(Msg::Podcast(PCMsg::DescriptionUpdate));
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),

            // Event::Keyboard(KeyEvent {
            //     code: Key::Enter, ..
            // }) => {
            //     if let State::One(StateValue::Usize(index)) = self.state() {
            //         return Some(Msg::DataBase(DBMsg::SearchResult(index)));
            //     }
            //     CmdResult::None
            // }
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(self.on_key_tab.clone())
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backtab.clone()),

            // Event::Keyboard(keyevent) if keyevent == self.keys.library_search.key_event() => {
            //     return Some(Msg::GeneralSearch(crate::ui::GSMsg::PopupShowDatabase))
            // }
            Event::Keyboard(keyevent) if keyevent == self.keys.global_right.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::EpisodeAdd(index)));
                }
                CmdResult::None
            }

            Event::Keyboard(keyevent) if keyevent == self.keys.podcast_mark_played.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::EpisodeMarkPlayed(index)));
                }
                CmdResult::None
            }

            Event::Keyboard(keyevent)
                if keyevent == self.keys.podcast_mark_all_played.key_event() =>
            {
                return Some(Msg::Podcast(PCMsg::EpisodeMarkAllPlayed));
            }
            // Event::Keyboard(keyevent) if keyevent == self.keys.database_add_all.key_event() => {
            //     return Some(Msg::DataBase(DBMsg::AddAllToPlaylist))
            // }
            Event::Keyboard(keyevent)
                if keyevent == self.keys.podcast_episode_download.key_event() =>
            {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::EpisodeDownload(index)));
                }
                CmdResult::None
            }

            Event::Keyboard(keyevent)
                if keyevent == self.keys.podcast_episode_delete_file.key_event() =>
            {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::EpisodeDeleteFile(index)));
                }
                CmdResult::None
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

impl Model {
    pub fn podcast_add(&mut self, url: &str) {
        let feed = PodcastFeed::new(None, url, None);

        crate::podcast::check_feed(
            feed,
            self.config.podcast_max_retries,
            &self.threadpool,
            self.tx_to_main.clone(),
        );
    }
    pub fn podcast_sync_feeds_and_episodes(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in self.podcasts.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }
            let new = record.num_unplayed();
            let total = record.episodes.len();
            if new > 0 {
                table.add_col(TextSpan::new(format!("{} ({new}/{total})", record.title)).bold());
                continue;
            }

            table.add_col(TextSpan::new(format!("{} ({new}/{total})", record.title)));
        }
        if self.podcasts.is_empty() {
            table.add_col(TextSpan::from("empty feeds list"));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::Podcast,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();
        if let Err(e) = self.podcast_sync_episodes() {
            self.mount_error_popup(format!("Error sync episodes: {e}"));
        }
    }

    pub fn podcast_sync_episodes(&mut self) -> Result<()> {
        if self.podcasts.is_empty() {
            return Ok(());
        }
        let podcast_selected = self
            .podcasts
            .get(self.podcasts_index)
            .ok_or_else(|| anyhow!("get podcast selected failed."))?;
        // let episodes = self.db_podcast.get_episodes(podcast_selected.id, true)?;
        let mut table: TableBuilder = TableBuilder::default();

        for (idx, record) in podcast_selected.episodes.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }

            let mut title = record.title.clone();
            // if let Some(_) = record.path {
            if record.path.is_some() {
                title = format!("[D] {title}");
            }
            if record.played {
                table.add_col(TextSpan::new(title).strikethrough());
                continue;
            }

            table.add_col(TextSpan::new(title).bold());
        }
        if podcast_selected.episodes.is_empty() {
            table.add_col(TextSpan::from("empty episodes list"));
        }

        let table = table.build();
        self.app
            .attr(
                &Id::Episode,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();

        self.lyric_update();
        Ok(())
    }
    pub fn episode_mark_played(&mut self, index: usize) -> Result<()> {
        if self.podcasts.is_empty() {
            return Ok(());
        }
        let podcast_selected = self
            .podcasts
            .get_mut(self.podcasts_index)
            .ok_or_else(|| anyhow!("get podcast selected failed."))?;
        let ep = podcast_selected
            .episodes
            .get_mut(index)
            .ok_or_else(|| anyhow!("get episode selected failed"))?;
        self.db_podcast.set_played_status(ep.id, !ep.played)?;
        ep.played = !ep.played;
        self.podcast_sync_feeds_and_episodes();

        Ok(())
    }

    pub fn episode_mark_all_played(&mut self) -> Result<()> {
        if self.podcasts.is_empty() {
            return Ok(());
        }
        let podcast_selected = self
            .podcasts
            .get_mut(self.podcasts_index)
            .ok_or_else(|| anyhow!("get podcast selected failed."))?;
        let played = podcast_selected
            .episodes
            .get(0)
            .ok_or_else(|| anyhow!("get first episode failed."))?
            .played;
        let mut epid_vec = Vec::new();
        for ep in &mut podcast_selected.episodes {
            epid_vec.push(ep.id);
            ep.played = !played;
        }

        self.db_podcast.set_all_played_status(&epid_vec, !played)?;
        self.podcast_sync_feeds_and_episodes();

        Ok(())
    }

    /// Handles the application logic for adding a new podcast, or
    /// synchronizing data from the RSS feed of an existing podcast.
    /// `pod_id` will be None if a new podcast is being added (i.e.,
    /// the database has not given it an id yet).
    pub fn add_or_sync_data(&mut self, pod: &PodcastNoId, pod_id: Option<i64>) -> Result<()> {
        // let title = pod.title.clone();
        let db_result;

        if let Some(id) = pod_id {
            db_result = self.db_podcast.update_podcast(id, pod);
        } else {
            db_result = self.db_podcast.insert_podcast(pod);
        }
        match db_result {
            Ok(_result) => {
                {
                    self.podcasts = self.db_podcast.get_podcasts()?;
                    self.podcast_sync_feeds_and_episodes();
                    Ok(())
                }
                // self.update_filters(self.filters, true);

                // if pod_id.is_some() {
                //     self.sync_tracker.push(result);
                //     self.sync_counter -= 1;
                //     self.update_tracker_notif();

                //     if self.sync_counter == 0 {
                //         // count up total new episodes and updated
                //         // episodes when sync process is finished
                //         let mut added = 0;
                //         let mut updated = 0;
                //         let mut new_eps = Vec::new();
                //         for res in self.sync_tracker.iter() {
                //             added += res.added.len();
                //             updated += res.updated.len();
                //             new_eps.extend(res.added.clone());
                //         }
                //         self.sync_tracker = Vec::new();
                //         self.notif_to_ui(
                //             format!("Sync complete: Added {added}, updated {updated} episodes."),
                //             false,
                //         );

                //         // deal with new episodes once syncing is
                //         // complete, based on user preferences
                //         if !new_eps.is_empty() {
                //             match self.config.download_new_episodes {
                //                 DownloadNewEpisodes::Always => {
                //                     for ep in new_eps.into_iter() {
                //                         self.download(ep.pod_id, Some(ep.id));
                //                     }
                //                 }
                //                 DownloadNewEpisodes::AskSelected => {
                //                     self.tx_to_ui
                //                         .send(MainMessage::UiSpawnDownloadPopup(new_eps, true))
                //                         .expect("Thread messaging error");
                //                 }
                //                 DownloadNewEpisodes::AskUnselected => {
                //                     self.tx_to_ui
                //                         .send(MainMessage::UiSpawnDownloadPopup(new_eps, false))
                //                         .expect("Thread messaging error");
                //                 }
                //                 _ => (),
                //             }
                //         }
                //     }
                // } else {
                //     self.notif_to_ui(
                //         format!("Successfully added {} episodes.", result.added.len()),
                //         false,
                //     );
                // }
            }
            Err(e) => Err(e),
        }
    }

    /// Synchronize RSS feed data for one or more podcasts.
    pub fn podcast_sync_pod(&mut self, index: Option<usize>) -> Result<()> {
        // We pull out the data we need here first, so we can
        // stop borrowing the podcast list as quickly as possible.
        // Slightly less efficient (two loops instead of
        // one), but then it won't block other tasks that
        // need to access the list.

        let mut pod_data = Vec::new();
        match index {
            // just grab one podcast
            Some(i) => {
                if self.podcasts.is_empty() {
                    return Ok(());
                }
                let pod_selected = self
                    .podcasts
                    .get(i)
                    .ok_or_else(|| anyhow!("get podcast selected failed."))?;
                let pcf = PodcastFeed::new(
                    Some(pod_selected.id),
                    &pod_selected.url.clone(),
                    Some(pod_selected.title.clone()),
                );
                pod_data.push(pcf);
            }

            // get all of 'em!
            None => {
                pod_data = self
                    .podcasts
                    .iter()
                    .map(|pod| {
                        PodcastFeed::new(Some(pod.id), &pod.url.clone(), Some(pod.title.clone()))
                    })
                    .collect();
            }
        }
        for feed in pod_data {
            crate::podcast::check_feed(
                feed,
                self.config.podcast_max_retries,
                &self.threadpool,
                self.tx_to_main.clone(),
            );
        }
        // self.update_tracker_notif();
        self.podcast_sync_feeds_and_episodes();
        Ok(())
    }

    pub fn episode_download(&mut self, index: Option<usize>) -> Result<()> {
        if self.podcasts.is_empty() {
            return Ok(());
        }
        let podcast_selected = self
            .podcasts
            .get_mut(self.podcasts_index)
            .ok_or_else(|| anyhow!("get podcast selected failed."))?;

        let pod_title;
        let mut ep_data = Vec::new();
        {
            pod_title = podcast_selected.title.clone();

            // if we are selecting one specific episode, just grab that
            // one; otherwise, loop through them all
            match index {
                Some(idx) => {
                    // grab just the relevant data we need

                    let ep = podcast_selected
                        .episodes
                        .get_mut(idx)
                        .ok_or_else(|| anyhow!("get episode selected failed"))?;
                    let data = EpData {
                        id: ep.id,
                        pod_id: ep.pod_id,
                        title: ep.title.clone(),
                        url: ep.url.clone(),
                        pubdate: ep.pubdate,
                        file_path: None,
                    };
                    if ep.path.is_none() {
                        ep_data.push(data);
                    }
                }
                None => {
                    // grab just the relevant data we need
                    ep_data = podcast_selected
                        .episodes
                        .iter()
                        .filter_map(|ep| {
                            if ep.path.is_none() {
                                Some(EpData {
                                    id: ep.id,
                                    pod_id: ep.pod_id,
                                    title: ep.title.clone(),
                                    url: ep.url.clone(),
                                    pubdate: ep.pubdate,
                                    file_path: None,
                                })
                            } else {
                                None
                            }
                        })
                        .collect();
                }
            }
        }

        // check against episodes currently being downloaded -- so we
        // don't needlessly download them again
        // ep_data.retain(|ep| !self.download_tracker.contains(&ep.id));

        if !ep_data.is_empty() {
            // add directory for podcast, create if it does not exist
            let dir_name = sanitize_with_options(
                &pod_title,
                Options {
                    truncate: true,
                    windows: true, // for simplicity, we'll just use Windows-friendly paths for everyone
                    replacement: "",
                },
            );
            match crate::utils::create_podcast_dir(&self.config, dir_name) {
                Ok(path) => {
                    // for ep in ep_data.iter() {
                    //     self.download_tracker.insert(ep.id);
                    // }
                    download_list(
                        ep_data,
                        &path,
                        self.config.podcast_max_retries,
                        &self.threadpool,
                        &self.tx_to_main,
                    );
                }
                Err(_) => bail!("Could not create dir: {pod_title}"),
            }
        }

        // self.podcast_sync_feeds_and_episodes();
        Ok(())
    }

    pub fn episode_download_complete(&mut self, ep_data: EpData) -> Result<()> {
        let file_path = ep_data.file_path.unwrap();
        let res = self.db_podcast.insert_file(ep_data.id, &file_path);
        if res.is_err() {
            bail!(
                "Could not add episode file to database: {}",
                file_path.to_string_lossy()
            );
        }

        let podcasts = self.db_podcast.get_podcasts()?;
        self.podcasts = podcasts;

        self.podcast_sync_feeds_and_episodes();
        self.episode_update_playlist();
        Ok(())
    }

    /// Deletes a downloaded file for an episode from the user's local
    /// system.
    pub fn episode_delete_file(&mut self, ep_index: usize) -> Result<()> {
        if self.podcasts.is_empty() {
            return Ok(());
        }
        let podcast_selected = self
            .podcasts
            .get_mut(self.podcasts_index)
            .ok_or_else(|| anyhow!("get podcast selected failed."))?;

        let ep = podcast_selected
            .episodes
            .get_mut(ep_index)
            .ok_or_else(|| anyhow!("get episode selected failed"))?;

        if ep.path.is_some() {
            let title = &ep.title;
            let path = ep.path.clone().unwrap();
            match std::fs::remove_file(path) {
                Ok(_) => {
                    self.db_podcast.remove_file(ep.id).map_err(|e| {
                        anyhow!(format!("Could not remove file from db: {title} {e}"))
                    })?;
                    ep.path = None;
                }
                Err(e) => bail!(format!("Error deleting \"{title}\": {e}")),
            }
        }
        self.podcast_sync_feeds_and_episodes();
        self.episode_update_playlist();
        Ok(())
    }

    fn episode_update_playlist(&mut self) {
        self.player.playlist.reload().ok();
        self.playlist_sync();
    }
}
