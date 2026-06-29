//! Phase 5 RED Tests: Test Quality Improvements for Synchronization Config
//!
//! These tests validate the Phase 5 test quality requirements:
//! - AC-19 / SCENARIO-023: Test TOML strings use the `indoc!` macro for readable indentation
//! - AC-20 / SCENARIO-024: Error-path tests assert against specific error type/message content
//! - AC-18 / SCENARIO-022: Test names are descriptive without AC/T/SCENARIO labels
//!
//! These tests are expected to FAIL (RED) because:
//! - The existing synchronization_tests.rs uses raw string TOML without indoc!
//! - The existing error-path tests only check is_err() without verifying message content
//!
//! The approach: meta-tests that inspect the source code of synchronization_tests.rs
//! to verify compliance with the style requirements.

#[cfg(test)]
mod tests {
    // =========================================================================
    // AC-19 / SCENARIO-023: Test TOML uses indoc! macro for readability
    //
    // All multi-line TOML content in synchronization_tests.rs MUST use indoc! macro.
    // No raw string TOML (`r#"..."#`) should appear without being wrapped in indoc!{}.
    // =========================================================================

    /// SCENARIO-023: Verify that synchronization_tests.rs uses indoc! for TOML strings.
    /// This inspects the actual test source file. Multi-line TOML in test code MUST
    /// be wrapped in indoc! macro.
    ///
    /// RED STATE: The current synchronization_tests.rs uses raw `r#"..."#` strings
    /// for TOML content without indoc! (e.g., lines 31-38, 68-73, 86-91).
    #[test]
    fn synchronization_tests_uses_indoc_for_toml_strings() {
        let source = include_str!("synchronization_tests.rs");

        // Count occurrences of raw TOML strings that are NOT inside indoc!
        // Pattern: `let toml_str = r#"` without preceding `indoc!`
        // If indoc is used, the pattern would be `indoc! { r#"` or similar
        let has_raw_toml_without_indoc = source.lines().any(|line| {
            let trimmed = line.trim();
            // Lines that assign a TOML string using raw literals directly
            // (not wrapped in indoc!)
            (trimmed.starts_with("let toml_str = r#\"")
                || trimmed.starts_with("let toml_str = r#\""))
                && !trimmed.contains("indoc!")
        });

        assert!(
            !has_raw_toml_without_indoc,
            "synchronization_tests.rs contains raw TOML strings (r#\"...\"#) that are not \
             wrapped in indoc! macro. All multi-line TOML test content must use indoc! \
             for readable indentation (AC-19, SCENARIO-023)."
        );
    }

    /// SCENARIO-023: Verify indoc crate is imported in synchronization_tests.rs.
    /// The `use indoc::indoc;` import must be present.
    ///
    /// RED STATE: Current synchronization_tests.rs does not import indoc.
    #[test]
    fn synchronization_tests_imports_indoc() {
        let source = include_str!("synchronization_tests.rs");

        let has_indoc_import = source.contains("use indoc::indoc")
            || source.contains("use indoc::{indoc")
            || source.contains("indoc::");

        assert!(
            has_indoc_import,
            "synchronization_tests.rs must import the indoc crate \
             (e.g., `use indoc::indoc;`) for TOML string formatting (AC-19)."
        );
    }

    // =========================================================================
    // AC-20 / SCENARIO-024: Error-path tests assert specific error types/messages
    //
    // A bare `is_err()` check without subsequent error content verification is
    // NOT sufficient. Tests must call `.unwrap_err()` and check the message.
    // =========================================================================

    /// SCENARIO-024: Verify that synchronization_tests.rs error tests do NOT
    /// use bare `is_err()` as the sole assertion. They must also check the error
    /// message content (e.g., via `unwrap_err().to_string().contains(...)`).
    ///
    /// RED STATE: Current error tests at lines 171-218 use `assert!(result.is_err(), ...)`
    /// as the only meaningful assertion without checking the error message content.
    #[test]
    fn synchronization_tests_error_assertions_check_message_content() {
        let source = include_str!("synchronization_tests.rs");

        // Find test functions that test error cases.
        // These are functions that contain `is_err()` assertions.
        // For each such function, verify it also contains `.unwrap_err()` or
        // similar pattern that inspects the error content.
        let mut in_error_test = false;
        let mut error_test_name = String::new();
        let mut has_is_err = false;
        let mut has_error_inspection = false;
        let mut violations = Vec::new();

        for line in source.lines() {
            let trimmed = line.trim();

            // Detect start of a test function
            if trimmed.starts_with("fn ") && trimmed.contains("error")
                || trimmed.contains("invalid")
                || trimmed.contains("produces_error")
            {
                // Save any previous test state
                if in_error_test && has_is_err && !has_error_inspection {
                    violations.push(error_test_name.clone());
                }
                in_error_test = true;
                error_test_name = trimmed.to_string();
                has_is_err = false;
                has_error_inspection = false;
            }

            if in_error_test {
                if trimmed.contains("is_err()") {
                    has_is_err = true;
                }
                if trimmed.contains("unwrap_err()")
                    || trimmed.contains("err.to_string()")
                    || trimmed.contains("err_msg")
                {
                    has_error_inspection = true;
                }
            }

            // Detect end of test function (closing brace at column 4)
            if in_error_test && line == "    }" {
                if has_is_err && !has_error_inspection {
                    violations.push(error_test_name.clone());
                }
                in_error_test = false;
            }
        }

        // Check last function
        if in_error_test && has_is_err && !has_error_inspection {
            violations.push(error_test_name.clone());
        }

        assert!(
            violations.is_empty(),
            "The following error-path tests in synchronization_tests.rs use bare is_err() \
             without inspecting the error message content (AC-20, SCENARIO-024): {:?}",
            violations
        );
    }

    // =========================================================================
    // AC-18 / SCENARIO-022: Test names explain convention or remove labels
    //
    // If AC/T/SCENARIO labels appear in section comments, a module-level
    // doc comment must explain the convention.
    // =========================================================================

    /// SCENARIO-022: Verify that if section comments contain AC/SCENARIO references,
    /// the module doc comment explains the labeling convention.
    ///
    /// The existing synchronization_tests.rs has section headers like:
    /// `// SCENARIO-001 / AC-01: ...`
    ///
    /// After Phase 5, either:
    /// (a) These labels are removed, OR
    /// (b) The module doc comment explicitly explains the convention.
    ///
    /// RED STATE: The current module doc comment mentions AC/SCENARIO IDs
    /// but does not explicitly explain WHY the convention is used or define it
    /// as an intentional traceability pattern.
    #[test]
    fn synchronization_tests_labels_convention_documented_or_removed() {
        let source = include_str!("synchronization_tests.rs");

        // Check if section comments still use the SCENARIO-XXX / AC-XX pattern
        let has_scenario_labels_in_comments = source.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//")
                && (trimmed.contains("SCENARIO-") || trimmed.contains("AC-"))
                && !trimmed.starts_with("//!")
        });

        if has_scenario_labels_in_comments {
            // If labels exist in comments, the module doc MUST explain the convention
            let doc_comment_section: String = source
                .lines()
                .take_while(|line| line.starts_with("//!"))
                .collect::<Vec<_>>()
                .join("\n");

            let explains_convention = doc_comment_section.contains("traceability")
                || doc_comment_section.contains("convention")
                || doc_comment_section.contains("labeling")
                || doc_comment_section.contains("These labels map");

            assert!(
                explains_convention,
                "synchronization_tests.rs contains AC/SCENARIO labels in section comments \
                 but the module-level doc comment does not explain the labeling convention. \
                 Either remove the labels or add an explanation (AC-18, SCENARIO-022)."
            );
        }
        // If no labels exist in comments, the test passes (labels were removed)
    }
}
