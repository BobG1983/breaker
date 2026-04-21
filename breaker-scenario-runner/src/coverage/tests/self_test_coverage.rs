use super::{super::system::*, helpers::minimal_scenario};
use crate::types::{InvariantKind, ScenarioDefinition};

// -----------------------------------------------------------------------
// Behavior 1: Missing self-test detection — finds uncovered invariant
// -----------------------------------------------------------------------

#[test]
fn missing_self_test_detection_finds_uncovered_invariants() {
    // One self-test scenario covers only BoltInBounds.
    let scenarios = vec![(
        "bolt_oob_detection".to_owned(),
        minimal_scenario("corridor", Some(vec![InvariantKind::BoltInBounds])),
    )];
    let self_test_names = vec!["bolt_oob_detection".to_owned()];
    let layout_names: Vec<String> = vec![];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    // Every variant in ALL except BoltInBounds should be missing.
    let expected_missing: Vec<InvariantKind> = InvariantKind::ALL
        .iter()
        .copied()
        .filter(|v| *v != InvariantKind::BoltInBounds)
        .collect();

    assert_eq!(
        report.missing_self_tests.len(),
        expected_missing.len(),
        "expected {} missing self-tests, got {}: {:?}",
        expected_missing.len(),
        report.missing_self_tests.len(),
        report.missing_self_tests
    );

    for variant in &expected_missing {
        assert!(
            report.missing_self_tests.contains(variant),
            "{variant:?} should be in missing_self_tests"
        );
    }

    // BoltInBounds should NOT be in the missing list.
    assert!(
        !report
            .missing_self_tests
            .contains(&InvariantKind::BoltInBounds),
        "BoltInBounds is covered by a self-test and must not be in missing_self_tests"
    );
}

#[test]
fn non_self_test_scenario_with_allowed_failures_does_not_count_as_coverage() {
    // A scenario NOT in self_test_names has allowed_failures for BoltInBounds,
    // but it should NOT count as coverage.
    let scenarios = vec![(
        "some_mechanic_test".to_owned(),
        minimal_scenario("corridor", Some(vec![InvariantKind::BoltInBounds])),
    )];
    // This scenario name is NOT in self_test_names.
    let self_test_names: Vec<String> = vec![];
    let layout_names: Vec<String> = vec![];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    // All variants should be missing, including BoltInBounds.
    assert_eq!(
        report.missing_self_tests.len(),
        InvariantKind::ALL.len(),
        "all invariant variants should be missing when no self-test scenarios exist"
    );
    assert!(
        report
            .missing_self_tests
            .contains(&InvariantKind::BoltInBounds),
        "BoltInBounds from a non-self-test scenario must still appear in missing_self_tests"
    );
}

// -----------------------------------------------------------------------
// Behavior 2: All invariants covered — empty missing list
// -----------------------------------------------------------------------

#[test]
fn all_invariants_covered_produces_empty_missing_list() {
    // Create one self-test scenario per invariant variant.
    let mut scenarios = Vec::new();
    let mut self_test_names = Vec::new();

    for (i, variant) in InvariantKind::ALL.iter().enumerate() {
        let name = format!("self_test_{i}");
        scenarios.push((
            name.clone(),
            minimal_scenario("corridor", Some(vec![*variant])),
        ));
        self_test_names.push(name);
    }

    let layout_names = vec!["corridor".to_owned()];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert!(
        report.missing_self_tests.is_empty(),
        "all invariants are covered — missing_self_tests should be empty, got: {:?}",
        report.missing_self_tests
    );
}

// -----------------------------------------------------------------------
// Behavior 1 (spec): covered_self_tests is complement of missing
// -----------------------------------------------------------------------

#[test]
fn covered_self_tests_contains_invariants_with_self_test_scenarios() {
    let scenarios = vec![(
        "bolt_oob_detection".to_owned(),
        minimal_scenario("corridor", Some(vec![InvariantKind::BoltInBounds])),
    )];
    let self_test_names = vec!["bolt_oob_detection".to_owned()];
    let layout_names: Vec<String> = vec![];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert_eq!(
        report.covered_self_tests,
        vec![InvariantKind::BoltInBounds],
        "covered_self_tests should contain exactly BoltInBounds"
    );
    assert_eq!(
        report.missing_self_tests.len(),
        InvariantKind::ALL.len() - 1,
        "missing_self_tests should contain all variants except BoltInBounds"
    );
    // Union of covered and missing equals ALL (no duplicates, no omissions).
    let mut union: Vec<InvariantKind> = report.covered_self_tests.clone();
    union.extend_from_slice(&report.missing_self_tests);
    for variant in InvariantKind::ALL {
        assert!(
            union.contains(variant),
            "{variant:?} missing from union of covered + missing"
        );
    }
    assert_eq!(
        union.len(),
        InvariantKind::ALL.len(),
        "union of covered + missing should have no duplicates"
    );
}

#[test]
fn non_self_test_scenario_does_not_contribute_to_covered_self_tests() {
    // A scenario NOT in self_test_names has allowed_failures for BoltInBounds,
    // but it should NOT appear in covered_self_tests.
    let scenarios = vec![(
        "some_mechanic_test".to_owned(),
        minimal_scenario("corridor", Some(vec![InvariantKind::BoltInBounds])),
    )];
    let self_test_names: Vec<String> = vec![];
    let layout_names: Vec<String> = vec![];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert!(
        !report
            .covered_self_tests
            .contains(&InvariantKind::BoltInBounds),
        "BoltInBounds from a non-self-test scenario must not appear in covered_self_tests"
    );
    assert!(
        report.covered_self_tests.is_empty(),
        "covered_self_tests should be empty when no self-test scenarios exist"
    );
}

// -----------------------------------------------------------------------
// Behavior 2 (spec): all invariants covered produces full covered list
// -----------------------------------------------------------------------

#[test]
fn all_invariants_covered_produces_full_covered_list_and_empty_missing() {
    let mut scenarios = Vec::new();
    let mut self_test_names = Vec::new();

    for (i, variant) in InvariantKind::ALL.iter().enumerate() {
        let name = format!("self_test_{i}");
        scenarios.push((
            name.clone(),
            minimal_scenario("corridor", Some(vec![*variant])),
        ));
        self_test_names.push(name);
    }

    let layout_names = vec!["corridor".to_owned()];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert_eq!(
        report.covered_self_tests.len(),
        InvariantKind::ALL.len(),
        "covered_self_tests should contain all {} variants",
        InvariantKind::ALL.len()
    );
    assert!(
        report.missing_self_tests.is_empty(),
        "missing_self_tests should be empty when all invariants are covered"
    );
    assert_eq!(
        report.covered_self_tests,
        InvariantKind::ALL,
        "covered_self_tests should match InvariantKind::ALL exactly (same order, same length)"
    );
}

#[test]
fn no_scenarios_produces_empty_covered_and_full_missing() {
    let scenarios: Vec<(String, ScenarioDefinition)> = vec![];
    let self_test_names: Vec<String> = vec![];
    let layout_names: Vec<String> = vec![];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert!(
        report.covered_self_tests.is_empty(),
        "covered_self_tests should be empty when no scenarios exist"
    );
    assert_eq!(
        report.missing_self_tests.len(),
        InvariantKind::ALL.len(),
        "missing_self_tests should contain all {} variants when no scenarios exist",
        InvariantKind::ALL.len()
    );
}

// -----------------------------------------------------------------------
// Behavior 3 (spec): duplicate coverage produces single entry
// -----------------------------------------------------------------------

#[test]
fn duplicate_self_test_coverage_produces_single_entry_in_covered() {
    let scenarios = vec![
        (
            "bolt_oob_a".to_owned(),
            minimal_scenario("corridor", Some(vec![InvariantKind::BoltInBounds])),
        ),
        (
            "bolt_oob_b".to_owned(),
            minimal_scenario("corridor", Some(vec![InvariantKind::BoltInBounds])),
        ),
    ];
    let self_test_names = vec!["bolt_oob_a".to_owned(), "bolt_oob_b".to_owned()];
    let layout_names: Vec<String> = vec![];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    let bolt_count = report
        .covered_self_tests
        .iter()
        .filter(|v| **v == InvariantKind::BoltInBounds)
        .count();
    assert_eq!(
        bolt_count, 1,
        "BoltInBounds should appear exactly once in covered_self_tests, got {bolt_count}"
    );
    assert!(
        !report
            .missing_self_tests
            .contains(&InvariantKind::BoltInBounds),
        "BoltInBounds must not appear in missing_self_tests when covered"
    );
}

#[test]
fn self_test_covering_multiple_invariants_adds_each_once() {
    let scenarios = vec![(
        "multi_cover".to_owned(),
        minimal_scenario(
            "corridor",
            Some(vec![
                InvariantKind::BoltInBounds,
                InvariantKind::BreakerInBounds,
            ]),
        ),
    )];
    let self_test_names = vec!["multi_cover".to_owned()];
    let layout_names: Vec<String> = vec![];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert!(
        report
            .covered_self_tests
            .contains(&InvariantKind::BoltInBounds),
        "BoltInBounds should be in covered_self_tests"
    );
    assert!(
        report
            .covered_self_tests
            .contains(&InvariantKind::BreakerInBounds),
        "BreakerInBounds should be in covered_self_tests"
    );
    // Each appears exactly once.
    let bolt_count = report
        .covered_self_tests
        .iter()
        .filter(|v| **v == InvariantKind::BoltInBounds)
        .count();
    let breaker_count = report
        .covered_self_tests
        .iter()
        .filter(|v| **v == InvariantKind::BreakerInBounds)
        .count();
    assert_eq!(bolt_count, 1, "BoltInBounds should appear exactly once");
    assert_eq!(
        breaker_count, 1,
        "BreakerInBounds should appear exactly once"
    );
}

// -----------------------------------------------------------------------
// Behavior 4 (spec): covered_self_tests preserves ALL ordering
// -----------------------------------------------------------------------

#[test]
fn covered_self_tests_preserves_invariant_kind_all_ordering() {
    // Scenarios discover BreakerInBounds, NoNaN, BoltInBounds (not ALL order).
    let scenarios = vec![
        (
            "breaker_test".to_owned(),
            minimal_scenario("corridor", Some(vec![InvariantKind::BreakerInBounds])),
        ),
        (
            "nan_test".to_owned(),
            minimal_scenario("corridor", Some(vec![InvariantKind::NoNaN])),
        ),
        (
            "bolt_test".to_owned(),
            minimal_scenario("corridor", Some(vec![InvariantKind::BoltInBounds])),
        ),
    ];
    let self_test_names = vec![
        "breaker_test".to_owned(),
        "nan_test".to_owned(),
        "bolt_test".to_owned(),
    ];
    let layout_names: Vec<String> = vec![];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    // InvariantKind::ALL has BoltInBounds at 0, BreakerInBounds at 3, NoNaN at 5.
    assert_eq!(
        report.covered_self_tests,
        vec![
            InvariantKind::BoltInBounds,
            InvariantKind::BreakerInBounds,
            InvariantKind::NoNaN,
        ],
        "covered_self_tests should be in InvariantKind::ALL order, not discovery order"
    );
}
