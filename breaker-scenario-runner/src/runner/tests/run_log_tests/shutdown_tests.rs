//! Tests for shutdown behavior — behaviors 21-22.

use std::fs;

use super::helpers::test_temp_dir;
use crate::runner::run_log::RunLog;

// =========================================================================
// Section G: Shutdown — behaviors 21-22
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 21: shutdown flushes remaining writes and joins the thread
// -------------------------------------------------------------------------

#[test]
fn shutdown_flushes_final_writes() {
    let base = test_temp_dir("rl_shutdown_flush");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");

    log.write_line("final line");
    log.shutdown();

    let contents = fs::read_to_string(&path).expect("must read log file");
    assert_eq!(
        contents, "final line\n",
        "shutdown must flush pending writes before returning"
    );
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 21 edge case: shutdown with no prior writes
// -------------------------------------------------------------------------

#[test]
fn shutdown_with_no_writes_succeeds_file_empty() {
    let base = test_temp_dir("rl_shutdown_empty");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");

    log.shutdown();

    let contents = fs::read_to_string(&path).expect("must read log file");
    assert!(
        contents.is_empty(),
        "file must be empty after shutdown with no writes"
    );
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 22: after shutdown, remaining clones' write_line calls are no-ops
// -------------------------------------------------------------------------

#[test]
fn write_line_after_shutdown_is_noop_no_panic() {
    let base = test_temp_dir("rl_write_after_shutdown");
    let path = base.join("test.log");
    let log1 = RunLog::new(&path).expect("RunLog::new must succeed");
    let log2 = log1.clone();

    log1.shutdown();
    // Must not panic.
    log2.write_line("after shutdown");

    let contents = fs::read_to_string(&path).expect("must read log file");
    assert!(
        !contents.contains("after shutdown"),
        "writes after shutdown must not appear in the file"
    );
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 22 edge case: write_lines after shutdown is also no-op
// -------------------------------------------------------------------------

#[test]
fn write_lines_after_shutdown_is_noop_no_panic() {
    let base = test_temp_dir("rl_write_lines_after_shutdown");
    let path = base.join("test.log");
    let log1 = RunLog::new(&path).expect("RunLog::new must succeed");
    let log2 = log1.clone();

    log1.shutdown();
    // Must not panic.
    log2.write_lines(["alpha", "beta", "gamma"]);

    let contents = fs::read_to_string(&path).expect("must read log file");
    assert!(
        contents.is_empty(),
        "write_lines after shutdown must not produce any output"
    );
    drop(fs::remove_dir_all(&base));
}
