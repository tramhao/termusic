//! Tests for `SynchronizationSettings` configuration struct.
//!
//! These tests cover Phase 1 of Server-Side Podcast Synchronization:
//! - AC-01: Config section existence with correct defaults
//! - AC-10: Config serialization roundtrip
//! - SCENARIO-001: Default config when section absent
//! - SCENARIO-002: Explicit non-default values honored
//! - SCENARIO-003: Roundtrip preserves all fields
//! - SCENARIO-004: Invalid duration string rejected
//!
//! Labeling convention: These labels map to BDD scenarios and acceptance criteria
//! defined in the specification for traceability between tests and requirements.

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use crate::config::v2::server::synchronization::SynchronizationSettings;
    use crate::config::v2::server::ServerSettings;

    // =========================================================================
    // SCENARIO-001 / AC-01: Default synchronization config applied when section absent
    // =========================================================================

    /// When a TOML config has no `[synchronization]` section, the defaults should apply:
    /// - enable: true
    /// - interval: 1 hour (3600 seconds)
    /// - refresh_on_startup: true
    #[test]
    fn default_config_when_synchronization_section_absent() {
        // A minimal server config TOML without any [synchronization] section
        let toml_str = indoc! { r#"
            [com]
            port = 5101

            [player]
            volume = 30

            [podcast]
            max_download_retries = 3
        "# };

        let settings: ServerSettings =
            toml::from_str(toml_str).expect("should parse without error");

        assert_eq!(settings.synchronization.enable, true);
        assert_eq!(settings.synchronization.interval, Duration::from_secs(3600));
        assert_eq!(settings.synchronization.refresh_on_startup, true);
    }

    /// The Default impl for SynchronizationSettings should produce the documented defaults.
    #[test]
    fn default_impl_produces_correct_values() {
        let defaults = SynchronizationSettings::default();

        assert_eq!(defaults.enable, true);
        assert_eq!(defaults.interval, Duration::from_secs(3600));
        assert_eq!(defaults.refresh_on_startup, true);
    }

    // =========================================================================
    // SCENARIO-002 / AC-01: Explicit synchronization configuration honored
    // =========================================================================

    /// When the config TOML specifies all synchronization fields explicitly with
    /// non-default values, the deserialized struct should reflect those values.
    #[test]
    fn explicit_non_default_values_deserialized_correctly() {
        let toml_str = indoc! { r#"
            [synchronization]
            enable = false
            interval = "30m"
            refresh_on_startup = false
        "# };

        let settings: SynchronizationSettings =
            toml::from_str(toml_str).expect("should parse explicit values");

        assert_eq!(settings.enable, false);
        assert_eq!(settings.interval, Duration::from_secs(30 * 60));
        assert_eq!(settings.refresh_on_startup, false);
    }

    /// Test with a different non-default interval value to prevent hardcoding.
    #[test]
    fn explicit_interval_2h30m_deserialized_correctly() {
        let toml_str = indoc! { r#"
            [synchronization]
            enable = true
            interval = "2h30m"
            refresh_on_startup = true
        "# };

        let settings: SynchronizationSettings =
            toml::from_str(toml_str).expect("should parse 2h30m interval");

        assert_eq!(settings.interval, Duration::from_secs(2 * 3600 + 30 * 60));
    }

    /// Test with a seconds-only interval to verify varied duration formats.
    #[test]
    fn explicit_interval_seconds_only() {
        let toml_str = indoc! { r#"
            [synchronization]
            interval = "45s"
        "# };

        let settings: SynchronizationSettings =
            toml::from_str(toml_str).expect("should parse 45s interval");

        assert_eq!(settings.interval, Duration::from_secs(45));
    }

    // =========================================================================
    // SCENARIO-003 / AC-01, AC-10: Configuration roundtrip preserves all fields
    // =========================================================================

    /// Serializing and then deserializing SynchronizationSettings with non-default
    /// values should produce identical output.
    #[test]
    fn serialization_roundtrip_preserves_all_fields() {
        let original = SynchronizationSettings {
            enable: false,
            interval: Duration::from_secs(1800), // 30 minutes
            refresh_on_startup: false,
            max_new_episodes: 5,
            auto_enqueue: true,
        };

        let serialized = toml::to_string(&original).expect("should serialize");
        let deserialized: SynchronizationSettings =
            toml::from_str(&serialized).expect("should deserialize roundtrip");

        assert_eq!(original, deserialized);
    }

    /// Roundtrip with default values should also be stable.
    #[test]
    fn serialization_roundtrip_default_values() {
        let original = SynchronizationSettings::default();

        let serialized = toml::to_string(&original).expect("should serialize defaults");
        let deserialized: SynchronizationSettings =
            toml::from_str(&serialized).expect("should deserialize roundtrip defaults");

        assert_eq!(original, deserialized);
    }

    /// Roundtrip with a complex interval value (multi-unit duration).
    #[test]
    fn serialization_roundtrip_complex_interval() {
        let original = SynchronizationSettings {
            enable: true,
            interval: Duration::from_secs(5 * 3600 + 15 * 60 + 30), // 5h15m30s
            refresh_on_startup: true,
            max_new_episodes: 5,
            auto_enqueue: true,
        };

        let serialized = toml::to_string(&original).expect("should serialize complex interval");
        let deserialized: SynchronizationSettings =
            toml::from_str(&serialized).expect("should deserialize complex interval roundtrip");

        assert_eq!(original, deserialized);
    }

    // =========================================================================
    // SCENARIO-004 / AC-01: Unparseable interval duration string rejected
    // =========================================================================

    /// An unparseable duration string should produce a deserialization error
    /// with a message indicating the parsing failure.
    #[test]
    fn invalid_duration_string_produces_error() {
        let toml_str = indoc! { r#"
            [synchronization]
            enable = true
            interval = "not_a_duration"
            refresh_on_startup = true
        "# };

        let result = toml::from_str::<SynchronizationSettings>(toml_str);
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("expected") || err_msg.contains("duration"),
            "error should mention parsing failure, got: {err_msg}"
        );
    }

    /// An empty string should also be rejected with a descriptive error.
    #[test]
    fn empty_duration_string_produces_error() {
        let toml_str = indoc! { r#"
            [synchronization]
            interval = ""
        "# };

        let result = toml::from_str::<SynchronizationSettings>(toml_str);
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("expected") || err_msg.contains("duration"),
            "error should mention parsing failure, got: {err_msg}"
        );
    }

    /// A numeric value without unit should be rejected with a descriptive error.
    #[test]
    fn numeric_without_unit_produces_error() {
        let toml_str = indoc! { r#"
            [synchronization]
            interval = "3600"
        "# };

        let result = toml::from_str::<SynchronizationSettings>(toml_str);
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("expected") || err_msg.contains("duration"),
            "error should mention parsing failure, got: {err_msg}"
        );
    }

    // =========================================================================
    // AC-01: Backward compatibility - ServerSettings parses with synchronization
    // =========================================================================

    /// Full ServerSettings should parse correctly even when synchronization section
    /// is present with all fields specified.
    #[test]
    fn server_settings_with_explicit_synchronization_section() {
        let toml_str = indoc! { r#"
            [com]
            port = 5101

            [player]
            volume = 30

            [podcast]
            max_download_retries = 3

            [synchronization]
            enable = true
            interval = "2h"
            refresh_on_startup = false
        "# };

        let settings: ServerSettings =
            toml::from_str(toml_str).expect("should parse full server settings");

        assert_eq!(settings.synchronization.enable, true);
        assert_eq!(settings.synchronization.interval, Duration::from_secs(7200));
        assert_eq!(settings.synchronization.refresh_on_startup, false);
    }

    /// ServerSettings with a completely empty config should use all defaults including
    /// synchronization defaults.
    #[test]
    fn server_settings_empty_config_uses_all_defaults() {
        let toml_str = "";

        let settings: ServerSettings =
            toml::from_str(toml_str).expect("should parse empty config with defaults");

        assert_eq!(settings.synchronization.enable, true);
        assert_eq!(settings.synchronization.interval, Duration::from_secs(3600));
        assert_eq!(settings.synchronization.refresh_on_startup, true);
    }

    // =========================================================================
    // AC-01: Partial synchronization section (some fields missing, use defaults)
    // =========================================================================

    /// When only `enable` is specified, other fields should use defaults.
    #[test]
    fn partial_synchronization_section_uses_defaults_for_missing_fields() {
        let toml_str = indoc! { r#"
            [synchronization]
            enable = false
        "# };

        let settings: SynchronizationSettings =
            toml::from_str(toml_str).expect("should parse partial config");

        assert_eq!(settings.enable, false);
        // interval should default to 1h
        assert_eq!(settings.interval, Duration::from_secs(3600));
        // refresh_on_startup should default to true
        assert_eq!(settings.refresh_on_startup, true);
    }

    /// When only `interval` is specified, other fields should use defaults.
    #[test]
    fn partial_synchronization_only_interval_specified() {
        let toml_str = indoc! { r#"
            [synchronization]
            interval = "15m"
        "# };

        let settings: SynchronizationSettings =
            toml::from_str(toml_str).expect("should parse interval-only config");

        assert_eq!(settings.enable, true);
        assert_eq!(settings.interval, Duration::from_secs(15 * 60));
        assert_eq!(settings.refresh_on_startup, true);
    }

    // =========================================================================
    // Struct-level properties
    // =========================================================================

    /// SynchronizationSettings should implement PartialEq for comparison in tests.
    #[test]
    fn synchronization_settings_equality() {
        let a = SynchronizationSettings {
            enable: true,
            interval: Duration::from_secs(3600),
            refresh_on_startup: true,
            max_new_episodes: 5,
            auto_enqueue: true,
        };
        let b = SynchronizationSettings::default();
        assert_eq!(a, b);
    }

    /// SynchronizationSettings with different values should not be equal.
    #[test]
    fn synchronization_settings_inequality() {
        let a = SynchronizationSettings::default();
        let b = SynchronizationSettings {
            enable: false,
            interval: Duration::from_secs(1800),
            refresh_on_startup: false,
            max_new_episodes: 5,
            auto_enqueue: true,
        };
        assert_ne!(a, b);
    }

    /// SynchronizationSettings should implement Clone.
    #[test]
    fn synchronization_settings_clone() {
        let original = SynchronizationSettings {
            enable: false,
            interval: Duration::from_secs(900),
            refresh_on_startup: false,
            max_new_episodes: 5,
            auto_enqueue: true,
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    /// SynchronizationSettings should implement Debug.
    #[test]
    fn synchronization_settings_debug() {
        let settings = SynchronizationSettings::default();
        let debug_str = format!("{:?}", settings);
        assert!(debug_str.contains("SynchronizationSettings"));
        assert!(debug_str.contains("enable"));
        assert!(debug_str.contains("interval"));
        assert!(debug_str.contains("refresh_on_startup"));
    }
}
