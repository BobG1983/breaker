//! Tests for `collect_and_evaluate` — pass/fail and early-exit snapshot handling.

use std::sync::{Arc, Mutex};

use crate::{
    invariants::{ScenarioStats, ViolationEntry},
    runner::app::{EvalSnapshot, SharedEvalBuffer, collect_and_evaluate},
    types::{InputStrategy, InvariantKind, ScenarioDefinition, ScriptedParams},
};

// -------------------------------------------------------------------------
// collect_and_evaluate fails when no snapshot was captured
// -------------------------------------------------------------------------

#[test]
fn collect_and_evaluate_fails_when_no_snapshot() {
    let buffer = SharedEvalBuffer(Arc::new(Mutex::new(None)));
    let passed = collect_and_evaluate(&buffer, "test_scenario", false, None);
    assert!(!passed, "should fail when no snapshot was captured");
}

// -------------------------------------------------------------------------
// collect_and_evaluate passes with a clean snapshot
// -------------------------------------------------------------------------

#[test]
fn collect_and_evaluate_passes_with_clean_snapshot() {
    let definition = ScenarioDefinition {
        breaker: "test".into(),
        layout: "test".into(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 100,
        disallowed_failures: vec![],
        ..Default::default()
    };
    let stats = ScenarioStats {
        actions_injected: 0,
        invariant_checks: 10,
        max_frame: 50,
        entered_playing: true,
        bolts_tagged: 1,
        breakers_tagged: 1,
        ..Default::default()
    };
    let snapshot = EvalSnapshot {
        violations: vec![],
        logs: vec![],
        stats,
        definition,
    };
    let buffer = SharedEvalBuffer(Arc::new(Mutex::new(Some(snapshot))));
    let passed = collect_and_evaluate(&buffer, "test_scenario", false, None);
    assert!(
        passed,
        "should pass with clean snapshot and empty scripted actions"
    );
}

// -------------------------------------------------------------------------
// Behavior 9: collect_and_evaluate processes violations normally after
//             fail-fast early exit (snapshot with violations, low frame count)
// -------------------------------------------------------------------------

#[test]
fn collect_and_evaluate_reports_failure_for_early_exit_snapshot_with_violations() {
    let definition = ScenarioDefinition {
        breaker: "test".into(),
        layout: "test".into(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 20000,
        disallowed_failures: vec![],
        allowed_failures: None,
        ..Default::default()
    };
    let stats = ScenarioStats {
        actions_injected: 0,
        invariant_checks: 5,
        max_frame: 5,
        entered_playing: true,
        bolts_tagged: 1,
        breakers_tagged: 1,
        ..Default::default()
    };
    let snapshot = EvalSnapshot {
        violations: vec![ViolationEntry {
            frame:     5,
            invariant: InvariantKind::BoltInBounds,
            entity:    None,
            message:   "bolt OOB at (999.0, 0.0)".into(),
        }],
        logs: vec![],
        stats,
        definition,
    };
    let buffer = SharedEvalBuffer(Arc::new(Mutex::new(Some(snapshot))));

    let passed = collect_and_evaluate(&buffer, "early_exit_test", false, None);

    assert!(
        !passed,
        "collect_and_evaluate must report failure when snapshot contains violations from early exit"
    );
}
