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
    pub unused_layouts: Vec<String>,
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
    let missing_self_tests: Vec<InvariantKind> = InvariantKind::ALL
        .iter()
        .copied()
        .filter(|variant| {
            !scenarios.iter().any(|(name, def)| {
                self_test_names.contains(name)
                    && def
                        .allowed_failures
                        .as_ref()
                        .is_some_and(|violations| violations.contains(variant))
            })
        })
        .collect();

    let unused_layouts: Vec<String> = layout_names
        .iter()
        .filter(|layout| {
            let normalized = normalize_layout_name(layout);
            !scenarios
                .iter()
                .any(|(_, def)| normalize_layout_name(&def.layout) == normalized)
        })
        .cloned()
        .collect();

    CoverageReport {
        missing_self_tests,
        unused_layouts,
    }
}

/// Normalizes a layout name for comparison: lowercase and strip underscores.
///
/// Layout RON files use `snake_case` (`boss_arena`) while scenarios use `PascalCase` (`BossArena`).
fn normalize_layout_name(name: &str) -> String {
    name.to_lowercase().replace('_', "")
}

/// Prints the coverage report to stdout.
/// Returns `true` if there are any gaps (missing self-tests or unused layouts).
#[must_use]
pub fn print_coverage_report(report: &CoverageReport) -> bool {
    println!("Coverage Report");
    println!("===============");

    let has_gaps = !report.missing_self_tests.is_empty() || !report.unused_layouts.is_empty();

    if !report.missing_self_tests.is_empty() {
        println!("\nMissing Self-Tests:");
        for variant in &report.missing_self_tests {
            println!("  - {variant:?}");
        }
    }

    if !report.unused_layouts.is_empty() {
        println!("\nUnused Layouts:");
        for layout in &report.unused_layouts {
            println!("  - {layout}");
        }
    }

    has_gaps
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
            unused_layouts: vec![],
        };

        let has_gaps = print_coverage_report(&report);

        assert!(
            has_gaps,
            "print_coverage_report must return true when missing_self_tests is non-empty"
        );
    }

    #[test]
    fn print_coverage_report_returns_true_when_unused_layouts() {
        let report = CoverageReport {
            missing_self_tests: vec![],
            unused_layouts: vec!["Fortress".to_owned()],
        };

        let has_gaps = print_coverage_report(&report);

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
            unused_layouts: vec![],
        };

        let has_gaps = print_coverage_report(&report);

        assert!(
            !has_gaps,
            "print_coverage_report must return false when both lists are empty"
        );
    }
}
