//! Scenario verdict consolidation.
//!
//! [`ScenarioVerdict`] collects pass/fail state across violation checks, log
//! capture, and health warnings. Defaults to `Fail` so any gap in the
//! evaluation pipeline produces a safe failure.

use bevy::prelude::*;

use crate::{
    invariants::{ScenarioStats, ViolationEntry},
    log_capture::LogEntry,
    types::{InputStrategy, ScenarioDefinition, ScriptedParams},
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
                // No expected violations: any violation is a failure.
                for v in violations {
                    self.add_fail_reason(v.invariant.fail_reason().to_owned());
                }
            }
            Some(expected) => {
                // Expected violations defined: check that all fired and none unexpected.
                for ev in expected {
                    if !violations.iter().any(|v| &v.invariant == ev) {
                        self.add_fail_reason(format!("expected violation {ev:?} never fired"));
                    }
                }
                for v in violations {
                    if !expected.iter().any(|ev| ev == &v.invariant) {
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

        if stats.actions_injected == 0 && !is_empty_scripted {
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

#[cfg(test)]
mod tests {
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
}
