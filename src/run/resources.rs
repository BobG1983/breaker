//! Run domain resources.

use bevy::prelude::*;

/// Outcome of the current run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RunOutcome {
    /// Run is still in progress.
    #[default]
    InProgress,
    /// Player cleared all nodes.
    Won,
    /// Timer expired before clearing.
    Lost,
}

/// Tracks the current run's progress.
#[derive(Resource, Debug, Clone, Default)]
pub struct RunState {
    /// Zero-indexed node within the current run.
    pub node_index: u32,
    /// Current run outcome.
    pub outcome: RunOutcome,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
