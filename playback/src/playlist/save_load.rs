use std::io::{BufRead, BufWriter, Write};
use std::{fs::File, io::BufReader};

use anyhow::{Context, Result};
use termusiclib::podcast::db::Database as DBPod;
use termusiclib::track::MediaTypes;
use termusiclib::{track::Track, utils::get_app_config_path};

use crate::Playlist;
use crate::playlist::get_playlist_path;

/// Load the playlist from the file.
///
/// Path in `$config$/playlist.log`.
///
/// Returns `(Position, Tracks[])`.
///
/// # Errors
/// - When the playlist path is not write-able
/// - When podcasts cannot be loaded
pub fn load() -> Result<(usize, Vec<Track>)> {
    let path = get_playlist_path()?;

    let Ok(file) = File::open(&path) else {
        // new file, nothing to parse from it
        File::create(&path)?;

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
    let db_path = get_app_config_path()?;
    let db_podcast = DBPod::new(&db_path)?;
    let podcasts = db_podcast
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
pub fn save(playlist: &Playlist) -> Result<()> {
    let path = get_playlist_path()?;

    let file = File::create(&path)?;

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
