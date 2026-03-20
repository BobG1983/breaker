//! Run domain resources.

use bevy::prelude::*;

use crate::run::definition::{DifficultyCurveDefaults, NodeType, TierDefinition};

/// A single node assignment in the generated sequence.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeAssignment {
    /// The type of this node (`Passive`, `Active`, or `Boss`).
    pub node_type: NodeType,
    /// Which tier this node belongs to (0-indexed).
    pub tier_index: u32,
    /// Hit-point multiplier for cells in this node.
    pub hp_mult: f32,
    /// Timer multiplier for this node.
    pub timer_mult: f32,
}

/// The full node sequence for a run.
#[derive(Resource, Debug, Clone)]
pub struct NodeSequence {
    /// Ordered list of node assignments from first to last.
    pub assignments: Vec<NodeAssignment>,
}

/// Runtime resource holding the active difficulty curve.
#[derive(Resource, Debug, Clone)]
pub struct DifficultyCurve {
    /// Ordered list of tier definitions.
    pub tiers: Vec<TierDefinition>,
    /// HP multiplier applied to boss nodes.
    pub boss_hp_mult: f32,
    /// Timer reduction applied after each boss encounter.
    pub timer_reduction_per_boss: f32,
}

impl Default for DifficultyCurve {
    fn default() -> Self {
        Self {
            tiers: vec![],
            boss_hp_mult: 1.0,
            timer_reduction_per_boss: 0.0,
        }
    }
}

impl From<DifficultyCurveDefaults> for DifficultyCurve {
    fn from(defaults: DifficultyCurveDefaults) -> Self {
        Self {
            tiers: defaults.tiers,
            boss_hp_mult: defaults.boss_hp_mult,
            timer_reduction_per_boss: defaults.timer_reduction_per_boss,
        }
    }
}

/// Outcome of the current run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RunOutcome {
    /// Run is still in progress.
    #[default]
    InProgress,
    /// Player cleared all nodes.
    Won,
    /// Timer expired before clearing all nodes.
    TimerExpired,
    /// All lives depleted (Aegis archetype).
    LivesDepleted,
}

/// Tracks the current run's progress.
#[derive(Resource, Debug, Clone, Default)]
pub struct RunState {
    /// Zero-indexed node within the current run.
    pub node_index: u32,
    /// Current run outcome.
    pub outcome: RunOutcome,
    /// Set to `true` when `handle_node_cleared` queues a state transition this
    /// frame. Checked by `handle_timer_expired` to yield to the node-cleared
    /// transition (player wins tie-frame: clear beats loss).
    pub transition_queued: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::definition::TierNodeCount;

    #[test]
    fn default_run_state_starts_at_node_zero() {
        let state = RunState::default();
        assert_eq!(state.node_index, 0);
    }

    #[test]
    fn default_outcome_is_in_progress() {
        let state = RunState::default();
        assert_eq!(state.outcome, RunOutcome::InProgress);
    }

    // -- DifficultyCurve From conversion --

    #[test]
    fn difficulty_curve_from_defaults_copies_all_fields() {
        let defaults = DifficultyCurveDefaults {
            tiers: vec![
                TierDefinition {
                    nodes: TierNodeCount::Fixed(3),
                    active_ratio: 0.0,
                    hp_mult: 1.0,
                    timer_mult: 1.0,
                    introduced_cells: vec![],
                },
                TierDefinition {
                    nodes: TierNodeCount::Range(4, 6),
                    active_ratio: 0.5,
                    hp_mult: 1.5,
                    timer_mult: 0.8,
                    introduced_cells: vec!['T'],
                },
            ],
            boss_hp_mult: 3.0,
            timer_reduction_per_boss: 0.1,
        };

        let curve = DifficultyCurve::from(defaults);

        assert_eq!(curve.tiers.len(), 2, "tier count should match");
        assert!(
            (curve.boss_hp_mult - 3.0).abs() < f32::EPSILON,
            "boss_hp_mult should be 3.0, got {}",
            curve.boss_hp_mult
        );
        assert!(
            (curve.timer_reduction_per_boss - 0.1).abs() < f32::EPSILON,
            "timer_reduction_per_boss should be 0.1, got {}",
            curve.timer_reduction_per_boss
        );
        // Spot-check first tier fields
        assert!(
            (curve.tiers[0].hp_mult - 1.0).abs() < f32::EPSILON,
            "first tier hp_mult should be 1.0"
        );
        assert!(
            (curve.tiers[0].active_ratio - 0.0).abs() < f32::EPSILON,
            "first tier active_ratio should be 0.0"
        );
    }
}
