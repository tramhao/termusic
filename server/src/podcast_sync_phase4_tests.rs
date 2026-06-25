//! Phase 4 RED Tests: Code Quality and Helper Extraction
//!
//! These tests verify:
//! - AC-08: Helper extraction reduces nesting (process_feed_result, download_and_enqueue)
//! - AC-09: Episode filtering excludes played episodes from download
//! - AC-17: max_new_episodes limit applied before disk-check loop
//! - Auto-enqueue guards: sync_once respects auto_enqueue=false config
//!
//! Coverage:
//! - SCENARIO-011: Sync logic nesting depth reduced via helper extraction
//! - SCENARIO-012: Episode filtering excludes played episodes
//! - SCENARIO-013: Never-downloaded but played episode is skipped
//! - SCENARIO-017: Max new episodes limit applied before disk-check loop
//! - SCENARIO-026: Sync pass with empty podcast feed
//! - SCENARIO-028: All episodes in feed are already played
//!
//! These tests are expected to FAIL (RED) because:
//! - `process_feed_result` and `download_and_enqueue` helper functions do not exist
//! - Episode played status is not checked during filtering
//! - auto_enqueue configuration is not read or used in sync_once

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::Duration;

    use termusiclib::config::v2::server::synchronization::SynchronizationSettings;
    use termusiclib::config::v2::server::{PodcastSettings, ServerSettings};
    use termusiclib::config::{ServerOverlay, SharedServerSettings, new_shared_server_settings};
    use termusiclib::player::playlist_helpers::PlaylistTrackSource;
    use termusiclib::podcast::PodcastNoId;
    use termusiclib::podcast::db::Database;
    use termusiclib::podcast::episode::EpisodeNoId;
    use termusicplayback::{PlayerCmd, PlayerCmdSender};
    use tokio::sync::mpsc::unbounded_channel;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::podcast_sync::sync_once;

    // =========================================================================
    // Helpers
    // =========================================================================

    fn make_test_config(download_dir: &Path) -> SharedServerSettings {
        let settings = ServerSettings {
            podcast: PodcastSettings {
                download_dir: download_dir.to_path_buf(),
                ..Default::default()
            },
            synchronization: SynchronizationSettings {
                enable: true,
                interval: Duration::from_secs(3600),
                refresh_on_startup: true,
                max_new_episodes: 5,
                ..Default::default()
            },
            ..Default::default()
        };
        new_shared_server_settings(ServerOverlay {
            settings,
            ..Default::default()
        })
    }

    fn make_test_config_with_auto_enqueue(
        download_dir: &Path,
        auto_enqueue: bool,
    ) -> SharedServerSettings {
        let settings = ServerSettings {
            podcast: PodcastSettings {
                download_dir: download_dir.to_path_buf(),
                ..Default::default()
            },
            synchronization: SynchronizationSettings {
                enable: true,
                interval: Duration::from_secs(3600),
                refresh_on_startup: true,
                max_new_episodes: 5,
                auto_enqueue,
                ..Default::default()
            },
            ..Default::default()
        };
        new_shared_server_settings(ServerOverlay {
            settings,
            ..Default::default()
        })
    }

    fn make_test_config_with_max_episodes(
        download_dir: &Path,
        max_new_episodes: u32,
    ) -> SharedServerSettings {
        let settings = ServerSettings {
            podcast: PodcastSettings {
                download_dir: download_dir.to_path_buf(),
                ..Default::default()
            },
            synchronization: SynchronizationSettings {
                enable: true,
                interval: Duration::from_secs(3600),
                refresh_on_startup: true,
                max_new_episodes,
                ..Default::default()
            },
            ..Default::default()
        };
        new_shared_server_settings(ServerOverlay {
            settings,
            ..Default::default()
        })
    }

    fn make_cmd_channel() -> (
        PlayerCmdSender,
        tokio::sync::mpsc::UnboundedReceiver<(
            PlayerCmd,
            termusicplayback::PlayerCmdCallbackSender,
        )>,
    ) {
        let (tx, rx) = unbounded_channel();
        (PlayerCmdSender::new(tx), rx)
    }

    fn generate_rss_feed(title: &str, episodes: &[(&str, &str, &str)]) -> String {
        let mut items = String::new();
        for (ep_title, guid, url) in episodes {
            items.push_str(&format!(
                r#"
        <item>
            <title>{ep_title}</title>
            <guid>{guid}</guid>
            <enclosure url="{url}" type="audio/mpeg" length="1024"/>
            <pubDate>Mon, 23 Jun 2025 12:00:00 +0000</pubDate>
        </item>"#
            ));
        }

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
    <channel>
        <title>{title}</title>
        <link>http://example.com</link>
        <description>A test podcast</description>
        {items}
    </channel>
</rss>"#
        )
    }

    fn fake_audio_content() -> Vec<u8> {
        let mut content = vec![0x49, 0x44, 0x33]; // "ID3" magic bytes
        content.extend_from_slice(&[0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        content.extend_from_slice(&[0xFF; 1024]);
        content
    }

    // =========================================================================
    // AC-08 / SCENARIO-011: Helper function extraction
    //
    // T-27: process_feed_result must exist as a callable async function
    // T-28: download_and_enqueue must exist as a callable async function
    //
    // These tests will FAIL because the helper functions don't exist yet.
    // The helpers are private to the module, so we test them indirectly by
    // verifying their behavioral effects.
    // =========================================================================

    /// SCENARIO-011: The sync logic must be decomposed into helper functions
    /// such that no code path exceeds 3 levels of nesting depth.
    ///
    /// We verify this indirectly by checking that sync_once still works correctly
    /// after the extraction. This test validates that the refactoring preserves
    /// behavior: a successful sync pass with a mock feed should still download
    /// episodes and enqueue them.
    ///
    /// AC-08: Processing logic delegated to extracted helper functions.
    #[tokio::test]
    async fn sync_once_behavior_preserved_after_helper_extraction() {
        let mock_server = MockServer::start().await;

        let ep_path = "/episodes/helper_test_ep.mp3";
        let feed_xml = generate_rss_feed(
            "Helper Extraction Test",
            &[(
                "Episode After Refactor",
                "guid-helper-001",
                &format!("{}{}", mock_server.uri(), ep_path),
            )],
        );

        Mock::given(method("GET"))
            .and(path("/helper_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(ep_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .mount(&mock_server)
            .await;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");
        let podcast = PodcastNoId {
            title: "Helper Extraction Test".to_string(),
            url: format!("{}/helper_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");
        drop(db);

        let config = make_test_config(&download_dir);
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once should still work after helper extraction: {:?}", result.err());
        let stats = result.unwrap();

        assert_eq!(stats.podcasts_checked, 1);
        assert_eq!(stats.episodes_downloaded, 1);
        assert_eq!(stats.episodes_enqueued, 1);

        // Verify PodcastUrl track source is preserved after refactoring
        let (cmd, _) = cmd_rx.try_recv().expect("should receive command");
        match cmd {
            PlayerCmd::PlaylistAddTrack(add_track) => {
                match &add_track.tracks[0] {
                    PlaylistTrackSource::PodcastUrl(url) => {
                        assert!(url.starts_with("http"), "Should be network URL: {url}");
                    }
                    other => panic!("Expected PodcastUrl after helper extraction, got: {:?}", other),
                }
            }
            other => panic!("Expected PlaylistAddTrack, got: {:?}", other),
        }
    }

    // =========================================================================
    // AC-09 / SCENARIO-012, SCENARIO-013, SCENARIO-028: Episode played filter
    //
    // T-29: Played episodes must be excluded from download consideration.
    // T-30: (Deleted episodes excluded -- handled by get_episodes hidden filter)
    //
    // These tests will FAIL because the current implementation does NOT filter
    // on ep.played -- it only checks ep.path.is_none().
    // =========================================================================

    /// SCENARIO-012: Episode filtering excludes played episodes.
    ///
    /// Given a podcast feed containing episodes where some are marked as played,
    /// when the sync process filters episodes for download,
    /// then only the unplayed episodes are considered for download.
    ///
    /// AC-09: The episode filtering logic MUST exclude already-played episodes.
    #[tokio::test]
    async fn sync_once_excludes_played_episodes_from_download() {
        let mock_server = MockServer::start().await;

        // Feed has 3 episodes
        let ep1_path = "/episodes/played_test_ep1.mp3";
        let ep2_path = "/episodes/played_test_ep2.mp3";
        let ep3_path = "/episodes/played_test_ep3.mp3";
        let feed_xml = generate_rss_feed(
            "Played Filter Podcast",
            &[
                (
                    "Unplayed Episode 1",
                    "guid-played-001",
                    &format!("{}{}", mock_server.uri(), ep1_path),
                ),
                (
                    "Played Episode 2",
                    "guid-played-002",
                    &format!("{}{}", mock_server.uri(), ep2_path),
                ),
                (
                    "Unplayed Episode 3",
                    "guid-played-003",
                    &format!("{}{}", mock_server.uri(), ep3_path),
                ),
            ],
        );

        Mock::given(method("GET"))
            .and(path("/played_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        // Only mock downloads for unplayed episodes
        Mock::given(method("GET"))
            .and(path(ep1_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .mount(&mock_server)
            .await;

        // ep2 is played -- should NOT be downloaded
        // We intentionally do NOT mock it so that if sync tries to download it,
        // the request goes unmatched.
        Mock::given(method("GET"))
            .and(path(ep2_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .expect(0) // Should NOT be requested
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(ep3_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .mount(&mock_server)
            .await;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");

        // Insert podcast with all 3 episodes pre-existing in DB
        // (simulating a previous sync that fetched the feed metadata)
        let episodes = vec![
            EpisodeNoId {
                title: "Unplayed Episode 1".to_string(),
                url: format!("{}{}", mock_server.uri(), ep1_path),
                guid: "guid-played-001".to_string(),
                description: String::new(),
                pubdate: None,
                duration: Some(300),
                image_url: None,
            },
            EpisodeNoId {
                title: "Played Episode 2".to_string(),
                url: format!("{}{}", mock_server.uri(), ep2_path),
                guid: "guid-played-002".to_string(),
                description: String::new(),
                pubdate: None,
                duration: Some(300),
                image_url: None,
            },
            EpisodeNoId {
                title: "Unplayed Episode 3".to_string(),
                url: format!("{}{}", mock_server.uri(), ep3_path),
                guid: "guid-played-003".to_string(),
                description: String::new(),
                pubdate: None,
                duration: Some(300),
                image_url: None,
            },
        ];
        let podcast = PodcastNoId {
            title: "Played Filter Podcast".to_string(),
            url: format!("{}/played_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes,
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        // Mark episode 2 as played
        let podcasts = db.get_podcasts().expect("get podcasts");
        let ep2 = podcasts[0]
            .episodes
            .iter()
            .find(|e| e.guid == "guid-played-002")
            .expect("find ep2");
        db.set_played_status(ep2.id, true)
            .expect("mark ep2 as played");
        drop(db);

        let config = make_test_config(&download_dir);
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());
        let stats = result.unwrap();

        // Only 2 episodes should be downloaded (ep1 and ep3; ep2 is played)
        assert_eq!(
            stats.episodes_downloaded, 2,
            "Only unplayed episodes should be downloaded. Got {} instead of 2. \
             Played episodes must be excluded from download (AC-09).",
            stats.episodes_downloaded
        );
        assert_eq!(
            stats.episodes_enqueued, 2,
            "Only unplayed episodes should be enqueued"
        );

        // Verify we got exactly 2 commands
        let mut commands_count = 0;
        while cmd_rx.try_recv().is_ok() {
            commands_count += 1;
        }
        assert_eq!(commands_count, 2, "should have 2 enqueue commands for 2 unplayed episodes");
    }

    /// SCENARIO-013: A podcast episode that has never been downloaded but is
    /// marked as played should be skipped during sync.
    ///
    /// AC-09: Never-downloaded but played episode is not downloaded.
    #[tokio::test]
    async fn sync_once_skips_never_downloaded_but_played_episode() {
        let mock_server = MockServer::start().await;

        let ep_path = "/episodes/never_dl_played.mp3";
        let feed_xml = generate_rss_feed(
            "Never DL Played Podcast",
            &[(
                "Never Downloaded But Played",
                "guid-never-dl-001",
                &format!("{}{}", mock_server.uri(), ep_path),
            )],
        );

        Mock::given(method("GET"))
            .and(path("/never_dl_played_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        // Should NOT be downloaded -- expect 0 requests
        Mock::given(method("GET"))
            .and(path(ep_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .expect(0) // Must NOT be requested
            .mount(&mock_server)
            .await;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");

        // Insert podcast with the episode in DB (no file path -- never downloaded)
        let episode = EpisodeNoId {
            title: "Never Downloaded But Played".to_string(),
            url: format!("{}{}", mock_server.uri(), ep_path),
            guid: "guid-never-dl-001".to_string(),
            description: String::new(),
            pubdate: None,
            duration: Some(300),
            image_url: None,
        };
        let podcast = PodcastNoId {
            title: "Never DL Played Podcast".to_string(),
            url: format!("{}/never_dl_played_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![episode],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        // Mark the episode as played (even though never downloaded)
        let podcasts = db.get_podcasts().expect("get podcasts");
        let ep = &podcasts[0].episodes[0];
        assert!(ep.path.is_none(), "Episode should not have a file path");
        db.set_played_status(ep.id, true)
            .expect("mark as played");
        drop(db);

        let config = make_test_config(&download_dir);
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());
        let stats = result.unwrap();

        // The played episode should NOT be downloaded even though it has no file path
        assert_eq!(
            stats.episodes_downloaded, 0,
            "Played episode should not be downloaded even without a file path (AC-09). \
             Got {} downloads but expected 0.",
            stats.episodes_downloaded
        );
        assert_eq!(
            stats.episodes_enqueued, 0,
            "Played episode should not be enqueued"
        );

        // No commands should have been sent
        assert!(
            cmd_rx.try_recv().is_err(),
            "No PlaylistAddTrack commands should be sent for played episodes"
        );
    }

    /// SCENARIO-028: All episodes in feed are already played.
    ///
    /// Given a podcast feed where every episode is already marked as played,
    /// when the sync process filters episodes for download,
    /// then zero episodes are selected for download.
    ///
    /// AC-09: When all episodes are played, no downloads should occur.
    #[tokio::test]
    async fn sync_once_all_played_episodes_downloads_zero() {
        let mock_server = MockServer::start().await;

        let ep1_path = "/episodes/all_played_ep1.mp3";
        let ep2_path = "/episodes/all_played_ep2.mp3";
        let ep3_path = "/episodes/all_played_ep3.mp3";
        let feed_xml = generate_rss_feed(
            "All Played Podcast",
            &[
                (
                    "Played Ep 1",
                    "guid-allplayed-001",
                    &format!("{}{}", mock_server.uri(), ep1_path),
                ),
                (
                    "Played Ep 2",
                    "guid-allplayed-002",
                    &format!("{}{}", mock_server.uri(), ep2_path),
                ),
                (
                    "Played Ep 3",
                    "guid-allplayed-003",
                    &format!("{}{}", mock_server.uri(), ep3_path),
                ),
            ],
        );

        Mock::given(method("GET"))
            .and(path("/all_played_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        // None should be downloaded
        for ep in [ep1_path, ep2_path, ep3_path] {
            Mock::given(method("GET"))
                .and(path(ep))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_bytes(fake_audio_content())
                        .insert_header("content-type", "audio/mpeg"),
                )
                .expect(0) // Must NOT be requested
                .mount(&mock_server)
                .await;
        }

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");

        let episodes: Vec<EpisodeNoId> = (1..=3)
            .map(|i| EpisodeNoId {
                title: format!("Played Ep {i}"),
                url: format!(
                    "{}/episodes/all_played_ep{i}.mp3",
                    mock_server.uri()
                ),
                guid: format!("guid-allplayed-{i:03}"),
                description: String::new(),
                pubdate: None,
                duration: Some(300),
                image_url: None,
            })
            .collect();

        let podcast = PodcastNoId {
            title: "All Played Podcast".to_string(),
            url: format!("{}/all_played_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes,
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        // Mark ALL episodes as played
        let podcasts = db.get_podcasts().expect("get podcasts");
        for ep in &podcasts[0].episodes {
            db.set_played_status(ep.id, true)
                .expect("mark as played");
        }
        drop(db);

        let config = make_test_config(&download_dir);
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());
        let stats = result.unwrap();

        assert_eq!(
            stats.episodes_downloaded, 0,
            "All-played podcast should produce zero downloads (AC-09, SCENARIO-028). \
             Got {} downloads but expected 0.",
            stats.episodes_downloaded
        );
        assert_eq!(stats.episodes_enqueued, 0, "No enqueue for all-played");
        assert_eq!(stats.episodes_failed, 0, "No failures expected");

        assert!(
            cmd_rx.try_recv().is_err(),
            "No commands should be sent when all episodes are played"
        );
    }

    // =========================================================================
    // AC-17 / SCENARIO-017: max_new_episodes limit applied BEFORE disk-check
    //
    // T-31: The max_new_episodes limit must be applied after played/deleted
    // filters but BEFORE the disk-check loop. This means if max_new_episodes=2
    // and there are 10 unplayed episodes, only 2 should have disk I/O performed.
    //
    // We verify this by checking that only max_new_episodes episodes are
    // downloaded, even when more are available and unplayed.
    // =========================================================================

    /// SCENARIO-017: Max new episodes limit applied before disk-check loop.
    ///
    /// Given a podcast feed with many new episodes and a max_new_episodes limit,
    /// only the limited number should be downloaded. This test ensures the limit
    /// is enforced and only the newest N episodes are considered.
    ///
    /// AC-17: max_new_episodes applied early in pipeline.
    #[tokio::test]
    async fn sync_once_respects_max_new_episodes_limit() {
        let mock_server = MockServer::start().await;

        // Create a feed with 10 episodes
        let episodes_data: Vec<(String, String, String)> = (1..=10)
            .map(|i| {
                (
                    format!("Episode {i}"),
                    format!("guid-limit-{i:03}"),
                    format!("{}/episodes/limit_ep{i}.mp3", mock_server.uri()),
                )
            })
            .collect();

        let ep_refs: Vec<(&str, &str, &str)> = episodes_data
            .iter()
            .map(|(t, g, u)| (t.as_str(), g.as_str(), u.as_str()))
            .collect();

        let feed_xml = generate_rss_feed("Limit Test Podcast", &ep_refs);

        Mock::given(method("GET"))
            .and(path("/limit_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        // Mock all episode downloads
        for i in 1..=10 {
            Mock::given(method("GET"))
                .and(path(format!("/episodes/limit_ep{i}.mp3")))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_bytes(fake_audio_content())
                        .insert_header("content-type", "audio/mpeg"),
                )
                .mount(&mock_server)
                .await;
        }

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");
        let podcast = PodcastNoId {
            title: "Limit Test Podcast".to_string(),
            url: format!("{}/limit_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");
        drop(db);

        // Set max_new_episodes to 3
        let config = make_test_config_with_max_episodes(&download_dir, 3);
        let (cmd_tx, _cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());
        let stats = result.unwrap();

        // Only 3 episodes should be downloaded (limited by max_new_episodes)
        assert_eq!(
            stats.episodes_downloaded, 3,
            "max_new_episodes=3 should limit downloads to 3. Got {}. \
             AC-17: limit must be applied before disk I/O.",
            stats.episodes_downloaded
        );
    }

    /// SCENARIO-017: When max_new_episodes limit is applied after played filter,
    /// played episodes should not consume limit slots.
    ///
    /// Given 5 episodes where 2 NEWEST are played and max_new_episodes=2,
    /// without the played filter, those 2 newest (played) ones would be selected.
    /// WITH the played filter, the 3 older unplayed episodes should fill the limit.
    ///
    /// AC-09 + AC-17: Played filter applied before limit.
    #[tokio::test]
    async fn sync_once_played_filter_applied_before_max_episodes_limit() {
        let mock_server = MockServer::start().await;

        // 5 episodes with descending pubdates (ep1 newest, ep5 oldest)
        // We mark eps 1 and 2 (NEWEST) as played.
        // Without played filter: take(2) would select eps 1,2 (newest first) = played ones
        // With played filter: take(2) selects from unplayed (eps 3,4,5) = 2 unplayed ones
        let feed_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
    <channel>
        <title>PlayedLimit Podcast</title>
        <link>http://example.com</link>
        <description>Tests played filter before limit</description>
        <item>
            <title>Episode 1 (newest, played)</title>
            <guid>guid-playedlimit-001</guid>
            <enclosure url="{server}/episodes/playedlimit_ep1.mp3" type="audio/mpeg" length="1024"/>
            <pubDate>Fri, 25 Jun 2025 12:00:00 +0000</pubDate>
        </item>
        <item>
            <title>Episode 2 (2nd newest, played)</title>
            <guid>guid-playedlimit-002</guid>
            <enclosure url="{server}/episodes/playedlimit_ep2.mp3" type="audio/mpeg" length="1024"/>
            <pubDate>Thu, 24 Jun 2025 12:00:00 +0000</pubDate>
        </item>
        <item>
            <title>Episode 3 (unplayed)</title>
            <guid>guid-playedlimit-003</guid>
            <enclosure url="{server}/episodes/playedlimit_ep3.mp3" type="audio/mpeg" length="1024"/>
            <pubDate>Wed, 23 Jun 2025 12:00:00 +0000</pubDate>
        </item>
        <item>
            <title>Episode 4 (unplayed)</title>
            <guid>guid-playedlimit-004</guid>
            <enclosure url="{server}/episodes/playedlimit_ep4.mp3" type="audio/mpeg" length="1024"/>
            <pubDate>Tue, 22 Jun 2025 12:00:00 +0000</pubDate>
        </item>
        <item>
            <title>Episode 5 (unplayed)</title>
            <guid>guid-playedlimit-005</guid>
            <enclosure url="{server}/episodes/playedlimit_ep5.mp3" type="audio/mpeg" length="1024"/>
            <pubDate>Mon, 21 Jun 2025 12:00:00 +0000</pubDate>
        </item>
    </channel>
</rss>"#,
            server = mock_server.uri()
        );

        Mock::given(method("GET"))
            .and(path("/playedlimit_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        // Episodes 1 and 2 are PLAYED -- they should NOT be downloaded
        // Without the played filter, these would be selected first (newest pubdate)
        Mock::given(method("GET"))
            .and(path("/episodes/playedlimit_ep1.mp3"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .expect(0) // Must NOT be requested -- episode is played
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/episodes/playedlimit_ep2.mp3"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .expect(0) // Must NOT be requested -- episode is played
            .mount(&mock_server)
            .await;

        // Episodes 3, 4, 5 are UNPLAYED -- eligible for download
        for i in 3..=5 {
            Mock::given(method("GET"))
                .and(path(format!("/episodes/playedlimit_ep{i}.mp3")))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_bytes(fake_audio_content())
                        .insert_header("content-type", "audio/mpeg"),
                )
                .mount(&mock_server)
                .await;
        }

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");

        // Pre-insert all 5 episodes with matching pubdates so DB order matches feed
        use chrono::TimeZone;
        let episodes: Vec<EpisodeNoId> = vec![
            EpisodeNoId {
                title: "Episode 1 (newest, played)".to_string(),
                url: format!("{}/episodes/playedlimit_ep1.mp3", mock_server.uri()),
                guid: "guid-playedlimit-001".to_string(),
                description: String::new(),
                pubdate: Some(chrono::Utc.with_ymd_and_hms(2025, 6, 25, 12, 0, 0).unwrap()),
                duration: Some(300),
                image_url: None,
            },
            EpisodeNoId {
                title: "Episode 2 (2nd newest, played)".to_string(),
                url: format!("{}/episodes/playedlimit_ep2.mp3", mock_server.uri()),
                guid: "guid-playedlimit-002".to_string(),
                description: String::new(),
                pubdate: Some(chrono::Utc.with_ymd_and_hms(2025, 6, 24, 12, 0, 0).unwrap()),
                duration: Some(300),
                image_url: None,
            },
            EpisodeNoId {
                title: "Episode 3 (unplayed)".to_string(),
                url: format!("{}/episodes/playedlimit_ep3.mp3", mock_server.uri()),
                guid: "guid-playedlimit-003".to_string(),
                description: String::new(),
                pubdate: Some(chrono::Utc.with_ymd_and_hms(2025, 6, 23, 12, 0, 0).unwrap()),
                duration: Some(300),
                image_url: None,
            },
            EpisodeNoId {
                title: "Episode 4 (unplayed)".to_string(),
                url: format!("{}/episodes/playedlimit_ep4.mp3", mock_server.uri()),
                guid: "guid-playedlimit-004".to_string(),
                description: String::new(),
                pubdate: Some(chrono::Utc.with_ymd_and_hms(2025, 6, 22, 12, 0, 0).unwrap()),
                duration: Some(300),
                image_url: None,
            },
            EpisodeNoId {
                title: "Episode 5 (unplayed)".to_string(),
                url: format!("{}/episodes/playedlimit_ep5.mp3", mock_server.uri()),
                guid: "guid-playedlimit-005".to_string(),
                description: String::new(),
                pubdate: Some(chrono::Utc.with_ymd_and_hms(2025, 6, 21, 12, 0, 0).unwrap()),
                duration: Some(300),
                image_url: None,
            },
        ];

        let podcast = PodcastNoId {
            title: "PlayedLimit Podcast".to_string(),
            url: format!("{}/playedlimit_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes,
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        // Mark episodes 1 and 2 (NEWEST by pubdate) as played
        let podcasts = db.get_podcasts().expect("get podcasts");
        for ep in &podcasts[0].episodes {
            if ep.guid == "guid-playedlimit-001" || ep.guid == "guid-playedlimit-002" {
                db.set_played_status(ep.id, true).expect("mark played");
            }
        }
        drop(db);

        // max_new_episodes = 2
        // Without played filter: take(2) from pubdate DESC = eps 1,2 (played!) -> would download played eps
        // With played filter: filter out played, then take(2) from remaining = eps 3,4 -> correct
        let config = make_test_config_with_max_episodes(&download_dir, 2);
        let (cmd_tx, _cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());
        let stats = result.unwrap();

        // Should download exactly 2 UNPLAYED episodes (eps 3 and 4, not played eps 1 and 2)
        // The expect(0) assertions on ep1 and ep2 mocks will panic on MockServer drop
        // if those episodes were downloaded, providing a second verification layer.
        assert_eq!(
            stats.episodes_downloaded, 2,
            "With 2 played (newest) and max_new_episodes=2, should download 2 unplayed episodes. \
             Got {}. AC-09: played filter before limit. AC-17: limit before disk I/O.",
            stats.episodes_downloaded
        );
    }

    // =========================================================================
    // T-32, T-33, T-34, T-35: Auto-enqueue guard
    //
    // When auto_enqueue is false, sync_once should download episodes but NOT
    // send PlaylistAddTrack commands.
    //
    // These tests will FAIL because the current sync_once does NOT read or
    // check the auto_enqueue config field.
    // =========================================================================

    /// T-35 / SCENARIO-011: sync_once with auto_enqueue=false downloads
    /// episodes but does NOT send any PlaylistAddTrack commands.
    ///
    /// AC-08: auto_enqueue guard around enqueue logic.
    #[tokio::test]
    async fn sync_once_auto_enqueue_false_downloads_without_enqueue() {
        let mock_server = MockServer::start().await;

        let ep_path = "/episodes/no_enqueue_ep.mp3";
        let feed_xml = generate_rss_feed(
            "No Enqueue Podcast",
            &[(
                "Episode Without Enqueue",
                "guid-noenqueue-001",
                &format!("{}{}", mock_server.uri(), ep_path),
            )],
        );

        Mock::given(method("GET"))
            .and(path("/no_enqueue_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(ep_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .mount(&mock_server)
            .await;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");
        let podcast = PodcastNoId {
            title: "No Enqueue Podcast".to_string(),
            url: format!("{}/no_enqueue_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");
        drop(db);

        // Configure auto_enqueue = false
        let config = make_test_config_with_auto_enqueue(&download_dir, false);
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());
        let stats = result.unwrap();

        // Episode should be downloaded
        assert_eq!(
            stats.episodes_downloaded, 1,
            "Episode should still be downloaded even with auto_enqueue=false"
        );

        // But NOT enqueued
        assert_eq!(
            stats.episodes_enqueued, 0,
            "With auto_enqueue=false, episodes_enqueued must be 0. Got {}. \
             The auto_enqueue config flag must be read and used as a guard \
             around enqueue logic (T-32, T-33, T-34).",
            stats.episodes_enqueued
        );

        // No PlaylistAddTrack commands should have been sent
        assert!(
            cmd_rx.try_recv().is_err(),
            "No PlaylistAddTrack commands should be sent when auto_enqueue=false. \
             The current implementation always enqueues regardless of config."
        );
    }

    /// T-35: sync_once with auto_enqueue=false should also suppress enqueue
    /// for episodes found already on disk (the existing-file-on-disk path).
    ///
    /// AC-08: auto_enqueue guard at the existing-file-on-disk enqueue point.
    #[tokio::test]
    async fn sync_once_auto_enqueue_false_suppresses_existing_file_enqueue() {
        let mock_server = MockServer::start().await;

        let ep_path = "/episodes/existing_no_enqueue.mp3";
        let feed_xml = generate_rss_feed(
            "Existing NoEnqueue Podcast",
            &[(
                "Episode Found On Disk",
                "guid-existnoenq-001",
                &format!("{}{}", mock_server.uri(), ep_path),
            )],
        );

        Mock::given(method("GET"))
            .and(path("/existing_no_enqueue_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        // Pre-create the podcast download directory and a file that matches
        // the episode title pattern
        let pod_dir = download_dir.join("Existing NoEnqueue Podcast");
        std::fs::create_dir_all(&pod_dir).expect("create podcast dir");

        let db = Database::new(db_path).expect("create database");

        let episode = EpisodeNoId {
            title: "Episode Found On Disk".to_string(),
            url: format!("{}{}", mock_server.uri(), ep_path),
            guid: "guid-existnoenq-001".to_string(),
            description: String::new(),
            pubdate: None,
            duration: Some(300),
            image_url: None,
        };
        let podcast = PodcastNoId {
            title: "Existing NoEnqueue Podcast".to_string(),
            url: format!("{}/existing_no_enqueue_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![episode],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        // Get total_episodes count for title format calculation
        let podcasts = db.get_podcasts().expect("get podcasts");
        let ep = &podcasts[0].episodes[0];
        let total_episodes = podcasts[0].episodes.len();

        // Calculate the filename pattern sync_once uses
        let ep_title = format!(
            "{:03} - {}",
            total_episodes - 0, // first episode, index 0
            ep.title
        );
        let sanitized_title = sanitize_filename::sanitize_with_options(
            &ep_title,
            sanitize_filename::Options {
                truncate: true,
                windows: true,
                replacement: "",
            },
        );
        // Create a matching file on disk
        let file_path = pod_dir.join(format!("{sanitized_title}.mp3"));
        std::fs::write(&file_path, fake_audio_content()).expect("write file");
        drop(db);

        // Configure auto_enqueue = false
        let config = make_test_config_with_auto_enqueue(&download_dir, false);
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());
        let stats = result.unwrap();

        // Even though the file was found on disk and registered in DB,
        // NO enqueue command should be sent when auto_enqueue=false
        assert_eq!(
            stats.episodes_enqueued, 0,
            "With auto_enqueue=false, existing files found on disk must NOT be enqueued. \
             Got {} enqueued. T-32: auto_enqueue guard at existing-file point.",
            stats.episodes_enqueued
        );

        assert!(
            cmd_rx.try_recv().is_err(),
            "No PlaylistAddTrack commands should be sent when auto_enqueue=false, \
             even for files found already on disk."
        );
    }

    /// T-35: sync_once with auto_enqueue=true (default) should still enqueue
    /// episodes as before, validating backward compatibility.
    ///
    /// This ensures the auto_enqueue guard doesn't accidentally suppress
    /// enqueue when the config is set to true.
    #[tokio::test]
    async fn sync_once_auto_enqueue_true_still_enqueues_episodes() {
        let mock_server = MockServer::start().await;

        let ep_path = "/episodes/yes_enqueue_ep.mp3";
        let feed_xml = generate_rss_feed(
            "Yes Enqueue Podcast",
            &[(
                "Episode With Enqueue",
                "guid-yesenqueue-001",
                &format!("{}{}", mock_server.uri(), ep_path),
            )],
        );

        Mock::given(method("GET"))
            .and(path("/yes_enqueue_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(ep_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .mount(&mock_server)
            .await;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");
        let podcast = PodcastNoId {
            title: "Yes Enqueue Podcast".to_string(),
            url: format!("{}/yes_enqueue_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");
        drop(db);

        // Configure auto_enqueue = true (explicit)
        let config = make_test_config_with_auto_enqueue(&download_dir, true);
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());
        let stats = result.unwrap();

        assert_eq!(stats.episodes_downloaded, 1);
        assert_eq!(
            stats.episodes_enqueued, 1,
            "With auto_enqueue=true, episodes should be enqueued as before"
        );

        // Should receive exactly 1 command
        let (cmd, _) = cmd_rx.try_recv().expect("should receive command with auto_enqueue=true");
        assert!(matches!(cmd, PlayerCmd::PlaylistAddTrack(_)));
    }

    // =========================================================================
    // T-34: auto_enqueue is read from config at the top of sync_once
    //
    // This test verifies that the config value is actually read, not hardcoded.
    // We do this by running sync_once twice with different configs and observing
    // different behavior.
    // =========================================================================

    /// T-34: The auto_enqueue value must be read from config, not hardcoded.
    /// Running sync_once with different configs must produce different behavior.
    #[tokio::test]
    async fn sync_once_reads_auto_enqueue_from_config_dynamically() {
        let mock_server = MockServer::start().await;

        let ep1_path = "/episodes/dynamic_enqueue_ep1.mp3";
        let ep2_path = "/episodes/dynamic_enqueue_ep2.mp3";

        // Two different feeds for two runs
        let feed1_xml = generate_rss_feed(
            "Dynamic Enqueue Test 1",
            &[(
                "Episode 1",
                "guid-dynamic-001",
                &format!("{}{}", mock_server.uri(), ep1_path),
            )],
        );
        let feed2_xml = generate_rss_feed(
            "Dynamic Enqueue Test 2",
            &[(
                "Episode 2",
                "guid-dynamic-002",
                &format!("{}{}", mock_server.uri(), ep2_path),
            )],
        );

        Mock::given(method("GET"))
            .and(path("/dynamic_feed1.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed1_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/dynamic_feed2.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed2_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(ep1_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(ep2_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .mount(&mock_server)
            .await;

        // Run 1: auto_enqueue = false
        let tmp_dir1 = tempfile::tempdir().expect("create temp dir 1");
        let db_path1 = tmp_dir1.path();
        let download_dir1 = tmp_dir1.path().join("downloads");
        std::fs::create_dir_all(&download_dir1).expect("create download dir 1");

        let db1 = Database::new(db_path1).expect("create database 1");
        let podcast1 = PodcastNoId {
            title: "Dynamic Enqueue Test 1".to_string(),
            url: format!("{}/dynamic_feed1.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db1.insert_podcast(&podcast1).expect("insert podcast 1");
        drop(db1);

        let config_false = make_test_config_with_auto_enqueue(&download_dir1, false);
        let (cmd_tx1, mut cmd_rx1) = make_cmd_channel();
        let result1 = sync_once(&config_false, &cmd_tx1, db_path1).await;
        assert!(result1.is_ok());
        let stats1 = result1.unwrap();

        // Run 2: auto_enqueue = true
        let tmp_dir2 = tempfile::tempdir().expect("create temp dir 2");
        let db_path2 = tmp_dir2.path();
        let download_dir2 = tmp_dir2.path().join("downloads");
        std::fs::create_dir_all(&download_dir2).expect("create download dir 2");

        let db2 = Database::new(db_path2).expect("create database 2");
        let podcast2 = PodcastNoId {
            title: "Dynamic Enqueue Test 2".to_string(),
            url: format!("{}/dynamic_feed2.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db2.insert_podcast(&podcast2).expect("insert podcast 2");
        drop(db2);

        let config_true = make_test_config_with_auto_enqueue(&download_dir2, true);
        let (cmd_tx2, mut cmd_rx2) = make_cmd_channel();
        let result2 = sync_once(&config_true, &cmd_tx2, db_path2).await;
        assert!(result2.is_ok());
        let stats2 = result2.unwrap();

        // Both should download
        assert_eq!(stats1.episodes_downloaded, 1, "run1 should download");
        assert_eq!(stats2.episodes_downloaded, 1, "run2 should download");

        // Only run2 (auto_enqueue=true) should enqueue
        assert_eq!(
            stats1.episodes_enqueued, 0,
            "auto_enqueue=false must produce 0 enqueued. Got {}. \
             T-34: auto_enqueue must be read from config, not hardcoded.",
            stats1.episodes_enqueued
        );
        assert_eq!(
            stats2.episodes_enqueued, 1,
            "auto_enqueue=true must produce 1 enqueued"
        );

        // Verify command channels
        assert!(
            cmd_rx1.try_recv().is_err(),
            "No commands with auto_enqueue=false"
        );
        assert!(
            cmd_rx2.try_recv().is_ok(),
            "Should have command with auto_enqueue=true"
        );
    }

    // =========================================================================
    // SCENARIO-026: Sync pass with empty podcast feed (edge case)
    //
    // After helper extraction, an empty feed should still work correctly
    // without panicking in the new helper functions.
    // =========================================================================

    /// SCENARIO-026: When a podcast feed returns zero new episodes after
    /// played filtering, no downloads should be spawned and processing
    /// continues without error.
    ///
    /// This validates that helper functions handle the empty case gracefully.
    #[tokio::test]
    async fn sync_once_empty_feed_after_played_filter_no_panic() {
        let mock_server = MockServer::start().await;

        // Feed with 2 episodes
        let ep1_path = "/episodes/emptyafter_ep1.mp3";
        let ep2_path = "/episodes/emptyafter_ep2.mp3";
        let feed_xml = generate_rss_feed(
            "Empty After Filter",
            &[
                (
                    "Played Ep 1",
                    "guid-emptyafter-001",
                    &format!("{}{}", mock_server.uri(), ep1_path),
                ),
                (
                    "Played Ep 2",
                    "guid-emptyafter-002",
                    &format!("{}{}", mock_server.uri(), ep2_path),
                ),
            ],
        );

        Mock::given(method("GET"))
            .and(path("/emptyafter_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        // Neither should be downloaded
        Mock::given(method("GET"))
            .and(path(ep1_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .expect(0)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(ep2_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .expect(0)
            .mount(&mock_server)
            .await;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");

        let episodes = vec![
            EpisodeNoId {
                title: "Played Ep 1".to_string(),
                url: format!("{}{}", mock_server.uri(), ep1_path),
                guid: "guid-emptyafter-001".to_string(),
                description: String::new(),
                pubdate: None,
                duration: Some(300),
                image_url: None,
            },
            EpisodeNoId {
                title: "Played Ep 2".to_string(),
                url: format!("{}{}", mock_server.uri(), ep2_path),
                guid: "guid-emptyafter-002".to_string(),
                description: String::new(),
                pubdate: None,
                duration: Some(300),
                image_url: None,
            },
        ];

        let podcast = PodcastNoId {
            title: "Empty After Filter".to_string(),
            url: format!("{}/emptyafter_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes,
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        // Mark all episodes as played
        let podcasts = db.get_podcasts().expect("get podcasts");
        for ep in &podcasts[0].episodes {
            db.set_played_status(ep.id, true).expect("mark played");
        }
        drop(db);

        let config = make_test_config(&download_dir);
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(
            result.is_ok(),
            "sync_once with all-played episodes after helper extraction must not panic: {:?}",
            result.err()
        );
        let stats = result.unwrap();

        assert_eq!(stats.podcasts_checked, 1, "podcast was checked");
        assert_eq!(stats.podcasts_failed, 0, "no failure expected");
        assert_eq!(
            stats.episodes_downloaded, 0,
            "no downloads when all played (SCENARIO-026 + SCENARIO-028)"
        );
        assert_eq!(stats.episodes_enqueued, 0);
        assert!(cmd_rx.try_recv().is_err());
    }

    // =========================================================================
    // T-36: Integration test for mixed played/unplayed with multiple podcasts
    //
    // Validates that the played filter works correctly across multiple podcasts
    // in the same sync pass after helper extraction.
    // =========================================================================

    /// T-36: Multiple podcasts with mixed played states should each be filtered
    /// independently. Played episodes in podcast A should not affect podcast B.
    #[tokio::test]
    async fn sync_once_played_filter_independent_per_podcast() {
        let mock_server = MockServer::start().await;

        // Podcast 1: 1 episode, played
        let pod1_ep = "/episodes/indep_pod1_ep1.mp3";
        let feed1_xml = generate_rss_feed(
            "Independent Played Podcast 1",
            &[(
                "Pod1 Played Episode",
                "guid-indep-pod1-001",
                &format!("{}{}", mock_server.uri(), pod1_ep),
            )],
        );

        // Podcast 2: 1 episode, NOT played
        let pod2_ep = "/episodes/indep_pod2_ep1.mp3";
        let feed2_xml = generate_rss_feed(
            "Independent Played Podcast 2",
            &[(
                "Pod2 Unplayed Episode",
                "guid-indep-pod2-001",
                &format!("{}{}", mock_server.uri(), pod2_ep),
            )],
        );

        Mock::given(method("GET"))
            .and(path("/indep_feed1.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed1_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/indep_feed2.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed2_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        // Pod1 ep should NOT be downloaded (played)
        Mock::given(method("GET"))
            .and(path(pod1_ep))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .expect(0) // Must NOT be requested
            .mount(&mock_server)
            .await;

        // Pod2 ep SHOULD be downloaded (unplayed)
        Mock::given(method("GET"))
            .and(path(pod2_ep))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .mount(&mock_server)
            .await;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");

        // Podcast 1 with pre-inserted episode
        let pod1_episode = EpisodeNoId {
            title: "Pod1 Played Episode".to_string(),
            url: format!("{}{}", mock_server.uri(), pod1_ep),
            guid: "guid-indep-pod1-001".to_string(),
            description: String::new(),
            pubdate: None,
            duration: Some(300),
            image_url: None,
        };
        let podcast1 = PodcastNoId {
            title: "Independent Played Podcast 1".to_string(),
            url: format!("{}/indep_feed1.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![pod1_episode],
            image_url: None,
        };
        db.insert_podcast(&podcast1).expect("insert podcast 1");

        // Podcast 2 with pre-inserted episode
        let pod2_episode = EpisodeNoId {
            title: "Pod2 Unplayed Episode".to_string(),
            url: format!("{}{}", mock_server.uri(), pod2_ep),
            guid: "guid-indep-pod2-001".to_string(),
            description: String::new(),
            pubdate: None,
            duration: Some(300),
            image_url: None,
        };
        let podcast2 = PodcastNoId {
            title: "Independent Played Podcast 2".to_string(),
            url: format!("{}/indep_feed2.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![pod2_episode],
            image_url: None,
        };
        db.insert_podcast(&podcast2).expect("insert podcast 2");

        // Mark podcast 1's episode as played
        let podcasts = db.get_podcasts().expect("get podcasts");
        for pod in &podcasts {
            for ep in &pod.episodes {
                if ep.guid == "guid-indep-pod1-001" {
                    db.set_played_status(ep.id, true).expect("mark played");
                }
            }
        }
        drop(db);

        let config = make_test_config(&download_dir);
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());
        let stats = result.unwrap();

        assert_eq!(stats.podcasts_checked, 2);
        assert_eq!(stats.podcasts_failed, 0);

        // Only podcast 2's unplayed episode should be downloaded
        assert_eq!(
            stats.episodes_downloaded, 1,
            "Only the unplayed episode from podcast 2 should be downloaded. \
             Got {}. Played filter must be applied per-podcast independently.",
            stats.episodes_downloaded
        );
        assert_eq!(stats.episodes_enqueued, 1);

        // Verify command has PodcastUrl with podcast 2's episode URL
        let (cmd, _) = cmd_rx.try_recv().expect("should have 1 command");
        match cmd {
            PlayerCmd::PlaylistAddTrack(add_track) => {
                match &add_track.tracks[0] {
                    PlaylistTrackSource::PodcastUrl(url) => {
                        assert!(
                            url.contains("indep_pod2_ep1"),
                            "Enqueued URL should be from podcast 2's unplayed episode: {url}"
                        );
                    }
                    other => panic!("Expected PodcastUrl, got: {:?}", other),
                }
            }
            other => panic!("Expected PlaylistAddTrack, got: {:?}", other),
        }

        // No more commands
        assert!(cmd_rx.try_recv().is_err(), "only 1 command expected");
    }
}
