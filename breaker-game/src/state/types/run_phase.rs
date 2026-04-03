//! Run phase state — sub-state of [`GameState::Run`].
//!
//! Temporary name `RunPhase` to avoid conflict with the existing `RunState`
//! resource. Will be renamed to `RunState` in Wave 4e when the resource
//! is renamed.

use bevy::prelude::*;

use super::GameState;

/// Run lifecycle phase.
///
/// Sub-state of [`GameState::Run`]. Controls the progression through
/// a single run: loading, setup, nodes, chip select, and run end.
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(GameState = GameState::Run)]
pub enum RunPhase {
    /// Run initialization — reset state, generate node sequence, capture seed.
    #[default]
    Loading,
    /// Spawn breaker and bolt (`setup_run` runs on exit).
    Setup,
    /// Active node gameplay — `NodeState` sub-states take over.
    Node,
    /// Chip selection between nodes.
    ChipSelect,
    /// Run end screen — win or lose.
    RunEnd,
    /// Run teardown — parent `GameState` watches for this.
    Teardown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_run_phase_is_loading() {
        assert_eq!(RunPhase::default(), RunPhase::Loading);
    }
}
