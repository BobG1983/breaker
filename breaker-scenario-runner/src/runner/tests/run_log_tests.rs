//! Tests for `run_log` — async file-based logging via background thread + mpsc channel.

use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    invariants::{ScenarioStats, ViolationEntry},
    runner::{
        app::{EvalSnapshot, SharedEvalBuffer},
        run_log::{
            RunLog, format_log_path_message, log_file_path, now_timestamp, resolve_log_file_path,
        },
    },
    types::{InputStrategy, InvariantKind, ScenarioDefinition, ScriptedParams},
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates an isolated temp directory for a single test, returning its path.
///
/// The caller is responsible for cleaning up via `fs::remove_dir_all`.
fn test_temp_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "breaker_test_run_log_{}_{test_name}",
        std::process::id()
    ));
    // Clean up any leftover from a previous interrupted run.
    drop(fs::remove_dir_all(&dir));
    fs::create_dir_all(&dir).expect("failed to create test temp dir");
    dir
}

/// Builds a clean `EvalSnapshot` with no violations and no logs.
fn clean_snapshot(_scenario_name: &str) -> EvalSnapshot {
    EvalSnapshot {
        violations: vec![],
        logs: vec![],
        stats: ScenarioStats {
            actions_injected: 0,
            invariant_checks: 10,
            max_frame: 50,
            entered_playing: true,
            bolts_tagged: 1,
            breakers_tagged: 1,
            ..Default::default()
        },
        definition: ScenarioDefinition {
            breaker: "test".into(),
            layout: "test".into(),
            input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
            max_frames: 100,
            disallowed_failures: vec![],
            ..Default::default()
        },
    }
}

/// Builds an `EvalSnapshot` with the given violations.
fn snapshot_with_violations(violations: Vec<ViolationEntry>) -> EvalSnapshot {
    let mut snap = clean_snapshot("test_scenario");
    snap.violations = violations;
    snap
}

fn make_violation(invariant: InvariantKind, frame: u32, message: &str) -> ViolationEntry {
    ViolationEntry {
        frame,
        invariant,
        entity: None,
        message: message.to_owned(),
    }
}

// =========================================================================
// Section A: Timestamp formatting (`now_timestamp`) — behaviors 1-2
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 1: now_timestamp returns a filesystem-safe ISO 8601 string
// -------------------------------------------------------------------------

#[test]
fn now_timestamp_returns_19_char_filesystem_safe_string() {
    let result = now_timestamp();
    assert_eq!(
        result.len(),
        19,
        "timestamp must be exactly 19 characters (YYYY-MM-DDTHH-MM-SS), got {} chars: {result:?}",
        result.len()
    );
}

#[test]
fn now_timestamp_matches_pattern_yyyy_mm_dd_t_hh_mm_ss() {
    let result = now_timestamp();
    // Validate: YYYY-MM-DDTHH-MM-SS
    let parts: Vec<&str> = result.split('T').collect();
    assert_eq!(
        parts.len(),
        2,
        "timestamp must have exactly one T separator, got: {result:?}"
    );

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    assert_eq!(
        date_parts.len(),
        3,
        "date portion must have 3 parts: {result:?}"
    );
    assert_eq!(date_parts[0].len(), 4, "year must be 4 digits: {result:?}");
    assert_eq!(date_parts[1].len(), 2, "month must be 2 digits: {result:?}");
    assert_eq!(date_parts[2].len(), 2, "day must be 2 digits: {result:?}");

    let time_parts: Vec<&str> = parts[1].split('-').collect();
    assert_eq!(
        time_parts.len(),
        3,
        "time portion must have 3 parts separated by hyphens: {result:?}"
    );
    assert_eq!(time_parts[0].len(), 2, "hour must be 2 digits: {result:?}");
    assert_eq!(
        time_parts[1].len(),
        2,
        "minute must be 2 digits: {result:?}"
    );
    assert_eq!(
        time_parts[2].len(),
        2,
        "second must be 2 digits: {result:?}"
    );
}

// -------------------------------------------------------------------------
// Behavior 1 edge case: no colons in the timestamp
// -------------------------------------------------------------------------

#[test]
fn now_timestamp_contains_no_colons() {
    let result = now_timestamp();
    assert!(
        !result.contains(':'),
        "timestamp must not contain colons (invalid in filenames), got: {result:?}"
    );
}

// -------------------------------------------------------------------------
// Behavior 2: hyphens used instead of colons for time components
// -------------------------------------------------------------------------

#[test]
fn now_timestamp_time_portion_uses_hyphens() {
    let result = now_timestamp();
    let time_portion = result
        .split('T')
        .nth(1)
        .expect("timestamp must contain a T separator");
    assert!(
        time_portion.contains('-'),
        "time portion must use hyphens as separators, got: {time_portion:?}"
    );
    // All characters in the time portion should be digits or hyphens.
    assert!(
        time_portion.chars().all(|c| c.is_ascii_digit() || c == '-'),
        "time portion must only contain digits and hyphens, got: {time_portion:?}"
    );
}

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

// =========================================================================
// Section H: Integration of RunLog with collect_and_evaluate — behaviors 23-27
// =========================================================================

use crate::runner::app::collect_and_evaluate;

// -------------------------------------------------------------------------
// Behavior 23: collect_and_evaluate writes verbose output to RunLog when Some
// -------------------------------------------------------------------------

#[test]
fn collect_and_evaluate_writes_to_run_log_when_some() {
    let base = test_temp_dir("rl_cae_writes");
    let log_path = base.join("test.log");
    let run_log = RunLog::new(&log_path).expect("RunLog::new must succeed");

    let violations = vec![
        make_violation(InvariantKind::BoltInBounds, 10, "bolt oob"),
        make_violation(InvariantKind::NoNaN, 20, "nan detected"),
    ];
    let snapshot = snapshot_with_violations(violations);
    let buffer = SharedEvalBuffer(Arc::new(Mutex::new(Some(snapshot))));

    collect_and_evaluate(&buffer, "test_scenario", false, Some(&run_log));
    run_log.flush();

    let contents = fs::read_to_string(&log_path).expect("must read log file");
    // The log file must contain some output about the scenario stats/violations.
    assert!(
        !contents.is_empty(),
        "log file must contain output when RunLog is Some and there are violations"
    );
    run_log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 23 edge case: zero violations still gets stats line
// -------------------------------------------------------------------------

#[test]
fn collect_and_evaluate_writes_stats_to_run_log_with_zero_violations() {
    let base = test_temp_dir("rl_cae_no_violations");
    let log_path = base.join("test.log");
    let run_log = RunLog::new(&log_path).expect("RunLog::new must succeed");

    let snapshot = clean_snapshot("test_scenario");
    let buffer = SharedEvalBuffer(Arc::new(Mutex::new(Some(snapshot))));

    collect_and_evaluate(&buffer, "test_scenario", false, Some(&run_log));
    run_log.flush();

    let contents = fs::read_to_string(&log_path).expect("must read log file");
    assert!(
        !contents.is_empty(),
        "log file must contain at least the stats line even with zero violations"
    );
    run_log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 24: collect_and_evaluate writes verbose violation details regardless of verbose flag
// -------------------------------------------------------------------------

#[test]
fn collect_and_evaluate_writes_verbose_details_to_log_even_when_not_verbose() {
    let base = test_temp_dir("rl_cae_verbose_details");
    let log_path = base.join("test.log");
    let run_log = RunLog::new(&log_path).expect("RunLog::new must succeed");

    let violations = vec![make_violation(
        InvariantKind::BoltInBounds,
        42,
        "bolt at (-50.0, 300.0) outside bounds",
    )];
    let snapshot = snapshot_with_violations(violations);
    let buffer = SharedEvalBuffer(Arc::new(Mutex::new(Some(snapshot))));

    // verbose = false — but the log should still get verbose output.
    collect_and_evaluate(&buffer, "test_scenario", false, Some(&run_log));
    run_log.flush();

    let contents = fs::read_to_string(&log_path).expect("must read log file");
    // Must contain violation details (frame, kind, message).
    assert!(
        contents.contains("BoltInBounds"),
        "log must contain invariant kind 'BoltInBounds', got: {contents:?}"
    );
    assert!(
        contents.contains("bolt at (-50.0, 300.0) outside bounds"),
        "log must contain violation message, got: {contents:?}"
    );
    run_log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 25: collect_and_evaluate writes pass/fail verdict to the log
// -------------------------------------------------------------------------

#[test]
fn collect_and_evaluate_writes_pass_verdict_to_log() {
    let base = test_temp_dir("rl_cae_pass_verdict");
    let log_path = base.join("test.log");
    let run_log = RunLog::new(&log_path).expect("RunLog::new must succeed");

    let snapshot = clean_snapshot("test_scenario");
    let buffer = SharedEvalBuffer(Arc::new(Mutex::new(Some(snapshot))));

    let passed = collect_and_evaluate(&buffer, "test_scenario", false, Some(&run_log));
    run_log.flush();

    assert!(passed, "clean snapshot must pass");
    let contents = fs::read_to_string(&log_path).expect("must read log file");
    assert!(
        contents.contains("PASS"),
        "log must contain PASS verdict for a passing scenario, got: {contents:?}"
    );
    run_log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 25 edge case: failing scenario writes FAIL verdict
// -------------------------------------------------------------------------

#[test]
fn collect_and_evaluate_writes_fail_verdict_to_log() {
    let base = test_temp_dir("rl_cae_fail_verdict");
    let log_path = base.join("test.log");
    let run_log = RunLog::new(&log_path).expect("RunLog::new must succeed");

    let violations = vec![make_violation(InvariantKind::BoltInBounds, 10, "bolt oob")];
    let snapshot = snapshot_with_violations(violations);
    let buffer = SharedEvalBuffer(Arc::new(Mutex::new(Some(snapshot))));

    let passed = collect_and_evaluate(&buffer, "test_scenario", false, Some(&run_log));
    run_log.flush();

    assert!(!passed, "snapshot with violations must fail");
    let contents = fs::read_to_string(&log_path).expect("must read log file");
    assert!(
        contents.contains("FAIL"),
        "log must contain FAIL verdict for a failing scenario, got: {contents:?}"
    );
    run_log.shutdown();
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 26: collect_and_evaluate with None for RunLog sends output to stdout only
// -------------------------------------------------------------------------

#[test]
fn collect_and_evaluate_with_none_run_log_does_not_create_log_file() {
    let base = test_temp_dir("rl_cae_none");
    let violations = vec![make_violation(InvariantKind::BoltInBounds, 10, "bolt oob")];
    let snapshot = snapshot_with_violations(violations);
    let buffer = SharedEvalBuffer(Arc::new(Mutex::new(Some(snapshot))));

    // No RunLog — pass None.
    collect_and_evaluate(&buffer, "test_scenario", true, None);

    // Verify no .log files were created in the temp dir.
    let log_files: Vec<_> = fs::read_dir(&base)
        .expect("must read dir")
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
        .collect();
    assert!(
        log_files.is_empty(),
        "no log files must be created when RunLog is None"
    );
    drop(fs::remove_dir_all(&base));
}

// -------------------------------------------------------------------------
// Behavior 27: collect_and_evaluate with None for RunLog does not panic
// -------------------------------------------------------------------------

#[test]
fn collect_and_evaluate_with_none_run_log_does_not_panic() {
    let violations = vec![make_violation(InvariantKind::BoltInBounds, 10, "bolt oob")];
    let snapshot = snapshot_with_violations(violations);
    let buffer = SharedEvalBuffer(Arc::new(Mutex::new(Some(snapshot))));

    // Must not panic.
    let _passed = collect_and_evaluate(&buffer, "test_scenario", false, None);
}

// -------------------------------------------------------------------------
// Behavior 27 edge case: empty snapshot (None captured) with None RunLog
// -------------------------------------------------------------------------

#[test]
fn collect_and_evaluate_no_snapshot_none_run_log_returns_false() {
    let buffer = SharedEvalBuffer(Arc::new(Mutex::new(None)));
    let passed = collect_and_evaluate(&buffer, "test_scenario", false, None);
    assert!(
        !passed,
        "must return false when no snapshot was captured, even with None RunLog"
    );
}

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
