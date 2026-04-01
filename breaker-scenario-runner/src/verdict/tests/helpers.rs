//! Shared test helpers for verdict tests.

use bevy::log::Level;

use crate::{
    invariants::{ScenarioStats, ViolationEntry},
    log_capture::LogEntry,
    types::{ChaosParams, InputStrategy, InvariantKind, ScenarioDefinition},
};

pub(super) fn make_violation(invariant: InvariantKind) -> ViolationEntry {
    ViolationEntry {
        frame: 42,
        invariant,
        entity: None,
        message: format!("test violation: {invariant:?}"),
    }
}

pub(super) fn make_log_entry(message: &str) -> LogEntry {
    LogEntry {
        level: Level::WARN,
        target: "breaker::test".to_owned(),
        message: message.to_owned(),
        frame: 10,
    }
}

pub(super) fn make_chaos_definition() -> ScenarioDefinition {
    ScenarioDefinition {
        breaker: "aegis".to_owned(),
        layout: "corridor".to_owned(),
        input: InputStrategy::Chaos(ChaosParams { action_prob: 0.3 }),
        max_frames: 1000,
        disallowed_failures: vec![],
        ..Default::default()
    }
}

pub(super) fn make_healthy_stats() -> ScenarioStats {
    ScenarioStats {
        actions_injected: 50,
        invariant_checks: 100,
        max_frame: 100,
        entered_playing: true,
        bolts_tagged: 1,
        breakers_tagged: 1,
        ..Default::default()
    }
}
