//! Tests for `StressResult`, `StressFailure`, and `partition_stress_scenarios`.

use super::super::execution::{StressFailure, StressResult, build_run_list};

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
    use super::super::execution::partition_stress_scenarios;

    let runs = build_run_list(None, true);
    assert!(
        !runs.is_empty(),
        "must have scenarios for this test to be meaningful"
    );

    let (normal, stress) = partition_stress_scenarios(&runs);

    // We know chain_lightning_arc_lifecycle and split_decision_cascade have stress fields
    let stress_names: Vec<&str> = stress.iter().map(|(n, ..)| n.as_str()).collect();
    assert!(
        stress_names.contains(&"chain_lightning_arc_lifecycle"),
        "chain_lightning_arc_lifecycle must be in stress partition, got: {stress_names:?}"
    );
    assert!(
        stress_names.contains(&"split_decision_cascade"),
        "split_decision_cascade must be in stress partition, got: {stress_names:?}"
    );

    // Normal scenarios should not include stress scenarios.
    let normal_names: Vec<&str> = normal.iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        !normal_names.contains(&"chain_lightning_arc_lifecycle"),
        "chain_lightning_arc_lifecycle must NOT be in normal partition"
    );
    assert!(
        !normal_names.contains(&"split_decision_cascade"),
        "split_decision_cascade must NOT be in normal partition"
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
    use super::super::execution::partition_stress_scenarios;

    // aegis_chaos has no stress field.
    let runs = build_run_list(Some("aegis_chaos"), false);
    let (normal, stress) = partition_stress_scenarios(&runs);

    assert_eq!(normal.len(), 1);
    assert!(stress.is_empty());
}

#[test]
fn partition_stress_scenarios_returns_default_stress_config() {
    use super::super::execution::partition_stress_scenarios;

    let runs = build_run_list(Some("split_decision_cascade"), false);
    let (_, stress) = partition_stress_scenarios(&runs);

    assert_eq!(stress.len(), 1, "split_decision_cascade must be stress");
    let (_, _, config) = &stress[0];
    assert_eq!(config.runs, 4, "runs must be 4");
    assert_eq!(config.parallelism, 4, "parallelism must be 4");
}

#[test]
fn partition_stress_scenarios_empty_input_returns_empty() {
    use super::super::execution::partition_stress_scenarios;

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
