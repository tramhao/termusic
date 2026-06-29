//! Phase 5 RED Tests: Style Polish, Test Quality, and API Cleanup
//!
//! These tests validate the Phase 5 requirements for PR #720 review action items:
//! - AC-12 / SCENARIO-016: SyncPassStats implements Default with all counters at zero
//! - AC-11 / SCENARIO-015: Module documentation uses `//!` doc-comment syntax
//! - AC-21 / SCENARIO-025: PlaylistAddTrack::new_append_single/vec delegate to positional constructors
//! - AC-19 / SCENARIO-023: Test TOML uses indoc! macro for readability
//! - AC-20 / SCENARIO-024: Error-path tests assert specific error types/messages
//!
//! These tests are expected to FAIL (RED) because:
//! - SyncPassStats does NOT yet derive Default (T-40)
//! - The module doc comments still use `//` instead of `//!` (T-39)

#[cfg(test)]
mod tests {
    use termusiclib::player::playlist_helpers::{PlaylistAddTrack, PlaylistTrackSource};

    use crate::podcast_sync::SyncPassStats;

    // =========================================================================
    // AC-12 / SCENARIO-016: SyncPassStats uses Default trait
    //
    // The SyncPassStats struct must derive Default so that initialization
    // uses SyncPassStats::default() instead of manual zero-valued struct literals.
    // This test will FAIL TO COMPILE if Default is not derived on SyncPassStats.
    // =========================================================================

    /// SCENARIO-016: SyncPassStats::default() produces all counters at zero.
    /// This test calls Default::default() on SyncPassStats which requires
    /// the trait to be implemented (either derived or manual impl).
    ///
    /// RED STATE: Currently SyncPassStats does NOT derive Default.
    #[test]
    fn sync_pass_stats_default_produces_all_zeros() {
        let stats = SyncPassStats::default();

        assert_eq!(stats.podcasts_checked, 0);
        assert_eq!(stats.podcasts_failed, 0);
        assert_eq!(stats.episodes_downloaded, 0);
        assert_eq!(stats.episodes_enqueued, 0);
        assert_eq!(stats.episodes_failed, 0);
    }

    /// SCENARIO-016: Default::default() for SyncPassStats is equivalent to
    /// manual zero initialization. Verifies that the derived Default produces
    /// the same struct as explicitly setting all fields to zero.
    #[test]
    fn sync_pass_stats_default_equals_manual_zeros() {
        let from_default = SyncPassStats::default();
        let manual = SyncPassStats {
            podcasts_checked: 0,
            podcasts_failed: 0,
            episodes_downloaded: 0,
            episodes_enqueued: 0,
            episodes_failed: 0,
        };

        assert_eq!(from_default, manual);
    }

    /// SCENARIO-016: Anti-hardcoding - verify that non-zero values are NOT
    /// produced by Default. After incrementing a field, it should differ from default.
    #[test]
    fn sync_pass_stats_default_differs_from_nonzero() {
        let mut stats = SyncPassStats::default();
        stats.podcasts_checked = 1;

        let fresh_default = SyncPassStats::default();
        assert_ne!(stats, fresh_default);
    }

    // =========================================================================
    // AC-21 / SCENARIO-025: PlaylistAddTrack append methods delegate to positional
    //
    // new_append_single must produce identical output to new_single(AT_END, track).
    // new_append_vec must produce identical output to new_vec(AT_END, tracks).
    //
    // This verifies behavioral equivalence. The delegation itself is an
    // implementation detail that ensures no duplicated logic.
    // =========================================================================

    /// SCENARIO-025: new_append_single produces same result as new_single(AT_END, track).
    /// Both methods must yield identical PlaylistAddTrack structs.
    #[test]
    fn new_append_single_equivalent_to_new_single_at_end() {
        let track = PlaylistTrackSource::PodcastUrl("https://example.com/ep1.mp3".to_string());

        let from_append = PlaylistAddTrack::new_append_single(track.clone());
        let from_positional = PlaylistAddTrack::new_single(PlaylistAddTrack::AT_END, track);

        assert_eq!(from_append, from_positional);
    }

    /// SCENARIO-025: new_append_vec produces same result as new_vec(AT_END, tracks).
    #[test]
    fn new_append_vec_equivalent_to_new_vec_at_end() {
        let tracks = vec![
            PlaylistTrackSource::PodcastUrl("https://example.com/ep1.mp3".to_string()),
            PlaylistTrackSource::Path("/music/song.flac".to_string()),
            PlaylistTrackSource::Url("https://stream.example.com/live".to_string()),
        ];

        let from_append = PlaylistAddTrack::new_append_vec(tracks.clone());
        let from_positional = PlaylistAddTrack::new_vec(PlaylistAddTrack::AT_END, tracks);

        assert_eq!(from_append, from_positional);
    }

    /// SCENARIO-025: Anti-hardcoding - verify with different track source types
    /// to ensure the delegation is not specific to one variant.
    #[test]
    fn new_append_single_works_with_path_variant() {
        let track = PlaylistTrackSource::Path("/home/user/music/track.mp3".to_string());

        let from_append = PlaylistAddTrack::new_append_single(track.clone());
        let from_positional = PlaylistAddTrack::new_single(PlaylistAddTrack::AT_END, track);

        assert_eq!(from_append, from_positional);
    }

    /// SCENARIO-025: Anti-hardcoding - verify with Url variant.
    #[test]
    fn new_append_single_works_with_url_variant() {
        let track = PlaylistTrackSource::Url("https://radio.example.com/stream".to_string());

        let from_append = PlaylistAddTrack::new_append_single(track.clone());
        let from_positional = PlaylistAddTrack::new_single(PlaylistAddTrack::AT_END, track);

        assert_eq!(from_append, from_positional);
    }

    /// SCENARIO-025: new_append_single sets at_index to AT_END (u64::MAX).
    #[test]
    fn new_append_single_uses_at_end_sentinel() {
        let track = PlaylistTrackSource::PodcastUrl("https://example.com/ep.mp3".to_string());
        let result = PlaylistAddTrack::new_append_single(track);

        assert_eq!(result.at_index, PlaylistAddTrack::AT_END);
        assert_eq!(result.at_index, u64::MAX);
    }

    /// SCENARIO-025: new_append_vec sets at_index to AT_END (u64::MAX).
    #[test]
    fn new_append_vec_uses_at_end_sentinel() {
        let tracks = vec![
            PlaylistTrackSource::PodcastUrl("https://example.com/ep1.mp3".to_string()),
            PlaylistTrackSource::PodcastUrl("https://example.com/ep2.mp3".to_string()),
        ];
        let result = PlaylistAddTrack::new_append_vec(tracks);

        assert_eq!(result.at_index, PlaylistAddTrack::AT_END);
        assert_eq!(result.tracks.len(), 2);
    }

    /// SCENARIO-025: new_append_vec with empty vec should still work.
    #[test]
    fn new_append_vec_empty_tracks() {
        let tracks: Vec<PlaylistTrackSource> = vec![];

        let from_append = PlaylistAddTrack::new_append_vec(tracks.clone());
        let from_positional = PlaylistAddTrack::new_vec(PlaylistAddTrack::AT_END, tracks);

        assert_eq!(from_append, from_positional);
        assert_eq!(from_append.tracks.len(), 0);
    }

    // =========================================================================
    // AC-11 / SCENARIO-015: Module documentation uses //! doc-comment syntax
    //
    // This is a compile-time/structural requirement. We verify it by checking
    // that the module-level doc attribute is accessible (which only works with //!).
    // The actual verification is done by code inspection or a source scan test below.
    // =========================================================================

    /// SCENARIO-015: Verify podcast_sync module has doc comments.
    /// This test reads the source file and checks that the first lines use //! syntax.
    /// It will FAIL if the module still uses // instead of //! for top-of-file comments.
    #[test]
    fn podcast_sync_module_uses_doc_comment_syntax() {
        let source = include_str!("podcast_sync.rs");
        let first_line = source.lines().next().unwrap_or("");

        // The first line MUST start with //! (module doc comment)
        // Currently it starts with // (regular comment), so this test will FAIL.
        assert!(
            first_line.starts_with("//!"),
            "First line of podcast_sync.rs must use //! module doc-comment syntax, \
             but found: {:?}",
            first_line
        );
    }

    /// SCENARIO-015: Verify second line also uses //! syntax.
    #[test]
    fn podcast_sync_module_second_line_uses_doc_comment_syntax() {
        let source = include_str!("podcast_sync.rs");
        let second_line = source.lines().nth(1).unwrap_or("");

        assert!(
            second_line.starts_with("//!"),
            "Second line of podcast_sync.rs must use //! module doc-comment syntax, \
             but found: {:?}",
            second_line
        );
    }
}
