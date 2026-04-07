//! Tests for `print_summary` and `build_run_list`.

use crate::runner::execution::{build_run_list, print_summary};

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
// build_run_list — returns enriched entries with ScenarioDefinition
// -------------------------------------------------------------------------

/// `build_run_list` returns a triple (name, path, definition) for a single known scenario.
#[test]
fn build_run_list_single_scenario_returns_one_enriched_entry() {
    let runs = build_run_list(Some("aegis_chaos"), false);
    assert_eq!(runs.len(), 1, "single scenario must produce 1 entry");
    let (name, path, definition) = &runs[0];
    assert_eq!(name, "aegis_chaos");
    assert!(
        path.to_string_lossy().ends_with("aegis_chaos.scenario.ron"),
        "path must end with aegis_chaos.scenario.ron, got: {}",
        path.display()
    );
    assert_eq!(
        definition.breaker, "Aegis",
        "definition.breaker must be 'Aegis'"
    );
}

/// `build_run_list` with `--all` returns one enriched entry per discovered scenario.
#[test]
fn build_run_list_all_returns_one_enriched_entry_per_scenario() {
    let runs = build_run_list(None, true);
    assert!(
        runs.len() > 1,
        "--all must discover multiple scenarios, got {}",
        runs.len()
    );
    // Verify no duplicates
    let names: Vec<&str> = runs.iter().map(|(n, ..)| n.as_str()).collect();
    let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
    assert_eq!(
        names.len(),
        unique.len(),
        "run list must not contain duplicate names"
    );
}

/// `build_run_list` with `--all` returns at least 50 entries (known lower bound).
#[test]
fn build_run_list_all_returns_at_least_50_scenarios() {
    let runs = build_run_list(None, true);
    assert!(
        runs.len() >= 50,
        "--all must discover >= 50 scenarios, got {}",
        runs.len()
    );
}

/// Each enriched entry from `build_run_list(None, true)` has non-empty breaker and layout.
#[test]
fn build_run_list_all_entries_have_nonempty_breaker_and_layout() {
    let runs = build_run_list(None, true);
    assert!(!runs.is_empty(), "must have scenarios for this test");
    for (name, _, definition) in &runs {
        assert!(
            !definition.breaker.is_empty(),
            "scenario '{name}' has empty breaker"
        );
        assert!(
            !definition.layout.is_empty(),
            "scenario '{name}' has empty layout"
        );
    }
}

/// `build_run_list` for a nonexistent scenario returns an empty vec.
#[test]
fn build_run_list_nonexistent_scenario_returns_empty() {
    let runs = build_run_list(Some("nonexistent_scenario_xyz"), false);
    assert!(
        runs.is_empty(),
        "nonexistent scenario must produce 0 entries"
    );
}

/// `build_run_list(None, false)` returns empty — neither `-s` nor `--all` was passed.
#[test]
fn build_run_list_neither_scenario_nor_all_returns_empty() {
    let runs = build_run_list(None, false);
    assert!(
        runs.is_empty(),
        "build_run_list(None, false) must return empty vec"
    );
}

/// Non-stress scenario (`aegis_chaos`) has `definition.stress == None`.
#[test]
fn build_run_list_nonstress_scenario_has_no_stress_config() {
    let runs = build_run_list(Some("aegis_chaos"), false);
    assert_eq!(runs.len(), 1);
    let (_, _, definition) = &runs[0];
    assert!(
        definition.stress.is_none(),
        "aegis_chaos should have stress == None, got {:?}",
        definition.stress
    );
}

/// Stress scenario (`split_decision_cascade`) has correct `StressConfig` in its definition.
#[test]
fn build_run_list_stress_scenario_has_correct_stress_config() {
    let runs = build_run_list(Some("split_decision_cascade"), false);
    assert_eq!(runs.len(), 1, "split_decision_cascade must produce 1 entry");
    let (_, _, definition) = &runs[0];
    let config = definition
        .stress
        .as_ref()
        .expect("split_decision_cascade must have stress config");
    assert_eq!(config.runs, 4, "stress config runs must be 4");
    assert_eq!(config.parallelism, 4, "stress config parallelism must be 4");
}

/// Stress scenario (`chain_lightning_arc_lifecycle`) has correct `StressConfig` in its definition.
#[test]
fn build_run_list_chain_lightning_stress_scenario_has_correct_config() {
    let runs = build_run_list(Some("chain_lightning_arc_lifecycle"), false);
    assert_eq!(
        runs.len(),
        1,
        "chain_lightning_arc_lifecycle must produce 1 entry"
    );
    let (_, _, definition) = &runs[0];
    let config = definition
        .stress
        .as_ref()
        .expect("chain_lightning_arc_lifecycle must have stress config");
    assert_eq!(config.runs, 4, "stress config runs must be 4");
    assert_eq!(config.parallelism, 4, "stress config parallelism must be 4");
}
