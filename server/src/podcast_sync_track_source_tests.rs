//! Phase 1 RED Tests: Critical Bug Fixes for PlaylistTrackSource
//!
//! These tests verify AC-01 and AC-02: podcast episodes MUST be enqueued
//! using `PlaylistTrackSource::PodcastUrl(episode_url)` instead of
//! `PlaylistTrackSource::Path(file_path)`.
//!
//! Coverage:
//! - AC-01: Existing file on disk uses PodcastUrl (SCENARIO-001)
//! - AC-02: Newly downloaded episode uses PodcastUrl (SCENARIO-002)
//! - AC-01+AC-02: PodcastUrl enables podcast-specific player behaviors (SCENARIO-003)
//!
//! These tests are expected to FAIL (RED) against the current implementation
//! which incorrectly uses PlaylistTrackSource::Path at both enqueue points.

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::Duration;

    use termusiclib::config::v2::server::synchronization::SynchronizationSettings;
    use termusiclib::config::v2::server::{PodcastSettings, ServerSettings};
    use termusiclib::config::{ServerOverlay, SharedServerSettings, new_shared_server_settings};
    use termusiclib::player::playlist_helpers::{PlaylistAddTrack, PlaylistTrackSource};
    use termusiclib::podcast::PodcastNoId;
    use termusiclib::podcast::db::Database;
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
    // AC-01 / SCENARIO-001: Already-downloaded episode uses PodcastUrl
    //
    // When a podcast episode file already exists on disk and is registered
    // during sync, the track source MUST be PodcastUrl(episode_url), NOT
    // Path(file_path).
    // =========================================================================

    /// SCENARIO-001: When an episode file already exists on disk and is found
    /// during the sync pass, the enqueue command must use PodcastUrl with
    /// the episode's original network URL, not the local file path.
    ///
    /// This test creates a podcast with an undownloaded episode in the DB,
    /// places a matching file on disk (simulating a prior manual download),
    /// then runs sync_once. The sync pass should detect the file, register it,
    /// and enqueue using PodcastUrl(episode_url).
    ///
    /// AC-01: MUST use PlaylistTrackSource::PodcastUrl(ep.url.clone())
    #[tokio::test]
    async fn existing_file_on_disk_enqueues_with_podcast_url_not_path() {
        let mock_server = MockServer::start().await;

        // RSS feed with one episode whose URL is the mock server address
        let episode_url = format!("{}/episodes/existing_ep.mp3", mock_server.uri());
        let feed_xml = generate_rss_feed(
            "Track Source Test Podcast",
            &[(
                "Existing On Disk Episode",
                "guid-existing-ondisk-001",
                &episode_url,
            )],
        );

        Mock::given(method("GET"))
            .and(path("/track_source_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        // Do NOT mock the download endpoint -- the file already exists on disk,
        // so no HTTP download should be attempted.

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        // Create the podcast subdirectory and place a file that matches
        // the episode title prefix (this is how sync detects existing files)
        let pod_dir = download_dir.join("Track Source Test Podcast");
        std::fs::create_dir_all(&pod_dir).expect("create podcast dir");

        // The sync code formats episode titles as "{idx:03} - {title}" then
        // sanitizes. For index 1 of 1 total episodes, it would be "001 - Existing On Disk Episode"
        // Actually the format is total_episodes - idx, so for 1 episode at index 0:
        // total_episodes = 1, idx = 0, so ep_title = "001 - Existing On Disk Episode"
        let fake_file_path = pod_dir.join("001 - Existing On Disk Episode.mp3");
        std::fs::write(&fake_file_path, fake_audio_content()).expect("write fake file");

        let db = Database::new(db_path).expect("create database");
        let podcast = PodcastNoId {
            title: "Track Source Test Podcast".to_string(),
            url: format!("{}/track_source_feed.xml", mock_server.uri()),
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
        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());

        let stats = result.unwrap();
        // The episode was found on disk, so it counts as "downloaded" and "enqueued"
        assert!(
            stats.episodes_downloaded >= 1 || stats.episodes_enqueued >= 1,
            "At least one episode should have been registered/enqueued, stats: {:?}",
            stats
        );

        // Verify the PlaylistAddTrack command uses PodcastUrl, NOT Path
        let mut found_podcast_url = false;
        while let Ok((cmd, _callback)) = cmd_rx.try_recv() {
            match cmd {
                PlayerCmd::PlaylistAddTrack(add_track) => {
                    for track in &add_track.tracks {
                        match track {
                            PlaylistTrackSource::PodcastUrl(url) => {
                                // CORRECT: PodcastUrl with the episode's network URL
                                assert!(
                                    url.contains("/episodes/existing_ep.mp3"),
                                    "PodcastUrl should contain the episode network URL, got: {url}"
                                );
                                found_podcast_url = true;
                            }
                            PlaylistTrackSource::Path(p) => {
                                panic!(
                                    "AC-01 VIOLATION: Existing file on disk was enqueued with \
                                     PlaylistTrackSource::Path({p}) instead of \
                                     PlaylistTrackSource::PodcastUrl(episode_url). \
                                     Podcast episodes MUST use PodcastUrl for resume tracking."
                                );
                            }
                            PlaylistTrackSource::Url(u) => {
                                panic!(
                                    "Unexpected PlaylistTrackSource::Url({u}) for podcast episode"
                                );
                            }
                        }
                    }
                }
                _ => {} // Ignore non-playlist commands
            }
        }

        assert!(
            found_podcast_url,
            "Expected at least one PlaylistAddTrack command with PodcastUrl variant"
        );
    }

    // =========================================================================
    // AC-02 / SCENARIO-002: Newly-downloaded episode uses PodcastUrl
    //
    // When a podcast episode is freshly downloaded during a sync pass,
    // the enqueue command MUST use PodcastUrl(episode_url), NOT Path(file_path).
    // =========================================================================

    /// SCENARIO-002: When an episode is newly downloaded during a sync pass,
    /// the enqueue command must use PodcastUrl with the episode's network URL,
    /// not the local file path where the download was saved.
    ///
    /// This test sets up a mock server serving both the RSS feed and the
    /// episode audio file, runs sync_once, and verifies that the enqueued
    /// track uses PodcastUrl.
    ///
    /// AC-02: MUST use PlaylistTrackSource::PodcastUrl(ep_data.url.clone())
    #[tokio::test]
    async fn newly_downloaded_episode_enqueues_with_podcast_url_not_path() {
        let mock_server = MockServer::start().await;

        let ep_download_path = "/episodes/new_download.mp3";
        let episode_url = format!("{}{}", mock_server.uri(), ep_download_path);

        let feed_xml = generate_rss_feed(
            "Download Track Source Podcast",
            &[(
                "Freshly Downloaded Episode",
                "guid-fresh-dl-001",
                &episode_url,
            )],
        );

        Mock::given(method("GET"))
            .and(path("/download_track_source_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(ep_download_path))
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
            title: "Download Track Source Podcast".to_string(),
            url: format!("{}/download_track_source_feed.xml", mock_server.uri()),
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
        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());

        let stats = result.unwrap();
        assert_eq!(
            stats.episodes_downloaded, 1,
            "Expected 1 episode to be downloaded"
        );
        assert_eq!(
            stats.episodes_enqueued, 1,
            "Expected 1 episode to be enqueued"
        );

        // Verify the PlaylistAddTrack command uses PodcastUrl, NOT Path
        let (cmd, _callback) = cmd_rx
            .try_recv()
            .expect("Should have received a PlaylistAddTrack command");

        match cmd {
            PlayerCmd::PlaylistAddTrack(add_track) => {
                assert_eq!(add_track.at_index, PlaylistAddTrack::AT_END);
                assert_eq!(add_track.tracks.len(), 1);

                match &add_track.tracks[0] {
                    PlaylistTrackSource::PodcastUrl(url) => {
                        // CORRECT: PodcastUrl with the episode's network URL
                        assert_eq!(
                            url, &episode_url,
                            "PodcastUrl should contain the exact episode network URL"
                        );
                    }
                    PlaylistTrackSource::Path(p) => {
                        panic!(
                            "AC-02 VIOLATION: Newly downloaded episode was enqueued with \
                             PlaylistTrackSource::Path({p}) instead of \
                             PlaylistTrackSource::PodcastUrl({episode_url}). \
                             Podcast episodes MUST use PodcastUrl for resume tracking \
                             and played-state marking."
                        );
                    }
                    PlaylistTrackSource::Url(u) => {
                        panic!(
                            "Unexpected PlaylistTrackSource::Url({u}) for podcast episode"
                        );
                    }
                }
            }
            other => panic!("Expected PlaylistAddTrack command, got: {:?}", other),
        }
    }

    // =========================================================================
    // AC-01 + AC-02 / SCENARIO-003: PodcastUrl enables podcast-specific behaviors
    //
    // Validates that using PodcastUrl (not Path) is what enables the player
    // to recognize a track as a podcast episode for resume/played tracking.
    // =========================================================================

    /// SCENARIO-003: When a podcast episode is enqueued with PodcastUrl, the
    /// track source contains the episode's network URL (not a file path).
    /// This enables the player to:
    /// - Track resume position per episode
    /// - Mark episodes as played
    /// - Differentiate podcast tracks from local music files
    ///
    /// This test verifies the semantic correctness: PodcastUrl variant must
    /// contain a valid HTTP/HTTPS URL (not a local path), which the player
    /// uses as a stable identifier for the episode.
    #[tokio::test]
    async fn podcast_url_source_contains_network_url_not_file_path() {
        let mock_server = MockServer::start().await;

        let ep_path = "/episodes/podcast_behavior.mp3";
        let episode_network_url = format!("{}{}", mock_server.uri(), ep_path);

        let feed_xml = generate_rss_feed(
            "Behavior Test Podcast",
            &[(
                "Behavior Episode",
                "guid-behavior-001",
                &episode_network_url,
            )],
        );

        Mock::given(method("GET"))
            .and(path("/behavior_feed.xml"))
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
            title: "Behavior Test Podcast".to_string(),
            url: format!("{}/behavior_feed.xml", mock_server.uri()),
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
        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());

        // Collect all enqueued track sources
        let mut track_sources: Vec<PlaylistTrackSource> = Vec::new();
        while let Ok((cmd, _callback)) = cmd_rx.try_recv() {
            if let PlayerCmd::PlaylistAddTrack(add_track) = cmd {
                track_sources.extend(add_track.tracks);
            }
        }

        assert!(
            !track_sources.is_empty(),
            "Expected at least one track to be enqueued"
        );

        for source in &track_sources {
            match source {
                PlaylistTrackSource::PodcastUrl(url) => {
                    // The URL must be a network URL (http/https), NOT a file path
                    assert!(
                        url.starts_with("http://") || url.starts_with("https://"),
                        "PodcastUrl must contain a network URL, got: {url}"
                    );
                    // It must NOT look like a local file path
                    assert!(
                        !url.starts_with('/'),
                        "PodcastUrl must NOT be a local file path, got: {url}"
                    );
                    // It should be the episode's original network address
                    assert!(
                        url.contains("/episodes/"),
                        "PodcastUrl should contain the episode path from the feed, got: {url}"
                    );
                }
                PlaylistTrackSource::Path(p) => {
                    panic!(
                        "SCENARIO-003 VIOLATION: Track source is Path({p}) instead of PodcastUrl. \
                         Podcast-specific player behaviors (resume, played-state) will NOT activate \
                         for Path-based track sources."
                    );
                }
                PlaylistTrackSource::Url(u) => {
                    panic!(
                        "Unexpected Url variant ({u}) for podcast episode. \
                         Expected PodcastUrl for podcast-specific behavior."
                    );
                }
            }
        }
    }

    // =========================================================================
    // Anti-hardcoding: Multiple episodes test
    //
    // Ensures the fix works for MULTIPLE episodes, not just a single one.
    // This prevents a hardcoded single-case fix.
    // =========================================================================

    /// Anti-hardcoding test: When multiple episodes are downloaded in a single
    /// sync pass, ALL of them must use PodcastUrl (not just the first or last).
    /// This forces a general implementation rather than a single-point fix.
    #[tokio::test]
    async fn multiple_downloaded_episodes_all_use_podcast_url() {
        let mock_server = MockServer::start().await;

        let ep1_path = "/episodes/multi_ep1.mp3";
        let ep2_path = "/episodes/multi_ep2.mp3";
        let ep3_path = "/episodes/multi_ep3.mp3";

        let ep1_url = format!("{}{}", mock_server.uri(), ep1_path);
        let ep2_url = format!("{}{}", mock_server.uri(), ep2_path);
        let ep3_url = format!("{}{}", mock_server.uri(), ep3_path);

        let feed_xml = generate_rss_feed(
            "Multi Episode Podcast",
            &[
                ("Episode One", "guid-multi-001", &ep1_url),
                ("Episode Two", "guid-multi-002", &ep2_url),
                ("Episode Three", "guid-multi-003", &ep3_url),
            ],
        );

        Mock::given(method("GET"))
            .and(path("/multi_track_source_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        // Mock all three episode downloads
        for ep_path_item in &[ep1_path, ep2_path, ep3_path] {
            Mock::given(method("GET"))
                .and(path(*ep_path_item))
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
            title: "Multi Episode Podcast".to_string(),
            url: format!("{}/multi_track_source_feed.xml", mock_server.uri()),
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
        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());

        let stats = result.unwrap();
        assert_eq!(
            stats.episodes_downloaded, 3,
            "Expected 3 episodes to be downloaded"
        );
        assert_eq!(
            stats.episodes_enqueued, 3,
            "Expected 3 episodes to be enqueued"
        );

        // Collect all track sources from commands
        let mut podcast_urls: Vec<String> = Vec::new();
        while let Ok((cmd, _callback)) = cmd_rx.try_recv() {
            if let PlayerCmd::PlaylistAddTrack(add_track) = cmd {
                for track in &add_track.tracks {
                    match track {
                        PlaylistTrackSource::PodcastUrl(url) => {
                            podcast_urls.push(url.clone());
                        }
                        PlaylistTrackSource::Path(p) => {
                            panic!(
                                "AC-02 VIOLATION: Episode enqueued with Path({p}) instead of PodcastUrl. \
                                 ALL podcast episodes must use PodcastUrl."
                            );
                        }
                        PlaylistTrackSource::Url(u) => {
                            panic!("Unexpected Url({u}) for podcast episode");
                        }
                    }
                }
            }
        }

        assert_eq!(
            podcast_urls.len(),
            3,
            "Expected 3 PodcastUrl track sources, got {}",
            podcast_urls.len()
        );

        // Verify each URL is a valid episode network URL
        for url in &podcast_urls {
            assert!(
                url.starts_with("http"),
                "Each PodcastUrl must be a network URL, got: {url}"
            );
            assert!(
                url.contains("/episodes/"),
                "Each PodcastUrl must contain the episode path, got: {url}"
            );
        }
    }

    // =========================================================================
    // Regression guard: existing integration test expectation
    //
    // The existing test `integration_full_flow_fetches_downloads_and_enqueues_new_episodes`
    // in podcast_sync.rs (line ~1700) currently asserts PlaylistTrackSource::Path.
    // After the fix, it should assert PlaylistTrackSource::PodcastUrl.
    // This test duplicates that assertion with the CORRECT expectation.
    // =========================================================================

    /// This test mirrors the existing integration_full_flow test but with the
    /// CORRECT assertion: enqueued episodes must use PodcastUrl (not Path).
    /// This will FAIL until the bug at lines ~193 and ~255 is fixed.
    #[tokio::test]
    async fn full_sync_flow_uses_podcast_url_for_all_enqueued_episodes() {
        let mock_server = MockServer::start().await;

        let ep1_path = "/episodes/full_flow_ep1.mp3";
        let ep2_path = "/episodes/full_flow_ep2.mp3";
        let ep1_url = format!("{}{}", mock_server.uri(), ep1_path);
        let ep2_url = format!("{}{}", mock_server.uri(), ep2_path);

        let feed_xml = generate_rss_feed(
            "Full Flow PodcastUrl Test",
            &[
                ("Episode Alpha", "guid-ff-001", &ep1_url),
                ("Episode Beta", "guid-ff-002", &ep2_url),
            ],
        );

        Mock::given(method("GET"))
            .and(path("/full_flow_podcast_url_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
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

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");
        let podcast = PodcastNoId {
            title: "Full Flow PodcastUrl Test".to_string(),
            url: format!("{}/full_flow_podcast_url_feed.xml", mock_server.uri()),
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
        assert!(result.is_ok(), "sync_once should succeed: {:?}", result.err());

        let stats = result.unwrap();
        assert_eq!(stats.episodes_downloaded, 2);
        assert_eq!(stats.episodes_enqueued, 2);

        // Verify ALL PlaylistAddTrack commands use PodcastUrl
        let mut commands_checked = 0;
        while let Ok((cmd, _callback)) = cmd_rx.try_recv() {
            match cmd {
                PlayerCmd::PlaylistAddTrack(add_track) => {
                    assert_eq!(
                        add_track.at_index,
                        PlaylistAddTrack::AT_END,
                        "track should be appended at end"
                    );
                    assert_eq!(add_track.tracks.len(), 1);
                    match &add_track.tracks[0] {
                        PlaylistTrackSource::PodcastUrl(url) => {
                            assert!(
                                url.starts_with("http"),
                                "PodcastUrl must be network URL: {url}"
                            );
                        }
                        PlaylistTrackSource::Path(p) => {
                            panic!(
                                "BUG: sync_once enqueued podcast episode with Path({p}). \
                                 Must use PodcastUrl for podcast-specific player behavior."
                            );
                        }
                        PlaylistTrackSource::Url(u) => {
                            panic!("Unexpected Url({u}) for podcast episode");
                        }
                    }
                    commands_checked += 1;
                }
                other => panic!("Expected PlaylistAddTrack, got: {:?}", other),
            }
        }

        assert_eq!(
            commands_checked, 2,
            "Expected 2 PlaylistAddTrack commands with PodcastUrl"
        );
    }

    // =========================================================================
    // Negative test: No PlaylistTrackSource::Path should ever appear for
    // podcast episodes after the fix
    // =========================================================================

    /// Verifies that after the fix, grep-like inspection of the sync output
    /// finds zero instances of PlaylistTrackSource::Path for podcast episodes.
    /// This tests the behavioral contract regardless of implementation details.
    #[tokio::test]
    async fn no_path_variant_used_for_any_podcast_episode_enqueue() {
        let mock_server = MockServer::start().await;

        let ep_path = "/episodes/no_path_variant.mp3";
        let ep_url = format!("{}{}", mock_server.uri(), ep_path);

        let feed_xml = generate_rss_feed(
            "No Path Variant Podcast",
            &[("Single Episode", "guid-nopath-001", &ep_url)],
        );

        Mock::given(method("GET"))
            .and(path("/no_path_variant_feed.xml"))
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
            title: "No Path Variant Podcast".to_string(),
            url: format!("{}/no_path_variant_feed.xml", mock_server.uri()),
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
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert!(stats.episodes_enqueued >= 1, "Need at least one enqueue to test");

        // Count Path vs PodcastUrl variants
        let mut path_count = 0u32;
        let mut podcast_url_count = 0u32;

        while let Ok((cmd, _callback)) = cmd_rx.try_recv() {
            if let PlayerCmd::PlaylistAddTrack(add_track) = cmd {
                for track in &add_track.tracks {
                    match track {
                        PlaylistTrackSource::Path(_) => path_count += 1,
                        PlaylistTrackSource::PodcastUrl(_) => podcast_url_count += 1,
                        PlaylistTrackSource::Url(_) => {}
                    }
                }
            }
        }

        assert_eq!(
            path_count, 0,
            "ZERO Path variants should be used for podcast episodes, but found {path_count}"
        );
        assert!(
            podcast_url_count >= 1,
            "At least one PodcastUrl variant should be used, got {podcast_url_count}"
        );
    }
}
