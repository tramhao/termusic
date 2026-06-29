//! Tests for `PlaylistAddTrack` API extension (Phase 2).
//!
//! These tests cover Phase 2 of Server-Side Podcast Synchronization:
//! - T-09: AT_END constant equals u64::MAX
//! - T-10: new_append_single constructs correct PlaylistAddTrack
//! - T-11: new_append_vec constructs correct PlaylistAddTrack
//! - T-12: Unit tests verifying AT_END value and constructor behavior
//!
//! AC References:
//! - AC-07: Downloaded episodes appended to end of play queue via PlaylistAddTrack
//!
//! BDD Scenario References:
//! - SCENARIO-015: Downloaded episode appended to end of play queue

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::player::playlist_helpers::{PlaylistAddTrack, PlaylistTrackSource};

    // =========================================================================
    // T-09: AT_END constant value
    // =========================================================================

    /// The AT_END constant must equal u64::MAX so that any playlist length
    /// comparison (at_index >= playlist.len()) triggers end-append behavior.
    #[test]
    fn at_end_constant_equals_u64_max() {
        assert_eq!(PlaylistAddTrack::AT_END, u64::MAX);
    }

    /// AT_END should be distinct from any realistic playlist index (0-based).
    /// This ensures no accidental collision with real indices.
    #[test]
    fn at_end_is_not_zero() {
        assert_ne!(PlaylistAddTrack::AT_END, 0);
    }

    /// AT_END should be larger than any reasonable playlist size.
    #[test]
    fn at_end_is_larger_than_any_reasonable_index() {
        // Even a hypothetical playlist with a billion tracks should never reach AT_END
        assert!(PlaylistAddTrack::AT_END > 1_000_000_000);
    }

    // =========================================================================
    // T-10: new_append_single constructor
    // =========================================================================

    /// new_append_single should set at_index to AT_END (u64::MAX).
    #[test]
    fn new_append_single_sets_at_index_to_at_end() {
        let track = PlaylistTrackSource::Path("/tmp/episode1.mp3".to_string());
        let request = PlaylistAddTrack::new_append_single(track);

        assert_eq!(request.at_index, PlaylistAddTrack::AT_END);
    }

    /// new_append_single should wrap the track in a Vec with exactly one element.
    #[test]
    fn new_append_single_contains_exactly_one_track() {
        let track = PlaylistTrackSource::Path("/tmp/episode1.mp3".to_string());
        let request = PlaylistAddTrack::new_append_single(track);

        assert_eq!(request.tracks.len(), 1);
    }

    /// new_append_single should preserve the exact track source provided.
    #[test]
    fn new_append_single_preserves_path_track() {
        let path = "/home/user/podcasts/episode_42.mp3".to_string();
        let track = PlaylistTrackSource::Path(path.clone());
        let request = PlaylistAddTrack::new_append_single(track);

        assert_eq!(request.tracks[0], PlaylistTrackSource::Path(path));
    }

    /// new_append_single works with URL track sources (anti-hardcoding: varied input).
    #[test]
    fn new_append_single_preserves_url_track() {
        let url = "https://example.com/feed/ep99.mp3".to_string();
        let track = PlaylistTrackSource::Url(url.clone());
        let request = PlaylistAddTrack::new_append_single(track);

        assert_eq!(request.at_index, PlaylistAddTrack::AT_END);
        assert_eq!(request.tracks.len(), 1);
        assert_eq!(request.tracks[0], PlaylistTrackSource::Url(url));
    }

    /// new_append_single works with PodcastUrl track sources (anti-hardcoding: varied input).
    #[test]
    fn new_append_single_preserves_podcast_url_track() {
        let podcast_url = "https://feeds.example.org/show/episode123.mp3".to_string();
        let track = PlaylistTrackSource::PodcastUrl(podcast_url.clone());
        let request = PlaylistAddTrack::new_append_single(track);

        assert_eq!(request.at_index, PlaylistAddTrack::AT_END);
        assert_eq!(request.tracks.len(), 1);
        assert_eq!(
            request.tracks[0],
            PlaylistTrackSource::PodcastUrl(podcast_url)
        );
    }

    // =========================================================================
    // T-11: new_append_vec constructor
    // =========================================================================

    /// new_append_vec should set at_index to AT_END (u64::MAX).
    #[test]
    fn new_append_vec_sets_at_index_to_at_end() {
        let tracks = vec![
            PlaylistTrackSource::Path("/tmp/ep1.mp3".to_string()),
            PlaylistTrackSource::Path("/tmp/ep2.mp3".to_string()),
        ];
        let request = PlaylistAddTrack::new_append_vec(tracks);

        assert_eq!(request.at_index, PlaylistAddTrack::AT_END);
    }

    /// new_append_vec should contain all tracks provided in order.
    #[test]
    fn new_append_vec_preserves_all_tracks_in_order() {
        let tracks = vec![
            PlaylistTrackSource::Path("/tmp/first.mp3".to_string()),
            PlaylistTrackSource::Url("https://example.com/second.mp3".to_string()),
            PlaylistTrackSource::PodcastUrl("https://feeds.org/third.mp3".to_string()),
        ];
        let expected = tracks.clone();
        let request = PlaylistAddTrack::new_append_vec(tracks);

        assert_eq!(request.tracks, expected);
    }

    /// new_append_vec with an empty Vec should produce a valid struct with no tracks.
    #[test]
    fn new_append_vec_with_empty_vec() {
        let tracks: Vec<PlaylistTrackSource> = vec![];
        let request = PlaylistAddTrack::new_append_vec(tracks);

        assert_eq!(request.at_index, PlaylistAddTrack::AT_END);
        assert_eq!(request.tracks.len(), 0);
    }

    /// new_append_vec with a single element should behave identically to new_append_single
    /// in terms of the resulting struct fields (anti-hardcoding: verify consistency).
    #[test]
    fn new_append_vec_single_element_matches_new_append_single() {
        let track = PlaylistTrackSource::Path("/tmp/same_track.mp3".to_string());
        let from_single = PlaylistAddTrack::new_append_single(track.clone());
        let from_vec = PlaylistAddTrack::new_append_vec(vec![track]);

        assert_eq!(from_single.at_index, from_vec.at_index);
        assert_eq!(from_single.tracks, from_vec.tracks);
    }

    /// new_append_vec with many tracks (anti-hardcoding: stress test with varied count).
    #[test]
    fn new_append_vec_with_many_tracks() {
        let tracks: Vec<PlaylistTrackSource> = (0..50)
            .map(|i| PlaylistTrackSource::Path(format!("/podcasts/episode_{i}.mp3")))
            .collect();
        let expected_len = tracks.len();
        let request = PlaylistAddTrack::new_append_vec(tracks);

        assert_eq!(request.at_index, PlaylistAddTrack::AT_END);
        assert_eq!(request.tracks.len(), expected_len);
        // Verify first and last to ensure ordering
        assert_eq!(
            request.tracks[0],
            PlaylistTrackSource::Path("/podcasts/episode_0.mp3".to_string())
        );
        assert_eq!(
            request.tracks[49],
            PlaylistTrackSource::Path("/podcasts/episode_49.mp3".to_string())
        );
    }

    // =========================================================================
    // T-12: Regression - existing new_single and new_vec remain functional
    // =========================================================================

    /// Existing new_single method should still work correctly with an explicit index.
    #[test]
    fn existing_new_single_still_works() {
        let track = PlaylistTrackSource::Path("/tmp/track.mp3".to_string());
        let request = PlaylistAddTrack::new_single(5, track.clone());

        assert_eq!(request.at_index, 5);
        assert_eq!(request.tracks.len(), 1);
        assert_eq!(request.tracks[0], track);
    }

    /// Existing new_vec method should still work correctly with an explicit index.
    #[test]
    fn existing_new_vec_still_works() {
        let tracks = vec![
            PlaylistTrackSource::Path("/tmp/a.mp3".to_string()),
            PlaylistTrackSource::Path("/tmp/b.mp3".to_string()),
        ];
        let expected = tracks.clone();
        let request = PlaylistAddTrack::new_vec(3, tracks);

        assert_eq!(request.at_index, 3);
        assert_eq!(request.tracks, expected);
    }

    /// new_single with at_index=0 should place track at beginning (distinct from AT_END).
    #[test]
    fn new_single_at_index_zero_is_distinct_from_at_end() {
        let track = PlaylistTrackSource::Path("/tmp/beginning.mp3".to_string());
        let at_beginning = PlaylistAddTrack::new_single(0, track.clone());
        let at_end = PlaylistAddTrack::new_append_single(track);

        assert_ne!(at_beginning.at_index, at_end.at_index);
    }

    // =========================================================================
    // Struct properties: PartialEq, Clone, Debug
    // =========================================================================

    /// PlaylistAddTrack constructed via new_append_single should support equality comparison.
    #[test]
    fn new_append_single_supports_equality() {
        let track = PlaylistTrackSource::Path("/tmp/same.mp3".to_string());
        let a = PlaylistAddTrack::new_append_single(track.clone());
        let b = PlaylistAddTrack::new_append_single(track);

        assert_eq!(a, b);
    }

    /// PlaylistAddTrack with different tracks should not be equal.
    #[test]
    fn new_append_single_different_tracks_not_equal() {
        let a = PlaylistAddTrack::new_append_single(PlaylistTrackSource::Path(
            "/tmp/one.mp3".to_string(),
        ));
        let b = PlaylistAddTrack::new_append_single(PlaylistTrackSource::Path(
            "/tmp/two.mp3".to_string(),
        ));

        assert_ne!(a, b);
    }

    /// PlaylistAddTrack constructed via new_append_vec should be clonable.
    #[test]
    fn new_append_vec_is_clonable() {
        let tracks = vec![
            PlaylistTrackSource::Path("/tmp/x.mp3".to_string()),
            PlaylistTrackSource::Url("https://example.com/y.mp3".to_string()),
        ];
        let original = PlaylistAddTrack::new_append_vec(tracks);
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    /// PlaylistAddTrack should implement Debug for logging purposes.
    #[test]
    fn new_append_single_implements_debug() {
        let track = PlaylistTrackSource::Path("/tmp/debug_test.mp3".to_string());
        let request = PlaylistAddTrack::new_append_single(track);
        let debug_str = format!("{:?}", request);

        assert!(debug_str.contains("PlaylistAddTrack"));
        assert!(debug_str.contains("at_index"));
        assert!(debug_str.contains("tracks"));
    }
}
