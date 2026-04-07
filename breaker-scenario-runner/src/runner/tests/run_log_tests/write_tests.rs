//! Tests for writing to the log — behaviors 14-17.

use std::fs;

use super::helpers::test_temp_dir;
use crate::runner::run_log::RunLog;

// =========================================================================
// Section E: Writing to the log — behaviors 14-17
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 14: write_line sends a line to the background thread
// -------------------------------------------------------------------------

#[test]
fn write_line_appears_in_file_after_flush() {
    let base = test_temp_dir("rl_write_line");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");

    log.write_line("Running [aegis_chaos] breaker=Aegis layout=Corridor");
    log.flush();

    let contents = fs::read_to_string(&path).expect("must read log file");
    assert_eq!(
        contents, "Running [aegis_chaos] breaker=Aegis layout=Corridor\n",
        "file must contain the exact line with trailing newline"
    );
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 14 edge case: empty string writes an empty line
// -------------------------------------------------------------------------

#[test]
fn write_line_empty_string_writes_empty_line() {
    let base = test_temp_dir("rl_write_line_empty");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");

    log.write_line("");
    log.flush();

    let contents = fs::read_to_string(&path).expect("must read log file");
    assert_eq!(
        contents, "\n",
        "empty string write_line must produce a single newline"
    );
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 15: write_lines sends multiple lines atomically in order
// -------------------------------------------------------------------------

#[test]
fn write_lines_sends_multiple_lines_in_order() {
    let base = test_temp_dir("rl_write_lines");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");

    log.write_lines(["line1", "line2", "line3"]);
    log.flush();

    let contents = fs::read_to_string(&path).expect("must read log file");
    assert_eq!(
        contents, "line1\nline2\nline3\n",
        "write_lines must produce lines in order with trailing newlines"
    );
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 15 edge case: empty iterator is a no-op
// -------------------------------------------------------------------------

#[test]
fn write_lines_empty_iterator_is_noop() {
    let base = test_temp_dir("rl_write_lines_empty");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");

    let empty: Vec<&str> = vec![];
    log.write_lines(empty);
    log.flush();

    let contents = fs::read_to_string(&path).expect("must read log file");
    assert!(
        contents.is_empty(),
        "empty iterator must not write anything, got: {contents:?}"
    );
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 16: Multiple write_line calls preserve ordering
// -------------------------------------------------------------------------

#[test]
fn multiple_write_line_calls_preserve_ordering() {
    let base = test_temp_dir("rl_write_order");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");

    log.write_line("A");
    log.write_line("B");
    log.write_line("C");
    log.flush();

    let contents = fs::read_to_string(&path).expect("must read log file");
    assert_eq!(
        contents, "A\nB\nC\n",
        "multiple write_line calls must preserve FIFO order"
    );
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 16 edge case: 1000 rapid write_line calls
// -------------------------------------------------------------------------

#[test]
fn thousand_rapid_write_lines_all_appear_in_order() {
    let base = test_temp_dir("rl_write_1000");
    let path = base.join("test.log");
    let log = RunLog::new(&path).expect("RunLog::new must succeed");

    for i in 0..1000 {
        log.write_line(&format!("line {i}"));
    }
    log.flush();

    let contents = fs::read_to_string(&path).expect("must read log file");
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(
        lines.len(),
        1000,
        "must have exactly 1000 lines after 1000 write_line calls"
    );
    for (i, line) in lines.iter().enumerate() {
        assert_eq!(
            *line,
            format!("line {i}"),
            "line {i} must match expected content"
        );
    }
    log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 17: writes from multiple clones interleave but each is complete
// -------------------------------------------------------------------------

#[test]
fn writes_from_clones_both_appear_as_complete_lines() {
    let base = test_temp_dir("rl_clone_writes");
    let path = base.join("test.log");
    let log1 = RunLog::new(&path).expect("RunLog::new must succeed");
    let log2 = log1.clone();

    log1.write_line("from-1");
    log2.write_line("from-2");
    log1.flush();

    let contents = fs::read_to_string(&path).expect("must read log file");
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 2, "must have exactly 2 lines");
    // Both writes happen sequentially in the same thread, so order is deterministic.
    assert_eq!(lines[0], "from-1", "first line must be from original");
    assert_eq!(lines[1], "from-2", "second line must be from clone");
    log1.shutdown();
    drop(fs::remove_dir_all(&base));
}
