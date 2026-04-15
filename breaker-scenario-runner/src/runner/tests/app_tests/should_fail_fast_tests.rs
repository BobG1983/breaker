//! Tests for `should_fail_fast` — pure function covering fail-fast conditions.

use crate::{
    invariants::{ViolationEntry, ViolationLog},
    runner::app::should_fail_fast,
    types::{InputStrategy, InvariantKind, ScenarioDefinition, ScriptedParams},
};

/// Helper: builds a minimal `ScenarioDefinition` with the given `allowed_failures`.
fn definition_with_allowed_failures(
    allowed_failures: Option<Vec<InvariantKind>>,
) -> ScenarioDefinition {
    ScenarioDefinition {
        breaker: "test".into(),
        layout: "test".into(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 100,
        disallowed_failures: vec![],
        allowed_failures,
        ..Default::default()
    }
}

/// Helper: builds a `ViolationLog` with the given entries.
fn violation_log_with(entries: Vec<ViolationEntry>) -> ViolationLog {
    ViolationLog(entries)
}

/// Helper: builds a single `ViolationEntry` for `BoltInBounds` at the given frame.
fn bolt_oob_violation(frame: u32) -> ViolationEntry {
    ViolationEntry {
        frame,
        invariant: InvariantKind::BoltInBounds,
        entity: None,
        message: "bolt OOB".into(),
    }
}

// -------------------------------------------------------------------------
// Behavior 5: returns true when fail-fast active, violations exist, no
//             allowed_failures
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_true_when_active_with_violations_and_no_allowed_failures() {
    let log = violation_log_with(vec![bolt_oob_violation(5)]);
    let definition = definition_with_allowed_failures(None);

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        result,
        "should_fail_fast must return true when fail_fast=true, violations exist, and allowed_failures=None"
    );
}

// -------------------------------------------------------------------------
// Behavior 5 edge: allowed_failures = Some(vec![]) also returns true
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_true_when_allowed_failures_is_empty_vec() {
    let log = violation_log_with(vec![bolt_oob_violation(5)]);
    let definition = definition_with_allowed_failures(Some(vec![]));

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        result,
        "should_fail_fast must return true when allowed_failures=Some(vec![]) because empty vec means no expected violations"
    );
}

// -------------------------------------------------------------------------
// Behavior 6: returns false when all violations are in allowed_failures
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_false_when_violation_is_in_allowed_failures() {
    let log = violation_log_with(vec![ViolationEntry {
        frame:     10,
        invariant: InvariantKind::BoltInBounds,
        entity:    None,
        message:   "expected violation".into(),
    }]);
    let definition = definition_with_allowed_failures(Some(vec![InvariantKind::BoltInBounds]));

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        !result,
        "should_fail_fast must return false when violation is in allowed_failures (expected self-test violation)"
    );
}

// -------------------------------------------------------------------------
// Behavior 6 edge: multiple allowed_failures covering the violation
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_false_when_violation_covered_by_multiple_allowed() {
    let log = violation_log_with(vec![bolt_oob_violation(10)]);
    let definition = definition_with_allowed_failures(Some(vec![
        InvariantKind::BoltInBounds,
        InvariantKind::NoNaN,
    ]));

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        !result,
        "should_fail_fast must return false when violation is in allowed_failures list"
    );
}

// -------------------------------------------------------------------------
// Behavior 6 edge: disallowed violation in self-test triggers fail-fast
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_true_when_violation_not_in_allowed_failures() {
    // Self-test allows BoltInBounds but gets NoNaN — should fail-fast
    let log = violation_log_with(vec![ViolationEntry {
        frame:     10,
        invariant: InvariantKind::NoNaN,
        entity:    None,
        message:   "unexpected NaN".into(),
    }]);
    let definition = definition_with_allowed_failures(Some(vec![InvariantKind::BoltInBounds]));

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        result,
        "should_fail_fast must return true when violation is NOT in allowed_failures (disallowed violation in self-test)"
    );
}

// -------------------------------------------------------------------------
// Behavior 6 edge: mixed allowed and disallowed violations triggers fail-fast
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_true_when_any_violation_not_in_allowed_failures() {
    // Self-test allows BoltInBounds, gets both BoltInBounds (allowed) and NoNaN (disallowed)
    let log = violation_log_with(vec![
        bolt_oob_violation(5),
        ViolationEntry {
            frame:     10,
            invariant: InvariantKind::NoNaN,
            entity:    None,
            message:   "unexpected NaN".into(),
        },
    ]);
    let definition = definition_with_allowed_failures(Some(vec![InvariantKind::BoltInBounds]));

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        result,
        "should_fail_fast must return true when any violation is NOT in allowed_failures"
    );
}

// -------------------------------------------------------------------------
// Behavior 7: returns false when fail_fast flag is false
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_false_when_flag_is_false() {
    let log = violation_log_with(vec![bolt_oob_violation(5)]);
    let definition = definition_with_allowed_failures(None);

    let result = should_fail_fast(&log, &definition, false);

    assert!(
        !result,
        "should_fail_fast must return false when fail_fast=false regardless of violations"
    );
}

// -------------------------------------------------------------------------
// Behavior 7 edge: multiple violations, still false when flag is false
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_false_with_multiple_violations_when_flag_is_false() {
    let log = violation_log_with(vec![bolt_oob_violation(5), bolt_oob_violation(10)]);
    let definition = definition_with_allowed_failures(None);

    let result = should_fail_fast(&log, &definition, false);

    assert!(
        !result,
        "should_fail_fast must return false when fail_fast=false even with multiple violations"
    );
}

// -------------------------------------------------------------------------
// Behavior 8: returns false when ViolationLog is empty
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_false_when_violation_log_is_empty() {
    let log = violation_log_with(vec![]);
    let definition = definition_with_allowed_failures(None);

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        !result,
        "should_fail_fast must return false when violation log is empty even with fail_fast=true"
    );
}

// -------------------------------------------------------------------------
// Behavior 8 edge: empty log + empty allowed_failures still returns false
// -------------------------------------------------------------------------

#[test]
fn should_fail_fast_returns_false_when_log_empty_and_allowed_failures_empty() {
    let log = violation_log_with(vec![]);
    let definition = definition_with_allowed_failures(Some(vec![]));

    let result = should_fail_fast(&log, &definition, true);

    assert!(
        !result,
        "should_fail_fast must return false when log is empty (empty log takes precedence)"
    );
}
