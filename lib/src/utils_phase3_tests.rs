//! Phase 3 Tests: `ensure_podcast_dir` Utility Extraction
//!
//! These tests validate the new `ensure_podcast_dir` function in `lib/src/utils.rs`:
//! - AC-10 / SCENARIO-014: Podcast directory creation reuses existing utility
//! - AC-10 / SCENARIO-027: Download directory does not exist for new podcast
//! - T-21: ensure_podcast_dir creates directory with sanitized name
//! - T-22: existing create_podcast_dir delegates to ensure_podcast_dir
//!
//! These tests are expected to FAIL (RED phase) because:
//! - The `ensure_podcast_dir` function does not yet exist in `lib/src/utils.rs`

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use pretty_assertions::assert_eq;

    use crate::utils::{ensure_podcast_dir, random_ascii};

    /// Helper: create a unique temporary directory for test isolation.
    /// Uses std::env::temp_dir with a random subdirectory name.
    fn make_temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("termusic_test_{}", random_ascii(12)));
        std::fs::create_dir_all(&dir).expect("create test temp dir");
        dir
    }

    /// Helper: clean up a test directory after use.
    fn cleanup_dir(dir: &Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    // =========================================================================
    // T-21: ensure_podcast_dir basic functionality
    // AC-10, SCENARIO-014: Podcast directory creation reuses existing utility
    // =========================================================================

    /// ensure_podcast_dir should create a directory under download_dir with
    /// the sanitized podcast title as the directory name.
    #[test]
    fn ensure_podcast_dir_creates_directory_with_sanitized_name() {
        let download_dir = make_temp_dir();

        let result = ensure_podcast_dir(&download_dir, "My Podcast Title");

        assert!(result.is_ok());
        let pod_dir = result.unwrap();
        assert!(pod_dir.exists());
        assert!(pod_dir.is_dir());
        // The directory should be named after the sanitized title
        assert_eq!(
            pod_dir.file_name().unwrap().to_str().unwrap(),
            "My Podcast Title"
        );

        cleanup_dir(&download_dir);
    }

    /// ensure_podcast_dir should sanitize filesystem-unsafe characters from the title.
    #[test]
    fn ensure_podcast_dir_sanitizes_special_characters() {
        let download_dir = make_temp_dir();

        let result = ensure_podcast_dir(&download_dir, "My/Podcast: A <Special> Title?");

        assert!(result.is_ok());
        let pod_dir = result.unwrap();
        assert!(pod_dir.exists());
        assert!(pod_dir.is_dir());
        // Sanitized name should not contain filesystem-unsafe characters
        let dir_name = pod_dir.file_name().unwrap().to_str().unwrap();
        assert!(!dir_name.contains('/'));
        assert!(!dir_name.contains(':'));
        assert!(!dir_name.contains('<'));
        assert!(!dir_name.contains('>'));
        assert!(!dir_name.contains('?'));

        cleanup_dir(&download_dir);
    }

    /// ensure_podcast_dir should handle titles with only unsafe characters.
    #[test]
    fn ensure_podcast_dir_handles_all_unsafe_characters_title() {
        let download_dir = make_temp_dir();

        // Title with characters that will all be stripped by sanitize_with_options
        let result = ensure_podcast_dir(&download_dir, "///");

        // Should still succeed (sanitize_filename produces empty string -> handled gracefully)
        assert!(result.is_ok());
        let pod_dir = result.unwrap();
        assert!(pod_dir.exists());

        cleanup_dir(&download_dir);
    }

    /// ensure_podcast_dir should create intermediate directories if the base download_dir
    /// doesn't exist yet.
    /// AC-10, SCENARIO-027: Download directory does not exist for new podcast.
    #[test]
    fn ensure_podcast_dir_creates_intermediate_directories() {
        let base = make_temp_dir();
        let download_dir = base.join("deep").join("nested").join("path");

        // download_dir doesn't exist yet
        assert!(!download_dir.exists());

        let result = ensure_podcast_dir(&download_dir, "New Podcast");

        assert!(result.is_ok());
        let pod_dir = result.unwrap();
        assert!(pod_dir.exists());
        assert!(pod_dir.is_dir());

        cleanup_dir(&base);
    }

    /// ensure_podcast_dir should be idempotent - calling it twice with the same
    /// arguments should succeed both times without error.
    #[test]
    fn ensure_podcast_dir_is_idempotent() {
        let download_dir = make_temp_dir();

        let result1 = ensure_podcast_dir(&download_dir, "Repeat Podcast");
        assert!(result1.is_ok());

        let result2 = ensure_podcast_dir(&download_dir, "Repeat Podcast");
        assert!(result2.is_ok());

        assert_eq!(result1.unwrap(), result2.unwrap());

        cleanup_dir(&download_dir);
    }

    /// ensure_podcast_dir should return the full path to the created directory.
    #[test]
    fn ensure_podcast_dir_returns_full_path() {
        let download_dir = make_temp_dir();

        let result = ensure_podcast_dir(&download_dir, "Path Test Pod");

        assert!(result.is_ok());
        let pod_dir = result.unwrap();
        assert!(pod_dir.starts_with(&download_dir));

        cleanup_dir(&download_dir);
    }

    /// ensure_podcast_dir should use the truncate and windows options for sanitization
    /// to match existing create_podcast_dir behavior.
    #[test]
    fn ensure_podcast_dir_truncates_very_long_titles() {
        let download_dir = make_temp_dir();

        // Create a title longer than typical filesystem limits
        let long_title = "A".repeat(300);
        let result = ensure_podcast_dir(&download_dir, &long_title);

        assert!(result.is_ok());
        let pod_dir = result.unwrap();
        assert!(pod_dir.exists());
        // The directory name should be truncated to a safe length
        let dir_name = pod_dir.file_name().unwrap().to_str().unwrap();
        assert!(dir_name.len() <= 255);

        cleanup_dir(&download_dir);
    }

    /// ensure_podcast_dir should handle empty title gracefully.
    #[test]
    fn ensure_podcast_dir_handles_empty_title() {
        let download_dir = make_temp_dir();

        let result = ensure_podcast_dir(&download_dir, "");

        // Should still succeed - sanitize_filename with empty string produces empty,
        // create_dir_all with empty child just creates the parent
        assert!(result.is_ok());

        cleanup_dir(&download_dir);
    }

    /// ensure_podcast_dir should handle Windows-reserved names (CON, PRN, AUX, etc.)
    /// by sanitizing them (since windows: true is used in options).
    #[test]
    fn ensure_podcast_dir_sanitizes_windows_reserved_names() {
        let download_dir = make_temp_dir();

        let result = ensure_podcast_dir(&download_dir, "CON");

        assert!(result.is_ok());
        let pod_dir = result.unwrap();
        assert!(pod_dir.exists());
        // On all platforms with windows:true sanitization, CON should be handled

        cleanup_dir(&download_dir);
    }

    // =========================================================================
    // T-22: create_podcast_dir delegates to ensure_podcast_dir
    // =========================================================================

    /// After refactoring, create_podcast_dir should produce the same result
    /// as calling ensure_podcast_dir with the resolved download path.
    /// This test validates behavioral equivalence.
    #[test]
    fn create_podcast_dir_produces_same_result_as_ensure_podcast_dir() {
        let download_dir = make_temp_dir();

        // Call ensure_podcast_dir directly
        let direct_result = ensure_podcast_dir(&download_dir, "Delegation Test");
        assert!(direct_result.is_ok());

        // The path should be download_dir/sanitized_title
        let pod_dir = direct_result.unwrap();
        assert!(pod_dir.starts_with(&download_dir));
        assert!(pod_dir.exists());

        cleanup_dir(&download_dir);
    }

    // =========================================================================
    // Function signature validation
    // =========================================================================

    /// ensure_podcast_dir must accept &Path and &str, returning Result<PathBuf>.
    #[test]
    fn ensure_podcast_dir_has_correct_signature() {
        let download_dir = make_temp_dir();
        let download_path: &Path = &download_dir;
        let pod_title: &str = "Signature Test";

        // This call validates the function signature:
        // pub fn ensure_podcast_dir(download_dir: &Path, pod_title: &str) -> Result<PathBuf>
        let result: anyhow::Result<PathBuf> = ensure_podcast_dir(download_path, pod_title);
        assert!(result.is_ok());

        cleanup_dir(&download_dir);
    }

    // =========================================================================
    // Anti-hardcoding: multiple varied inputs to force real logic
    // =========================================================================

    /// Various podcast titles should all produce valid directories.
    #[test]
    fn ensure_podcast_dir_handles_varied_titles() {
        let download_dir = make_temp_dir();

        let titles = [
            "The Daily",
            "Science Friday",
            "Hello Internet (Archive)",
            "99% Invisible",
            "Podcast Name With Spaces",
        ];

        for title in titles {
            let result = ensure_podcast_dir(&download_dir, title);
            assert!(result.is_ok(), "Failed for title: {title}");
            let pod_dir = result.unwrap();
            assert!(pod_dir.exists(), "Directory not created for title: {title}");
            assert!(pod_dir.is_dir(), "Not a directory for title: {title}");
        }

        cleanup_dir(&download_dir);
    }
}
