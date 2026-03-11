//! Run domain resources.

use bevy::prelude::*;

/// Tracks the current run's progress.
#[derive(Resource, Debug, Clone, Default)]
pub struct RunState {
    /// Zero-indexed node within the current run.
    pub node_index: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_run_state_starts_at_node_zero() {
        let state = RunState::default();
        assert_eq!(state.node_index, 0);
    }
}
