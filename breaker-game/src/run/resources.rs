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

// -- Highlight detection constants --

/// Node cleared with less than this many seconds remaining.
pub const CLUTCH_CLEAR_THRESHOLD: f32 = 3.0;
/// Number of cells destroyed within a 1-second window to trigger `MassDestruction`.
pub const MASS_DESTRUCTION_COUNT: usize = 10;
/// Consecutive perfect bumps required for a `PerfectStreak` highlight.
pub const PERFECT_STREAK_THRESHOLD: u32 = 5;
/// Node cleared in less than this fraction of allotted time triggers `FastClear`.
pub const FAST_CLEAR_FRACTION: f32 = 0.5;

/// Categories of memorable run moments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HighlightKind {
    /// Node cleared with < 3.0s remaining.
    ClutchClear,
    /// 10+ cells destroyed within a 1-second window.
    MassDestruction,
    /// 5+ consecutive perfect bumps.
    PerfectStreak,
    /// Node cleared in < 50% of allotted time.
    FastClear,
    /// First chip evolution in a run.
    FirstEvolution,
    /// Node cleared without losing a bolt.
    NoDamageNode,
}

/// A memorable moment recorded during the run.
#[derive(Clone, Debug)]
pub struct RunHighlight {
    /// What kind of highlight.
    pub kind: HighlightKind,
    /// Which node (0-indexed) it occurred on.
    pub node_index: u32,
    /// Associated value (e.g., seconds remaining for `ClutchClear`, streak count).
    pub value: f32,
}

/// Cumulative statistics for the current run.
#[derive(Resource, Debug, Clone, Default)]
pub struct RunStats {
    /// Number of nodes cleared.
    pub nodes_cleared: u32,
    /// Total cells destroyed across all nodes.
    pub cells_destroyed: u32,
    /// Total bumps performed.
    pub bumps_performed: u32,
    /// Total perfect bumps.
    pub perfect_bumps: u32,
    /// Total bolts lost.
    pub bolts_lost: u32,
    /// Names of chips collected (in order).
    pub chips_collected: Vec<String>,
    /// Number of chip evolutions performed.
    pub evolutions_performed: u32,
    /// Cumulative simulation time elapsed (seconds).
    pub time_elapsed: f32,
    /// The seed used for this run (0 = not yet captured).
    pub seed: u64,
    /// Memorable moments from the run.
    pub highlights: Vec<RunHighlight>,
}

impl RunStats {
    /// Calculate Flux earned from this run's stats.
    #[must_use]
    pub fn flux_earned(&self) -> u32 {
        let base = self.nodes_cleared * 10;
        let bump_bonus = self.perfect_bumps * 2;
        let evo_bonus = self.evolutions_performed * 25;
        let penalty = self.bolts_lost * 3;
        (base + bump_bonus + evo_bonus).saturating_sub(penalty)
    }
}

/// Per-node intermediate tracking state for highlight detection.
#[derive(Resource, Debug, Clone, Default)]
pub struct HighlightTracker {
    /// Consecutive perfect bumps in the current node.
    pub consecutive_perfect_bumps: u32,
    /// Best perfect streak in the current node.
    pub best_perfect_streak: u32,
    /// Bolts lost in the current node.
    pub node_bolts_lost: u32,
    /// Cell destruction timestamps (simulation time) within the current node.
    pub cell_destroyed_times: Vec<f32>,
    /// Simulation time when the current node started.
    pub node_start_time: f32,
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

    // -- flux_earned calculation --

    #[test]
    fn flux_earned_with_concrete_values() {
        let stats = RunStats {
            nodes_cleared: 5,
            perfect_bumps: 10,
            evolutions_performed: 1,
            bolts_lost: 3,
            ..Default::default()
        };
        // (5*10) + (10*2) + (1*25) - (3*3) = 50 + 20 + 25 - 9 = 86
        assert_eq!(
            stats.flux_earned(),
            86,
            "flux = (5*10) + (10*2) + (1*25) - (3*3) = 86"
        );
    }

    #[test]
    fn flux_earned_floors_at_zero_when_penalty_exceeds_bonuses() {
        let stats = RunStats {
            nodes_cleared: 0,
            perfect_bumps: 0,
            evolutions_performed: 0,
            bolts_lost: 10,
            ..Default::default()
        };
        assert_eq!(
            stats.flux_earned(),
            0,
            "flux should floor at 0, not go negative"
        );
    }
}
