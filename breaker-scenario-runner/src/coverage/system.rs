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
