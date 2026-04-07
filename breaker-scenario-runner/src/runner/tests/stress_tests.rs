//! Tests for `StressResult`, `StressFailure`, and `partition_stress_scenarios`.

use std::path::PathBuf;

use crate::{
    runner::execution::{StressFailure, StressResult, build_run_list, partition_stress_scenarios},
    types::{ScenarioDefinition, StressConfig},
};

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
// partition_stress_scenarios — separates normal from stress using enriched entries
// -------------------------------------------------------------------------

/// `partition_stress_scenarios` separates stress from normal using pre-parsed definitions.
#[test]
fn partition_stress_scenarios_separates_stress_from_normal() {
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

/// `partition_stress_scenarios` correctly identifies known stress scenarios.
#[test]
fn partition_stress_scenarios_known_stress_and_normal() {
    let runs = build_run_list(None, true);
    assert!(!runs.is_empty(), "must have scenarios for this test");

    let (normal, stress) = partition_stress_scenarios(&runs);

    // Stress partition must contain known stress scenarios.
    let stress_names: Vec<&str> = stress.iter().map(|(n, ..)| n.as_str()).collect();
    assert!(
        stress_names.contains(&"chain_lightning_arc_lifecycle"),
        "chain_lightning_arc_lifecycle must be in stress"
    );
    assert!(
        stress_names.contains(&"split_decision_cascade"),
        "split_decision_cascade must be in stress"
    );

    // aegis_chaos (known non-stress) must be in normal partition.
    assert!(
        normal.iter().any(|(n, _)| n == "aegis_chaos"),
        "aegis_chaos must be in normal partition"
    );
}

/// `partition_stress_scenarios` with a single non-stress scenario returns empty stress vec.
#[test]
fn partition_stress_scenarios_all_normal_when_no_stress() {
    // aegis_chaos has no stress field.
    let runs = build_run_list(Some("aegis_chaos"), false);
    let (normal, stress) = partition_stress_scenarios(&runs);

    assert_eq!(normal.len(), 1);
    assert!(stress.is_empty());
}

/// `partition_stress_scenarios` preserves `StressConfig` values from the pre-parsed definition.
#[test]
fn partition_stress_scenarios_returns_correct_stress_config() {
    let runs = build_run_list(Some("split_decision_cascade"), false);
    let (_, stress) = partition_stress_scenarios(&runs);

    assert_eq!(stress.len(), 1, "split_decision_cascade must be stress");
    let (_, _, config) = &stress[0];
    assert_eq!(config.runs, 4, "runs must be 4");
    assert_eq!(config.parallelism, 4, "parallelism must be 4");
}

/// `partition_stress_scenarios` with empty input returns empty normal and stress vecs.
#[test]
fn partition_stress_scenarios_empty_input_returns_empty() {
    let empty: Vec<(String, PathBuf, ScenarioDefinition)> = vec![];
    let (normal, stress) = partition_stress_scenarios(&empty);
    assert!(normal.is_empty());
    assert!(stress.is_empty());
}

/// `partition_stress_scenarios` preserves a synthesized `StressConfig` with non-default values.
#[test]
fn partition_stress_scenarios_preserves_synthesized_stress_config() {
    let def = ScenarioDefinition {
        stress: Some(StressConfig {
            runs: 32,
            parallelism: 32,
        }),
        ..Default::default()
    };
    let runs = vec![(
        "synthetic_stress".to_owned(),
        PathBuf::from("/tmp/synthetic.scenario.ron"),
        def,
    )];

    let (normal, stress) = partition_stress_scenarios(&runs);
    assert!(
        normal.is_empty(),
        "synthetic stress scenario should not be in normal"
    );
    assert_eq!(
        stress.len(),
        1,
        "synthetic stress scenario should be in stress"
    );
    let (name, _, config) = &stress[0];
    assert_eq!(name, "synthetic_stress");
    assert_eq!(config.runs, 32, "runs must be 32");
    assert_eq!(config.parallelism, 32, "parallelism must be 32");
}

/// `partition_stress_scenarios` projects out `ScenarioDefinition` from normal entries.
/// Normal entries are `(String, PathBuf)` — no `ScenarioDefinition` in output.
#[test]
fn partition_stress_scenarios_normal_entries_are_name_path_only() {
    let def = ScenarioDefinition {
        breaker: "TestBreaker".to_owned(),
        layout: "TestLayout".to_owned(),
        stress: None,
        ..Default::default()
    };
    let runs = vec![(
        "test_normal".to_owned(),
        PathBuf::from("/tmp/test.scenario.ron"),
        def,
    )];

    let (normal, stress) = partition_stress_scenarios(&runs);
    assert_eq!(normal.len(), 1);
    assert!(stress.is_empty());
    // Normal entries are (String, PathBuf) — the ScenarioDefinition is projected out.
    let (name, path) = &normal[0];
    assert_eq!(name, "test_normal");
    assert_eq!(path, &PathBuf::from("/tmp/test.scenario.ron"));
}

/// `partition_stress_scenarios` projects out `ScenarioDefinition` from stress entries too.
/// Stress entries are `(String, PathBuf, StressConfig)` — definition is replaced with just config.
#[test]
fn partition_stress_scenarios_stress_entries_are_name_path_config() {
    let def = ScenarioDefinition {
        breaker: "TestBreaker".to_owned(),
        layout: "TestLayout".to_owned(),
        stress: Some(StressConfig {
            runs: 10,
            parallelism: 5,
        }),
        ..Default::default()
    };
    let runs = vec![(
        "test_stress".to_owned(),
        PathBuf::from("/tmp/stress.scenario.ron"),
        def,
    )];

    let (normal, stress) = partition_stress_scenarios(&runs);
    assert!(normal.is_empty());
    assert_eq!(stress.len(), 1);
    let (name, path, config) = &stress[0];
    assert_eq!(name, "test_stress");
    assert_eq!(path, &PathBuf::from("/tmp/stress.scenario.ron"));
    assert_eq!(config.runs, 10);
    assert_eq!(config.parallelism, 5);
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
