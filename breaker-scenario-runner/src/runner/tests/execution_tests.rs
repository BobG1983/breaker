//! Tests for `print_summary` and `build_run_list`.

use super::super::execution::{build_run_list, print_summary};

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
