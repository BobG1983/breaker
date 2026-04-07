//! Tests for `format_log_path_message` — behaviors 28-29.

use std::path::PathBuf;

use crate::runner::run_log::format_log_path_message;

// =========================================================================
// Section I: End-to-end log file location message — behaviors 28-29
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 28: format_log_path_message returns the announcement string
// -------------------------------------------------------------------------

#[test]
fn format_log_path_message_returns_full_log_available_at() {
    let path = PathBuf::from("/tmp/breaker-scenario-runner/2026-04-07T14-32-05.log");
    let result = format_log_path_message(&path);
    assert_eq!(
        result,
        "Full log available at: /tmp/breaker-scenario-runner/2026-04-07T14-32-05.log"
    );
}

// -------------------------------------------------------------------------
// Behavior 28 edge case: path with spaces
// -------------------------------------------------------------------------

#[test]
fn format_log_path_message_path_with_spaces_no_quoting() {
    let path = PathBuf::from("/tmp/my logs/2026-04-07T14-32-05.log");
    let result = format_log_path_message(&path);
    assert_eq!(
        result,
        "Full log available at: /tmp/my logs/2026-04-07T14-32-05.log"
    );
}
