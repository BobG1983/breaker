use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use bevy::prelude::*;

use super::{
    app::{
        SharedEvalBuffer, drain_remaining_logs, guarded_update, is_timed_out, snapshot_eval_data,
    },
    execution::{
        Parallelism, StressFailure, StressResult, build_run_list, parse_parallelism, print_summary,
    },
    output::{group_logs, group_violations, is_invariant_fail_reason},
};
use crate::{
    invariants::{ScenarioFrame, ScenarioStats, ViolationEntry, ViolationLog},
    lifecycle::ScenarioConfig,
    log_capture::{CapturedLogs, LogBuffer, LogEntry},
    types::InvariantKind,
};

// -------------------------------------------------------------------------
// parse_parallelism — parses "all" or a positive integer
// -------------------------------------------------------------------------

#[test]
fn parse_parallelism_parses_all() {
    let result = parse_parallelism("all");
    assert_eq!(result, Ok(Parallelism::All));
}

#[test]
fn parse_parallelism_parses_all_case_insensitive() {
    assert_eq!(parse_parallelism("ALL"), Ok(Parallelism::All));
    assert_eq!(parse_parallelism("All"), Ok(Parallelism::All));
}

#[test]
fn parse_parallelism_parses_positive_number() {
    let result = parse_parallelism("8");
    assert_eq!(result, Ok(Parallelism::Count(8)));
}

#[test]
fn parse_parallelism_rejects_zero() {
    let result = parse_parallelism("0");
    assert!(result.is_err(), "expected error for 0, got: {result:?}");
}

#[test]
fn parse_parallelism_rejects_non_numeric_string() {
    let result = parse_parallelism("abc");
    assert!(result.is_err(), "expected error for 'abc', got: {result:?}");
}

// -------------------------------------------------------------------------
// Parallelism::resolve — resolves to concrete batch size
// -------------------------------------------------------------------------

#[test]
fn parallelism_count_resolves_to_given_value() {
    assert_eq!(Parallelism::Count(4).resolve(100), 4);
}

#[test]
fn parallelism_all_resolves_to_total() {
    assert_eq!(Parallelism::All.resolve(100), 100);
}

#[test]
fn parallelism_all_resolves_to_at_least_one() {
    assert_eq!(Parallelism::All.resolve(0), 1);
}

#[test]
fn parallelism_count_zero_resolves_to_one() {
    assert_eq!(Parallelism::Count(0).resolve(100), 1);
}

// -------------------------------------------------------------------------
// Parallelism::Display — formats for user-facing output
// -------------------------------------------------------------------------

#[test]
fn parallelism_display_count() {
    assert_eq!(Parallelism::Count(4).to_string(), "4");
}

#[test]
fn parallelism_display_all() {
    assert_eq!(Parallelism::All.to_string(), "all");
}

// -------------------------------------------------------------------------
// print_summary — prints cross-scenario summary and returns exit code
// -------------------------------------------------------------------------

#[test]
fn print_summary_returns_zero_when_all_pass() {
    let results = vec![("a".to_owned(), true), ("b".to_owned(), true)];
    assert_eq!(print_summary(&results), 0);
}

#[test]
fn print_summary_returns_one_when_any_fail() {
    let results = vec![("a".to_owned(), true), ("b".to_owned(), false)];
    assert_eq!(print_summary(&results), 1);
}

// -------------------------------------------------------------------------
// build_run_list — builds run entries from scenario discovery
// -------------------------------------------------------------------------

#[test]
fn build_run_list_single_scenario_returns_one_entry() {
    // Use a known scenario name that exists in the scenarios/ directory.
    let runs = build_run_list(Some("aegis_chaos"), false);
    assert_eq!(runs.len(), 1, "single scenario must produce 1 entry");
    assert_eq!(runs[0].0, "aegis_chaos");
}

#[test]
fn build_run_list_all_returns_one_entry_per_scenario() {
    let runs = build_run_list(None, true);
    assert!(
        runs.len() > 1,
        "--all must discover multiple scenarios, got {}",
        runs.len()
    );
    // Verify no duplicates
    let names: Vec<&str> = runs.iter().map(|(n, _)| n.as_str()).collect();
    let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
    assert_eq!(
        names.len(),
        unique.len(),
        "run list must not contain duplicate names"
    );
}

#[test]
fn build_run_list_nonexistent_scenario_returns_empty() {
    let runs = build_run_list(Some("nonexistent_scenario_xyz"), false);
    assert!(
        runs.is_empty(),
        "nonexistent scenario must produce 0 entries"
    );
}

// -------------------------------------------------------------------------
// is_timed_out — returns true when start is in the past beyond timeout
// -------------------------------------------------------------------------

/// A start `Instant` 5 seconds in the past with a 1-second timeout must
/// return `true` from `is_timed_out`.
#[test]
fn is_timed_out_returns_true_when_timeout_exceeded() {
    let start = Instant::now()
        .checked_sub(Duration::from_secs(5))
        .expect("5s subtraction must succeed");
    let timeout = Duration::from_secs(1);

    let result = is_timed_out(start, timeout);

    assert!(
        result,
        "expected is_timed_out to return true when 5s elapsed against a 1s timeout"
    );
}

// -------------------------------------------------------------------------
// is_timed_out — returns false when timeout has not yet elapsed
// -------------------------------------------------------------------------

/// A start `Instant::now()` with a 60-second timeout must return `false`
/// from `is_timed_out` immediately.
#[test]
fn is_timed_out_returns_false_when_timeout_not_exceeded() {
    let start = Instant::now();
    let timeout = Duration::from_mins(1);

    let result = is_timed_out(start, timeout);

    assert!(
        !result,
        "expected is_timed_out to return false when called immediately after start with a 60s timeout"
    );
}

// -------------------------------------------------------------------------
// drain_remaining_logs — transfers buffered entries into CapturedLogs
// -------------------------------------------------------------------------

/// `drain_remaining_logs` must move all entries from `LogBuffer` into
/// `CapturedLogs` with the frame number from `ScenarioFrame`, and leave
/// the buffer empty afterward.
#[test]
fn drain_remaining_logs_transfers_buffered_entries_to_captured_logs() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Populate the LogBuffer with 2 entries before inserting as resource.
    let buffer_entries: Vec<(bevy::log::Level, String, String)> = vec![
        (
            bevy::log::Level::WARN,
            "breaker::test".to_owned(),
            "msg1".to_owned(),
        ),
        (
            bevy::log::Level::ERROR,
            "breaker::test".to_owned(),
            "msg2".to_owned(),
        ),
    ];
    let log_buffer = LogBuffer(Arc::new(Mutex::new(buffer_entries)));
    app.insert_resource(log_buffer)
        .insert_resource(CapturedLogs::default())
        .insert_resource(ScenarioFrame(42));

    drain_remaining_logs(&mut app);

    let captured = app.world().resource::<CapturedLogs>();
    assert_eq!(
        captured.0.len(),
        2,
        "expected 2 captured log entries after drain, got {}",
        captured.0.len()
    );
    assert_eq!(captured.0[0].frame, 42, "expected frame=42 on first entry");
    assert_eq!(captured.0[0].message, "msg1");
    assert_eq!(captured.0[1].message, "msg2");

    let buffer = app.world().resource::<LogBuffer>();
    assert!(
        buffer
            .0
            .lock()
            .expect("lock must not be poisoned")
            .is_empty(),
        "expected LogBuffer to be empty after drain"
    );
}

// -------------------------------------------------------------------------
// guarded_update — returns Err when a system panics
// -------------------------------------------------------------------------

/// `guarded_update` must return `Err` containing the panic message when a
/// registered system calls `panic!("test panic")`.
#[test]
fn guarded_update_returns_err_when_system_panics() {
    fn panicking_system() {
        panic!("test panic");
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, panicking_system);

    let result = guarded_update(&mut app);

    assert!(
        result.is_err(),
        "expected guarded_update to return Err when a system panics"
    );
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("test panic"),
        "expected error message to contain 'test panic', got: {err_msg:?}"
    );
}

// -------------------------------------------------------------------------
// guarded_update — returns Ok when update succeeds
// -------------------------------------------------------------------------

/// `guarded_update` must return `Ok(())` when `app.update()` completes
/// without a panic.
#[test]
fn guarded_update_returns_ok_when_update_succeeds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let result = guarded_update(&mut app);

    assert!(
        result.is_ok(),
        "expected guarded_update to return Ok when update completes normally, got: {result:?}"
    );
}

// -------------------------------------------------------------------------
// group_violations — groups by invariant kind
// -------------------------------------------------------------------------

fn make_violation(invariant: InvariantKind, frame: u32) -> ViolationEntry {
    ViolationEntry {
        frame,
        invariant,
        entity: None,
        message: format!("test: {invariant:?}"),
    }
}

#[test]
fn group_violations_groups_by_invariant_kind() {
    let violations = vec![
        make_violation(InvariantKind::BoltInBounds, 100),
        make_violation(InvariantKind::BoltInBounds, 101),
        make_violation(InvariantKind::BoltInBounds, 105),
    ];

    let groups = group_violations(&violations);

    assert_eq!(
        groups.len(),
        1,
        "3 same-kind violations must produce 1 group"
    );
    assert_eq!(groups[0].invariant, InvariantKind::BoltInBounds);
    assert_eq!(groups[0].count, 3);
    assert_eq!(groups[0].first_frame, 100);
    assert_eq!(groups[0].last_frame, 105);
}

#[test]
fn group_violations_separates_different_invariant_kinds() {
    let violations = vec![
        make_violation(InvariantKind::BoltInBounds, 10),
        make_violation(InvariantKind::NoNaN, 20),
        make_violation(InvariantKind::BoltInBounds, 30),
    ];

    let groups = group_violations(&violations);

    assert_eq!(
        groups.len(),
        2,
        "BoltInBounds + NoNaN must produce 2 groups"
    );
    let bolt = groups
        .iter()
        .find(|g| g.invariant == InvariantKind::BoltInBounds)
        .unwrap();
    let nan = groups
        .iter()
        .find(|g| g.invariant == InvariantKind::NoNaN)
        .unwrap();
    assert_eq!(bolt.count, 2);
    assert_eq!(bolt.first_frame, 10);
    assert_eq!(bolt.last_frame, 30);
    assert_eq!(nan.count, 1);
    assert_eq!(nan.first_frame, 20);
    assert_eq!(nan.last_frame, 20);
}

#[test]
fn group_violations_single_entry_has_matching_first_last_frame() {
    let violations = vec![make_violation(InvariantKind::NoNaN, 42)];

    let groups = group_violations(&violations);

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].first_frame, 42);
    assert_eq!(groups[0].last_frame, 42);
    assert_eq!(groups[0].count, 1);
}

// -------------------------------------------------------------------------
// group_logs — groups by level + message
// -------------------------------------------------------------------------

fn make_log(level: bevy::log::Level, message: &str, frame: u32) -> LogEntry {
    LogEntry {
        level,
        target: "breaker::test".to_owned(),
        message: message.to_owned(),
        frame,
    }
}

#[test]
fn group_logs_groups_by_level_and_message() {
    let logs = vec![
        make_log(bevy::log::Level::WARN, "bad thing", 100),
        make_log(bevy::log::Level::WARN, "bad thing", 200),
        make_log(bevy::log::Level::WARN, "bad thing", 300),
    ];

    let groups = group_logs(&logs);

    assert_eq!(groups.len(), 1, "3 identical logs must produce 1 group");
    assert_eq!(groups[0].count, 3);
    assert_eq!(groups[0].first_frame, 100);
    assert_eq!(groups[0].last_frame, 300);
    assert_eq!(groups[0].message, "bad thing");
}

#[test]
fn group_logs_separates_different_messages() {
    let logs = vec![
        make_log(bevy::log::Level::WARN, "msg a", 10),
        make_log(bevy::log::Level::WARN, "msg b", 20),
    ];

    let groups = group_logs(&logs);

    assert_eq!(
        groups.len(),
        2,
        "2 different messages must produce 2 groups"
    );
}

#[test]
fn group_logs_separates_different_levels_same_message() {
    let logs = vec![
        make_log(bevy::log::Level::WARN, "same msg", 10),
        make_log(bevy::log::Level::ERROR, "same msg", 20),
    ];

    let groups = group_logs(&logs);

    assert_eq!(
        groups.len(),
        2,
        "WARN + ERROR with same message must produce 2 groups"
    );
}

// -------------------------------------------------------------------------
// is_invariant_fail_reason — matches all InvariantKind fail reasons
// -------------------------------------------------------------------------

#[test]
fn is_invariant_fail_reason_returns_true_for_all_invariant_kinds() {
    for variant in InvariantKind::ALL {
        assert!(
            is_invariant_fail_reason(variant.fail_reason()),
            "is_invariant_fail_reason must return true for {:?} fail_reason: {:?}",
            variant,
            variant.fail_reason()
        );
    }
}

#[test]
fn is_invariant_fail_reason_returns_false_for_health_check_strings() {
    assert!(
        !is_invariant_fail_reason("no actions were injected during scenario run"),
        "health check string must not match as invariant fail reason"
    );
    assert!(
        !is_invariant_fail_reason("scenario never entered Playing state"),
        "health check string must not match as invariant fail reason"
    );
}

// -------------------------------------------------------------------------
// snapshot_eval_data — captures results into shared buffer
// -------------------------------------------------------------------------

#[test]
fn snapshot_eval_data_captures_results_into_shared_buffer() {
    use crate::types::{InputStrategy, InvariantParams, ScenarioDefinition, ScriptedParams};

    let shared = SharedEvalBuffer(Arc::new(Mutex::new(None)));

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ViolationLog(vec![ViolationEntry {
            frame: 42,
            invariant: InvariantKind::BoltInBounds,
            entity: None,
            message: "test violation".into(),
        }]))
        .insert_resource(CapturedLogs::default())
        .insert_resource(ScenarioStats {
            actions_injected: 100,
            invariant_checks: 50,
            max_frame: 500,
            entered_playing: true,
            bolts_tagged: 1,
            breakers_tagged: 1,
        })
        .insert_resource(ScenarioConfig {
            definition: ScenarioDefinition {
                breaker: "Aegis".to_owned(),
                layout: "Corridor".to_owned(),
                input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
                max_frames: 1000,
                invariants: vec![],
                expected_violations: None,
                debug_setup: None,
                invariant_params: InvariantParams {
                    max_bolt_count: 8,
                    ..InvariantParams::default()
                },
                allow_early_end: true,
                stress: None,
                ..Default::default()
            },
        })
        .insert_resource(shared.clone())
        .add_systems(Last, snapshot_eval_data);

    // Before tick: buffer is None
    assert!(shared.0.lock().unwrap().is_none());

    // Tick once to run the Last system
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();

    // After tick: buffer has the snapshot
    let snapshot = shared
        .0
        .lock()
        .unwrap()
        .take()
        .expect("snapshot should be Some after tick");
    assert_eq!(snapshot.violations.len(), 1);
    assert_eq!(snapshot.violations[0].frame, 42);
    assert_eq!(snapshot.stats.max_frame, 500);
    assert_eq!(snapshot.stats.actions_injected, 100);
}

// -------------------------------------------------------------------------
// StressResult::passed — all copies pass or any copy fails
// -------------------------------------------------------------------------

/// `StressResult::passed` returns `true` when failures is empty.
#[test]
fn stress_result_passed_returns_true_when_all_copies_pass() {
    let result = StressResult {
        name: "test".into(),
        total: 64,
        failures: vec![],
    };
    assert!(
        result.passed(),
        "expected passed() to return true when all 64 copies passed"
    );
}

/// `StressResult::passed` returns `false` when any copy has failed.
#[test]
fn stress_result_passed_returns_false_when_any_copy_fails() {
    let result = StressResult {
        name: "test".into(),
        total: 64,
        failures: vec![
            StressFailure {
                copy_index: 3,
                stdout: "output".into(),
                stderr: "err".into(),
            },
            StressFailure {
                copy_index: 7,
                stdout: "output".into(),
                stderr: "err".into(),
            },
            StressFailure {
                copy_index: 12,
                stdout: "output".into(),
                stderr: "err".into(),
            },
            StressFailure {
                copy_index: 55,
                stdout: "output".into(),
                stderr: "err".into(),
            },
        ],
    };
    assert!(
        !result.passed(),
        "expected passed() to return false when 4 copies failed"
    );
}

// -------------------------------------------------------------------------
// StressResult::summary_line — human-readable pass/fail line
// -------------------------------------------------------------------------

/// `StressResult::summary_line` returns `"32/32 passed"` when all copies pass.
#[test]
fn stress_result_summary_line_all_pass() {
    let result = StressResult {
        name: "my_scenario".into(),
        total: 32,
        failures: vec![],
    };
    assert_eq!(
        result.summary_line(),
        "32/32 passed",
        "summary_line must be '32/32 passed' when all copies pass"
    );
}

/// `StressResult::summary_line` returns `"28/32 passed (4 failures)"` when
/// some copies failed.
#[test]
fn stress_result_summary_line_with_failures() {
    let result = StressResult {
        name: "my_scenario".into(),
        total: 32,
        failures: vec![
            StressFailure {
                copy_index: 1,
                stdout: String::new(),
                stderr: String::new(),
            },
            StressFailure {
                copy_index: 5,
                stdout: String::new(),
                stderr: String::new(),
            },
            StressFailure {
                copy_index: 10,
                stdout: String::new(),
                stderr: String::new(),
            },
            StressFailure {
                copy_index: 20,
                stdout: String::new(),
                stderr: String::new(),
            },
        ],
    };
    assert_eq!(
        result.summary_line(),
        "28/32 passed (4 failures)",
        "summary_line must be '28/32 passed (4 failures)' when 4 copies failed"
    );
}

// -------------------------------------------------------------------------
// partition_stress_scenarios — separates normal from stress scenarios
// -------------------------------------------------------------------------

#[test]
fn partition_stress_scenarios_separates_stress_from_normal() {
    use super::execution::partition_stress_scenarios;

    let runs = build_run_list(None, true);
    assert!(
        !runs.is_empty(),
        "must have scenarios for this test to be meaningful"
    );

    let (normal, stress) = partition_stress_scenarios(&runs);

    // We know breaker_oob_stress and prism_scatter_stress have stress: Some(())
    let stress_names: Vec<&str> = stress.iter().map(|(n, ..)| n.as_str()).collect();
    assert!(
        stress_names.contains(&"breaker_oob_stress"),
        "breaker_oob_stress must be in stress partition, got: {stress_names:?}"
    );
    assert!(
        stress_names.contains(&"prism_scatter_stress"),
        "prism_scatter_stress must be in stress partition, got: {stress_names:?}"
    );

    // Normal scenarios should not include stress scenarios.
    let normal_names: Vec<&str> = normal.iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        !normal_names.contains(&"breaker_oob_stress"),
        "breaker_oob_stress must NOT be in normal partition"
    );
    assert!(
        !normal_names.contains(&"prism_scatter_stress"),
        "prism_scatter_stress must NOT be in normal partition"
    );

    // Total must equal original.
    assert_eq!(
        normal.len() + stress.len(),
        runs.len(),
        "partition must preserve total count"
    );
}

#[test]
fn partition_stress_scenarios_all_normal_when_no_stress() {
    use super::execution::partition_stress_scenarios;

    // aegis_chaos has no stress field.
    let runs = build_run_list(Some("aegis_chaos"), false);
    let (normal, stress) = partition_stress_scenarios(&runs);

    assert_eq!(normal.len(), 1);
    assert!(stress.is_empty());
}

#[test]
fn partition_stress_scenarios_returns_default_stress_config() {
    use super::execution::partition_stress_scenarios;

    let runs = build_run_list(Some("breaker_oob_stress"), false);
    let (_, stress) = partition_stress_scenarios(&runs);

    assert_eq!(stress.len(), 1, "breaker_oob_stress must be stress");
    let (_, _, config) = &stress[0];
    assert_eq!(config.runs, 32, "default runs must be 32");
    assert_eq!(config.parallelism, 32, "default parallelism must be 32");
}

#[test]
fn partition_stress_scenarios_empty_input_returns_empty() {
    use super::execution::partition_stress_scenarios;

    let (normal, stress) = partition_stress_scenarios(&[]);
    assert!(normal.is_empty());
    assert!(stress.is_empty());
}

// -------------------------------------------------------------------------
// StressResult::pass_count — derived from total minus failures
// -------------------------------------------------------------------------

#[test]
fn stress_result_pass_count_derived_from_total_minus_failures() {
    let result = StressResult {
        name: "test".into(),
        total: 10,
        failures: vec![
            StressFailure {
                copy_index: 2,
                stdout: String::new(),
                stderr: String::new(),
            },
            StressFailure {
                copy_index: 7,
                stdout: String::new(),
                stderr: String::new(),
            },
        ],
    };
    assert_eq!(result.pass_count(), 8);
}
