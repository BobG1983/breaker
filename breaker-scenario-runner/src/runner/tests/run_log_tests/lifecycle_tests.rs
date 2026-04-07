//! Tests for `RunLog` construction and lifecycle — behaviors 9-13.

use std::{fs, path::PathBuf};

use super::helpers::test_temp_dir;
use crate::runner::run_log::RunLog;

// =========================================================================
// Section D: RunLog construction and lifecycle — behaviors 9-13
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 9: RunLog::new creates the log file on disk
// -------------------------------------------------------------------------

#[test]
fn run_log_new_creates_file_on_disk() {
    let base = test_temp_dir("rl_new_creates");
    let path = base.join("2026-04-07T14-32-05.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");
    assert!(
        path.exists(),
        "log file must exist on disk after RunLog::new"
    );
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 9 edge case: two separate RunLogs in the same directory
// -------------------------------------------------------------------------

#[test]
fn run_log_new_two_separate_logs_in_same_dir() {
    let base = test_temp_dir("rl_new_two");
    let path1 = base.join("log1.log");
    let path2 = base.join("log2.log");
    let log1 = RunLog::new(&path1).expect("first RunLog::new must succeed");
    let log2 = RunLog::new(&path2).expect("second RunLog::new must succeed");
    assert!(path1.exists(), "first log file must exist");
    assert!(path2.exists(), "second log file must exist");
    log1.shutdown();
    log2.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 10: RunLog::new creates intermediate parent directories
// -------------------------------------------------------------------------

#[test]
fn run_log_new_creates_intermediate_parent_directories() {
    let base = test_temp_dir("rl_new_nested");
    let path = base.join("nested/deeply/test.log");
    assert!(
        !base.join("nested").exists(),
        "nested dir must not exist before test"
    );
    let log = RunLog::new(&path).expect("RunLog::new must succeed for nested path");
    assert!(
        path.exists(),
        "log file must exist on disk after creating nested directories"
    );
    assert!(
        base.join("nested/deeply").is_dir(),
        "intermediate directories must be created"
    );
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 10 edge case: parent directory already exists
// -------------------------------------------------------------------------

#[test]
fn run_log_new_succeeds_when_parent_already_exists() {
    let base = test_temp_dir("rl_new_existing_parent");
    // Parent already exists (base itself).
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed when parent exists");
    assert!(path.exists());
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 11: RunLog::new fails when path is truly uncreatable
// -------------------------------------------------------------------------

#[test]
fn run_log_new_fails_for_impossible_path() {
    // /dev/null is a file on Unix, so /dev/null/impossible/log.log cannot be created.
    let path = PathBuf::from("/dev/null/impossible/log.log");
    let result = RunLog::new(&path);
    assert!(
        result.is_err(),
        "RunLog::new must return Err for an uncreatable path"
    );
}

// -------------------------------------------------------------------------
// Behavior 12: RunLog::path returns the path passed to new
// -------------------------------------------------------------------------

#[test]
fn run_log_path_returns_construction_path() {
    let base = test_temp_dir("rl_path");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");
    assert_eq!(
        log.path(),
        path,
        "path() must return the same path passed to new()"
    );
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 13: RunLog is Clone and both clones share the same file
// -------------------------------------------------------------------------

#[test]
fn run_log_clone_writes_to_same_file() {
    let base = test_temp_dir("rl_clone");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");
    let log2 = log.clone();

    log.write_line("from original");
    log2.write_line("from clone");
    log.flush();

    let contents = fs::read_to_string(&path).expect("must read log file");
    assert!(
        contents.contains("from original"),
        "log file must contain 'from original', got: {contents:?}"
    );
    assert!(
        contents.contains("from clone"),
        "log file must contain 'from clone', got: {contents:?}"
    );
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}
