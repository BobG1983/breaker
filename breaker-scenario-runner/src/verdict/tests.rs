use bevy::log::Level;

use super::*;
use crate::{
    invariants::ScenarioStats,
    log_capture::LogEntry,
    types::{ChaosParams, InputStrategy, InvariantKind, InvariantParams, ScenarioDefinition},
};

// -------------------------------------------------------------------------
// Test helpers
// -------------------------------------------------------------------------

fn make_violation(invariant: InvariantKind) -> ViolationEntry {
    ViolationEntry {
        frame: 42,
        invariant,
        entity: None,
        message: format!("test violation: {invariant:?}"),
    }
}

fn make_log_entry(message: &str) -> LogEntry {
    LogEntry {
        level: Level::WARN,
        target: "breaker::test".to_owned(),
        message: message.to_owned(),
        frame: 10,
    }
}

fn make_chaos_definition() -> ScenarioDefinition {
    ScenarioDefinition {
        breaker: "aegis".to_owned(),
        layout: "corridor".to_owned(),
        input: InputStrategy::Chaos(ChaosParams {
            seed: 0,
            action_prob: 0.3,
        }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: None,
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
    }
}

fn make_healthy_stats() -> ScenarioStats {
    ScenarioStats {
        actions_injected: 50,
        invariant_checks: 100,
        max_frame: 100,
        entered_playing: true,
        bolts_tagged: 1,
        breakers_tagged: 1,
    }
}

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

// -------------------------------------------------------------------------
// Behavior 3: Clean run evaluates to Pass
// -------------------------------------------------------------------------

#[test]
fn clean_run_evaluates_to_pass() {
    let mut verdict = ScenarioVerdict::default();
    let stats = make_healthy_stats();
    let definition = make_chaos_definition();

    verdict.evaluate(&[], &[], &stats, &definition);

    assert!(verdict.passed(), "clean run must evaluate to Pass");
    assert!(
        verdict.reasons.is_empty(),
        "clean run must produce no reasons, got: {:?}",
        verdict.reasons
    );
}

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
        "unexpected violation must cause Fail when no expected_violations list"
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
fn expected_violations_match_exactly_evaluates_to_pass() {
    let mut verdict = ScenarioVerdict::default();
    let violations = vec![make_violation(InvariantKind::BoltInBounds)];
    let stats = make_healthy_stats();
    let mut definition = make_chaos_definition();
    definition.expected_violations = Some(vec![InvariantKind::BoltInBounds]);

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
    definition.expected_violations = Some(vec![InvariantKind::BoltInBounds]);

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
    definition.expected_violations = Some(vec![InvariantKind::BoltInBounds]);

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
// Behavior 8: Logs cause failure even when violations match
// -------------------------------------------------------------------------

#[test]
fn logs_cause_failure_even_when_violations_match() {
    let mut verdict = ScenarioVerdict::default();
    let violations = vec![make_violation(InvariantKind::BoltInBounds)];
    let logs = vec![make_log_entry("bad thing")];
    let stats = make_healthy_stats();
    let mut definition = make_chaos_definition();
    definition.expected_violations = Some(vec![InvariantKind::BoltInBounds]);

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
// Behavior 9: Empty expected list with no violations evaluates to Pass
// -------------------------------------------------------------------------

#[test]
fn empty_expected_list_with_no_violations_evaluates_to_pass() {
    let mut verdict = ScenarioVerdict::default();
    let stats = make_healthy_stats();
    let mut definition = make_chaos_definition();
    definition.expected_violations = Some(vec![]);

    verdict.evaluate(&[], &[], &stats, &definition);

    assert!(
        verdict.passed(),
        "Some([]) expected with no violations must evaluate to Pass"
    );
}

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

// -------------------------------------------------------------------------
// Behavior 16: Healthy stats produce no health reasons (same as Behavior 3,
// stated from the health angle)
// -------------------------------------------------------------------------

#[test]
fn healthy_stats_produce_no_health_reasons() {
    let mut verdict = ScenarioVerdict::default();
    let stats = make_healthy_stats();
    let definition = make_chaos_definition();

    verdict.evaluate(&[], &[], &stats, &definition);

    assert!(
        verdict.passed(),
        "healthy stats with no violations or logs must evaluate to Pass"
    );
    assert!(
        verdict.reasons.is_empty(),
        "healthy stats must produce no reasons, got: {:?}",
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
    definition.expected_violations = Some(vec![InvariantKind::BoltInBounds]);

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
