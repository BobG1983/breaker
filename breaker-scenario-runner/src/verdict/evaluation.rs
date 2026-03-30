//! Scenario verdict consolidation.
//!
//! [`ScenarioVerdict`] collects pass/fail state across violation checks, log
//! capture, and health warnings. Defaults to `Fail` so any gap in the
//! evaluation pipeline produces a safe failure.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    invariants::{ScenarioStats, ViolationEntry},
    log_capture::LogEntry,
    types::{BumpMode, InputStrategy, InvariantKind, ScenarioDefinition, ScriptedParams},
};

/// Whether a scenario run passed or failed evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerdictStatus {
    /// The scenario satisfied all invariants and health checks.
    Pass,
    /// The scenario failed one or more checks.
    Fail,
}

/// Consolidated pass/fail verdict for a completed scenario run.
///
/// Defaults to [`VerdictStatus::Fail`] so any unevaluated run is a safe failure.
#[derive(Resource, Debug, Clone)]
pub struct ScenarioVerdict {
    /// Overall result of the scenario.
    pub status: VerdictStatus,
    /// Human-readable reasons for failure (empty when passing).
    pub reasons: Vec<String>,
}

impl Default for ScenarioVerdict {
    fn default() -> Self {
        Self {
            status: VerdictStatus::Fail,
            reasons: vec!["scenario did not complete evaluation".into()],
        }
    }
}

impl ScenarioVerdict {
    /// Runs combined evaluation over violations, captured logs, stats, and
    /// scenario definition, updating `status` and `reasons` in place.
    ///
    /// Clears any prior failure reasons before re-evaluating.
    pub fn evaluate(
        &mut self,
        violations: &[ViolationEntry],
        logs: &[LogEntry],
        stats: &ScenarioStats,
        definition: &ScenarioDefinition,
    ) {
        self.reasons.clear();

        // Check violations against expected_violations.
        match definition.expected_violations.as_deref() {
            None | Some([]) => {
                // No expected violations: any violation is a failure (one reason per kind).
                let mut seen = HashSet::<InvariantKind>::new();
                for v in violations {
                    if seen.insert(v.invariant) {
                        self.add_fail_reason(v.invariant.fail_reason().to_owned());
                    }
                }
            }
            Some(expected) => {
                // Expected violations defined: check that all fired and none unexpected.
                for ev in expected {
                    if !violations.iter().any(|v| &v.invariant == ev) {
                        self.add_fail_reason(format!("expected violation {ev:?} never fired"));
                    }
                }
                let mut seen = HashSet::<InvariantKind>::new();
                for v in violations {
                    if !expected.iter().any(|ev| ev == &v.invariant) && seen.insert(v.invariant) {
                        self.add_fail_reason(v.invariant.fail_reason().to_owned());
                    }
                }
            }
        }

        // Captured logs always cause failure.
        for l in logs {
            self.add_fail_reason(format!(
                "captured {:?} log at frame {}: {}",
                l.level, l.frame, l.message
            ));
        }

        // Health checks.
        let is_empty_scripted = matches!(
            &definition.input,
            InputStrategy::Scripted(ScriptedParams { actions }) if actions.is_empty()
        );

        let is_never_bump = matches!(
            &definition.input,
            InputStrategy::Perfect(BumpMode::NeverBump)
        );

        if stats.actions_injected == 0 && !is_empty_scripted && !is_never_bump {
            self.add_fail_reason(format!(
                "no actions were injected during scenario run (input strategy: {:?})",
                definition.input
            ));
        }

        if !stats.entered_playing {
            self.add_fail_reason("scenario never entered Playing state".to_owned());
        }

        if stats.bolts_tagged == 0 {
            self.add_fail_reason("no bolts were tagged — bolt invariants are vacuous".to_owned());
        }

        if stats.breakers_tagged == 0 {
            self.add_fail_reason(
                "no breakers were tagged — breaker invariants are vacuous".to_owned(),
            );
        }

        if stats.max_frame < 10 {
            self.add_fail_reason(format!(
                "scenario exited very early (max_frame={})",
                stats.max_frame
            ));
        }

        if stats.invariant_checks == 0 {
            self.add_fail_reason(
                "no invariant checks ran — game loop may not have executed".to_owned(),
            );
        }

        // If no reasons accumulated, the run passed.
        if self.reasons.is_empty() {
            self.status = VerdictStatus::Pass;
        }
    }

    /// Returns `true` when the verdict is [`VerdictStatus::Pass`].
    #[must_use]
    pub fn passed(&self) -> bool {
        self.status == VerdictStatus::Pass
    }

    /// Appends a failure reason and ensures `status` is [`VerdictStatus::Fail`].
    ///
    /// Reverts a [`VerdictStatus::Pass`] to [`VerdictStatus::Fail`] if called
    /// after a successful [`evaluate`](Self::evaluate).
    pub fn add_fail_reason(&mut self, reason: String) {
        self.status = VerdictStatus::Fail;
        self.reasons.push(reason);
    }
}
