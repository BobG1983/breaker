//! Survival turret behavior components.
//!
//! Marker, pattern, timer, and immunity components for survival cells.
//! Behavior lives in the sibling `systems/` module.

use bevy::prelude::*;

use crate::cells::definition::AttackPattern;

/// Permanent marker identifying a cell as a survival turret.
#[derive(Component, Debug)]
pub struct SurvivalTurret;

/// The turret's attack pattern — determines salvo spawn shape.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurvivalPattern(pub(crate) AttackPattern);

/// Self-destruct countdown timer. Decremented each tick once `started`
/// is true. When it reaches zero, the turret self-destructs (Wave 6B).
/// `started` flips to true on first salvo fired.
#[derive(Component, Debug)]
pub struct SurvivalTimer {
    /// Remaining seconds until self-destruct.
    pub remaining: f32,
    /// Whether the countdown has started (flips on first shot).
    pub started:   bool,
}

/// Marker: entity is immune to bolt-sourced damage.
#[derive(Component, Debug)]
pub struct BoltImmune;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 22: SurvivalTurret is a marker Component with Debug ──

    #[test]
    fn survival_turret_debug_contains_name() {
        let marker = SurvivalTurret;
        let debug_str = format!("{marker:?}");
        assert!(
            debug_str.contains("SurvivalTurret"),
            "debug output should contain 'SurvivalTurret', got: {debug_str}"
        );
    }

    // ── Behavior 23: SurvivalPattern wraps AttackPattern ──

    #[test]
    fn survival_pattern_wraps_attack_pattern_spread() {
        let pattern = SurvivalPattern(AttackPattern::Spread(3));
        assert_eq!(
            pattern.0,
            AttackPattern::Spread(3),
            "SurvivalPattern.0 should be AttackPattern::Spread(3)"
        );
        let debug_str = format!("{pattern:?}");
        assert!(
            debug_str.contains("SurvivalPattern"),
            "debug output should contain 'SurvivalPattern', got: {debug_str}"
        );
    }

    // ── Behavior 24: SurvivalTimer has remaining and started fields ──

    #[test]
    fn survival_timer_has_remaining_and_started_fields() {
        let timer = SurvivalTimer {
            remaining: 10.0,
            started:   false,
        };
        assert!(
            (timer.remaining - 10.0).abs() < f32::EPSILON,
            "remaining should be 10.0, got {}",
            timer.remaining
        );
        assert!(!timer.started, "started should be false");
    }

    // Behavior 24 edge case
    #[test]
    fn survival_timer_zero_remaining_started_true() {
        let timer = SurvivalTimer {
            remaining: 0.0,
            started:   true,
        };
        assert!(
            (timer.remaining - 0.0).abs() < f32::EPSILON,
            "remaining should be 0.0, got {}",
            timer.remaining
        );
        assert!(timer.started, "started should be true");
    }

    // ── Behavior 25: BoltImmune is a marker Component with Debug ──

    #[test]
    fn bolt_immune_debug_contains_name() {
        let marker = BoltImmune;
        let debug_str = format!("{marker:?}");
        assert!(
            debug_str.contains("BoltImmune"),
            "debug output should contain 'BoltImmune', got: {debug_str}"
        );
    }
}
