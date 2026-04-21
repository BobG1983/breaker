//! Tests for `collect_and_evaluate` — pass/fail and early-exit snapshot handling.

use std::{
    fs,
    sync::{Arc, Mutex},
};

use crate::{
    invariants::{ScenarioStats, ViolationEntry},
    runner::app::{EvalSnapshot, SharedEvalBuffer, collect_and_evaluate, write_chaos_regression},
    types::{
        ChaosParams, GameAction, InputStrategy, InvariantKind, ScenarioDefinition, ScriptedFrame,
        ScriptedParams,
    },
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
        chaos_input_log: None,
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
        chaos_input_log: None,
    };
    let buffer = SharedEvalBuffer(Arc::new(Mutex::new(Some(snapshot))));

    let passed = collect_and_evaluate(&buffer, "early_exit_test", false, None);

    assert!(
        !passed,
        "collect_and_evaluate must report failure when snapshot contains violations from early exit"
    );
}

// -------------------------------------------------------------------------
// write_chaos_regression — replayable scenario file is emitted on chaos fail
// -------------------------------------------------------------------------

#[test]
fn write_chaos_regression_produces_replayable_scripted_scenario() {
    let tmp_root = std::env::temp_dir().join(format!(
        "breaker_write_chaos_regression_{}",
        std::process::id()
    ));
    drop(fs::remove_dir_all(&tmp_root));
    fs::create_dir_all(&tmp_root).expect("create tmp root");

    let original = ScenarioDefinition {
        breaker: "Aegis".into(),
        layout: "Scatter".into(),
        input: InputStrategy::Chaos(ChaosParams { action_prob: 0.7 }),
        max_frames: 3000,
        disallowed_failures: vec![InvariantKind::NoNaN],
        seed: Some(9173),
        ..Default::default()
    };
    let chaos_log = vec![
        ScriptedFrame {
            frame:   5,
            actions: vec![GameAction::MoveLeft],
        },
        ScriptedFrame {
            frame:   42,
            actions: vec![GameAction::Bump],
        },
    ];

    let written = write_chaos_regression(
        &tmp_root,
        "some_chaos_scenario",
        &original,
        chaos_log.clone(),
    )
    .expect("regression write must succeed");

    assert!(written.exists(), "regression file must be created on disk");
    let parent = written.parent().expect("written file has a parent");
    assert_eq!(
        parent.file_name().and_then(|s| s.to_str()),
        Some("regressions"),
        "regression file must live under scenarios/regressions/",
    );
    let stem = written.file_name().and_then(|s| s.to_str()).unwrap_or("");
    assert!(
        stem.contains("-chaos-some_chaos_scenario.scenario.ron"),
        "filename must follow <timestamp>-chaos-<name>.scenario.ron pattern, got {stem}",
    );

    let contents = fs::read_to_string(&written).expect("regression file readable");
    let replay: ScenarioDefinition =
        ron::de::from_str(&contents).expect("regression file must parse as ScenarioDefinition");

    assert!(
        matches!(replay.input, InputStrategy::Scripted(_)),
        "replay scenario input must be Scripted, got {:?}",
        replay.input,
    );
    if let InputStrategy::Scripted(ref params) = replay.input {
        assert_eq!(
            params.actions, chaos_log,
            "replay Scripted actions must match the recorded chaos log"
        );
    }
    assert_eq!(
        replay.seed, None,
        "replay scenario must drop the seed (scripted input is deterministic by action list)",
    );
    assert_eq!(replay.breaker, original.breaker);
    assert_eq!(replay.layout, original.layout);
    assert_eq!(replay.max_frames, original.max_frames);
    assert_eq!(replay.disallowed_failures, original.disallowed_failures);

    drop(fs::remove_dir_all(&tmp_root));
}

// -------------------------------------------------------------------------
// write_chaos_regression — path traversal in scenario_name is neutralised
// -------------------------------------------------------------------------

#[test]
fn write_chaos_regression_sanitizes_path_traversal_in_scenario_name() {
    let tmp_root = std::env::temp_dir().join(format!(
        "breaker_write_chaos_regression_traversal_{}",
        std::process::id()
    ));
    drop(fs::remove_dir_all(&tmp_root));
    fs::create_dir_all(&tmp_root).expect("create tmp root");

    let original = ScenarioDefinition {
        breaker: "Aegis".into(),
        layout: "Scatter".into(),
        input: InputStrategy::Chaos(ChaosParams { action_prob: 0.5 }),
        max_frames: 500,
        disallowed_failures: vec![InvariantKind::NoNaN],
        seed: Some(1),
        ..Default::default()
    };

    // Attacker-shaped scenario name: `..` separators, embedded dots, slashes.
    let hostile = "../../../etc/passwd";
    let written = write_chaos_regression(
        &tmp_root,
        hostile,
        &original,
        vec![ScriptedFrame {
            frame:   1,
            actions: vec![GameAction::Bump],
        }],
    )
    .expect("regression write must succeed even for hostile names");

    // The written path MUST remain inside the regressions subdirectory of
    // tmp_root. Without sanitization, `join("..-chaos-..")` would land in
    // tmp_root itself — or worse, above it.
    let expected_regressions = tmp_root.join("regressions");
    let canonical_written = written.canonicalize().expect("written file canonicalizes");
    let canonical_regressions = expected_regressions
        .canonicalize()
        .expect("regressions dir canonicalizes");
    assert!(
        canonical_written.starts_with(&canonical_regressions),
        "sanitized filename must keep the file inside {}, got {}",
        canonical_regressions.display(),
        canonical_written.display(),
    );

    // Filename must not contain `/`, `\`, or `..` sequences after sanitization.
    let stem = written.file_name().and_then(|s| s.to_str()).unwrap_or("");
    assert!(
        !stem.contains('/') && !stem.contains('\\') && !stem.contains(".."),
        "sanitized filename must not carry path-traversal tokens, got {stem}",
    );

    drop(fs::remove_dir_all(&tmp_root));
}
