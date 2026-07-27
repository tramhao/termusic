use std::cmp::Ordering;

use termusiclib::{
    new_database::{Database, track_ops},
    player::{SortCriterion, SortDirection},
    track::Track,
    utils::frecency_score,
};

/// A track paired with its sort key and title for deterministic ordering.
pub struct ScoredTrack {
    track: Track,
    /// Primary sort key (score, duration, etc.).
    key: f64,
}

impl From<ScoredTrack> for Track {
    fn from(value: ScoredTrack) -> Self {
        value.track
    }
}

/// Compute the sort key for a single track against the given criterion.
#[allow(clippy::cast_precision_loss)]
pub fn score_track(track: Track, criterion: SortCriterion, db: &Database, now: u64) -> ScoredTrack {
    let key = match criterion {
        SortCriterion::Alphanumeric => f64::NEG_INFINITY,
        SortCriterion::Duration => track.duration().map_or(0.0, |d| d.as_secs_f64()),
        SortCriterion::MostPlayed
        | SortCriterion::Recency
        | SortCriterion::FirstAdded
        | SortCriterion::Frecency => {
            let conn = db.get_connection();
            let tr = track
                .path()
                .and_then(|p| track_ops::get_track_from_path(&conn, p).ok());
            let (pc, lp, added) = tr.as_ref().map_or((0, None, None), |x| {
                (x.total_play_count, x.last_played_at, x.added_at)
            });

            match criterion {
                SortCriterion::MostPlayed => pc as f64,
                SortCriterion::Recency => lp.map_or(f64::MIN, |v| v as f64),
                SortCriterion::FirstAdded => added.map_or(f64::MIN, |v| v as f64),
                SortCriterion::Frecency => frecency_score(pc, lp, now),
                _ => unreachable!(),
            }
        }
    };
    ScoredTrack { track, key }
}

/// Apply the [`SortDirection`] to the given [`Ordering`].
///
/// Effectively this means it returns:
/// - `initial` as-is if [`SortDirection::Asc`]
/// - `initial` reversed if [`SortDirection::Desc`]
fn apply_direction(initial: Ordering, dir: SortDirection) -> Ordering {
    if dir == SortDirection::Desc {
        initial.reverse()
    } else {
        initial
    }
}

/// Sort a scored track list in-place according to `criterion` + `direction`.
pub fn sort_scored(scored: &mut [ScoredTrack], criterion: SortCriterion, direction: SortDirection) {
    if criterion == SortCriterion::Alphanumeric {
        scored.sort_by(|a, b| {
            apply_direction(
                alphanumeric_sort::compare_str(
                    a.track.title().unwrap_or_default(),
                    b.track.title().unwrap_or_default(),
                ),
                direction,
            )
        });
    } else {
        scored.sort_by(|a, b| {
            apply_direction(
                a.key.partial_cmp(&b.key).unwrap_or(Ordering::Equal),
                direction,
            )
            .then_with(|| {
                alphanumeric_sort::compare_str(
                    a.track.title().unwrap_or_default(),
                    b.track.title().unwrap_or_default(),
                )
            })
        });
    }
}
