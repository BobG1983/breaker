use super::super::system::*;
use crate::types::InvariantKind;

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
