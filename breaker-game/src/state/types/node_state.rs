//! Node state — sub-state of [`RunState::Node`].

use bevy::prelude::*;

use super::RunState;

/// Node lifecycle state.
///
/// Sub-state of [`RunState::Node`]. Controls the progression through
/// a single node: loading, animate-in, active gameplay, animate-out,
/// and teardown.
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(RunState = RunState::Node)]
pub enum NodeState {
    /// Spawn cells, walls, HUD. Apply node scaling.
    #[default]
    Loading,
    /// Animate node entrance (pass-through until transitions are wired).
    AnimateIn,
    /// Active gameplay — physics, timers, and input all active.
    Playing,
    /// Animate node exit (pass-through until transitions are wired).
    AnimateOut,
    /// Node teardown — cleanup entities, parent `RunState` watches for this.
    Teardown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_node_state_is_loading() {
        assert_eq!(NodeState::default(), NodeState::Loading);
    }
}
