//! Coverage parity checking for invariant self-tests and layout usage.
//!
//! Pure analysis module — no Bevy, no ECS. Checks that every
//! [`InvariantKind`] variant has at least one self-test scenario and that
//! every layout RON file is referenced by at least one scenario.

use crate::types::{InvariantKind, ScenarioDefinition};

/// Result of a coverage parity check.
#[derive(Debug)]
pub struct CoverageReport {
    /// `InvariantKind` variants that have no self-test scenario.
    pub missing_self_tests: Vec<InvariantKind>,
    /// Layout names (from `.node.ron` files) that no scenario references.
    pub unused_layouts:     Vec<String>,
    /// `InvariantKind` variants that have at least one self-test scenario.
    pub covered_self_tests: Vec<InvariantKind>,
    /// Layout names that are referenced by at least one scenario, with counts.
    pub used_layouts:       Vec<(String, usize)>,
}

/// Checks self-test parity and layout coverage from loaded scenario definitions.
///
/// `scenarios` is a list of (name, definition) pairs for all discovered scenarios.
/// `self_test_names` is the set of scenario names that live in `self_tests/` directory.
/// `layout_names` is the list of layout names (stems of `.node.ron` files).
#[must_use]
pub fn check_coverage(
    scenarios: &[(String, ScenarioDefinition)],
    self_test_names: &[String],
    layout_names: &[String],
) -> CoverageReport {
    let is_covered = |variant: &InvariantKind| {
        scenarios.iter().any(|(name, def)| {
            self_test_names.contains(name)
                && def
                    .allowed_failures
                    .as_ref()
                    .is_some_and(|violations| violations.contains(variant))
        })
    };

    let mut missing_self_tests = Vec::new();
    let mut covered_self_tests = Vec::new();

    for variant in InvariantKind::ALL {
        if is_covered(variant) {
            covered_self_tests.push(*variant);
        } else {
            missing_self_tests.push(*variant);
        }
    }

    let mut unused_layouts = Vec::new();
    let mut used_layouts = Vec::new();

    for layout in layout_names {
        let normalized = normalize_layout_name(layout);
        let count = scenarios
            .iter()
            .filter(|(_, def)| normalize_layout_name(&def.layout) == normalized)
            .count();
        if count > 0 {
            used_layouts.push((layout.clone(), count));
        } else {
            unused_layouts.push(layout.clone());
        }
    }

    CoverageReport {
        missing_self_tests,
        unused_layouts,
        covered_self_tests,
        used_layouts,
    }
}

/// Normalizes a layout name for comparison: lowercase and strip underscores.
///
/// Layout RON files use `snake_case` (`boss_arena`) while scenarios use `PascalCase` (`BossArena`).
fn normalize_layout_name(name: &str) -> String {
    name.to_lowercase().replace('_', "")
}

/// Formats the coverage report as a plain-text string (no ANSI escapes).
///
/// Only reports gaps — missing self-tests and unused layouts. When `verbose`
/// is true and coverage is complete, returns a success summary instead of an
/// empty string. When `verbose` is false, returns empty on success (suitable
/// for `--all` runs where silence means success).
#[must_use]
pub fn format_coverage_report(report: &CoverageReport, verbose: bool) -> String {
    use std::fmt::Write as _;

    let has_missing_tests = !report.missing_self_tests.is_empty();
    let has_unused_layouts = !report.unused_layouts.is_empty();

    if !has_missing_tests && !has_unused_layouts {
        if verbose {
            let covered = report.covered_self_tests.len();
            let total_invariants = covered + report.missing_self_tests.len();
            let used = report.used_layouts.len();
            let total_layouts = used + report.unused_layouts.len();
            return format!(
                "Coverage: {covered}/{total_invariants} invariants have self-tests, \
                 {used}/{total_layouts} layouts used by scenarios\n"
            );
        }
        return String::new();
    }

    let mut out = String::new();
    out.push_str("Coverage Gaps\n=============\n");

    if has_missing_tests {
        let _ = writeln!(
            out,
            "Missing self-tests ({}):",
            report.missing_self_tests.len()
        );
        for variant in &report.missing_self_tests {
            let _ = writeln!(out, "  [ ] {variant:?}");
        }
    }

    if has_unused_layouts {
        if has_missing_tests {
            out.push('\n');
        }
        let _ = writeln!(out, "Unused layouts ({}):", report.unused_layouts.len());
        for name in &report.unused_layouts {
            let _ = writeln!(out, "  [ ] {name}");
        }
    }

    out
}

/// Prints coverage report to stdout. When `verbose` is true, prints a success
/// summary even when coverage is complete. Returns `true` if there are any gaps.
#[must_use]
pub fn print_coverage_report(report: &CoverageReport, verbose: bool) -> bool {
    let formatted = format_coverage_report(report, verbose);
    if !formatted.is_empty() {
        print!("{formatted}");
    }
    !report.missing_self_tests.is_empty() || !report.unused_layouts.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChaosParams, InputStrategy};

    // -----------------------------------------------------------------------
    // Helper — minimal ScenarioDefinition builder
    // -----------------------------------------------------------------------

    /// Creates a minimal `ScenarioDefinition` with sensible defaults.
    /// Override fields after construction for test-specific values.
    fn minimal_scenario(
        layout: &str,
        allowed_failures: Option<Vec<InvariantKind>>,
    ) -> ScenarioDefinition {
        ScenarioDefinition {
            breaker: "aegis".to_owned(),
            layout: layout.to_owned(),
            input: InputStrategy::Chaos(ChaosParams { action_prob: 0.1 }),
            max_frames: 100,
            disallowed_failures: vec![],
            allowed_failures,
            ..Default::default()
        }
    }

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
    // Behavior 3: Unused layout detection — finds unreferenced layout
    // -----------------------------------------------------------------------

    #[test]
    fn unused_layout_detection_finds_unreferenced_layouts() {
        let scenarios = vec![(
            "corridor_test".to_owned(),
            minimal_scenario("Corridor", None),
        )];
        let self_test_names: Vec<String> = vec![];
        let layout_names = vec![
            "Corridor".to_owned(),
            "Fortress".to_owned(),
            "Scatter".to_owned(),
        ];

        let report = check_coverage(&scenarios, &self_test_names, &layout_names);

        assert_eq!(
            report.unused_layouts.len(),
            2,
            "expected 2 unused layouts, got: {:?}",
            report.unused_layouts
        );
        assert!(
            report.unused_layouts.contains(&"Fortress".to_owned()),
            "Fortress should be in unused_layouts"
        );
        assert!(
            report.unused_layouts.contains(&"Scatter".to_owned()),
            "Scatter should be in unused_layouts"
        );
        assert!(
            !report.unused_layouts.contains(&"Corridor".to_owned()),
            "Corridor is used and must not be in unused_layouts"
        );
    }

    #[test]
    fn layout_name_comparison_is_case_insensitive() {
        // Scenario uses lowercase "corridor", layout file is "Corridor".
        let scenarios = vec![(
            "corridor_test".to_owned(),
            minimal_scenario("corridor", None),
        )];
        let self_test_names: Vec<String> = vec![];
        let layout_names = vec!["Corridor".to_owned()];

        let report = check_coverage(&scenarios, &self_test_names, &layout_names);

        assert!(
            report.unused_layouts.is_empty(),
            "case-insensitive match should mark Corridor as used, got unused: {:?}",
            report.unused_layouts
        );
    }

    #[test]
    fn layout_name_comparison_normalizes_underscores_to_pascal_case() {
        // Layout file is "boss_arena" (snake_case), scenario uses "BossArena" (PascalCase).
        let scenarios = vec![("boss_test".to_owned(), minimal_scenario("BossArena", None))];
        let self_test_names: Vec<String> = vec![];
        let layout_names = vec!["boss_arena".to_owned()];

        let report = check_coverage(&scenarios, &self_test_names, &layout_names);

        assert!(
            report.unused_layouts.is_empty(),
            "boss_arena should match BossArena after normalization, got unused: {:?}",
            report.unused_layouts
        );
    }

    // -----------------------------------------------------------------------
    // Behavior 4: All layouts used — empty unused list
    // -----------------------------------------------------------------------

    #[test]
    fn all_layouts_used_produces_empty_unused_list() {
        let scenarios = vec![(
            "corridor_test".to_owned(),
            minimal_scenario("Corridor", None),
        )];
        let self_test_names: Vec<String> = vec![];
        let layout_names = vec!["Corridor".to_owned()];

        let report = check_coverage(&scenarios, &self_test_names, &layout_names);

        assert!(
            report.unused_layouts.is_empty(),
            "all layouts are used — unused_layouts should be empty, got: {:?}",
            report.unused_layouts
        );
    }

    // -----------------------------------------------------------------------
    // Behavior 5: print_coverage_report returns true when gaps exist
    // -----------------------------------------------------------------------

    #[test]
    fn print_coverage_report_returns_true_when_missing_self_tests() {
        let report = CoverageReport {
            missing_self_tests: vec![InvariantKind::BoltInBounds],
            unused_layouts:     vec![],
            covered_self_tests: vec![],
            used_layouts:       vec![],
        };

        let has_gaps = print_coverage_report(&report, false);

        assert!(
            has_gaps,
            "print_coverage_report must return true when missing_self_tests is non-empty"
        );
    }

    #[test]
    fn print_coverage_report_returns_true_when_unused_layouts() {
        let report = CoverageReport {
            missing_self_tests: vec![],
            unused_layouts:     vec!["Fortress".to_owned()],
            covered_self_tests: vec![],
            used_layouts:       vec![],
        };

        let has_gaps = print_coverage_report(&report, false);

        assert!(
            has_gaps,
            "print_coverage_report must return true when unused_layouts is non-empty"
        );
    }

    // -----------------------------------------------------------------------
    // Behavior 6: print_coverage_report returns false when no gaps
    // -----------------------------------------------------------------------

    #[test]
    fn print_coverage_report_returns_false_when_no_gaps() {
        let report = CoverageReport {
            missing_self_tests: vec![],
            unused_layouts:     vec![],
            covered_self_tests: vec![],
            used_layouts:       vec![],
        };

        let has_gaps = print_coverage_report(&report, false);

        assert!(
            !has_gaps,
            "print_coverage_report must return false when both lists are empty"
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

    // -----------------------------------------------------------------------
    // Behavior 5 (spec): used_layouts populated with scenario counts
    // -----------------------------------------------------------------------

    #[test]
    fn used_layouts_populated_with_scenario_counts() {
        let scenarios = vec![
            (
                "corridor_test".to_owned(),
                minimal_scenario("Corridor", None),
            ),
            (
                "corridor_chaos".to_owned(),
                minimal_scenario("Corridor", None),
            ),
            (
                "fortress_test".to_owned(),
                minimal_scenario("Fortress", None),
            ),
        ];
        let self_test_names: Vec<String> = vec![];
        let layout_names = vec![
            "Corridor".to_owned(),
            "Fortress".to_owned(),
            "Scatter".to_owned(),
        ];

        let report = check_coverage(&scenarios, &self_test_names, &layout_names);

        assert!(
            report.used_layouts.contains(&("Corridor".to_owned(), 2)),
            "used_layouts should contain (Corridor, 2), got: {:?}",
            report.used_layouts
        );
        assert!(
            report.used_layouts.contains(&("Fortress".to_owned(), 1)),
            "used_layouts should contain (Fortress, 1), got: {:?}",
            report.used_layouts
        );
        assert!(
            !report
                .used_layouts
                .iter()
                .any(|(name, _)| name == "Scatter"),
            "Scatter should not appear in used_layouts"
        );
        assert_eq!(
            report.unused_layouts,
            vec!["Scatter".to_owned()],
            "unused_layouts should contain only Scatter"
        );
    }

    // -----------------------------------------------------------------------
    // Behavior 6 (spec): layout name normalization applies to used_layouts
    // -----------------------------------------------------------------------

    #[test]
    fn used_layouts_counting_applies_normalization() {
        let scenarios = vec![
            (
                "corridor_lower".to_owned(),
                minimal_scenario("corridor", None),
            ),
            (
                "corridor_pascal".to_owned(),
                minimal_scenario("Corridor", None),
            ),
        ];
        let self_test_names: Vec<String> = vec![];
        let layout_names = vec!["Corridor".to_owned()];

        let report = check_coverage(&scenarios, &self_test_names, &layout_names);

        assert_eq!(
            report.used_layouts,
            vec![("Corridor".to_owned(), 2)],
            "both scenarios should match via normalization; name should be the layout_names entry"
        );
    }

    #[test]
    fn used_layouts_normalization_strips_underscores() {
        // Layout file "boss_arena" (snake_case), scenario uses "BossArena" (PascalCase).
        let scenarios = vec![("boss_test".to_owned(), minimal_scenario("BossArena", None))];
        let self_test_names: Vec<String> = vec![];
        let layout_names = vec!["boss_arena".to_owned()];

        let report = check_coverage(&scenarios, &self_test_names, &layout_names);

        assert_eq!(
            report.used_layouts,
            vec![("boss_arena".to_owned(), 1)],
            "boss_arena should match BossArena after normalization"
        );
    }

    // -----------------------------------------------------------------------
    // Behavior 7 (spec): all layouts used produces full used list
    // -----------------------------------------------------------------------

    #[test]
    fn all_layouts_used_produces_full_used_list_and_empty_unused() {
        let scenarios = vec![(
            "corridor_test".to_owned(),
            minimal_scenario("Corridor", None),
        )];
        let self_test_names: Vec<String> = vec![];
        let layout_names = vec!["Corridor".to_owned()];

        let report = check_coverage(&scenarios, &self_test_names, &layout_names);

        assert_eq!(
            report.used_layouts,
            vec![("Corridor".to_owned(), 1)],
            "used_layouts should contain Corridor with count 1"
        );
        assert!(
            report.unused_layouts.is_empty(),
            "unused_layouts should be empty when all layouts are used"
        );
    }

    #[test]
    fn no_layouts_produces_empty_used_and_unused() {
        let scenarios = vec![(
            "corridor_test".to_owned(),
            minimal_scenario("Corridor", None),
        )];
        let self_test_names: Vec<String> = vec![];
        let layout_names: Vec<String> = vec![];

        let report = check_coverage(&scenarios, &self_test_names, &layout_names);

        assert!(
            report.used_layouts.is_empty(),
            "used_layouts should be empty when no layout_names exist"
        );
        assert!(
            report.unused_layouts.is_empty(),
            "unused_layouts should be empty when no layout_names exist"
        );
    }

    // -----------------------------------------------------------------------
    // Behavior 8 (spec): no scenarios means all layouts unused, none used
    // -----------------------------------------------------------------------

    #[test]
    fn no_scenarios_means_all_layouts_unused_and_none_used() {
        let scenarios: Vec<(String, ScenarioDefinition)> = vec![];
        let self_test_names: Vec<String> = vec![];
        let layout_names = vec!["Corridor".to_owned(), "Fortress".to_owned()];

        let report = check_coverage(&scenarios, &self_test_names, &layout_names);

        assert!(
            report.used_layouts.is_empty(),
            "used_layouts should be empty when no scenarios exist"
        );
        assert_eq!(
            report.unused_layouts,
            vec!["Corridor".to_owned(), "Fortress".to_owned()],
            "all layouts should be unused when no scenarios exist"
        );
    }

    // -----------------------------------------------------------------------
    // Behavior 9 (spec): used_layouts preserves layout_names ordering
    // -----------------------------------------------------------------------

    #[test]
    fn used_layouts_preserves_layout_names_ordering() {
        let scenarios = vec![
            (
                "corridor_test".to_owned(),
                minimal_scenario("Corridor", None),
            ),
            (
                "fortress_test".to_owned(),
                minimal_scenario("Fortress", None),
            ),
        ];
        let self_test_names: Vec<String> = vec![];
        let layout_names = vec![
            "Fortress".to_owned(),
            "Corridor".to_owned(),
            "Scatter".to_owned(),
        ];

        let report = check_coverage(&scenarios, &self_test_names, &layout_names);

        assert_eq!(
            report.used_layouts,
            vec![("Fortress".to_owned(), 1), ("Corridor".to_owned(), 1),],
            "used_layouts should preserve layout_names ordering, not scenario discovery order"
        );
        assert_eq!(
            report.unused_layouts,
            vec!["Scatter".to_owned()],
            "unused_layouts should contain Scatter"
        );
    }

    // -----------------------------------------------------------------------
    // format_coverage_report — gaps-only output
    // -----------------------------------------------------------------------

    #[test]
    fn format_report_returns_empty_when_no_gaps_and_not_verbose() {
        let report = CoverageReport {
            covered_self_tests: InvariantKind::ALL.to_vec(),
            missing_self_tests: vec![],
            used_layouts:       vec![("Corridor".to_owned(), 2)],
            unused_layouts:     vec![],
        };
        assert!(format_coverage_report(&report, false).is_empty());
    }

    #[test]
    fn format_report_verbose_shows_summary_when_no_gaps() {
        let report = CoverageReport {
            covered_self_tests: vec![InvariantKind::BoltInBounds, InvariantKind::BreakerInBounds],
            missing_self_tests: vec![],
            used_layouts:       vec![("Corridor".to_owned(), 2), ("Fortress".to_owned(), 1)],
            unused_layouts:     vec![],
        };
        let output = format_coverage_report(&report, true);
        assert!(output.contains("2/2 invariants have self-tests"));
        assert!(output.contains("2/2 layouts used by scenarios"));
    }

    #[test]
    fn format_report_lists_only_gaps_when_present() {
        let report = CoverageReport {
            covered_self_tests: vec![InvariantKind::BoltInBounds],
            missing_self_tests: vec![InvariantKind::NoNaN],
            used_layouts:       vec![("Corridor".to_owned(), 3)],
            unused_layouts:     vec!["Fortress".to_owned()],
        };
        let output = format_coverage_report(&report, false);
        assert!(output.contains("[ ] NoNaN"));
        assert!(output.contains("[ ] Fortress"));
        assert!(!output.contains("BoltInBounds"));
        assert!(!output.contains("Corridor"));
    }

    #[test]
    fn print_coverage_report_returns_correct_bool() {
        let gaps = CoverageReport {
            missing_self_tests: vec![InvariantKind::NoNaN],
            unused_layouts:     vec![],
            covered_self_tests: vec![],
            used_layouts:       vec![],
        };
        assert!(print_coverage_report(&gaps, false));

        let clean = CoverageReport {
            missing_self_tests: vec![],
            unused_layouts:     vec![],
            covered_self_tests: vec![InvariantKind::BoltInBounds],
            used_layouts:       vec![],
        };
        assert!(!print_coverage_report(&clean, false));
    }
}
