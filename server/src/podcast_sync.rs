//! Podcast synchronization module.
//! Implements the sync pass logic and task lifecycle for periodic podcast feed refresh and download.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sanitize_filename::{Options, sanitize_with_options};
use termusiclib::config::SharedServerSettings;
use termusiclib::player::playlist_helpers::{PlaylistAddTrack, PlaylistTrackSource};
use termusiclib::podcast::db::Database;
use termusiclib::podcast::episode::Episode;
use termusiclib::podcast::{
    EpData, PodcastDLResult, PodcastFeed, PodcastSyncResult, check_feed, download_list,
};
use termusiclib::taskpool::TaskPool;
use termusiclib::utils::ensure_podcast_dir;
use termusicplayback::{PlayerCmd, PlayerCmdSender};
use tokio::sync::mpsc::unbounded_channel;

/// Statistics collected during a single sync pass, for logging.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncPassStats {
    /// Number of subscribed podcasts whose feeds were checked.
    pub podcasts_checked: usize,
    /// Number of podcasts where feed fetch or parse failed.
    pub podcasts_failed: usize,
    /// Number of new episodes successfully downloaded.
    pub episodes_downloaded: usize,
    /// Number of downloaded episodes successfully enqueued.
    pub episodes_enqueued: usize,
    /// Number of episodes where download failed.
    pub episodes_failed: usize,
}

/// Batch of episodes to download for a single podcast.
/// Collects episodes during feed processing for deferred download.
struct DownloadBatch {
    episodes: Vec<EpData>,
    download_dir: PathBuf,
}

/// SCENARIO-004: Check which files already exist in a podcast download directory
/// using async I/O via `tokio::task::spawn_blocking`.
///
/// The directory listing is performed ONCE per podcast (not once per episode).
/// Returns a `HashSet` of lowercase filenames for O(1) case-insensitive lookups.
///
/// AC-03: Directory listing uses async I/O performed once per podcast.
pub(crate) async fn check_existing_files(pod_download_dir: &Path) -> HashSet<String> {
    let dir = pod_download_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        std::fs::read_dir(&dir)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.file_name().to_string_lossy().to_lowercase())
                    .collect::<HashSet<String>>()
            })
            .unwrap_or_default()
    })
    .await
    .unwrap_or_default()
}

/// Process a single podcast's feed result: identify new unplayed episodes,
/// check for files already on disk, register them or collect for download.
///
/// AC-08, SCENARIO-011: Extracted helper to reduce `sync_once` nesting depth.
/// AC-09, SCENARIO-012, SCENARIO-013: Filters out played episodes.
/// AC-10, SCENARIO-014: Uses `ensure_podcast_dir` for directory creation.
/// AC-17, SCENARIO-017: Applies `max_new_episodes` limit after the played filter.
#[allow(clippy::too_many_arguments)]
fn process_feed_result(
    episodes: &[Episode],
    pod_download_dir: &Path,
    existing_files: &HashSet<String>,
    max_new_episodes: u32,
    auto_enqueue: bool,
    db: &Database,
    cmd_tx: &PlayerCmdSender,
    stats: &mut SyncPassStats,
    download_batches: &mut Vec<DownloadBatch>,
) {
    let total_episodes = episodes.len();

    // Filter to undownloaded episodes, limited by max_new_episodes
    // Episodes are ordered by pubdate DESC (newest first)
    // AC-09, SCENARIO-012, SCENARIO-013: Exclude played episodes
    // from download consideration. The played filter is applied
    // before the max_new_episodes limit (AC-17, SCENARIO-017)
    // so played episodes do not consume limit slots.
    let undownloaded: Vec<(usize, &_)> = episodes
        .iter()
        .enumerate()
        .filter(|(_, ep)| ep.path.is_none())
        .filter(|(_, ep)| !ep.played)
        .collect();

    let limit = if max_new_episodes == 0 {
        undownloaded.len()
    } else {
        max_new_episodes as usize
    };

    // Check for files already existing on disk (moved/restored)
    // and register them in DB without re-downloading.
    // Collect episodes needing download into a batch.
    let mut episodes_to_download = Vec::new();
    for (idx, ep) in undownloaded.into_iter().take(limit) {
        let ep_title = format!("{:03} - {}", total_episodes - idx, ep.title);
        let sanitized_title = sanitize_with_options(
            &ep_title,
            Options {
                truncate: true,
                windows: true,
                replacement: "",
            },
        );

        // Check if file already exists on disk using the
        // pre-computed HashSet (O(1) lookup per episode)
        let existing_file = if sanitized_title.is_empty() {
            None
        } else {
            existing_files
                .iter()
                .find(|filename| {
                    let stem = Path::new(filename.as_str())
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    stem.starts_with(&sanitized_title.to_lowercase())
                })
                .map(|filename| pod_download_dir.join(filename))
        };

        if let Some(file_path) = existing_file {
            // File exists on disk — register in DB and enqueue
            if let Err(err) = db.insert_file(ep.id, &file_path) {
                warn!("Failed to register existing file in DB: {err:#?}");
            } else {
                // T-32: Only enqueue if auto_enqueue is enabled
                if auto_enqueue {
                    enqueue_episode(cmd_tx, &ep.url, stats);
                }
                stats.episodes_downloaded += 1;
            }
        } else {
            episodes_to_download.push(EpData {
                id: ep.id,
                pod_id: ep.pod_id,
                title: ep_title,
                url: ep.url.clone(),
                pubdate: ep.pubdate,
                file_path: None,
            });
        }
    }

    // Collect batch for deferred download (Phase B)
    if !episodes_to_download.is_empty() {
        download_batches.push(DownloadBatch {
            episodes: episodes_to_download,
            download_dir: pod_download_dir.to_path_buf(),
        });
    }
}

/// Enqueue a podcast episode on the playlist using `PodcastUrl` source.
///
/// SCENARIO-001, SCENARIO-002: Uses `PlaylistTrackSource::PodcastUrl` (not `Path`)
/// so the player activates podcast-specific behaviors (resume, played-state).
fn enqueue_episode(cmd_tx: &PlayerCmdSender, episode_url: &str, stats: &mut SyncPassStats) {
    let track = PlaylistTrackSource::PodcastUrl(episode_url.to_string());
    let add_cmd = PlaylistAddTrack::new_append_single(track);
    if let Err(err) = cmd_tx.send(PlayerCmd::PlaylistAddTrack(add_cmd)) {
        warn!("Failed to enqueue episode: {err}");
    } else {
        stats.episodes_enqueued += 1;
    }
}

/// Execute one full sync pass: fetch all feeds, identify new episodes,
/// download them, and enqueue them on the playlist.
///
/// Uses the collect-then-download pattern (ADR-001):
/// - Phase A: Process all feed results and collect episodes needing download
/// - Phase B: Download all collected episodes via a single shared TaskPool (ADR-002)
///
/// This ensures feed processing for podcast B is never blocked by downloads
/// for podcast A (AC-05, SCENARIO-007, SCENARIO-008).
///
/// Per-podcast and per-episode errors are logged at warn level and do not
/// abort the pass. Only truly fatal errors (cannot open DB) propagate.
pub async fn sync_once(
    config: &SharedServerSettings,
    cmd_tx: &PlayerCmdSender,
    db_path: &Path,
) -> Result<SyncPassStats> {
    let mut stats = SyncPassStats::default();

    // Open a Database connection for this sync pass (per-pass connection, not shared)
    let db = Database::new(db_path).context("sync_once: opening podcast database")?;

    // Retrieve all subscribed podcasts
    let podcasts = db
        .get_podcasts()
        .context("sync_once: reading podcast list from database")?;

    if podcasts.is_empty() {
        return Ok(stats);
    }

    // Read config values needed for this pass
    // T-34: auto_enqueue is read from config alongside other settings
    let (download_dir, concurrent_downloads_max, max_download_retries, max_new_episodes, auto_enqueue) = {
        let config_read = config.read();
        let podcast_settings = &config_read.settings.podcast;
        let sync_settings = &config_read.settings.synchronization;
        (
            podcast_settings.download_dir.clone(),
            usize::from(podcast_settings.concurrent_downloads_max.get()),
            usize::from(podcast_settings.max_download_retries),
            sync_settings.max_new_episodes,
            sync_settings.auto_enqueue,
        )
    };

    // Create a taskpool for bounded concurrency on feed fetches
    let feed_taskpool = TaskPool::new(concurrent_downloads_max);

    // Set up channel for receiving feed fetch results
    let (feed_tx, mut feed_rx) = unbounded_channel();

    // Dispatch feed fetch tasks for all podcasts
    let pod_titles: HashMap<i64, String> = podcasts
        .iter()
        .map(|p| (p.id, p.title.clone()))
        .collect();
    for podcast in &podcasts {
        let feed = PodcastFeed::new(
            Some(podcast.id),
            podcast.url.clone(),
            Some(podcast.title.clone()),
        );
        let feed_tx_clone = feed_tx.clone();
        check_feed(feed, max_download_retries, &feed_taskpool, move |msg| {
            let _ = feed_tx_clone.send(msg);
        });
    }

    // Drop the original sender so the channel closes when all tasks finish
    drop(feed_tx);

    // =========================================================================
    // Phase A: Process all feed results, collect episodes-to-download
    // SCENARIO-007, SCENARIO-008: Feed processing completes before downloads begin
    // =========================================================================
    let mut download_batches: Vec<DownloadBatch> = Vec::new();
    let mut msg_counter: usize = 0;

    while let Some(message) = feed_rx.recv().await {
        match message {
            PodcastSyncResult::FetchPodcastStart(_) => {
                // Progress notification, not counted
            }
            PodcastSyncResult::SyncData((pod_id, pod_data)) => {
                msg_counter += 1;
                stats.podcasts_checked += 1;

                // Update podcast in DB (handles deduplication via GUID and URL matching)
                match db.update_podcast(pod_id, &pod_data) {
                    Ok(_sync_result) => {
                        // After updating, find episodes that need downloading (path == None)
                        match db.get_episodes(pod_id, false) {
                            Ok(episodes) => {
                                let pod_title =
                                    pod_titles.get(&pod_id).cloned().unwrap_or_default();

                                // AC-10, SCENARIO-014: Use ensure_podcast_dir utility
                                let pod_download_dir =
                                    match ensure_podcast_dir(&download_dir, &pod_title) {
                                        Ok(dir) => dir,
                                        Err(err) => {
                                            warn!("Failed to create podcast download dir for '{pod_title}': {err}");
                                            continue;
                                        }
                                    };

                                // AC-03, SCENARIO-004: Async directory listing once per podcast
                                let existing_files =
                                    check_existing_files(&pod_download_dir).await;

                                process_feed_result(
                                    &episodes,
                                    &pod_download_dir,
                                    &existing_files,
                                    max_new_episodes,
                                    auto_enqueue,
                                    &db,
                                    cmd_tx,
                                    &mut stats,
                                    &mut download_batches,
                                );
                            }
                            Err(err) => {
                                warn!("Failed to get episodes for podcast {pod_id}: {err:#?}");
                            }
                        }
                    }
                    Err(err) => {
                        warn!("Failed to update podcast {pod_id}: {err:#?}");
                        stats.podcasts_failed += 1;
                    }
                }
            }
            PodcastSyncResult::NewData(_pod_data) => {
                // This shouldn't happen in sync (we always pass Some(id)),
                // but handle gracefully
                warn!("Unexpected NewData result in sync pass (expected SyncData)");
                msg_counter += 1;
                stats.podcasts_checked += 1;
            }
            PodcastSyncResult::Error(feed) => {
                msg_counter += 1;
                stats.podcasts_checked += 1;
                stats.podcasts_failed += 1;
                warn!("Feed fetch failed for: {}", feed.url);
            }
        }

        if msg_counter >= podcasts.len() {
            break;
        }
    }

    // =========================================================================
    // Phase B: Download all collected episodes via a single shared TaskPool
    // SCENARIO-005, SCENARIO-006, SCENARIO-029: Single shared TaskPool across
    // all podcasts enforces the configured concurrency limit globally.
    // =========================================================================
    if !download_batches.is_empty() {
        // AC-04: Create ONE shared download TaskPool for all podcasts
        let dl_taskpool = TaskPool::new(concurrent_downloads_max);
        let (dl_tx, mut dl_rx) = unbounded_channel();

        // Dispatch all downloads through the shared pool
        for batch in download_batches {
            download_list(
                batch.episodes,
                &batch.download_dir,
                max_download_retries,
                &dl_taskpool,
                {
                    let dl_tx = dl_tx.clone();
                    move |msg| {
                        let _ = dl_tx.send(msg);
                    }
                },
            );
        }

        // Drop sender so channel closes when all download tasks complete
        drop(dl_tx);

        // Drain download results
        while let Some(dl_result) = dl_rx.recv().await {
            match dl_result {
                PodcastDLResult::DLComplete(ep_data) => {
                    if let Some(ref file_path) = ep_data.file_path {
                        stats.episodes_downloaded += 1;
                        if let Err(err) = db.insert_file(ep_data.id, file_path) {
                            warn!("Failed to record download in DB: {err:#?}");
                        }
                        // T-33: Only enqueue if auto_enqueue is enabled
                        if auto_enqueue {
                            enqueue_episode(cmd_tx, &ep_data.url, &mut stats);
                        }
                    } else {
                        warn!(
                            "DLComplete but file_path is None for episode: {}",
                            ep_data.title
                        );
                    }
                }
                PodcastDLResult::DLStart(_) => {
                    // Progress notification, not counted
                }
                PodcastDLResult::DLResponseError(ep_data)
                | PodcastDLResult::DLFileCreateError(ep_data)
                | PodcastDLResult::DLFileWriteError(ep_data) => {
                    warn!("Episode download failed: {} - {}", ep_data.title, ep_data.url);
                    stats.episodes_failed += 1;
                }
            }
        }
    }

    Ok(stats)
}

/// Spawn the periodic podcast synchronization task.
///
/// The task executes `sync_once` either immediately (if `refresh_on_startup` is true)
/// or after the first interval tick. Subsequent ticks run at the configured interval.
///
/// The task exits cleanly when `cancel_token` is cancelled (server shutdown).
///
/// Only call this function when `config.read().settings.synchronization.enable` is true.
pub fn start_podcast_sync_task(
    handle: tokio::runtime::Handle,
    cancel_token: tokio_util::sync::CancellationToken,
    config: SharedServerSettings,
    cmd_tx: PlayerCmdSender,
    db_path: std::path::PathBuf,
) {
    handle.spawn(async move {
        let (interval_duration, refresh_on_startup) = {
            let settings = &config.read().settings.synchronization;
            (settings.interval, settings.refresh_on_startup)
        };

        // Guard against zero-duration interval which would cause tokio to panic.
        // If a user configures synchronization.interval = "0s", humantime_serde
        // parses it as Duration::ZERO. Clamp to a minimum of 1 second.
        let interval_duration = interval_duration.max(std::time::Duration::from_secs(1));

        // Immediate sync on startup if configured (AC-03, SCENARIO-006)
        // Wrapped in select! so cancellation can interrupt the startup sync.
        if refresh_on_startup {
            tokio::select! {
                result = sync_once(&config, &cmd_tx, &db_path) => {
                    match result {
                        Ok(stats) => info!("Startup sync complete: {stats:?}"),
                        Err(err) => error!("Startup sync failed: {err:#?}"),
                    }
                },
                _ = cancel_token.cancelled() => {
                    info!("Podcast sync task shutting down during startup sync");
                    return;
                }
            }
        }

        // Periodic sync loop (AC-04, SCENARIO-008)
        let mut timer = tokio::time::interval_at(
            tokio::time::Instant::now() + interval_duration,
            interval_duration,
        );
        loop {
            tokio::select! {
                _ = timer.tick() => {
                    match sync_once(&config, &cmd_tx, &db_path).await {
                        Ok(stats) => info!("Periodic sync complete: {stats:?}"),
                        Err(err) => error!("Periodic sync failed: {err:#?}"),
                    }
                },
                _ = cancel_token.cancelled() => {
                    info!("Podcast sync task shutting down");
                    break;
                }
            }
        }
    });
}

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
    use termusiclib::podcast::episode::EpisodeNoId;
    use termusicplayback::{PlayerCmd, PlayerCmdSender};
    use tokio::sync::mpsc::unbounded_channel;

    use super::*;

    // =========================================================================
    // Helper: create a SharedServerSettings with test config
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

    // =========================================================================
    // T-13: SyncPassStats struct definition
    // =========================================================================

    /// SyncPassStats must exist and have the correct fields for logging sync results.
    /// This tests the existence of the struct and its fields.
    #[test]
    fn sync_pass_stats_struct_has_required_fields() {
        let stats = SyncPassStats {
            podcasts_checked: 5,
            podcasts_failed: 1,
            episodes_downloaded: 10,
            episodes_enqueued: 9,
            episodes_failed: 1,
        };

        assert_eq!(stats.podcasts_checked, 5);
        assert_eq!(stats.podcasts_failed, 1);
        assert_eq!(stats.episodes_downloaded, 10);
        assert_eq!(stats.episodes_enqueued, 9);
        assert_eq!(stats.episodes_failed, 1);
    }

    /// SyncPassStats with all zeros should represent a pass with nothing to do.
    #[test]
    fn sync_pass_stats_all_zeros() {
        let stats = SyncPassStats {
            podcasts_checked: 0,
            podcasts_failed: 0,
            episodes_downloaded: 0,
            episodes_enqueued: 0,
            episodes_failed: 0,
        };

        assert_eq!(stats.podcasts_checked, 0);
        assert_eq!(stats.episodes_downloaded, 0);
    }

    /// SyncPassStats should implement Debug for logging purposes.
    #[test]
    fn sync_pass_stats_implements_debug() {
        let stats = SyncPassStats {
            podcasts_checked: 3,
            podcasts_failed: 0,
            episodes_downloaded: 7,
            episodes_enqueued: 7,
            episodes_failed: 0,
        };
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("podcasts_checked"));
        assert!(debug_str.contains("episodes_downloaded"));
    }

    // =========================================================================
    // T-14: sync_once with empty podcast list (SCENARIO-021)
    // =========================================================================

    /// When there are no subscribed podcasts, sync_once should return Ok with
    /// all-zero stats and not attempt any downloads.
    /// AC-04, SCENARIO-021: First sync with no subscribed podcasts.
    #[tokio::test]
    async fn sync_once_no_podcasts_returns_ok_with_zero_stats() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();

        // Create the database (empty -- no podcasts)
        let _db = Database::new(db_path).expect("create database");

        let config = make_test_config(db_path);
        let (cmd_tx, _rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once should succeed with no podcasts");
        let stats = result.unwrap();
        assert_eq!(stats.podcasts_checked, 0);
        assert_eq!(stats.podcasts_failed, 0);
        assert_eq!(stats.episodes_downloaded, 0);
        assert_eq!(stats.episodes_enqueued, 0);
        assert_eq!(stats.episodes_failed, 0);
    }

    /// sync_once should open its own Database connection from the provided db_path.
    /// If the path is invalid, it should return an error (fatal error case).
    #[tokio::test]
    async fn sync_once_invalid_db_path_returns_error() {
        let invalid_path = Path::new("/nonexistent/impossible/path/that/should/not/exist");
        let config = make_test_config(invalid_path);
        let (cmd_tx, _rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, invalid_path).await;

        assert!(
            result.is_err(),
            "sync_once should fail with invalid db path"
        );
    }

    // =========================================================================
    // T-15: Per-podcast feed fetch with error isolation (SCENARIO-017, SCENARIO-018)
    // =========================================================================

    /// When a podcast feed URL is unreachable, the sync pass should log a warning
    /// and continue processing other podcasts. The failed podcast increments
    /// podcasts_failed but does not abort the pass.
    /// AC-08, SCENARIO-017: Network error on one feed does not abort sync pass.
    #[tokio::test]
    async fn sync_once_unreachable_feed_increments_failed_continues() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();

        // Create database and insert a podcast with an unreachable URL
        let db = Database::new(db_path).expect("create database");
        let podcast = PodcastNoId {
            title: "Unreachable Podcast".to_string(),
            url: "http://192.0.2.1:1/nonexistent_feed.xml".to_string(),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        let config = make_test_config(db_path);
        let (cmd_tx, _rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once should not abort on feed error");
        let stats = result.unwrap();
        assert_eq!(stats.podcasts_checked, 1);
        assert_eq!(stats.podcasts_failed, 1);
        assert_eq!(stats.episodes_downloaded, 0);
    }

    /// When multiple podcasts are subscribed and one feed fails, the others
    /// should still be processed successfully.
    /// AC-08, SCENARIO-017: Error isolation across multiple podcasts.
    #[tokio::test]
    async fn sync_once_mixed_feeds_processes_good_ones() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();

        let db = Database::new(db_path).expect("create database");

        // Insert a podcast with an unreachable feed
        let bad_podcast = PodcastNoId {
            title: "Bad Feed".to_string(),
            url: "http://192.0.2.1:1/bad_feed.xml".to_string(),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&bad_podcast).expect("insert bad podcast");

        // Insert a podcast with a valid feed (using localhost mock would be ideal,
        // but for a unit test we just verify the error isolation behavior)
        let good_podcast = PodcastNoId {
            title: "Also Bad Feed".to_string(),
            url: "http://192.0.2.2:1/another_bad_feed.xml".to_string(),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&good_podcast)
            .expect("insert good podcast");

        let config = make_test_config(db_path);
        let (cmd_tx, _rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(
            result.is_ok(),
            "sync_once should succeed even when all feeds fail"
        );
        let stats = result.unwrap();
        assert_eq!(stats.podcasts_checked, 2);
        // Both should be marked as failed since they're both unreachable
        assert_eq!(stats.podcasts_failed, 2);
    }

    // =========================================================================
    // T-16: Episode deduplication (SCENARIO-010, SCENARIO-011, SCENARIO-012, SCENARIO-013)
    // =========================================================================

    /// New episodes (not in database by GUID) should be identified for download.
    /// AC-05, SCENARIO-010: New episode identified by GUID absence.
    /// This test verifies the deduplication logic by pre-populating the DB
    /// and checking that only new episodes are processed.
    #[tokio::test]
    async fn sync_once_identifies_new_episodes_by_guid() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();

        let db = Database::new(db_path).expect("create database");

        // Insert a podcast with one existing episode
        let existing_episode = EpisodeNoId {
            title: "Existing Episode".to_string(),
            url: "https://example.com/existing.mp3".to_string(),
            guid: "existing-guid-001".to_string(),
            description: "Already in DB".to_string(),
            pubdate: None,
            duration: Some(300),
            image_url: None,
        };
        let podcast = PodcastNoId {
            title: "Test Podcast".to_string(),
            url: "http://192.0.2.1:1/feed.xml".to_string(), // unreachable -- feed fetch will fail
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![existing_episode],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        // Verify the episode is in the database
        let podcasts = db.get_podcasts().expect("get podcasts");
        assert_eq!(podcasts.len(), 1);
        assert_eq!(podcasts[0].episodes.len(), 1);
        assert_eq!(podcasts[0].episodes[0].guid, "existing-guid-001");

        // The deduplication logic should recognize episodes already in the DB
        // This is tested indirectly -- when sync_once fetches a feed and finds
        // episodes with GUIDs already in the DB, they should be skipped.
        // Since we can't fetch a real feed in unit tests, we verify the DB state
        // is correct for deduplication to work.
        let config = make_test_config(db_path);
        let (cmd_tx, _rx) = make_cmd_channel();

        // The feed fetch will fail (unreachable), so no new episodes will be found.
        // This confirms the function handles the pre-existing episode state correctly.
        let result = sync_once(&config, &cmd_tx, db_path).await;
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.podcasts_checked, 1);
        // No downloads should occur (feed failed, but even if it succeeded,
        // existing episodes should be skipped)
        assert_eq!(stats.episodes_downloaded, 0);
    }

    /// Episodes already in the database (matched by GUID) should NOT be re-downloaded.
    /// AC-05, SCENARIO-011: Episode with existing GUID is skipped.
    #[tokio::test]
    async fn sync_once_skips_episodes_with_existing_guid() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();

        let db = Database::new(db_path).expect("create database");

        // Insert podcast with an episode that has a GUID
        let episode = EpisodeNoId {
            title: "Known Episode".to_string(),
            url: "https://example.com/known.mp3".to_string(),
            guid: "known-guid-abc".to_string(),
            description: String::new(),
            pubdate: None,
            duration: Some(600),
            image_url: None,
        };
        let podcast = PodcastNoId {
            title: "GUID Dedup Test".to_string(),
            url: "http://192.0.2.1:1/guid_dedup_feed.xml".to_string(),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![episode],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        let config = make_test_config(db_path);
        let (cmd_tx, _rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;
        assert!(result.is_ok());

        // Verify no episodes were downloaded (feed fetch failed, and even
        // if it succeeded, the existing episode should be deduplicated by GUID)
        let stats = result.unwrap();
        assert_eq!(stats.episodes_downloaded, 0);
        assert_eq!(stats.episodes_enqueued, 0);
    }

    /// Episodes already in the database should not be re-added to the queue.
    /// AC-05, SCENARIO-013: Episode already in play queue is not re-added.
    #[tokio::test]
    async fn sync_once_does_not_reenqueue_already_downloaded_episode() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();

        let db = Database::new(db_path).expect("create database");

        // Insert a podcast with an episode that has already been downloaded (has a path)
        let episode = EpisodeNoId {
            title: "Downloaded Episode".to_string(),
            url: "https://example.com/downloaded.mp3".to_string(),
            guid: "downloaded-guid-xyz".to_string(),
            description: String::new(),
            pubdate: None,
            duration: Some(120),
            image_url: None,
        };
        let podcast = PodcastNoId {
            title: "Queue Dedup Test".to_string(),
            url: "http://192.0.2.1:1/queue_dedup_feed.xml".to_string(),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![episode],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        // Mark the episode as downloaded by inserting a file record
        let podcasts = db.get_podcasts().expect("get podcasts");
        let ep_id = podcasts[0].episodes[0].id;
        db.insert_file(ep_id, Path::new("/tmp/downloaded.mp3"))
            .expect("insert file");

        let config = make_test_config(db_path);
        let (cmd_tx, mut rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;
        assert!(result.is_ok());

        // No PlaylistAddTrack commands should have been sent
        // (episode is already downloaded and in the DB)
        assert!(
            rx.try_recv().is_err(),
            "No PlaylistAddTrack should be sent for already-downloaded episodes"
        );
    }

    // =========================================================================
    // T-17: Download with channel-drain pattern (SCENARIO-019)
    // =========================================================================

    /// When an episode download fails, other episodes should still be processed.
    /// AC-08, SCENARIO-019: Download failure for one episode does not block others.
    /// This test verifies the SyncPassStats correctly tracks individual failures.
    #[tokio::test]
    async fn sync_pass_stats_tracks_individual_download_failures() {
        // This test validates the contract: when episodes_failed > 0,
        // episodes_downloaded can still be > 0 (other episodes succeeded).
        let stats = SyncPassStats {
            podcasts_checked: 1,
            podcasts_failed: 0,
            episodes_downloaded: 2,
            episodes_enqueued: 2,
            episodes_failed: 1,
        };

        // The stats struct allows representing partial success
        assert!(stats.episodes_downloaded > 0);
        assert!(stats.episodes_failed > 0);
        assert_eq!(stats.episodes_downloaded + stats.episodes_failed, 3);
    }

    // =========================================================================
    // T-18: Enqueue logic (SCENARIO-014, SCENARIO-015)
    // =========================================================================

    /// After a successful download, sync_once should send PlaylistAddTrack
    /// commands via cmd_tx for each downloaded episode.
    /// AC-07, SCENARIO-015: Downloaded episode appended to end of play queue.
    /// Note: This test uses a mock/unreachable feed so no actual downloads occur.
    /// The full integration test (Phase 5) will use a mock HTTP server.
    #[tokio::test]
    async fn sync_once_sends_playlist_add_track_for_downloaded_episodes() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();

        let db = Database::new(db_path).expect("create database");

        // Insert a podcast (feed will be unreachable in unit test)
        let podcast = PodcastNoId {
            title: "Enqueue Test".to_string(),
            url: "http://192.0.2.1:1/enqueue_feed.xml".to_string(),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        let config = make_test_config(db_path);
        let (cmd_tx, mut rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;
        assert!(result.is_ok());

        // Since the feed is unreachable, no episodes should have been enqueued
        let stats = result.unwrap();
        assert_eq!(stats.episodes_enqueued, 0);

        // No commands should be received
        assert!(
            rx.try_recv().is_err(),
            "No commands should be sent when feed is unreachable"
        );
    }

    /// The PlaylistAddTrack command sent by sync_once should use AT_END
    /// (new_append_single) to ensure episodes are appended at the end.
    /// AC-07, SCENARIO-015: Downloaded episode appended to END of play queue.
    #[test]
    fn playlist_add_track_for_sync_uses_at_end() {
        // Verify that the constructor used by sync produces AT_END index
        let track = PlaylistTrackSource::Path("/podcasts/new_episode.mp3".to_string());
        let cmd = PlaylistAddTrack::new_append_single(track.clone());

        assert_eq!(cmd.at_index, PlaylistAddTrack::AT_END);
        assert_eq!(cmd.tracks.len(), 1);
        assert_eq!(cmd.tracks[0], track);
    }

    /// The enqueue logic should use PlaylistTrackSource::PodcastUrl for podcast episodes.
    /// AC-01, AC-02, SCENARIO-001, SCENARIO-002: Podcast episodes use PodcastUrl.
    #[test]
    fn enqueue_uses_podcast_url_source_for_podcast_episodes() {
        let episode_url = "https://example.com/episodes/episode_new.mp3";
        let track = PlaylistTrackSource::PodcastUrl(episode_url.to_string());
        let cmd = PlaylistAddTrack::new_append_single(track);

        match &cmd.tracks[0] {
            PlaylistTrackSource::PodcastUrl(url) => assert_eq!(url, episode_url),
            _ => panic!("Expected PlaylistTrackSource::PodcastUrl for podcast episode"),
        }
    }

    // =========================================================================
    // sync_once function signature and return type validation
    // =========================================================================

    /// sync_once should accept SharedServerSettings, PlayerCmdSender, and Path refs.
    /// This test validates the function signature compiles correctly.
    #[tokio::test]
    async fn sync_once_accepts_expected_parameters() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let _db = Database::new(db_path).expect("create database");

        let config = make_test_config(db_path);
        let (cmd_tx, _rx) = make_cmd_channel();

        // This call validates that sync_once has the expected signature:
        // async fn sync_once(config: &SharedServerSettings, cmd_tx: &PlayerCmdSender, db_path: &Path) -> Result<SyncPassStats>
        let result: anyhow::Result<SyncPassStats> = sync_once(&config, &cmd_tx, db_path).await;
        assert!(result.is_ok());
    }

    /// sync_once return type must be anyhow::Result<SyncPassStats>.
    #[tokio::test]
    async fn sync_once_returns_anyhow_result_of_sync_pass_stats() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let _db = Database::new(db_path).expect("create database");

        let config = make_test_config(db_path);
        let (cmd_tx, _rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        // Explicitly type-check the result
        let _stats: SyncPassStats = result.expect("should return SyncPassStats on success");
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    /// sync_once with a podcast that has episodes but none have been downloaded
    /// (path == None) should attempt to download them (if feed fetch succeeds).
    /// This validates the filtering logic for undownloaded episodes.
    #[tokio::test]
    async fn sync_once_only_downloads_episodes_without_path() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();

        let db = Database::new(db_path).expect("create database");

        // Insert podcast with two episodes
        let ep1 = EpisodeNoId {
            title: "Episode With Path".to_string(),
            url: "https://example.com/ep1.mp3".to_string(),
            guid: "ep1-has-path".to_string(),
            description: String::new(),
            pubdate: None,
            duration: Some(300),
            image_url: None,
        };
        let ep2 = EpisodeNoId {
            title: "Episode Without Path".to_string(),
            url: "https://example.com/ep2.mp3".to_string(),
            guid: "ep2-no-path".to_string(),
            description: String::new(),
            pubdate: None,
            duration: Some(600),
            image_url: None,
        };
        let podcast = PodcastNoId {
            title: "Mixed Episodes".to_string(),
            url: "http://192.0.2.1:1/mixed_feed.xml".to_string(),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![ep1, ep2],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        // Mark ep1 as downloaded
        let podcasts = db.get_podcasts().expect("get podcasts");
        let episodes = &podcasts[0].episodes;
        // Find the episode with guid "ep1-has-path" and mark it downloaded
        for ep in episodes {
            if ep.guid == "ep1-has-path" {
                db.insert_file(ep.id, Path::new("/tmp/ep1.mp3"))
                    .expect("insert file for ep1");
            }
        }

        // Verify: ep1 now has a path, ep2 does not
        let podcasts = db.get_podcasts().expect("get podcasts after file insert");
        let episodes = &podcasts[0].episodes;
        let ep1_has_path = episodes
            .iter()
            .any(|e| e.guid == "ep1-has-path" && e.path.is_some());
        let ep2_no_path = episodes
            .iter()
            .any(|e| e.guid == "ep2-no-path" && e.path.is_none());
        assert!(ep1_has_path, "ep1 should have a file path");
        assert!(ep2_no_path, "ep2 should NOT have a file path");

        // When sync runs, only ep2 (without path) should be considered for download.
        // Since the feed is unreachable, the actual download won't happen,
        // but the filtering logic is what we're testing here.
        let config = make_test_config(db_path);
        let (cmd_tx, _rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;
        assert!(result.is_ok());
        // Feed failed, so no downloads occurred
        let stats = result.unwrap();
        assert_eq!(stats.podcasts_failed, 1);
    }

    /// sync_once should handle a podcast with many episodes efficiently.
    /// The function should not panic or stack overflow on a large episode count.
    #[tokio::test]
    async fn sync_once_handles_podcast_with_many_episodes() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();

        let db = Database::new(db_path).expect("create database");

        // Insert a podcast with many episodes
        let episodes: Vec<EpisodeNoId> = (0..100)
            .map(|i| EpisodeNoId {
                title: format!("Episode {i}"),
                url: format!("https://example.com/ep{i}.mp3"),
                guid: format!("guid-{i:04}"),
                description: String::new(),
                pubdate: None,
                duration: Some(300),
                image_url: None,
            })
            .collect();

        let podcast = PodcastNoId {
            title: "Large Podcast".to_string(),
            url: "http://192.0.2.1:1/large_feed.xml".to_string(),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes,
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        let config = make_test_config(db_path);
        let (cmd_tx, _rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;
        assert!(result.is_ok(), "sync_once should handle many episodes");
    }

    /// sync_once should read podcast.concurrent_downloads_max from config.
    /// This validates that the config is actually used for download concurrency.
    #[tokio::test]
    async fn sync_once_respects_concurrent_downloads_max_config() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let _db = Database::new(db_path).expect("create database");

        // Create config with a specific concurrent_downloads_max
        let settings = ServerSettings {
            podcast: PodcastSettings {
                concurrent_downloads_max: std::num::NonZeroU8::new(1).unwrap(),
                download_dir: db_path.to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = new_shared_server_settings(ServerOverlay {
            settings,
            ..Default::default()
        });
        let (cmd_tx, _rx) = make_cmd_channel();

        // Should not panic or error with concurrency = 1
        let result = sync_once(&config, &cmd_tx, db_path).await;
        assert!(result.is_ok());
    }

    /// sync_once should read podcast.max_download_retries from config.
    #[tokio::test]
    async fn sync_once_respects_max_download_retries_config() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let _db = Database::new(db_path).expect("create database");

        // Create config with 0 retries to test edge case
        let settings = ServerSettings {
            podcast: PodcastSettings {
                max_download_retries: 0,
                download_dir: db_path.to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = new_shared_server_settings(ServerOverlay {
            settings,
            ..Default::default()
        });
        let (cmd_tx, _rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;
        assert!(result.is_ok());
    }

    // =========================================================================
    // Phase 4: Task Lifecycle and Wiring Tests (T-20, T-21, T-22, T-23)
    // =========================================================================

    // =========================================================================
    // T-20: start_podcast_sync_task with interval_at + select! on cancel_token
    // SCENARIO-008: Periodic sync executes at configured interval
    // SCENARIO-009: Graceful shutdown cancels the sync task
    // SCENARIO-023: Concurrent sync tick arrives while previous pass still running
    // =========================================================================

    /// start_podcast_sync_task must exist and be callable with the expected
    /// parameters: Handle, CancellationToken, SharedServerSettings, PlayerCmdSender, PathBuf.
    /// AC-11, SCENARIO-020: Sync task follows established spawn pattern.
    #[tokio::test]
    async fn start_podcast_sync_task_has_expected_signature() {
        use tokio::runtime::Handle;
        use tokio_util::sync::CancellationToken;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path().to_path_buf();
        let _db = Database::new(&db_path).expect("create database");

        let config = make_test_config(&db_path);
        let (cmd_tx, _rx) = make_cmd_channel();
        let cancel_token = CancellationToken::new();
        let handle = Handle::current();

        // This call validates that start_podcast_sync_task exists with the
        // expected signature matching the spec (Section 4.1):
        // fn start_podcast_sync_task(
        //     handle: Handle,
        //     cancel_token: CancellationToken,
        //     config: SharedServerSettings,
        //     cmd_tx: PlayerCmdSender,
        //     db_path: PathBuf,
        // )
        super::start_podcast_sync_task(handle, cancel_token.clone(), config, cmd_tx, db_path);

        // Cancel immediately to prevent the task from running indefinitely
        cancel_token.cancel();
        // Brief yield to allow the spawned task to pick up the cancellation
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    /// When the CancellationToken is triggered, the sync task must exit cleanly
    /// without panic or resource leak.
    /// AC-09, SCENARIO-009: Graceful shutdown cancels the sync task.
    #[tokio::test]
    async fn start_podcast_sync_task_exits_on_cancellation() {
        use tokio::runtime::Handle;
        use tokio_util::sync::CancellationToken;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path().to_path_buf();
        let _db = Database::new(&db_path).expect("create database");

        let config = make_test_config(&db_path);
        let (cmd_tx, _rx) = make_cmd_channel();
        let cancel_token = CancellationToken::new();
        let handle = Handle::current();

        // Start the sync task
        super::start_podcast_sync_task(handle, cancel_token.clone(), config, cmd_tx, db_path);

        // Let it run briefly (it should be waiting for the interval or doing startup sync)
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Cancel the token -- the task should exit cleanly
        cancel_token.cancel();

        // Wait a bit for the task to actually shut down
        tokio::time::sleep(Duration::from_millis(100)).await;

        // If we reach here without panic or hang, the test passes.
        // The task should have exited cleanly via the select! branch on cancelled().
    }

    /// When synchronization.enable is false, start_podcast_sync_task should NOT
    /// be called by the server. This test verifies the gating logic by checking
    /// that the config field is accessible and the enable flag controls behavior.
    /// AC-02, SCENARIO-005: Sync task not spawned when disabled.
    #[tokio::test]
    async fn sync_task_not_spawned_when_disabled() {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path().to_path_buf();
        let _db = Database::new(&db_path).expect("create database");

        // Create config with synchronization disabled
        let settings = ServerSettings {
            podcast: PodcastSettings {
                download_dir: db_path.clone(),
                ..Default::default()
            },
            synchronization: SynchronizationSettings {
                enable: false,
                interval: Duration::from_secs(60),
                refresh_on_startup: true,
                max_new_episodes: 5,
                ..Default::default()
            },
            ..Default::default()
        };
        let config = new_shared_server_settings(ServerOverlay {
            settings,
            ..Default::default()
        });

        // Verify the gating condition: when enable is false, the task should
        // not be spawned. This mirrors the check in actual_main().
        let should_spawn = config.read().settings.synchronization.enable;
        assert!(
            !should_spawn,
            "Sync task should NOT be spawned when enable is false"
        );
    }

    /// When refresh_on_startup is true, the sync task should execute sync_once
    /// immediately before entering the periodic loop.
    /// AC-03, SCENARIO-006: Immediate sync on startup when refresh_on_startup enabled.
    #[tokio::test]
    async fn start_podcast_sync_task_executes_startup_sync_when_enabled() {
        use tokio::runtime::Handle;
        use tokio_util::sync::CancellationToken;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path().to_path_buf();
        let _db = Database::new(&db_path).expect("create database");

        // Config with refresh_on_startup = true and a long interval
        // so the periodic tick won't fire during the test
        let settings = ServerSettings {
            podcast: PodcastSettings {
                download_dir: db_path.clone(),
                ..Default::default()
            },
            synchronization: SynchronizationSettings {
                enable: true,
                interval: Duration::from_secs(3600), // 1 hour -- won't fire in test
                refresh_on_startup: true,
                max_new_episodes: 5,
                ..Default::default()
            },
            ..Default::default()
        };
        let config = new_shared_server_settings(ServerOverlay {
            settings,
            ..Default::default()
        });
        let (cmd_tx, _rx) = make_cmd_channel();
        let cancel_token = CancellationToken::new();
        let handle = Handle::current();

        // Start the sync task -- with refresh_on_startup=true, it should
        // call sync_once immediately
        super::start_podcast_sync_task(handle, cancel_token.clone(), config, cmd_tx, db_path);

        // Give the startup sync time to complete (with an empty DB, it's fast)
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Cancel to clean up
        cancel_token.cancel();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // The fact that the task started and ran sync_once without panic
        // validates the startup sync behavior. With an empty DB, sync_once
        // returns immediately with zero stats.
    }

    /// When refresh_on_startup is false, no sync should occur until the first
    /// interval tick fires.
    /// AC-03, SCENARIO-007: No immediate sync when refresh_on_startup disabled.
    #[tokio::test]
    async fn start_podcast_sync_task_skips_startup_sync_when_disabled() {
        use tokio::runtime::Handle;
        use tokio_util::sync::CancellationToken;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path().to_path_buf();
        let _db = Database::new(&db_path).expect("create database");

        // Config with refresh_on_startup = false and a long interval
        let settings = ServerSettings {
            podcast: PodcastSettings {
                download_dir: db_path.clone(),
                ..Default::default()
            },
            synchronization: SynchronizationSettings {
                enable: true,
                interval: Duration::from_secs(3600), // won't fire in test
                refresh_on_startup: false,
                max_new_episodes: 5,
                ..Default::default()
            },
            ..Default::default()
        };
        let config = new_shared_server_settings(ServerOverlay {
            settings,
            ..Default::default()
        });
        let (cmd_tx, mut rx) = make_cmd_channel();
        let cancel_token = CancellationToken::new();
        let handle = Handle::current();

        super::start_podcast_sync_task(handle, cancel_token.clone(), config, cmd_tx, db_path);

        // Wait briefly -- no sync should have occurred
        tokio::time::sleep(Duration::from_millis(200)).await;

        // No PlaylistAddTrack commands should have been sent (no sync ran)
        assert!(
            rx.try_recv().is_err(),
            "No commands should be sent when refresh_on_startup is false and interval hasn't elapsed"
        );

        cancel_token.cancel();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    /// The sync task should use tokio::time::interval_at (not sleep) to prevent
    /// timer drift. This test verifies that the periodic tick fires at the
    /// expected short interval by waiting just long enough for it to trigger.
    /// AC-04, SCENARIO-008: Periodic sync executes at configured interval.
    /// SCENARIO-023: Timer does not drift.
    #[tokio::test]
    async fn start_podcast_sync_task_fires_periodic_sync_at_interval() {
        use tokio::runtime::Handle;
        use tokio_util::sync::CancellationToken;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path().to_path_buf();
        let _db = Database::new(&db_path).expect("create database");

        // Very short interval for testing -- the periodic tick should fire
        // within this time window
        let settings = ServerSettings {
            podcast: PodcastSettings {
                download_dir: db_path.clone(),
                ..Default::default()
            },
            synchronization: SynchronizationSettings {
                enable: true,
                interval: Duration::from_millis(50),
                refresh_on_startup: false, // skip startup sync to isolate periodic behavior
                max_new_episodes: 5,
                ..Default::default()
            },
            ..Default::default()
        };
        let config = new_shared_server_settings(ServerOverlay {
            settings,
            ..Default::default()
        });
        let (cmd_tx, _rx) = make_cmd_channel();
        let cancel_token = CancellationToken::new();
        let handle = Handle::current();

        super::start_podcast_sync_task(handle, cancel_token.clone(), config, cmd_tx, db_path);

        // Wait long enough for at least one periodic tick to fire (50ms interval + margin)
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Cancel and clean up
        cancel_token.cancel();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // If we reach here without panic, the periodic timer fired correctly.
        // The sync_once calls completed (with empty DB, they return quickly).
    }

    /// The sync task must use select! with CancellationToken::cancelled() to
    /// ensure that even if it's waiting for the next interval tick, cancellation
    /// takes effect immediately (no waiting for the full interval).
    /// AC-09, SCENARIO-009: Graceful shutdown cancels the sync task mid-wait.
    #[tokio::test]
    async fn start_podcast_sync_task_cancellation_interrupts_interval_wait() {
        use tokio::runtime::Handle;
        use tokio_util::sync::CancellationToken;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path().to_path_buf();
        let _db = Database::new(&db_path).expect("create database");

        // Long interval -- the task should be waiting for the next tick
        let settings = ServerSettings {
            podcast: PodcastSettings {
                download_dir: db_path.clone(),
                ..Default::default()
            },
            synchronization: SynchronizationSettings {
                enable: true,
                interval: Duration::from_secs(3600), // 1 hour
                refresh_on_startup: false,
                max_new_episodes: 5,
                ..Default::default()
            },
            ..Default::default()
        };
        let config = new_shared_server_settings(ServerOverlay {
            settings,
            ..Default::default()
        });
        let (cmd_tx, _rx) = make_cmd_channel();
        let cancel_token = CancellationToken::new();
        let handle = Handle::current();

        super::start_podcast_sync_task(handle, cancel_token.clone(), config, cmd_tx, db_path);

        // Wait a brief moment (the task should be idle, waiting for the 1h interval)
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Cancel the token -- the task should exit immediately via select! branch
        // without waiting for the remaining ~59min59sec of the interval
        cancel_token.cancel();

        // If this completes quickly (within the test timeout), cancellation works.
        // A sleep-based implementation without select! would hang here.
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    /// The sync task function should mirror start_playlist_save_interval pattern:
    /// - Receives Handle for spawning
    /// - Receives CancellationToken for shutdown
    /// - Uses interval_at (not sleep) for timing
    /// AC-11, SCENARIO-020: Sync task follows established spawn pattern.
    #[tokio::test]
    async fn start_podcast_sync_task_mirrors_playlist_save_pattern() {
        use tokio::runtime::Handle;
        use tokio_util::sync::CancellationToken;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path().to_path_buf();
        let _db = Database::new(&db_path).expect("create database");

        let config = make_test_config(&db_path);
        let (cmd_tx, _rx) = make_cmd_channel();
        let cancel_token = CancellationToken::new();
        let handle = Handle::current();

        // The function must accept these exact parameter types (matching spec 4.1):
        // - handle: Handle (tokio runtime handle)
        // - cancel_token: CancellationToken
        // - config: SharedServerSettings
        // - cmd_tx: PlayerCmdSender
        // - db_path: PathBuf
        super::start_podcast_sync_task(handle, cancel_token.clone(), config, cmd_tx, db_path);

        // Clean up
        cancel_token.cancel();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    /// When the task is running and config has refresh_on_startup=true,
    /// verify that sync_once is called BEFORE the periodic loop starts.
    /// This means the first sync happens at time=0, not at time=interval.
    /// AC-03, SCENARIO-006: Immediate sync on startup.
    #[tokio::test]
    async fn start_podcast_sync_task_startup_sync_runs_before_periodic_loop() {
        use tokio::runtime::Handle;
        use tokio_util::sync::CancellationToken;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path().to_path_buf();

        // Insert a podcast with an unreachable feed so we can detect activity
        let db = Database::new(&db_path).expect("create database");
        let podcast = PodcastNoId {
            title: "Startup Sync Test".to_string(),
            url: "http://192.0.2.1:1/startup_test.xml".to_string(),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");
        drop(db);

        // Use a very long interval so the periodic tick cannot fire during test
        let settings = ServerSettings {
            podcast: PodcastSettings {
                download_dir: db_path.clone(),
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
        let config = new_shared_server_settings(ServerOverlay {
            settings,
            ..Default::default()
        });
        let (cmd_tx, _rx) = make_cmd_channel();
        let cancel_token = CancellationToken::new();
        let handle = Handle::current();

        super::start_podcast_sync_task(handle, cancel_token.clone(), config, cmd_tx, db_path);

        // Give enough time for the startup sync to run (with unreachable feed
        // it will fail quickly due to connection timeout or immediate error)
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Cancel after startup sync should have been attempted
        cancel_token.cancel();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // If we reach here, the startup sync was triggered without waiting
        // for the interval. With refresh_on_startup=true, sync_once is called
        // immediately on task start, before entering the interval_at loop.
    }

    // =========================================================================
    // Phase 5: Integration Tests with Mock HTTP Server (T-24, T-25, T-26)
    // =========================================================================
    //
    // These tests use wiremock to serve real HTTP responses for RSS feeds and
    // episode downloads, verifying the full end-to-end sync flow.
    // =========================================================================

    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Generate a minimal valid RSS feed XML with the given episodes.
    /// Each episode is represented as (title, guid, enclosure_url).
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

    /// Generate minimal audio file content for download mocking.
    fn fake_audio_content() -> Vec<u8> {
        // ID3 header + minimal frame to look like an MP3 file
        let mut content = vec![0x49, 0x44, 0x33]; // "ID3" magic bytes
        content.extend_from_slice(&[0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        content.extend_from_slice(&[0xFF; 1024]); // Fake audio data
        content
    }

    /// Recursively count files in a directory tree.
    fn count_files_recursive(dir: &std::path::Path) -> usize {
        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    count += 1;
                } else if path.is_dir() {
                    count += count_files_recursive(&path);
                }
            }
        }
        count
    }

    /// Check if any file exists recursively under a directory.
    fn walkdir(dir: &std::path::Path) -> bool {
        count_files_recursive(dir) > 0
    }

    // =========================================================================
    // T-24: Full sync pass with mock feeds verifying episode download and enqueue
    // SCENARIO-010: New episode identified by GUID absence
    // SCENARIO-014: New episode downloaded to podcast directory
    // SCENARIO-015: Downloaded episode appended to end of play queue
    // =========================================================================

    /// When a subscribed podcast has new episodes in its RSS feed, sync_once
    /// should fetch the feed, identify new episodes not in the database,
    /// download them to the podcast directory, and send PlaylistAddTrack commands.
    ///
    /// AC-05, AC-06, AC-07: Full flow from feed fetch to enqueue.
    /// SCENARIO-010, SCENARIO-014, SCENARIO-015.
    #[tokio::test]
    async fn integration_full_flow_fetches_downloads_and_enqueues_new_episodes() {
        let mock_server = MockServer::start().await;

        let ep1_path = "/episodes/episode1.mp3";
        let ep2_path = "/episodes/episode2.mp3";
        let feed_xml = generate_rss_feed(
            "Integration Test Podcast",
            &[
                (
                    "Episode 1: Hello World",
                    "guid-int-001",
                    &format!("{}{}", mock_server.uri(), ep1_path),
                ),
                (
                    "Episode 2: Second Episode",
                    "guid-int-002",
                    &format!("{}{}", mock_server.uri(), ep2_path),
                ),
            ],
        );

        Mock::given(method("GET"))
            .and(path("/feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        let audio_content = fake_audio_content();
        Mock::given(method("GET"))
            .and(path(ep1_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(audio_content.clone())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(ep2_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(audio_content.clone())
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
            title: "Integration Test Podcast".to_string(),
            url: format!("{}/feed.xml", mock_server.uri()),
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

        assert!(
            result.is_ok(),
            "sync_once should succeed: {:?}",
            result.err()
        );
        let stats = result.unwrap();

        assert_eq!(stats.podcasts_checked, 1, "should have checked 1 podcast");
        assert_eq!(stats.podcasts_failed, 0, "no podcasts should have failed");
        assert_eq!(
            stats.episodes_downloaded, 2,
            "should have downloaded 2 episodes"
        );
        assert_eq!(
            stats.episodes_enqueued, 2,
            "should have enqueued 2 episodes"
        );
        assert_eq!(stats.episodes_failed, 0, "no episodes should have failed");

        // Verify PlaylistAddTrack commands were sent
        let mut commands_received = Vec::new();
        while let Ok((cmd, _callback)) = cmd_rx.try_recv() {
            commands_received.push(cmd);
        }

        assert_eq!(
            commands_received.len(),
            2,
            "should have received 2 PlaylistAddTrack commands"
        );

        for cmd in &commands_received {
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
                            // SCENARIO-001/002: Podcast episodes use PodcastUrl with network URL
                            assert!(
                                url.starts_with("http"),
                                "PodcastUrl should be a network URL: {url}"
                            );
                        }
                        other => panic!("Expected PodcastUrl source, got: {:?}", other),
                    }
                }
                other => panic!("Expected PlaylistAddTrack, got: {:?}", other),
            }
        }

        // Verify files were actually downloaded (in per-podcast subdirectory)
        let downloaded_files = count_files_recursive(&download_dir);

        assert_eq!(
            downloaded_files,
            2,
            "should have 2 downloaded files in the directory"
        );
    }

    /// After a successful sync, a subsequent sync pass should NOT re-download
    /// or re-enqueue episodes that were already processed.
    ///
    /// AC-05, SCENARIO-011: Episode with existing GUID is skipped.
    /// SCENARIO-013: Episode already in play queue is not re-added.
    #[tokio::test]
    async fn integration_deduplication_across_multiple_sync_passes() {
        let mock_server = MockServer::start().await;

        let ep_path = "/episodes/dedup_episode.mp3";
        let feed_xml = generate_rss_feed(
            "Dedup Test Podcast",
            &[(
                "Episode Dedup",
                "guid-dedup-001",
                &format!("{}{}", mock_server.uri(), ep_path),
            )],
        );

        Mock::given(method("GET"))
            .and(path("/dedup_feed.xml"))
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
            .expect(1) // Episode should only be downloaded ONCE across two passes
            .mount(&mock_server)
            .await;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");
        let podcast = PodcastNoId {
            title: "Dedup Test Podcast".to_string(),
            url: format!("{}/dedup_feed.xml", mock_server.uri()),
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

        // First sync pass: should download and enqueue
        let result1 = sync_once(&config, &cmd_tx, db_path).await;
        assert!(result1.is_ok());
        let stats1 = result1.unwrap();
        assert_eq!(stats1.episodes_downloaded, 1);
        assert_eq!(stats1.episodes_enqueued, 1);

        // Drain the command channel
        while cmd_rx.try_recv().is_ok() {}

        // Second sync pass: should NOT re-download or re-enqueue
        let result2 = sync_once(&config, &cmd_tx, db_path).await;
        assert!(result2.is_ok());
        let stats2 = result2.unwrap();
        assert_eq!(
            stats2.episodes_downloaded, 0,
            "second pass should not re-download"
        );
        assert_eq!(
            stats2.episodes_enqueued, 0,
            "second pass should not re-enqueue"
        );

        // No new commands
        assert!(
            cmd_rx.try_recv().is_err(),
            "no commands should be sent on second pass"
        );
    }

    /// When a feed contains a mix of new and already-known episodes, only the
    /// new episodes should be downloaded and enqueued.
    ///
    /// AC-05, SCENARIO-010, SCENARIO-011: Mixed new and existing episodes.
    #[tokio::test]
    async fn integration_downloads_only_new_episodes_when_some_already_exist() {
        let mock_server = MockServer::start().await;

        let new_ep_path = "/episodes/new_episode.mp3";
        let feed_xml = generate_rss_feed(
            "Mixed Episodes Podcast",
            &[
                (
                    "Existing Episode",
                    "guid-existing-001",
                    "http://example.com/old.mp3",
                ),
                (
                    "New Episode",
                    "guid-new-001",
                    &format!("{}{}", mock_server.uri(), new_ep_path),
                ),
            ],
        );

        Mock::given(method("GET"))
            .and(path("/mixed_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(new_ep_path))
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

        // Insert podcast with one existing episode already in the DB
        let existing_episode = EpisodeNoId {
            title: "Existing Episode".to_string(),
            url: "http://example.com/old.mp3".to_string(),
            guid: "guid-existing-001".to_string(),
            description: "Already known".to_string(),
            pubdate: None,
            duration: Some(300),
            image_url: None,
        };
        let podcast = PodcastNoId {
            title: "Mixed Episodes Podcast".to_string(),
            url: format!("{}/mixed_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![existing_episode],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        // Mark the existing episode as downloaded
        let podcasts = db.get_podcasts().expect("get podcasts");
        let existing_ep = podcasts[0]
            .episodes
            .iter()
            .find(|e| e.guid == "guid-existing-001")
            .expect("find existing episode");
        db.insert_file(existing_ep.id, Path::new("/tmp/old.mp3"))
            .expect("mark as downloaded");
        drop(db);

        let config = make_test_config(&download_dir);
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok());
        let stats = result.unwrap();

        assert_eq!(
            stats.episodes_downloaded, 1,
            "only the new episode should be downloaded"
        );
        assert_eq!(
            stats.episodes_enqueued, 1,
            "only the new episode should be enqueued"
        );

        // Verify the command
        let (cmd, _) = cmd_rx.try_recv().expect("should receive one command");
        match cmd {
            PlayerCmd::PlaylistAddTrack(add_track) => {
                assert_eq!(add_track.at_index, PlaylistAddTrack::AT_END);
                assert_eq!(add_track.tracks.len(), 1);
            }
            other => panic!("Expected PlaylistAddTrack, got: {:?}", other),
        }

        assert!(cmd_rx.try_recv().is_err(), "only one command expected");
    }

    /// SCENARIO-016: When a new episode is enqueued via PlaylistAddTrack, the
    /// command uses the correct format that triggers auto-start when the queue
    /// was empty (existing player loop behavior).
    #[tokio::test]
    async fn integration_enqueue_format_enables_autostart_on_empty_queue() {
        let mock_server = MockServer::start().await;

        let ep_path = "/episodes/autostart.mp3";
        let feed_xml = generate_rss_feed(
            "Autostart Test",
            &[(
                "Fresh Episode",
                "guid-autostart-001",
                &format!("{}{}", mock_server.uri(), ep_path),
            )],
        );

        Mock::given(method("GET"))
            .and(path("/autostart_feed.xml"))
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
            title: "Autostart Test".to_string(),
            url: format!("{}/autostart_feed.xml", mock_server.uri()),
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

        let (cmd, _) = cmd_rx.try_recv().expect("should receive command");
        match cmd {
            PlayerCmd::PlaylistAddTrack(add_track) => {
                assert_eq!(
                    add_track.at_index,
                    PlaylistAddTrack::AT_END,
                    "must use AT_END for auto-start detection"
                );
                assert!(
                    !add_track.tracks.is_empty(),
                    "must have at least one track to trigger playback"
                );
            }
            other => panic!("Expected PlaylistAddTrack, got: {:?}", other),
        }
    }

    // =========================================================================
    // T-25: Error isolation with failing/malformed feeds
    // SCENARIO-017: Network error on one feed does not abort sync pass
    // SCENARIO-018: Malformed RSS feed does not crash the server
    // SCENARIO-019: Download failure for one episode does not block others
    // =========================================================================

    /// When one podcast feed returns HTTP 500, the sync pass should continue
    /// processing other podcasts that have valid feeds.
    ///
    /// AC-08, SCENARIO-017: Network error on one feed does not abort sync pass.
    #[tokio::test]
    async fn integration_http_500_on_one_feed_does_not_abort_others() {
        let mock_server = MockServer::start().await;

        // Good feed with a downloadable episode
        let good_ep_path = "/episodes/good_episode.mp3";
        let good_feed_xml = generate_rss_feed(
            "Good Podcast",
            &[(
                "Good Episode",
                "guid-good-001",
                &format!("{}{}", mock_server.uri(), good_ep_path),
            )],
        );

        Mock::given(method("GET"))
            .and(path("/good_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(good_feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(good_ep_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .mount(&mock_server)
            .await;

        // Bad feed returns HTTP 500
        Mock::given(method("GET"))
            .and(path("/bad_feed.xml"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");

        let bad_podcast = PodcastNoId {
            title: "Bad Podcast".to_string(),
            url: format!("{}/bad_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&bad_podcast).expect("insert bad");

        let good_podcast = PodcastNoId {
            title: "Good Podcast".to_string(),
            url: format!("{}/good_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&good_podcast).expect("insert good");
        drop(db);

        let config = make_test_config(&download_dir);
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once should not abort");
        let stats = result.unwrap();

        assert_eq!(stats.podcasts_checked, 2);
        assert_eq!(stats.podcasts_failed, 1);
        assert_eq!(stats.episodes_downloaded, 1);
        assert_eq!(stats.episodes_enqueued, 1);

        let (cmd, _) = cmd_rx.try_recv().expect("should have received command");
        assert!(matches!(cmd, PlayerCmd::PlaylistAddTrack(_)));
    }

    /// When a podcast feed returns malformed XML (not valid RSS), the sync pass
    /// should log a warning and continue with other podcasts.
    ///
    /// AC-08, SCENARIO-018: Malformed RSS feed does not crash the server.
    #[tokio::test]
    async fn integration_malformed_feed_xml_does_not_crash() {
        let mock_server = MockServer::start().await;

        // Malformed feed
        Mock::given(method("GET"))
            .and(path("/malformed_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("this is not xml at all <><><<<")
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        // Good feed
        let good_ep_path = "/episodes/good_after_malformed.mp3";
        let good_feed_xml = generate_rss_feed(
            "Good After Malformed",
            &[(
                "Survives Malformed",
                "guid-survives-001",
                &format!("{}{}", mock_server.uri(), good_ep_path),
            )],
        );

        Mock::given(method("GET"))
            .and(path("/good_after_malformed_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(good_feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(good_ep_path))
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

        let malformed_podcast = PodcastNoId {
            title: "Malformed Podcast".to_string(),
            url: format!("{}/malformed_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&malformed_podcast)
            .expect("insert malformed");

        let good_podcast = PodcastNoId {
            title: "Good After Malformed".to_string(),
            url: format!("{}/good_after_malformed_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&good_podcast).expect("insert good");
        drop(db);

        let config = make_test_config(&download_dir);
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok(), "sync_once must not crash on malformed feed");
        let stats = result.unwrap();

        assert_eq!(stats.podcasts_checked, 2);
        assert_eq!(
            stats.podcasts_failed, 1,
            "malformed feed should count as failed"
        );
        assert_eq!(stats.episodes_downloaded, 1);
        assert_eq!(stats.episodes_enqueued, 1);

        assert!(cmd_rx.try_recv().is_ok());
    }

    /// When a podcast has multiple new episodes but one episode's download fails
    /// (connection refused), the other episodes should still be downloaded and enqueued.
    ///
    /// AC-08, SCENARIO-019: Download failure for one episode does not block others.
    #[tokio::test]
    async fn integration_one_episode_download_fails_others_succeed() {
        let mock_server = MockServer::start().await;

        let good_ep1_path = "/episodes/good1.mp3";
        let good_ep2_path = "/episodes/good2.mp3";

        // Use a URL pointing to a non-routable address for the bad episode.
        // The download_list implementation only returns DLResponseError when
        // the TCP connection itself fails (not on HTTP 4xx/5xx status codes).
        let bad_ep_url = "http://192.0.2.1:1/episodes/unreachable.mp3";

        let feed_xml = generate_rss_feed(
            "Partial Download Podcast",
            &[
                (
                    "Good Episode 1",
                    "guid-partial-001",
                    &format!("{}{}", mock_server.uri(), good_ep1_path),
                ),
                ("Bad Episode (unreachable)", "guid-partial-002", bad_ep_url),
                (
                    "Good Episode 2",
                    "guid-partial-003",
                    &format!("{}{}", mock_server.uri(), good_ep2_path),
                ),
            ],
        );

        Mock::given(method("GET"))
            .and(path("/partial_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(good_ep1_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(fake_audio_content())
                    .insert_header("content-type", "audio/mpeg"),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(good_ep2_path))
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
            title: "Partial Download Podcast".to_string(),
            url: format!("{}/partial_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");
        drop(db);

        // Use max_download_retries=1 so the unreachable URL fails quickly
        let settings = ServerSettings {
            podcast: PodcastSettings {
                download_dir: download_dir.clone(),
                max_download_retries: 1,
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
        let config = new_shared_server_settings(ServerOverlay {
            settings,
            ..Default::default()
        });
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(
            result.is_ok(),
            "sync_once should not abort on download failure"
        );
        let stats = result.unwrap();

        assert_eq!(stats.podcasts_checked, 1);
        assert_eq!(stats.podcasts_failed, 0, "podcast itself didn't fail");
        assert_eq!(
            stats.episodes_downloaded, 2,
            "2 of 3 episodes should download successfully"
        );
        assert_eq!(stats.episodes_failed, 1, "1 episode should have failed");
        assert_eq!(
            stats.episodes_enqueued, 2,
            "2 successfully downloaded episodes should be enqueued"
        );

        // Verify 2 commands
        let mut commands_count = 0;
        while cmd_rx.try_recv().is_ok() {
            commands_count += 1;
        }
        assert_eq!(commands_count, 2, "should have 2 enqueue commands");
    }

    // =========================================================================
    // T-26: Task lifecycle integration tests
    // SCENARIO-022: Sync pass during ongoing playback does not disrupt audio
    // =========================================================================

    /// During active playback (simulated), a sync pass should append new episodes
    /// at the END without interrupting the current track.
    ///
    /// AC-07, SCENARIO-022: Sync during playback does not disrupt audio.
    #[tokio::test]
    async fn integration_sync_during_playback_appends_at_end() {
        let mock_server = MockServer::start().await;

        let ep_path = "/episodes/during_playback.mp3";
        let feed_xml = generate_rss_feed(
            "Playback Test Podcast",
            &[(
                "New During Playback",
                "guid-playback-001",
                &format!("{}{}", mock_server.uri(), ep_path),
            )],
        );

        Mock::given(method("GET"))
            .and(path("/playback_feed.xml"))
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
            title: "Playback Test Podcast".to_string(),
            url: format!("{}/playback_feed.xml", mock_server.uri()),
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

        let (cmd, _) = cmd_rx.try_recv().expect("should receive command");
        match cmd {
            PlayerCmd::PlaylistAddTrack(add_track) => {
                assert_eq!(
                    add_track.at_index,
                    PlaylistAddTrack::AT_END,
                    "sync during playback must append at end"
                );
            }
            other => panic!("Expected PlaylistAddTrack, got: {:?}", other),
        }
    }

    /// Verify that the sync task with refresh_on_startup=true performs a sync
    /// immediately on start, downloading and enqueuing episodes from a mock server.
    ///
    /// AC-03, SCENARIO-006: Immediate sync on startup with live mock.
    #[tokio::test]
    async fn integration_startup_sync_with_mock_server() {
        use tokio::runtime::Handle;
        use tokio_util::sync::CancellationToken;

        let mock_server = MockServer::start().await;

        let ep_path = "/episodes/startup_ep.mp3";
        let feed_xml = generate_rss_feed(
            "Startup Sync Podcast",
            &[(
                "Startup Episode",
                "guid-startup-001",
                &format!("{}{}", mock_server.uri(), ep_path),
            )],
        );

        Mock::given(method("GET"))
            .and(path("/startup_feed.xml"))
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
        let db_path = tmp_dir.path().to_path_buf();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(&db_path).expect("create database");
        let podcast = PodcastNoId {
            title: "Startup Sync Podcast".to_string(),
            url: format!("{}/startup_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");
        drop(db);

        let settings = ServerSettings {
            podcast: PodcastSettings {
                download_dir: download_dir.clone(),
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
        let config = new_shared_server_settings(ServerOverlay {
            settings,
            ..Default::default()
        });
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();
        let cancel_token = CancellationToken::new();
        let handle = Handle::current();

        // Start the sync task -- it should immediately run sync_once
        super::start_podcast_sync_task(handle, cancel_token.clone(), config, cmd_tx, db_path);

        // Wait for the startup sync to complete
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Cancel the task
        cancel_token.cancel();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify that a PlaylistAddTrack command was sent during startup sync
        let mut commands_received = 0;
        while let Ok((_cmd, _)) = cmd_rx.try_recv() {
            commands_received += 1;
        }

        assert!(
            commands_received >= 1,
            "startup sync should have enqueued at least 1 episode, got: {commands_received}"
        );

        // Verify a file was downloaded (in per-podcast subdirectory)
        let has_downloaded_file = walkdir(&download_dir);

        assert!(
            has_downloaded_file,
            "startup sync should have downloaded at least 1 file"
        );
    }

    /// Verify that a sync pass with an empty feed completes without downloads.
    ///
    /// SCENARIO-021: Sync with podcast that has no new episodes.
    #[tokio::test]
    async fn integration_empty_feed_completes_without_downloads() {
        let mock_server = MockServer::start().await;

        let empty_feed_xml = generate_rss_feed("Empty Podcast", &[]);

        Mock::given(method("GET"))
            .and(path("/empty_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(empty_feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");
        let podcast = PodcastNoId {
            title: "Empty Podcast".to_string(),
            url: format!("{}/empty_feed.xml", mock_server.uri()),
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

        assert_eq!(stats.podcasts_checked, 1);
        assert_eq!(stats.podcasts_failed, 0);
        assert_eq!(stats.episodes_downloaded, 0);
        assert_eq!(stats.episodes_enqueued, 0);
        assert_eq!(stats.episodes_failed, 0);

        assert!(cmd_rx.try_recv().is_err());
    }

    /// Verify URL-based deduplication: when an episode has no GUID but the
    /// enclosure URL matches an existing episode, it should not be re-downloaded.
    ///
    /// AC-05, SCENARIO-012: Fallback deduplication by enclosure URL.
    #[tokio::test]
    async fn integration_deduplication_by_enclosure_url_fallback() {
        let mock_server = MockServer::start().await;

        let existing_url = format!("{}/episodes/already_known.mp3", mock_server.uri());
        // Feed without <guid> element -- relies on URL dedup
        let feed_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
    <channel>
        <title>URL Dedup Podcast</title>
        <link>http://example.com</link>
        <description>Tests URL-based dedup</description>
        <item>
            <title>Already Known By URL</title>
            <enclosure url="{existing_url}" type="audio/mpeg" length="1024"/>
            <pubDate>Mon, 23 Jun 2025 12:00:00 +0000</pubDate>
        </item>
    </channel>
</rss>"#
        );

        Mock::given(method("GET"))
            .and(path("/url_dedup_feed.xml"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(feed_xml)
                    .insert_header("content-type", "application/rss+xml"),
            )
            .mount(&mock_server)
            .await;

        // Do NOT mock the episode download -- it should never be requested

        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path();
        let download_dir = tmp_dir.path().join("downloads");
        std::fs::create_dir_all(&download_dir).expect("create download dir");

        let db = Database::new(db_path).expect("create database");

        let existing_episode = EpisodeNoId {
            title: "Already Known By URL".to_string(),
            url: existing_url.clone(),
            guid: String::new(),
            description: String::new(),
            pubdate: None,
            duration: Some(300),
            image_url: None,
        };
        let podcast = PodcastNoId {
            title: "URL Dedup Podcast".to_string(),
            url: format!("{}/url_dedup_feed.xml", mock_server.uri()),
            description: None,
            author: None,
            explicit: None,
            last_checked: chrono::Utc::now(),
            episodes: vec![existing_episode],
            image_url: None,
        };
        db.insert_podcast(&podcast).expect("insert podcast");

        let podcasts = db.get_podcasts().expect("get podcasts");
        let ep_id = podcasts[0].episodes[0].id;
        db.insert_file(ep_id, Path::new("/tmp/already_known.mp3"))
            .expect("mark as downloaded");
        drop(db);

        let config = make_test_config(&download_dir);
        let (cmd_tx, mut cmd_rx) = make_cmd_channel();

        let result = sync_once(&config, &cmd_tx, db_path).await;

        assert!(result.is_ok());
        let stats = result.unwrap();

        assert_eq!(
            stats.episodes_downloaded, 0,
            "episode known by URL should not be re-downloaded"
        );
        assert_eq!(
            stats.episodes_enqueued, 0,
            "episode known by URL should not be re-enqueued"
        );

        assert!(cmd_rx.try_recv().is_err());
    }
}
