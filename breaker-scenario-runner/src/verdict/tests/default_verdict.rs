//! Tests for `ScenarioVerdict` default state and `passed()` on defaults.

use crate::verdict::evaluation::*;

// -------------------------------------------------------------------------
// Behavior 1: Default verdict is Fail with default reason
// -------------------------------------------------------------------------

#[test]
fn default_verdict_is_fail() {
    let verdict = ScenarioVerdict::default();
    assert_eq!(verdict.status, VerdictStatus::Fail);
}

#[test]
fn default_verdict_has_single_default_reason() {
    let verdict = ScenarioVerdict::default();
    assert_eq!(
        verdict.reasons,
        vec!["scenario did not complete evaluation".to_owned()],
        "expected default reason message"
    );
}

// -------------------------------------------------------------------------
// Behavior 2: passed() returns false for default verdict
// -------------------------------------------------------------------------

#[test]
fn passed_returns_false_for_default_verdict() {
    let verdict = ScenarioVerdict::default();
    assert!(
        !verdict.passed(),
        "passed() must return false for default verdict"
    );
}
