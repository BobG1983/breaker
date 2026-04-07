//! Tests for flush behavior — behaviors 18-20.

use std::fs;

use super::helpers::test_temp_dir;
use crate::runner::run_log::RunLog;

// =========================================================================
// Section F: Flush — behaviors 18-20
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 18: flush blocks until all pending writes are on disk
// -------------------------------------------------------------------------

#[test]
fn flush_blocks_until_all_pending_writes_on_disk() {
    let base = test_temp_dir("rl_flush_100");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");

    for i in 0..100 {
        log.write_line(&format!("line {i}"));
    }
    log.flush();

    let contents = fs::read_to_string(&path).expect("must read log file");
    let line_count = contents.lines().count();
    assert_eq!(
        line_count, 100,
        "all 100 lines must be present after flush, got {line_count}"
    );
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 18 edge case: flush with no pending writes
// -------------------------------------------------------------------------

#[test]
fn flush_with_no_pending_writes_is_noop() {
    let base = test_temp_dir("rl_flush_empty");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");

    log.flush(); // No writes pending.

    let contents = fs::read_to_string(&path).expect("must read log file");
    assert!(
        contents.is_empty(),
        "file must remain empty after flush with no writes"
    );
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 19: flush can be called multiple times
// -------------------------------------------------------------------------

#[test]
fn flush_called_multiple_times_accumulates_writes() {
    let base = test_temp_dir("rl_flush_multi");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");

    log.write_line("first");
    log.flush();
    log.write_line("second");
    log.flush();

    let contents = fs::read_to_string(&path).expect("must read log file");
    assert_eq!(
        contents, "first\nsecond\n",
        "multiple flush cycles must accumulate writes"
    );
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 19 edge case: three consecutive flushes with no writes between
// -------------------------------------------------------------------------

#[test]
fn three_consecutive_flushes_no_writes_between_all_succeed() {
    let base = test_temp_dir("rl_flush_triple");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");

    log.write_line("data");
    log.flush();
    log.flush();
    log.flush();

    let contents = fs::read_to_string(&path).expect("must read log file");
    assert_eq!(
        contents, "data\n",
        "consecutive flushes with no writes must not change file"
    );
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 20: flush on a clone after shutdown is a no-op
// -------------------------------------------------------------------------

#[test]
fn flush_on_clone_after_shutdown_is_noop() {
    let base = test_temp_dir("rl_flush_after_shutdown");
    let path = base.join("test.log");
    let log1 = RunLog::new(&path).expect("RunLog::new must succeed");
    let log2 = log1.clone();

    log1.shutdown();
    // Must not hang or panic.
    log2.flush();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 20 edge case: multiple flushes on clone after shutdown
// -------------------------------------------------------------------------

#[test]
fn multiple_flushes_on_clone_after_shutdown_all_noop() {
    let base = test_temp_dir("rl_flush_multi_after_shutdown");
    let path = base.join("test.log");
    let log1 = RunLog::new(&path).expect("RunLog::new must succeed");
    let log2 = log1.clone();

    log1.shutdown();
    log2.flush();
    log2.flush();
    log2.flush();
    // All must return immediately without hanging or panicking.
    drop(fs::remove_dir_all(&base));
}
