//! Shared test helpers for `run_log` tests.

use std::{fs, path::PathBuf};

use crate::{
    invariants::{ScenarioStats, ViolationEntry},
    runner::app::EvalSnapshot,
    types::{InputStrategy, InvariantKind, ScenarioDefinition, ScriptedParams},
};

/// Creates an isolated temp directory for a single test, returning its path.
///
/// The caller is responsible for cleaning up via `fs::remove_dir_all`.
pub(super) fn test_temp_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "breaker_test_run_log_{}_{test_name}",
        std::process::id()
    ));
    // Clean up any leftover from a previous interrupted run.
    drop(fs::remove_dir_all(&dir));
    fs::create_dir_all(&dir).expect("failed to create test temp dir");
    dir
}

/// Builds a clean `EvalSnapshot` with no violations and no logs.
pub(super) fn clean_snapshot(_scenario_name: &str) -> EvalSnapshot {
    EvalSnapshot {
        violations: vec![],
        logs:       vec![],
        stats:      ScenarioStats {
            actions_injected: 0,
            invariant_checks: 10,
            max_frame: 50,
            entered_playing: true,
            bolts_tagged: 1,
            breakers_tagged: 1,
            ..Default::default()
        },
        definition: ScenarioDefinition {
            breaker: "test".into(),
            layout: "test".into(),
            input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
            max_frames: 100,
            disallowed_failures: vec![],
            ..Default::default()
        },
    }
}

/// Builds an `EvalSnapshot` with the given violations.
pub(super) fn snapshot_with_violations(violations: Vec<ViolationEntry>) -> EvalSnapshot {
    let mut snap = clean_snapshot("test_scenario");
    snap.violations = violations;
    snap
}

pub(super) fn make_violation(
    invariant: InvariantKind,
    frame: u32,
    message: &str,
) -> ViolationEntry {
    ViolationEntry {
        frame,
        invariant,
        entity: None,
        message: message.to_owned(),
    }
}
