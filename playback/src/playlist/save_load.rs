use std::io::{BufRead, BufWriter, Write};
use std::path::Path;
use std::{fs::File, io::BufReader};

use anyhow::{Context, Result};
use termusiclib::podcast::db::Database as DBPod;
use termusiclib::track::MediaTypes;
use termusiclib::track::Track;

use crate::Playlist;

/// Load the playlist from the file.
///
/// Path in `$config$/playlist.log`.
///
/// Returns `(Position, Tracks[])`.
///
/// # Errors
/// - When the playlist path is not write-able
/// - When podcasts cannot be loaded
pub fn load(from_path: &Path, db_pod: &DBPod) -> Result<(usize, Vec<Track>)> {
    let Ok(file) = File::open(from_path) else {
        // new file, nothing to parse from it
        File::create(from_path)?;

        return Ok((0, Vec::new()));
    };

    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut current_track_index = 0;
    if let Some(line) = lines.next() {
        let index_line = line?;
        if let Ok(index) = index_line.trim().parse() {
            current_track_index = index;
        }
    } else {
        // empty file, nothing to parse from it
        return Ok((0, Vec::new()));
    }

    let mut playlist_items = Vec::new();
    let podcasts = db_pod
        .get_podcasts()
        .context("failed to get podcasts from db.")?;
    for line in lines {
        let line = line?;

        let trimmed_line = line.trim();

        // skip empty lines without trying to process them
        // skip lines that are comments (m3u-like)
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }

        if line.starts_with("http") {
            let mut is_podcast = false;
            'outer: for pod in &podcasts {
                for ep in &pod.episodes {
                    if ep.url == line.as_str() {
                        is_podcast = true;
                        let track = Track::from_podcast_episode(ep);
                        playlist_items.push(track);
                        break 'outer;
                    }
                }
            }
            if !is_podcast {
                let track = Track::new_radio(&line);
                playlist_items.push(track);
            }
            continue;
        }
        if let Ok(track) = Track::read_track_from_path(&line) {
            playlist_items.push(track);
        }
    }

    // protect against the listed index in the playlist file not matching the elements in the playlist
    // for example lets say it has "100", but there are only 2 elements in the playlist
    let current_track_index = current_track_index.min(playlist_items.len().saturating_sub(1));

    Ok((current_track_index, playlist_items))
}

/// Save the given playlist to the playlist location
///
/// Path in `$config$/playlist.log`
///
/// # Errors
///
/// Errors could happen when writing files
pub fn save(playlist: &Playlist, to_path: &Path) -> Result<()> {
    let file = File::create(to_path)?;

    // If the playlist is empty, truncate the file, but dont write anything else (like a index number)
    if playlist.is_empty() {
        return Ok(());
    }

    let mut writer = BufWriter::new(file);
    writer.write_all(playlist.current_track_index.to_string().as_bytes())?;
    writer.write_all(b"\n")?;
    for track in &playlist.tracks {
        let id = match track.inner() {
            MediaTypes::Track(track_data) => track_data.path().to_string_lossy(),
            MediaTypes::Radio(radio_track_data) => radio_track_data.url().into(),
            MediaTypes::Podcast(podcast_track_data) => podcast_track_data.url().into(),
        };
        writeln!(writer, "{id}")?;
    }

    writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use assert_fs::{
        TempDir,
        assert::PathAssert,
        fixture::{FileWriteStr, PathChild},
    };
    use indoc::indoc;
    use predicates::boolean::PredicateBooleanExt;
    use pretty_assertions::assert_eq;
    use termusiclib::{
        config::{ServerOverlay, new_shared_server_settings},
        player::playlist_helpers::{PlaylistPlaySpecific, PlaylistTrackSource},
        podcast::{
            PodcastNoId,
            db::Database as DBPod,
            episode::{Episode, EpisodeNoId},
        },
        track::{Track, TrackMetadata},
    };
    use tokio::sync::broadcast;

    use crate::Playlist;

    use super::{load, save};

    fn new_dir() -> TempDir {
        TempDir::with_prefix("termusic-playlist-saveload-").unwrap()
    }

    fn new_playlist() -> Playlist {
        let config = new_shared_server_settings(ServerOverlay::default());
        let (stream_tx, _) = broadcast::channel(1);
        Playlist::new(&config, stream_tx)
    }

    fn new_db_pod(config_path: &Path) -> DBPod {
        let db_pod = DBPod::new(config_path).unwrap();

        let episode = EpisodeNoId {
            title: "ep1".to_string(),
            url: "https://some.podcast/ep1".to_string(),
            ..EpisodeNoId::dummy()
        };

        db_pod
            .insert_podcast(&PodcastNoId {
                title: "test".to_string(),
                episodes: vec![episode],
                ..PodcastNoId::dummy()
            })
            .unwrap();

        db_pod
    }

    #[test]
    fn save_should_not_write_content_if_empty() {
        let tmp = new_dir();
        let playlist_file = tmp.child("playlist.log");
        let playlist = new_playlist();

        playlist_file.assert(predicates::path::exists().not());

        save(&playlist, playlist_file.path()).unwrap();

        playlist_file.assert(predicates::path::exists());
        playlist_file.assert(predicates::str::is_empty());
    }

    #[test]
    fn save_should_overwrite_even_if_empty() {
        let tmp = new_dir();
        let playlist_file = tmp.child("playlist.log");
        let playlist = new_playlist();

        playlist_file.write_str("testdata").unwrap();

        playlist_file.assert(predicates::path::exists());
        playlist_file.assert(predicates::str::is_empty().not());

        save(&playlist, playlist_file.path()).unwrap();

        playlist_file.assert(predicates::path::exists());
        playlist_file.assert(predicates::str::is_empty());
    }

    #[test]
    fn save_should_work() {
        let tmp = new_dir();
        let playlist_file = tmp.child("playlist.log");
        let mut playlist = new_playlist();

        playlist.add_track_test(Track::from_track_metadata(
            "/some/file/somewhere.mp3",
            TrackMetadata::default(),
        ));
        playlist.add_track_test(Track::new_radio("https://some.radio/"));
        playlist.add_track_test(Track::from_podcast_episode(&Episode {
            id: 1,
            pod_id: 1,
            title: "test".to_string(),
            url: "https://some.podcast/ep1".to_string(),
            ..Default::default()
        }));

        playlist_file.assert(predicates::path::exists().not());

        save(&playlist, playlist_file.path()).unwrap();

        playlist_file.assert(predicates::path::exists());
        playlist_file.assert(indoc! {"
            0
            /some/file/somewhere.mp3
            https://some.radio/
            https://some.podcast/ep1
        "});
    }

    #[test]
    fn save_should_set_current_index() {
        let tmp = new_dir();
        let playlist_file = tmp.child("playlist.log");
        let mut playlist = new_playlist();

        playlist.add_track_test(Track::from_track_metadata(
            "/some/file/somewhere.mp3",
            TrackMetadata::default(),
        ));
        playlist.add_track_test(Track::new_radio("https://some.radio/"));
        playlist.add_track_test(Track::from_podcast_episode(&Episode {
            id: 1,
            pod_id: 1,
            title: "test".to_string(),
            url: "https://some.podcast/ep1".to_string(),
            ..Default::default()
        }));
        playlist
            .set_play_specific(&PlaylistPlaySpecific {
                track_index: 2,
                id: PlaylistTrackSource::PodcastUrl("https://some.podcast/ep1".to_string()),
            })
            .unwrap();

        playlist_file.assert(predicates::path::exists().not());

        save(&playlist, playlist_file.path()).unwrap();

        playlist_file.assert(predicates::path::exists());
        playlist_file.assert(indoc! {"
            2
            /some/file/somewhere.mp3
            https://some.radio/
            https://some.podcast/ep1
        "});
    }

    #[test]
    fn load_should_create_file_if_not_existing() {
        let tmp = new_dir();
        let playlist_file = tmp.child("playlist.log");
        let db_pod_file = tmp.child("data.db");

        playlist_file.assert(predicates::path::exists().not());
        db_pod_file.assert(predicates::path::exists().not());

        let db_pod = new_db_pod(tmp.path());

        db_pod_file.assert(predicates::path::exists());

        let res = load(playlist_file.path(), &db_pod).unwrap();

        playlist_file.assert(predicates::path::exists());
        playlist_file.assert(predicates::str::is_empty());

        assert_eq!(res, (0, Vec::new()));
    }

    #[test]
    fn load_should_work() {
        let tmp = new_dir();
        let playlist_file = tmp.child("playlist.log");
        let db_pod_file = tmp.child("data.db");

        playlist_file.assert(predicates::path::exists().not());
        db_pod_file.assert(predicates::path::exists().not());

        let db_pod = new_db_pod(tmp.path());

        db_pod_file.assert(predicates::path::exists());

        // Not testing normal "Track"s here as those would need to exist, which is a pain to create
        playlist_file
            .write_str(indoc! {"
            0
            https://some.radio/
            https://some.podcast/ep1
        "})
            .unwrap();

        let res = load(playlist_file.path(), &db_pod).unwrap();

        playlist_file.assert(predicates::path::exists());
        playlist_file.assert(predicates::str::is_empty().not());

        assert_eq!(
            res,
            (
                0,
                vec![
                    Track::new_radio("https://some.radio/"),
                    Track::from_podcast_episode(&Episode {
                        id: 0,
                        pod_id: 0,
                        title: "test".to_string(),
                        url: "https://some.podcast/ep1".to_string(),
                        ..Default::default()
                    })
                ]
            )
        );
    }

    #[test]
    fn load_should_get_correct_starting_index() {
        let tmp = new_dir();
        let playlist_file = tmp.child("playlist.log");
        let db_pod_file = tmp.child("data.db");

        playlist_file.assert(predicates::path::exists().not());
        db_pod_file.assert(predicates::path::exists().not());

        let db_pod = new_db_pod(tmp.path());

        db_pod_file.assert(predicates::path::exists());

        // Not testing normal "Track"s here as those would need to exist, which is a pain to create
        playlist_file
            .write_str(indoc! {"
            1
            https://some.radio/
            https://some.podcast/ep1
        "})
            .unwrap();

        let res = load(playlist_file.path(), &db_pod).unwrap();

        playlist_file.assert(predicates::path::exists());
        playlist_file.assert(predicates::str::is_empty().not());

        assert_eq!(
            res,
            (
                1,
                vec![
                    Track::new_radio("https://some.radio/"),
                    Track::from_podcast_episode(&Episode {
                        id: 0,
                        pod_id: 0,
                        title: "test".to_string(),
                        url: "https://some.podcast/ep1".to_string(),
                        ..Default::default()
                    })
                ]
            )
        );
    }
}
