use crate::ui::Model;
use anyhow::{anyhow, bail, Result};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::ClientBuilder;
use sanitize_filename::{sanitize_with_options, Options};
use serde_json::Value;
use std::time::Duration;
use termusiclib::podcast::{download_list, EpData, PodcastFeed, PodcastNoId};
use termusiclib::track::MediaType;
use termusiclib::types::{Id, Msg, PCMsg};
use termusicplayback::SharedSettings;
use tokio::runtime::Handle;
use tui_realm_stdlib::List;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::props::{Alignment, BorderType, TableBuilder, TextSpan};
use tuirealm::props::{Borders, Color, PropPayload, PropValue};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers, NoUserEvent},
    AttrValue, Attribute, Component, Event, MockComponent, State, StateValue,
};
// left for debug
// use std::io::Write;

#[derive(MockComponent)]
pub struct FeedsList {
    component: List,
    on_key_tab: Msg,
    on_key_backtab: Msg,
    config: SharedSettings,
}

impl FeedsList {
    pub fn new(config: SharedSettings, on_key_tab: Msg, on_key_backtab: Msg) -> Self {
        let component = {
            let config = config.read();
            List::default()
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
                .title(" Podcast Feeds: ", Alignment::Left)
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
                )
        };

        Self {
            component,
            on_key_tab,
            on_key_backtab,
            config,
        }
    }
}

impl Component<Msg, NoUserEvent> for FeedsList {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().keys;
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
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
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(key) if key == keys.global_down.key_event() => {
                if let Some(AttrValue::Table(t)) = self.query(Attribute::Content) {
                    if let State::One(StateValue::Usize(index)) = self.state() {
                        if index >= t.len() - 1 {
                            return Some(self.on_key_tab.clone());
                        }
                    }
                }
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == keys.global_up.key_event() => {
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
            Event::Keyboard(key) if key == keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::Begin)),

            Event::Keyboard(KeyEvent {
                code: Key::Enter | Key::Right,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::PodcastSelected(index)));
                }
                CmdResult::None
            }

            Event::Keyboard(key) if key == keys.global_right.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::PodcastSelected(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::End)),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => {
                return Some(self.on_key_tab.clone());
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backtab.clone()),

            Event::Keyboard(keyevent) if keyevent == keys.podcast_search_add_feed.key_event() => {
                return Some(Msg::Podcast(PCMsg::PodcastAddPopupShow));
            }

            Event::Keyboard(keyevent) if keyevent == keys.podcast_refresh_feed.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::PodcastRefreshOne(index)));
                }
                CmdResult::None
            }

            Event::Keyboard(keyevent) if keyevent == keys.podcast_refresh_all_feeds.key_event() => {
                return Some(Msg::Podcast(PCMsg::PodcastRefreshAll));
            }

            Event::Keyboard(keyevent) if keyevent == keys.podcast_delete_feed.key_event() => {
                return Some(Msg::Podcast(PCMsg::FeedDeleteShow));
            }
            Event::Keyboard(keyevent) if keyevent == keys.podcast_delete_all_feeds.key_event() => {
                return Some(Msg::Podcast(PCMsg::FeedsDeleteShow));
            }

            Event::Keyboard(keyevent) if keyevent == keys.library_search.key_event() => {
                return Some(Msg::GeneralSearch(crate::ui::GSMsg::PopupShowPodcast))
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
    config: SharedSettings,
}

impl EpisodeList {
    pub fn new(config: SharedSettings, on_key_tab: Msg, on_key_backtab: Msg) -> Self {
        let component = {
            let config = config.read();
            List::default()
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
                )
        };

        Self {
            component,
            on_key_tab,
            on_key_backtab,
            config,
        }
    }
}

impl Component<Msg, NoUserEvent> for EpisodeList {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().keys;
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.perform(Cmd::Move(Direction::Down));
                return Some(Msg::Podcast(PCMsg::DescriptionUpdate));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    if index == 0 {
                        return Some(self.on_key_backtab.clone());
                    }
                }
                self.perform(Cmd::Move(Direction::Up));
                return Some(Msg::Podcast(PCMsg::DescriptionUpdate));
            }
            Event::Keyboard(key) if key == keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down));
                return Some(Msg::Podcast(PCMsg::DescriptionUpdate));
            }
            Event::Keyboard(key) if key == keys.global_up.key_event() => {
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
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(key) if key == keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(key) if key == keys.global_goto_bottom.key_event() => {
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
            }) => return Some(self.on_key_tab.clone()),
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(self.on_key_backtab.clone()),

            Event::Keyboard(KeyEvent {
                code: Key::Enter | Key::Right,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::EpisodeAdd(index)));
                }
                CmdResult::None
            }

            Event::Keyboard(keyevent) if keyevent == keys.global_right.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::EpisodeAdd(index)));
                }
                CmdResult::None
            }

            Event::Keyboard(keyevent) if keyevent == keys.podcast_mark_played.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::EpisodeMarkPlayed(index)));
                }
                CmdResult::None
            }

            Event::Keyboard(keyevent) if keyevent == keys.podcast_mark_all_played.key_event() => {
                return Some(Msg::Podcast(PCMsg::EpisodeMarkAllPlayed));
            }

            Event::Keyboard(keyevent) if keyevent == keys.podcast_episode_download.key_event() => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::EpisodeDownload(index)));
                }
                CmdResult::None
            }

            Event::Keyboard(keyevent)
                if keyevent == keys.podcast_episode_delete_file.key_event() =>
            {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::EpisodeDeleteFile(index)));
                }
                CmdResult::None
            }
            Event::Keyboard(keyevent) if keyevent == keys.library_search.key_event() => {
                return Some(Msg::GeneralSearch(crate::ui::GSMsg::PopupShowEpisode))
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

impl Model {
    #[allow(clippy::doc_markdown)]
    /// Search ITunes for podcasts and send it to `Model::tx_to_main` as [`Msg::Podcast`] and [`PCMsg::Search*`](PCMsg).
    ///
    /// Requires that the current thread has a entered runtime
    pub fn podcast_search_itunes(&self, search_str: &str) {
        let encoded: String = utf8_percent_encode(search_str, NON_ALPHANUMERIC).to_string();
        let url =
            format!("https://itunes.apple.com/search?media=podcast&entity=podcast&term={encoded}",);
        let agent = ClientBuilder::new()
            .connect_timeout(Duration::from_secs(5))
            .build()
            .expect("error build client");
        // let result = agent.get(&url).call()?;

        let mut max_retries = self.config.read().podcast_max_retries;

        let tx = self.tx_to_main.clone();

        // this will work for now as the tui loop is a async function, and this function is called on the same thread
        Handle::current().spawn(async move {
            let request: Result<reqwest::Response> = loop {
                let response = agent.get(&url).send().await;
                if let Ok(resp) = response {
                    break Ok(resp);
                }
                max_retries -= 1;
                if max_retries == 0 {
                    break Err(anyhow!("No response from feed"));
                }
            };
            // below two lines are left for debug purpose
            // let mut file = std::fs::File::create("data.txt").expect("create failed");
            // file.write_all(result.into_string()?.as_bytes())
            //     .expect("write failed");
            match request {
                Ok(result) => match result.status() {
                    reqwest::StatusCode::OK => match result.text().await {
                        Ok(text) => {
                            if let Some(vec) = parse_itunes_results(&text) {
                                tx.send(Msg::Podcast(PCMsg::SearchSuccess(vec))).ok();
                            } else {
                                tx.send(Msg::Podcast(PCMsg::SearchError(
                                    "Error parsing result".to_string(),
                                )))
                                .ok();
                            }
                        }
                        Err(_) => {
                            tx.send(Msg::Podcast(PCMsg::SearchError(
                                "Error in into_string".to_string(),
                            )))
                            .ok();
                        }
                    },
                    code => {
                        tx.send(Msg::Podcast(PCMsg::SearchError(format!(
                            "Error result status code: {code}"
                        ))))
                        .ok();
                    }
                },
                Err(e) => {
                    tx.send(Msg::Podcast(PCMsg::SearchError(e.to_string())))
                        .ok();
                }
            }
        });
    }

    pub fn podcast_add(&mut self, url: &str) {
        let feed = PodcastFeed::new(None, url, None);

        crate::podcast::check_feed(
            feed,
            self.config.read().podcast_max_retries,
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
            self.mount_error_popup(e.context("podcast sync episodes"));
        }
    }

    pub fn podcast_sync_episodes(&mut self) -> Result<()> {
        if self.podcasts.is_empty() {
            let mut table: TableBuilder = TableBuilder::default();
            table.add_col(TextSpan::from("empty episodes list"));

            let table = table.build();
            self.app
                .attr(
                    &Id::Episode,
                    tuirealm::Attribute::Content,
                    tuirealm::AttrValue::Table(table),
                )
                .ok();

            self.lyric_update();
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
        ep.played = !ep.played;
        self.db_podcast.set_played_status(ep.id, ep.played)?;
        self.podcast_sync_feeds_and_episodes();

        Ok(())
    }

    pub fn episode_mark_all_played(&mut self) -> Result<()> {
        if self.podcasts.is_empty() {
            return Ok(());
        }

        let mut ep_index = 0;
        if let Ok(idx) = self.podcast_get_episode_index() {
            ep_index = idx;
        }
        let podcast_selected = self
            .podcasts
            .get_mut(self.podcasts_index)
            .ok_or_else(|| anyhow!("get podcast selected failed."))?;
        let played = podcast_selected
            .episodes
            .get(ep_index)
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
    pub fn podcast_refresh_feeds(&mut self, index: Option<usize>) -> Result<()> {
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
                self.config.read().podcast_max_retries,
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
                    if ep.path.is_none() && !self.download_tracker.contains(&ep.url) {
                        ep_data.push(data);
                    }
                }
                None => {
                    // grab just the relevant data we need
                    ep_data = podcast_selected
                        .episodes
                        .iter()
                        .filter_map(|ep| {
                            if ep.path.is_none() && !self.download_tracker.contains(&ep.url) {
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
            match crate::utils::create_podcast_dir(&self.config.read(), dir_name) {
                Ok(path) => {
                    // for ep in ep_data.iter() {
                    //     self.download_tracker.insert(ep.id);
                    // }
                    download_list(
                        ep_data,
                        &path,
                        self.config.read().podcast_max_retries,
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
                Ok(()) => {
                    self.db_podcast.remove_file(ep.id).map_err(|e| {
                        anyhow!(format!("Could not remove file from db: {title} {e}"))
                    })?;
                    ep.path = None;
                }
                Err(e) => {
                    // Repeat the same thing in case the local file is missing. Update db
                    self.db_podcast.remove_file(ep.id).map_err(|e| {
                        anyhow!(format!("Could not remove file from db: {title} {e}"))
                    })?;
                    ep.path = None;
                    bail!(format!("Error deleting \"{title}\": {e}"));
                }
            }
        }
        self.podcast_sync_feeds_and_episodes();
        self.episode_update_playlist();
        Ok(())
    }

    fn episode_update_playlist(&mut self) {
        // self.player.playlist.reload().ok();
        self.playlist_sync();
    }

    pub fn podcast_delete_files(&mut self, pod_index: usize) -> Result<()> {
        let mut eps_to_remove = Vec::new();
        let mut success = true;
        {
            let podcast_selected = self
                .podcasts
                .get_mut(pod_index)
                .ok_or_else(|| anyhow!("get podcast selected failed."))?;

            for ep in &mut podcast_selected.episodes {
                if ep.path.is_some() {
                    match std::fs::remove_file(ep.path.clone().unwrap()) {
                        Ok(()) => {
                            eps_to_remove.push(ep.id);
                            ep.path = None;
                        }
                        Err(_) => success = false,
                    }
                }
            }
        }

        self.db_podcast.remove_files(&eps_to_remove)?;
        if !success {
            bail!("Error happend when removing local file. Please check.");
        }

        Ok(())
    }

    pub fn podcast_remove_all_feeds(&mut self) -> Result<()> {
        if self.podcasts.is_empty() {
            return Ok(());
        }

        let len = self.podcasts.len();

        for index in 0..len {
            self.podcast_delete_files(index).ok();
        }

        self.db_podcast.clear_db()?;

        self.podcasts = Vec::new();
        self.podcasts_index = 0;

        self.podcast_sync_feeds_and_episodes();
        self.episode_update_playlist();
        Ok(())
    }

    pub fn podcast_remove_feed(&mut self) -> Result<()> {
        if self.podcasts.is_empty() {
            return Ok(());
        }

        if let Ok(feed_index) = self.podcast_get_feed_index() {
            self.podcast_delete_files(feed_index)?;
            let podcast_selected = self.podcasts.remove(feed_index);
            self.db_podcast.remove_podcast(podcast_selected.id)?;
        }

        self.podcasts_index = self.podcasts_index.saturating_sub(1);
        self.podcast_sync_feeds_and_episodes();
        self.episode_update_playlist();
        Ok(())
    }

    fn podcast_get_feed_index(&self) -> Result<usize> {
        if let Ok(State::One(StateValue::Usize(feed_index))) = self.app.state(&Id::Podcast) {
            return Ok(feed_index);
        }
        Err(anyhow!("cannot get feed index"))
    }

    fn podcast_get_episode_index(&self) -> Result<usize> {
        if let Ok(State::One(StateValue::Usize(episode_index))) = self.app.state(&Id::Episode) {
            return Ok(episode_index);
        }
        Err(anyhow!("cannot get feed index"))
    }

    pub fn podcast_mark_current_track_played(&mut self) -> Result<()> {
        if self.podcasts.is_empty() {
            return Ok(());
        }
        if let Some(track) = self.playlist.current_track() {
            if MediaType::Podcast == track.media_type {
                if let Some(url) = track.file() {
                    'outer: for pod in &mut self.podcasts {
                        for ep in &mut pod.episodes {
                            if ep.url == url {
                                if !ep.played {
                                    ep.played = true;
                                    self.db_podcast.set_played_status(ep.id, ep.played)?;
                                }
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }

        self.podcast_sync_feeds_and_episodes();

        Ok(())
    }

    pub fn podcast_get_album_photo_by_url(&self, url: &str) -> Option<String> {
        if self.podcasts.is_empty() {
            return None;
        }
        for pod in &self.podcasts {
            for ep in &pod.episodes {
                if ep.url == url {
                    return pod.image_url.clone();
                }
            }
        }
        None
    }

    // #[cfg(not(any(feature = "mpv", feature = "gst")))]
    // pub fn podcast_get_episode_index_by_url(&mut self, url: &str) -> Option<usize> {
    //     if self.podcasts.is_empty() {
    //         return None;
    //     }
    //     for (idx_pod, pod) in self.podcasts.iter().enumerate() {
    //         for (idx_ep, ep) in pod.episodes.iter().enumerate() {
    //             if ep.url == url {
    //                 self.podcasts_index = idx_pod;
    //                 return Some(idx_ep);
    //             }
    //         }
    //     }
    //     None
    // }

    pub fn podcast_update_search_episode(&mut self, input: &str) {
        let mut table: TableBuilder = TableBuilder::default();
        let mut idx = 0;
        let search = format!("*{}*", input.to_lowercase());
        let mut db_tracks = vec![];
        // Get all episodes
        let podcasts = self.podcasts.clone();
        for podcast in podcasts {
            if let Ok(episodes) = self.db_podcast.get_episodes(podcast.id, true) {
                db_tracks.extend(episodes);
            }
        }

        if db_tracks.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty tracks from podcasts db"));
            table.add_col(TextSpan::from(""));
        } else {
            for record in db_tracks {
                if wildmatch::WildMatch::new(&search).matches(&record.title.to_lowercase()) {
                    if idx > 0 {
                        table.add_row();
                    }
                    table
                        .add_col(TextSpan::new(idx.to_string()))
                        .add_col(TextSpan::new(record.title).bold())
                        .add_col(TextSpan::new(format!("{}", record.id)));
                    idx += 1;
                }
            }
        }

        let table = table.build();
        self.general_search_update_show(table);
    }

    pub fn podcast_update_search_podcast(&mut self, input: &str) {
        let mut table: TableBuilder = TableBuilder::default();
        let mut idx = 0;
        let search = format!("*{}*", input.to_lowercase());
        // Get all episodes
        let db_tracks = self.podcasts.clone();

        if db_tracks.is_empty() {
            table.add_col(TextSpan::from("0"));
            table.add_col(TextSpan::from("empty tracks from podcasts db"));
            table.add_col(TextSpan::from(""));
        } else {
            for record in db_tracks {
                if wildmatch::WildMatch::new(&search).matches(&record.title.to_lowercase()) {
                    if idx > 0 {
                        table.add_row();
                    }
                    table
                        .add_col(TextSpan::new(idx.to_string()))
                        .add_col(TextSpan::new(record.title).bold())
                        .add_col(TextSpan::new(format!("{}", record.id)));
                    idx += 1;
                }
            }
        }

        let table = table.build();
        self.general_search_update_show(table);
    }

    pub fn podcast_locate_episode(&mut self, pod_index: usize, ep_index: usize) {
        assert!(self
            .app
            .attr(
                &Id::Podcast,
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::Usize(pod_index))),
            )
            .is_ok());
        self.podcast_sync_episodes().ok();
        assert!(self
            .app
            .attr(
                &Id::Episode,
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::Usize(ep_index))),
            )
            .is_ok());
        // update description of episode
        self.lyric_update();
    }

    #[allow(clippy::cast_possible_wrap)]
    pub fn podcast_find_by_ep_id(&mut self, ep_id: usize) -> Result<(usize, usize)> {
        for (podcast_index, podcast) in self.podcasts.iter().enumerate() {
            for (episode_index, episode) in podcast.episodes.iter().enumerate() {
                if episode.id == ep_id as i64 {
                    // Need to set podcast index here, otherwise the wrong episodes will be added
                    self.podcasts_index = podcast_index;
                    return Ok((podcast_index, episode_index));
                }
            }
        }
        bail!("Cannot find ep_id")
    }

    #[allow(clippy::cast_possible_wrap)]
    pub fn podcast_find_by_pod_id(&mut self, pod_id: usize) -> Result<usize> {
        for (podcast_index, podcast) in self.podcasts.iter().enumerate() {
            if podcast.id == pod_id as i64 {
                // Need to set podcast index here
                self.podcasts_index = podcast_index;
                return Ok(podcast_index);
            }
        }
        bail!("Cannot find pod_id")
    }

    pub fn podcast_focus_episode_list(&mut self) {
        // Set focus to episode list
        let mut need_to_set_focus = true;

        if let Ok(Some(AttrValue::Flag(true))) = self.app.query(&Id::Episode, Attribute::Focus) {
            need_to_set_focus = false;
        }
        if need_to_set_focus {
            self.app.active(&Id::Episode).ok();
        }
    }

    pub fn podcast_focus_podcast_list(&mut self) {
        // Set focus to episode list
        let mut need_to_set_focus = true;

        if let Ok(Some(AttrValue::Flag(true))) = self.app.query(&Id::Podcast, Attribute::Focus) {
            need_to_set_focus = false;
        }
        if need_to_set_focus {
            self.app.active(&Id::Podcast).ok();
        }
    }
}

fn parse_itunes_results(data: &str) -> Option<Vec<PodcastFeed>> {
    if let Ok(value) = serde_json::from_str::<Value>(data) {
        // below two lines are left for debug purpose
        // let mut file = std::fs::File::create("data.txt").expect("create failed");
        // file.write_all(data.as_bytes()).expect("write failed");

        let mut vec: Vec<PodcastFeed> = Vec::new();
        let array = value.get("results")?.as_array()?;
        for v in array {
            if let Some((title, url)) = parse_itunes_item(v) {
                vec.push(PodcastFeed {
                    id: None,
                    url,
                    title: Some(title),
                });
            }
        }
        return Some(vec);
    }
    None
}

fn parse_itunes_item(v: &Value) -> Option<(String, String)> {
    let title = v.get("collectionName")?.as_str()?.to_owned();
    let url = v.get("feedUrl")?.as_str()?.to_owned();
    Some((title, url))
}
