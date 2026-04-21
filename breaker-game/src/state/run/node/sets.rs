//! Node subdomain system sets.

use bevy::prelude::*;

/// System sets exposed by the node subdomain for ordering constraints.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeSystems {
    /// Cell spawning from the active layout. Systems that need cells
    /// to exist (e.g. `init_clear_remaining`) should run `.after(NodeSystems::Spawn)`.
    Spawn,
    /// Track whether all target cells are cleared.
    TrackCompletion,
    /// Tick the node countdown timer.
    TickTimer,
    /// Apply and reverse time penalties from effect consequences.
    ///
    /// Contains both `apply_time_penalty` (subtracts) and `reverse_time_penalty` (adds back).
    /// `reverse_time_penalty` runs before `apply_time_penalty` within this set.
    /// Runs after `TickTimer`. Systems that read `TimerExpired` should
    /// order `.after(NodeSystems::ApplyTimePenalty)` to see penalty-induced
    /// expirations in the same tick.
    ApplyTimePenalty,
    /// The `init_node_timer` system — initializes the node countdown timer.
    InitTimer,
    /// Cell cleanup on node teardown (`cleanup_on_exit::<NodeState>`). Runs
    /// in `OnEnter(NodeState::Teardown)`. Systems that need to observe a
    /// cleaned-up world (e.g. `effect_v3::triggers::node::on_node_end_occurred`)
    /// should run `.after(NodeSystems::Cleanup)`.
    Cleanup,
    /// The `advance_node` system — increments `NodeOutcome.node_index`,
    /// updates `tier` / `position_in_tier` from the just-completed node's
    /// type. Runs in `OnEnter(RunState::Node)`.
    ///
    /// Protocol systems that need to capture pre-advance state (e.g.
    /// `tier_regression::snapshot_pre_advance_state`) should order
    /// `.before(NodeSystems::AdvanceNode)`. Protocol systems that mutate
    /// `NodeSequence` / `NodeOutcome` using the snapshotted state (e.g.
    /// `tier_regression::apply_tier_regression`) should order
    /// `.after(NodeSystems::AdvanceNode)` so their writes survive the
    /// advance.
    AdvanceNode,
}
