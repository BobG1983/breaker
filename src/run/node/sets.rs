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
}
