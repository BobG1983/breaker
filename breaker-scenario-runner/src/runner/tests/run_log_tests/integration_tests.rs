//! Tests for `RunLog` integration with `collect_and_evaluate` — behaviors 23-27.

use std::{
    fs,
    sync::{Arc, Mutex},
};

use super::helpers::{clean_snapshot, make_violation, snapshot_with_violations, test_temp_dir};
use crate::{
    runner::{
        app::{SharedEvalBuffer, collect_and_evaluate},
        run_log::RunLog,
    },
    types::InvariantKind,
};

// =========================================================================
// Section H: Integration of RunLog with collect_and_evaluate — behaviors 23-27
// =========================================================================

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
