//! Tests for `log_file_path` and `resolve_log_file_path` — behaviors 3-8.

use std::{fs, path::PathBuf};

use super::helpers::test_temp_dir;
use crate::runner::run_log::{log_file_path, resolve_log_file_path};

// =========================================================================
// Section B: Log file path construction (`log_file_path`) — behaviors 3-4
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 3: log_file_path constructs <base_dir>/<timestamp>.log
// -------------------------------------------------------------------------

#[test]
fn log_file_path_constructs_base_dir_timestamp_dot_log() {
    let base_dir = PathBuf::from("/tmp/breaker-scenario-runner");
    let result = log_file_path(&base_dir, "2026-04-07T14-32-05");
    assert_eq!(
        result,
        PathBuf::from("/tmp/breaker-scenario-runner/2026-04-07T14-32-05.log")
    );
}

// -------------------------------------------------------------------------
// Behavior 3 edge case: midnight timestamp
// -------------------------------------------------------------------------

#[test]
fn log_file_path_midnight_timestamp() {
    let base_dir = PathBuf::from("/tmp/breaker-scenario-runner");
    let result = log_file_path(&base_dir, "2026-01-01T00-00-00");
    assert_eq!(
        result,
        PathBuf::from("/tmp/breaker-scenario-runner/2026-01-01T00-00-00.log")
    );
}

// -------------------------------------------------------------------------
// Behavior 4: log_file_path works with non-standard base directories
// -------------------------------------------------------------------------

#[test]
fn log_file_path_non_standard_base_dir() {
    let base_dir = PathBuf::from("/home/user/custom-output");
    let result = log_file_path(&base_dir, "2026-01-01T00-00-00");
    assert_eq!(
        result,
        PathBuf::from("/home/user/custom-output/2026-01-01T00-00-00.log")
    );
}

// -------------------------------------------------------------------------
// Behavior 4 edge case: trailing slash on base_dir
// -------------------------------------------------------------------------

#[test]
fn log_file_path_base_dir_with_trailing_slash() {
    let base_dir = PathBuf::from("/tmp/foo/");
    let result_with_slash = log_file_path(&base_dir, "2026-01-01T00-00-00");
    let result_without_slash = log_file_path(&PathBuf::from("/tmp/foo"), "2026-01-01T00-00-00");
    assert_eq!(
        result_with_slash, result_without_slash,
        "trailing slash must not affect the result"
    );
}

// =========================================================================
// Section C: Log file collision resolution (`resolve_log_file_path`) — behaviors 5-8
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 5: resolve_log_file_path returns <timestamp>.log when no collision
// -------------------------------------------------------------------------

#[test]
fn resolve_log_file_path_returns_unsuffixed_when_no_collision() {
    let base = test_temp_dir("rlfp_no_collision");
    let result = resolve_log_file_path(&base, "2026-04-07T14-32-05");
    assert_eq!(
        result,
        base.join("2026-04-07T14-32-05.log"),
        "must return unsuffixed path when no collision exists"
    );
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 5 edge case: base_dir does not exist
// -------------------------------------------------------------------------

#[test]
fn resolve_log_file_path_nonexistent_base_dir_returns_unsuffixed() {
    let base = std::env::temp_dir().join(format!(
        "breaker_test_run_log_{}_rlfp_nodir",
        std::process::id()
    ));
    // Ensure it does not exist.
    drop(fs::remove_dir_all(&base));
    let result = resolve_log_file_path(&base, "2026-04-07T14-32-05");
    assert_eq!(
        result,
        base.join("2026-04-07T14-32-05.log"),
        "must return unsuffixed path when base_dir does not exist"
    );
}

// -------------------------------------------------------------------------
// Behavior 6: resolve_log_file_path appends -1 when base path exists
// -------------------------------------------------------------------------

#[test]
fn resolve_log_file_path_appends_suffix_1_when_base_exists() {
    let base = test_temp_dir("rlfp_collision_1");
    // Create the colliding file.
    fs::write(base.join("2026-04-07T14-32-05.log"), "existing").unwrap();

    let result = resolve_log_file_path(&base, "2026-04-07T14-32-05");
    assert_eq!(
        result,
        base.join("2026-04-07T14-32-05-1.log"),
        "must return -1 suffix when base file exists"
    );
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 7: resolve_log_file_path increments suffix until unused
// -------------------------------------------------------------------------

#[test]
fn resolve_log_file_path_increments_suffix_past_existing_collisions() {
    let base = test_temp_dir("rlfp_collision_3");
    // Create colliding files: base, -1, -2.
    fs::write(base.join("2026-04-07T14-32-05.log"), "").unwrap();
    fs::write(base.join("2026-04-07T14-32-05-1.log"), "").unwrap();
    fs::write(base.join("2026-04-07T14-32-05-2.log"), "").unwrap();

    let result = resolve_log_file_path(&base, "2026-04-07T14-32-05");
    assert_eq!(
        result,
        base.join("2026-04-07T14-32-05-3.log"),
        "must return -3 suffix when base, -1, and -2 all exist"
    );
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 7 edge case: 100 collisions
// -------------------------------------------------------------------------

#[test]
fn resolve_log_file_path_handles_100_collisions() {
    let base = test_temp_dir("rlfp_collision_100");
    // Create 100 files: base file + -1 through -99.
    fs::write(base.join("2026-04-07T14-32-05.log"), "").unwrap();
    for i in 1..100 {
        fs::write(base.join(format!("2026-04-07T14-32-05-{i}.log")), "").unwrap();
    }

    let result = resolve_log_file_path(&base, "2026-04-07T14-32-05");
    assert_eq!(
        result,
        base.join("2026-04-07T14-32-05-100.log"),
        "must return -100 suffix when base and -1 through -99 all exist"
    );
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 8: resolve_log_file_path ignores files with different timestamps
// -------------------------------------------------------------------------

#[test]
fn resolve_log_file_path_ignores_different_timestamps() {
    let base = test_temp_dir("rlfp_diff_ts");
    // Create a file with a different timestamp.
    fs::write(base.join("2026-04-07T15-00-00.log"), "").unwrap();

    let result = resolve_log_file_path(&base, "2026-04-07T14-32-05");
    assert_eq!(
        result,
        base.join("2026-04-07T14-32-05.log"),
        "must return unsuffixed path when only different timestamps exist"
    );
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 8 edge case: non-.log file with same timestamp prefix
// -------------------------------------------------------------------------

#[test]
fn resolve_log_file_path_ignores_non_log_extensions() {
    let base = test_temp_dir("rlfp_non_log_ext");
    // Create a .txt file with the same timestamp prefix.
    fs::write(base.join("2026-04-07T14-32-05.txt"), "").unwrap();

    let result = resolve_log_file_path(&base, "2026-04-07T14-32-05");
    assert_eq!(
        result,
        base.join("2026-04-07T14-32-05.log"),
        "non-.log files must not be treated as collisions"
    );
    drop(fs::remove_dir_all(&base));
}
