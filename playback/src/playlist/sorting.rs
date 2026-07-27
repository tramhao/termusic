use std::cmp::Ordering;

use termusiclib::{
    new_database::{Database, track_ops},
    player::{SortCriterion, SortDirection},
    track::Track,
};

/// "Frecency" score — a hybrid of **fre**quency and re**cency**, popularized
/// by zoxide and Firefox. Tracks that are played often *and* recently score
/// highest.
///
/// `total_play_count` is the rank (starts at 1, incremented per access).
/// `last_played_at` is the last-access time (unix epoch seconds).
/// Buckets match zoxide's `dir.rs` exactly:
///   < 1 hour  → × 4.0
///   < 1 day   → × 2.0
///   < 1 week  → × 0.5
///   else      → × 0.25
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn frecency_score(total_play_count: u64, last_played_at: Option<u64>, now: u64) -> f64 {
    if total_play_count == 0 {
        return 0.0;
    }
    let rank = total_play_count as f64;
    let multiplier = match last_played_at {
        None => 0.25,
        Some(lp) => {
            let elapsed = now.saturating_sub(lp);
            if elapsed < FrecencyTimeBucket::Hour.as_secs() {
                FrecencyTimeBucket::Hour.multiplier()
            } else if elapsed < FrecencyTimeBucket::Day.as_secs() {
                FrecencyTimeBucket::Day.multiplier()
            } else if elapsed < FrecencyTimeBucket::Week.as_secs() {
                FrecencyTimeBucket::Week.multiplier()
            } else {
                FrecencyTimeBucket::Old.multiplier()
            }
        }
    };
    rank * multiplier
}

/// Time buckets for the frecency scoring algorithm.
///
/// Each bucket maps a recency window to a score multiplier, matching zoxide's
/// `dir.rs` exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
enum FrecencyTimeBucket {
    /// < 1 hour  → × 4.0
    Hour = 3_600,
    /// < 1 day   → × 2.0
    Day = 86_400,
    /// < 1 week  → × 0.5
    Week = 604_800,
    /// ≥ 1 week  → × 0.25
    Old = u64::MAX,
}

impl FrecencyTimeBucket {
    const fn as_secs(self) -> u64 {
        self as u64
    }

    const fn multiplier(self) -> f64 {
        match self {
            Self::Hour => 4.0,
            Self::Day => 2.0,
            Self::Week => 0.5,
            Self::Old => 0.25,
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::frecency_score;

    #[test]
    fn frecency_score_basic() {
        let now = 1_700_000_000;
        let eps = f64::EPSILON;

        // total_play_count=0 → score is 0 regardless of last_played_at
        assert!(frecency_score(0, Some(now), now).abs() < eps);
        assert!(frecency_score(0, None, now).abs() < eps);

        // 1 play, < 1 hour ago → rank 1 × 4.0 = 4.0
        assert!((frecency_score(1, Some(now - 1_800), now) - 4.0).abs() < eps);

        // 1 play, < 1 day ago → rank 1 × 2.0 = 2.0
        assert!((frecency_score(1, Some(now - 7_200), now) - 2.0).abs() < eps);

        // 1 play, < 1 week ago → rank 1 × 0.5 = 0.5
        assert!((frecency_score(1, Some(now - 100_000), now) - 0.5).abs() < eps);

        // 1 play, ≥ 1 week ago → rank 1 × 0.25 = 0.25
        assert!((frecency_score(1, Some(now - 700_000), now) - 0.25).abs() < eps);

        // 1 play, never played → rank 1 × 0.25 = 0.25
        assert!((frecency_score(1, None, now) - 0.25).abs() < eps);

        // 3 plays, < 1 week ago → rank 3 × 0.5 = 1.5
        assert!((frecency_score(3, Some(now - 100_000), now) - 1.5).abs() < eps);
    }
}
