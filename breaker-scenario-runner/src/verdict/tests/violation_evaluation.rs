//! Tests for violation matching: unexpected violations, expected violations, duplicates.

use super::{super::evaluation::*, helpers::*};
use crate::types::InvariantKind;

// -------------------------------------------------------------------------
// Behavior 4: Unexpected violation with no expected list evaluates to Fail
// -------------------------------------------------------------------------

#[test]
fn unexpected_violation_with_no_expected_list_evaluates_to_fail() {
    let mut verdict = ScenarioVerdict::default();
    let violations = vec![make_violation(InvariantKind::BoltInBounds)];
    let stats = make_healthy_stats();
    let definition = make_chaos_definition();

    verdict.evaluate(&violations, &[], &stats, &definition);

    assert!(
        !verdict.passed(),
        "unexpected violation must cause Fail when no allowed_failures list"
    );
    let has_reason = verdict
        .reasons
        .iter()
        .any(|r| r.contains("bolt position outside playfield bounds"));
    assert!(
        has_reason,
        "reasons must contain BoltInBounds fail_reason(), got: {:?}",
        verdict.reasons
    );
}

// -------------------------------------------------------------------------
// Behavior 5: Expected violations match exactly evaluates to Pass
// -------------------------------------------------------------------------

#[test]
fn allowed_failures_match_exactly_evaluates_to_pass() {
    let mut verdict = ScenarioVerdict::default();
    let violations = vec![make_violation(InvariantKind::BoltInBounds)];
    let stats = make_healthy_stats();
    let mut definition = make_chaos_definition();
    definition.allowed_failures = Some(vec![InvariantKind::BoltInBounds]);

    verdict.evaluate(&violations, &[], &stats, &definition);

    assert!(
        verdict.passed(),
        "exactly-matched expected violations must evaluate to Pass"
    );
    assert!(
        verdict.reasons.is_empty(),
        "exactly-matched expected violations must produce no reasons, got: {:?}",
        verdict.reasons
    );
}

// -------------------------------------------------------------------------
// Behavior 6: Expected violation not fired evaluates to Fail
// -------------------------------------------------------------------------

#[test]
fn expected_violation_not_fired_evaluates_to_fail() {
    let mut verdict = ScenarioVerdict::default();
    let stats = make_healthy_stats();
    let mut definition = make_chaos_definition();
    definition.allowed_failures = Some(vec![InvariantKind::BoltInBounds]);

    verdict.evaluate(&[], &[], &stats, &definition);

    assert!(
        !verdict.passed(),
        "expected violation that never fires must cause Fail"
    );
    let has_reason = verdict
        .reasons
        .iter()
        .any(|r| r.contains("expected violation BoltInBounds never fired"));
    assert!(
        has_reason,
        "reasons must contain 'expected violation BoltInBounds never fired', got: {:?}",
        verdict.reasons
    );
}

// -------------------------------------------------------------------------
// Behavior 7: Unexpected violation not in expected list evaluates to Fail
// -------------------------------------------------------------------------

#[test]
fn unexpected_violation_not_in_expected_list_evaluates_to_fail() {
    let mut verdict = ScenarioVerdict::default();
    let violations = vec![make_violation(InvariantKind::NoNaN)];
    let stats = make_healthy_stats();
    let mut definition = make_chaos_definition();
    definition.allowed_failures = Some(vec![InvariantKind::BoltInBounds]);

    verdict.evaluate(&violations, &[], &stats, &definition);

    assert!(
        !verdict.passed(),
        "unexpected NoNaN violation when only BoltInBounds expected must cause Fail"
    );
    let has_reason = verdict
        .reasons
        .iter()
        .any(|r| r.contains("NaN detected in transform or velocity"));
    assert!(
        has_reason,
        "reasons must contain NoNaN fail_reason(), got: {:?}",
        verdict.reasons
    );
}

// -------------------------------------------------------------------------
// Behavior 19: Duplicate violations produce a single reason
// -------------------------------------------------------------------------

#[test]
fn duplicate_violations_produce_single_reason() {
    let mut verdict = ScenarioVerdict::default();
    let violations = vec![
        make_violation(InvariantKind::BoltInBounds),
        make_violation(InvariantKind::BoltInBounds),
        make_violation(InvariantKind::BoltInBounds),
    ];
    let stats = make_healthy_stats();
    let definition = make_chaos_definition();

    verdict.evaluate(&violations, &[], &stats, &definition);

    let bolt_reasons: Vec<_> = verdict
        .reasons
        .iter()
        .filter(|r| r.contains("bolt position outside playfield bounds"))
        .collect();
    assert_eq!(
        bolt_reasons.len(),
        1,
        "3 identical BoltInBounds violations must produce exactly 1 reason, got {}",
        bolt_reasons.len()
    );
}

// -------------------------------------------------------------------------
// Behavior 20: Multiple distinct violations produce multiple reasons
// -------------------------------------------------------------------------

#[test]
fn multiple_distinct_violations_produce_multiple_reasons() {
    let mut verdict = ScenarioVerdict::default();
    let violations = vec![
        make_violation(InvariantKind::BoltInBounds),
        make_violation(InvariantKind::NoNaN),
    ];
    let stats = make_healthy_stats();
    let definition = make_chaos_definition();

    verdict.evaluate(&violations, &[], &stats, &definition);

    assert!(
        verdict
            .reasons
            .iter()
            .any(|r| r.contains("bolt position outside playfield bounds")),
        "must contain BoltInBounds reason"
    );
    assert!(
        verdict.reasons.iter().any(|r| r.contains("NaN detected")),
        "must contain NoNaN reason"
    );
}

// -------------------------------------------------------------------------
// Behavior 21: Duplicate unexpected violations with expected list produce single reason
// -------------------------------------------------------------------------

#[test]
fn duplicate_unexpected_with_expected_list_produce_single_reason() {
    let mut verdict = ScenarioVerdict::default();
    let violations = vec![
        make_violation(InvariantKind::BoltInBounds),
        make_violation(InvariantKind::NoNaN),
        make_violation(InvariantKind::NoNaN),
        make_violation(InvariantKind::NoNaN),
    ];
    let stats = make_healthy_stats();
    let mut definition = make_chaos_definition();
    definition.allowed_failures = Some(vec![InvariantKind::BoltInBounds]);

    verdict.evaluate(&violations, &[], &stats, &definition);

    let nan_reasons: Vec<_> = verdict
        .reasons
        .iter()
        .filter(|r| r.contains("NaN detected"))
        .collect();
    assert_eq!(
        nan_reasons.len(),
        1,
        "3 identical unexpected NoNaN violations must produce exactly 1 reason, got {}",
        nan_reasons.len()
    );
}
