//! Run end state — sub-state of [`RunState::RunEnd`].

use bevy::prelude::*;

use super::RunState;

/// Run end lifecycle state.
///
/// Sub-state of [`RunState::RunEnd`]. Controls the run end screen
/// showing highlights and final stats.
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(RunState = RunState::RunEnd)]
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
    /// Run end teardown — parent `RunState` watches for this.
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
