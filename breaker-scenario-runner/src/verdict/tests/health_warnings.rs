//! Tests for health warning detection in `ScenarioVerdict::evaluate`.

use super::{super::evaluation::*, helpers::*};
use crate::invariants::ScenarioStats;

// -------------------------------------------------------------------------
// Behavior 10: Health warning — no actions injected
// -------------------------------------------------------------------------

#[test]
fn health_warning_no_actions_injected_evaluates_to_fail() {
    let mut verdict = ScenarioVerdict::default();
    let stats = ScenarioStats {
        actions_injected: 0,
        invariant_checks: 100,
        max_frame: 100,
        entered_playing: true,
        bolts_tagged: 1,
        breakers_tagged: 1,
        ..Default::default()
    };
    let definition = make_chaos_definition();

    verdict.evaluate(&[], &[], &stats, &definition);

    assert!(!verdict.passed(), "zero actions injected must cause Fail");
    let has_reason = verdict
        .reasons
        .iter()
        .any(|r| r.to_lowercase().contains("no actions were injected"));
    assert!(
        has_reason,
        "reasons must contain 'no actions were injected', got: {:?}",
        verdict.reasons
    );
}

// -------------------------------------------------------------------------
// Behavior 11: Health warning — never entered Playing
// -------------------------------------------------------------------------

#[test]
fn health_warning_never_entered_playing_evaluates_to_fail() {
    let mut verdict = ScenarioVerdict::default();
    let stats = ScenarioStats {
        actions_injected: 50,
        invariant_checks: 100,
        max_frame: 100,
        entered_playing: false,
        bolts_tagged: 1,
        breakers_tagged: 1,
        ..Default::default()
    };
    let definition = make_chaos_definition();

    verdict.evaluate(&[], &[], &stats, &definition);

    assert!(!verdict.passed(), "never entering Playing must cause Fail");
    let has_reason = verdict
        .reasons
        .iter()
        .any(|r| r.to_lowercase().contains("never entered playing"));
    assert!(
        has_reason,
        "reasons must contain 'never entered Playing', got: {:?}",
        verdict.reasons
    );
}

// -------------------------------------------------------------------------
// Behavior 12: Health warning — no bolts tagged
// -------------------------------------------------------------------------

#[test]
fn health_warning_no_bolts_tagged_evaluates_to_fail() {
    let mut verdict = ScenarioVerdict::default();
    let stats = ScenarioStats {
        actions_injected: 50,
        invariant_checks: 100,
        max_frame: 100,
        entered_playing: true,
        bolts_tagged: 0,
        breakers_tagged: 1,
        ..Default::default()
    };
    let definition = make_chaos_definition();

    verdict.evaluate(&[], &[], &stats, &definition);

    assert!(!verdict.passed(), "no bolts tagged must cause Fail");
    let has_reason = verdict
        .reasons
        .iter()
        .any(|r| r.to_lowercase().contains("no bolts were tagged"));
    assert!(
        has_reason,
        "reasons must contain 'no bolts were tagged', got: {:?}",
        verdict.reasons
    );
}

// -------------------------------------------------------------------------
// Behavior 13: Health warning — no breakers tagged
// -------------------------------------------------------------------------

#[test]
fn health_warning_no_breakers_tagged_evaluates_to_fail() {
    let mut verdict = ScenarioVerdict::default();
    let stats = ScenarioStats {
        actions_injected: 50,
        invariant_checks: 100,
        max_frame: 100,
        entered_playing: true,
        bolts_tagged: 1,
        breakers_tagged: 0,
        ..Default::default()
    };
    let definition = make_chaos_definition();

    verdict.evaluate(&[], &[], &stats, &definition);

    assert!(!verdict.passed(), "no breakers tagged must cause Fail");
    let has_reason = verdict
        .reasons
        .iter()
        .any(|r| r.to_lowercase().contains("no breakers were tagged"));
    assert!(
        has_reason,
        "reasons must contain 'no breakers were tagged', got: {:?}",
        verdict.reasons
    );
}

// -------------------------------------------------------------------------
// Behavior 14: Health warning — early exit
// -------------------------------------------------------------------------

#[test]
fn health_warning_early_exit_evaluates_to_fail() {
    let mut verdict = ScenarioVerdict::default();
    let stats = ScenarioStats {
        actions_injected: 50,
        invariant_checks: 10,
        max_frame: 5,
        entered_playing: true,
        bolts_tagged: 1,
        breakers_tagged: 1,
        ..Default::default()
    };
    let definition = make_chaos_definition();

    verdict.evaluate(&[], &[], &stats, &definition);

    assert!(
        !verdict.passed(),
        "early exit (max_frame=5) must cause Fail"
    );
    let has_reason = verdict.reasons.iter().any(|r| {
        let lower = r.to_lowercase();
        lower.contains("exited") || lower.contains("very early")
    });
    assert!(
        has_reason,
        "reasons must mention 'exited' or 'very early', got: {:?}",
        verdict.reasons
    );
}

// -------------------------------------------------------------------------
// Behavior 15: Health warning — no invariant checks
// -------------------------------------------------------------------------

#[test]
fn health_warning_no_invariant_checks_evaluates_to_fail() {
    let mut verdict = ScenarioVerdict::default();
    let stats = ScenarioStats {
        actions_injected: 50,
        invariant_checks: 0,
        max_frame: 100,
        entered_playing: true,
        bolts_tagged: 1,
        breakers_tagged: 1,
        ..Default::default()
    };
    let definition = make_chaos_definition();

    verdict.evaluate(&[], &[], &stats, &definition);

    assert!(!verdict.passed(), "zero invariant checks must cause Fail");
    let has_reason = verdict
        .reasons
        .iter()
        .any(|r| r.to_lowercase().contains("no invariant checks ran"));
    assert!(
        has_reason,
        "reasons must contain 'no invariant checks ran', got: {:?}",
        verdict.reasons
    );
}
