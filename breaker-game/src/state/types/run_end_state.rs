//! Run end state — sub-state of [`RunPhase::RunEnd`].

use bevy::prelude::*;

use super::RunPhase;

/// Run end lifecycle state.
///
/// Sub-state of [`RunPhase::RunEnd`]. Controls the run end screen
/// showing highlights and final stats.
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(RunPhase = RunPhase::RunEnd)]
pub enum RunEndState {
    /// Run end loading (pass-through).
    #[default]
    Loading,
    /// Animate run end entrance (pass-through until transitions are wired).
    AnimateIn,
    /// Run end screen is active — player reviews highlights.
    Active,
    /// Animate run end exit (pass-through until transitions are wired).
    AnimateOut,
    /// Run end teardown — parent `RunPhase` watches for this.
    Teardown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_run_end_state_is_loading() {
        assert_eq!(RunEndState::default(), RunEndState::Loading);
    }
}
