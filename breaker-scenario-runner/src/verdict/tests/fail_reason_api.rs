//! Tests for `add_fail_reason` and log-induced failures.

use super::{super::evaluation::*, helpers::*};

// -------------------------------------------------------------------------
// Behavior 8: Logs cause failure even when violations match
// -------------------------------------------------------------------------

#[test]
fn logs_cause_failure_even_when_violations_match() {
    use crate::types::InvariantKind;

    let mut verdict = ScenarioVerdict::default();
    let violations = vec![make_violation(InvariantKind::BoltInBounds)];
    let logs = vec![make_log_entry("bad thing")];
    let stats = make_healthy_stats();
    let mut definition = make_chaos_definition();
    definition.allowed_failures = Some(vec![InvariantKind::BoltInBounds]);

    verdict.evaluate(&violations, &logs, &stats, &definition);

    assert!(
        !verdict.passed(),
        "captured logs must cause Fail even when violations match expected"
    );
    let has_reason = verdict.reasons.iter().any(|r| {
        let lower = r.to_lowercase();
        lower.contains("captured") && r.contains("bad thing")
    });
    assert!(
        has_reason,
        "reasons must mention 'captured' and 'bad thing', got: {:?}",
        verdict.reasons
    );
}

// -------------------------------------------------------------------------
// Behavior 17: add_fail_reason keeps Fail and appends
// -------------------------------------------------------------------------

#[test]
fn add_fail_reason_keeps_fail_and_appends_reason() {
    let mut verdict = ScenarioVerdict::default();

    verdict.add_fail_reason("extra reason".to_owned());

    assert!(
        !verdict.passed(),
        "add_fail_reason must keep status as Fail"
    );
    assert!(
        verdict
            .reasons
            .contains(&"scenario did not complete evaluation".to_owned()),
        "original default reason must still be present, got: {:?}",
        verdict.reasons
    );
    assert!(
        verdict.reasons.contains(&"extra reason".to_owned()),
        "appended reason must be present, got: {:?}",
        verdict.reasons
    );
}

// -------------------------------------------------------------------------
// Behavior 18: add_fail_reason on a Pass verdict reverts to Fail
// -------------------------------------------------------------------------

#[test]
fn add_fail_reason_on_pass_verdict_reverts_to_fail() {
    let mut verdict = ScenarioVerdict::default();
    let stats = make_healthy_stats();
    let definition = make_chaos_definition();

    // First bring the verdict to Pass via a clean evaluate.
    verdict.evaluate(&[], &[], &stats, &definition);
    assert!(
        verdict.passed(),
        "prerequisite: clean evaluate must produce Pass before testing add_fail_reason"
    );

    verdict.add_fail_reason("late failure".to_owned());

    assert!(
        !verdict.passed(),
        "add_fail_reason must revert a Pass verdict to Fail"
    );
    assert!(
        verdict.reasons.contains(&"late failure".to_owned()),
        "appended reason must be present, got: {:?}",
        verdict.reasons
    );
}
