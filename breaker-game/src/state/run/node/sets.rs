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
}
