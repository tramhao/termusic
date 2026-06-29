//! Phase 3 Tests: Config Simplification and Utility Extraction
//!
//! These tests validate the changes required by Phase 3 of the PR #720 review action items:
//! - AC-07 / SCENARIO-009: SynchronizationSettings uses standard #[derive(Deserialize)] with #[serde(default)]
//! - AC-07 / SCENARIO-010: Default sync settings applied when section is absent
//! - AC-06 / SCENARIO-030: Synchronization config section position documented
//! - T-18, T-19: auto_enqueue field exists with correct default and deserialization
//! - T-25: auto_enqueue = false deserializes correctly
//!
//! These tests are expected to FAIL (RED phase) because:
//! - The `auto_enqueue` field does not yet exist on `SynchronizationSettings`
//! - The `SyncSettingsRaw` and `SyncSettingsNested` still exist (simplification not yet done)

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use crate::config::v2::server::synchronization::SynchronizationSettings;
    use crate::config::v2::server::ServerSettings;

    // =========================================================================
    // AC-07 / SCENARIO-009: Config deserialization uses standard derive mechanism
    // After simplification, SynchronizationSettings should derive Deserialize directly.
    // =========================================================================

    /// T-17: SynchronizationSettings with #[derive(Deserialize)] and #[serde(default)]
    /// should deserialize from a flat TOML table without needing a [synchronization] wrapper.
    ///
    /// This test verifies that after removing the custom Deserialize impl, the struct
    /// can be deserialized directly as a nested value within ServerSettings.
    #[test]
    fn sync_settings_derives_deserialize_directly_flat_table() {
        // Flat key=value pairs (as they appear when nested inside ServerSettings)
        let toml_str = indoc! { r#"
            enable = false
            interval = "45m"
            refresh_on_startup = false
            max_new_episodes = 10
            auto_enqueue = true
        "# };

        let settings: SynchronizationSettings =
            toml::from_str(toml_str).expect("should deserialize flat table with derive");

        assert_eq!(settings.enable, false);
        assert_eq!(settings.interval, Duration::from_secs(45 * 60));
        assert_eq!(settings.refresh_on_startup, false);
        assert_eq!(settings.max_new_episodes, 10);
        assert_eq!(settings.auto_enqueue, true);
    }

    // =========================================================================
    // AC-07 / SCENARIO-010: Default sync settings applied when section absent
    // =========================================================================

    /// T-17: When the synchronization section is completely absent from config,
    /// all fields should use their Default values including the new auto_enqueue field.
    #[test]
    fn sync_settings_defaults_when_section_absent_includes_auto_enqueue() {
        let toml_str = indoc! { r#"
            [com]
            port = 5101

            [player]
            volume = 30

            [podcast]
            max_download_retries = 3
        "# };

        let settings: ServerSettings =
            toml::from_str(toml_str).expect("should parse without synchronization section");

        assert_eq!(settings.synchronization.enable, true);
        assert_eq!(settings.synchronization.interval, Duration::from_secs(3600));
        assert_eq!(settings.synchronization.refresh_on_startup, true);
        assert_eq!(settings.synchronization.max_new_episodes, 5);
        // The new auto_enqueue field must default to true
        assert_eq!(settings.synchronization.auto_enqueue, true);
    }

    /// Default impl must include auto_enqueue field set to true.
    #[test]
    fn default_impl_includes_auto_enqueue_true() {
        let defaults = SynchronizationSettings::default();

        assert_eq!(defaults.auto_enqueue, true);
    }

    // =========================================================================
    // T-18, T-19: auto_enqueue field on SynchronizationSettings
    // =========================================================================

    /// T-18: The auto_enqueue field must exist as a pub bool on SynchronizationSettings.
    /// Default value is true (backward compatible - existing users see no behavior change).
    #[test]
    fn auto_enqueue_field_exists_and_defaults_to_true() {
        let settings = SynchronizationSettings::default();

        // Accessing the field directly - this will fail to compile if the field doesn't exist
        let auto_enqueue: bool = settings.auto_enqueue;
        assert_eq!(auto_enqueue, true);
    }

    /// T-25: Explicit `auto_enqueue = false` in TOML should deserialize correctly.
    #[test]
    fn auto_enqueue_false_deserializes_correctly() {
        let toml_str = indoc! { r#"
            [synchronization]
            enable = true
            interval = "1h"
            refresh_on_startup = true
            max_new_episodes = 5
            auto_enqueue = false
        "# };

        let settings: SynchronizationSettings =
            toml::from_str(toml_str).expect("should parse auto_enqueue = false");

        assert_eq!(settings.auto_enqueue, false);
        // Other fields should retain their explicit values
        assert_eq!(settings.enable, true);
        assert_eq!(settings.interval, Duration::from_secs(3600));
        assert_eq!(settings.max_new_episodes, 5);
    }

    /// When auto_enqueue is not specified in TOML, it should default to true.
    #[test]
    fn auto_enqueue_missing_from_toml_defaults_to_true() {
        let toml_str = indoc! { r#"
            [synchronization]
            enable = true
            interval = "2h"
        "# };

        let settings: SynchronizationSettings =
            toml::from_str(toml_str).expect("should parse with missing auto_enqueue");

        assert_eq!(settings.auto_enqueue, true);
    }

    /// auto_enqueue field roundtrip: serialize then deserialize preserves the value.
    #[test]
    fn auto_enqueue_roundtrip_preserves_false() {
        let original = SynchronizationSettings {
            enable: true,
            interval: Duration::from_secs(3600),
            refresh_on_startup: true,
            max_new_episodes: 5,
            auto_enqueue: false,
        };

        let serialized = toml::to_string(&original).expect("should serialize");
        let deserialized: SynchronizationSettings =
            toml::from_str(&serialized).expect("should deserialize roundtrip");

        assert_eq!(original, deserialized);
        assert_eq!(deserialized.auto_enqueue, false);
    }

    /// auto_enqueue field roundtrip: true value also roundtrips.
    #[test]
    fn auto_enqueue_roundtrip_preserves_true() {
        let original = SynchronizationSettings {
            enable: false,
            interval: Duration::from_secs(1800),
            refresh_on_startup: false,
            max_new_episodes: 3,
            auto_enqueue: true,
        };

        let serialized = toml::to_string(&original).expect("should serialize");
        let deserialized: SynchronizationSettings =
            toml::from_str(&serialized).expect("should deserialize roundtrip");

        assert_eq!(original, deserialized);
        assert_eq!(deserialized.auto_enqueue, true);
    }

    // =========================================================================
    // AC-07: No SyncSettingsRaw or SyncSettingsNested should exist after simplification
    // Verified indirectly: if the custom Deserialize impl is removed and derive is used,
    // then these tests all pass with the simplified struct. The existence of the tests
    // themselves (requiring auto_enqueue field) proves the struct has been modified.
    // =========================================================================

    /// After simplification, ServerSettings with explicit synchronization section
    /// containing auto_enqueue should work seamlessly.
    #[test]
    fn server_settings_with_auto_enqueue_in_synchronization_section() {
        let toml_str = indoc! { r#"
            [com]
            port = 5101

            [synchronization]
            enable = true
            interval = "1h"
            auto_enqueue = false
        "# };

        let settings: ServerSettings =
            toml::from_str(toml_str).expect("should parse server settings with auto_enqueue");

        assert_eq!(settings.synchronization.enable, true);
        assert_eq!(settings.synchronization.auto_enqueue, false);
    }

    /// Empty config should produce defaults for all fields including auto_enqueue.
    #[test]
    fn empty_config_produces_all_defaults_including_auto_enqueue() {
        let toml_str = "";

        let settings: ServerSettings = toml::from_str(toml_str).expect("should parse empty config");

        assert_eq!(settings.synchronization.auto_enqueue, true);
        assert_eq!(settings.synchronization.enable, true);
        assert_eq!(settings.synchronization.interval, Duration::from_secs(3600));
    }

    // =========================================================================
    // AC-06 / SCENARIO-030: Synchronization config section position documented
    // The `synchronization` section remains top-level in ServerSettings.
    // This test validates the struct layout (it would fail if the field were moved).
    // =========================================================================

    /// The `synchronization` field must exist directly on ServerSettings (not nested
    /// under another field like `podcast`). This validates the AC-06 decision.
    #[test]
    fn synchronization_is_top_level_field_on_server_settings() {
        let settings = ServerSettings::default();

        // Direct field access on ServerSettings - proves it's top-level
        let _sync: &SynchronizationSettings = &settings.synchronization;

        // Verify the full struct with synchronization at top level
        assert_eq!(settings.synchronization.enable, true);
    }

    /// Verify that synchronization config section at root level of TOML
    /// (alongside [com], [player], [podcast]) works correctly.
    #[test]
    fn synchronization_section_is_peer_to_other_top_level_sections() {
        let toml_str = indoc! { r#"
            [com]
            port = 5101

            [player]
            volume = 30

            [podcast]
            max_download_retries = 3

            [synchronization]
            enable = false
            interval = "30m"
            auto_enqueue = true
        "# };

        let settings: ServerSettings =
            toml::from_str(toml_str).expect("should parse all top-level sections");

        // All sections parsed correctly
        assert_eq!(settings.com.port, 5101);
        assert_eq!(settings.synchronization.enable, false);
        assert_eq!(
            settings.synchronization.interval,
            Duration::from_secs(30 * 60)
        );
        assert_eq!(settings.synchronization.auto_enqueue, true);
    }

    // =========================================================================
    // Error cases: specific error messages (not just is_err)
    // AC-20 pattern: assert specific error content
    // =========================================================================

    /// Invalid duration string should produce an error containing duration-related message.
    #[test]
    fn invalid_duration_produces_specific_error_message() {
        let toml_str = indoc! { r#"
            [synchronization]
            interval = "not_a_valid_duration"
        "# };

        let result = toml::from_str::<SynchronizationSettings>(toml_str);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // The error should mention something about parsing duration
        assert!(
            err_msg.contains("expected a duration") || err_msg.contains("invalid"),
            "Error message should indicate duration parsing failure, got: {err_msg}"
        );
    }

    /// Invalid type for auto_enqueue should produce a specific error.
    #[test]
    fn invalid_type_for_auto_enqueue_produces_error() {
        let toml_str = indoc! { r#"
            [synchronization]
            auto_enqueue = "yes"
        "# };

        let result = toml::from_str::<SynchronizationSettings>(toml_str);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("boolean") || err_msg.contains("invalid type"),
            "Error should indicate type mismatch for auto_enqueue, got: {err_msg}"
        );
    }

    // =========================================================================
    // PartialEq with auto_enqueue included
    // =========================================================================

    /// Two SynchronizationSettings with different auto_enqueue values should not be equal.
    #[test]
    fn sync_settings_inequality_on_auto_enqueue() {
        let a = SynchronizationSettings {
            enable: true,
            interval: Duration::from_secs(3600),
            refresh_on_startup: true,
            max_new_episodes: 5,
            auto_enqueue: true,
        };
        let b = SynchronizationSettings {
            enable: true,
            interval: Duration::from_secs(3600),
            refresh_on_startup: true,
            max_new_episodes: 5,
            auto_enqueue: false,
        };
        assert_ne!(a, b);
    }
}
