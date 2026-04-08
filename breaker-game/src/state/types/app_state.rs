//! Top-level application state.

use bevy::prelude::*;

/// Top-level application state.
///
/// Controls the highest-level lifecycle: asset loading, game,
/// and application teardown.
#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AppState {
    /// Initial state — load disk assets, show progress bar.
    #[default]
    Loading,
    /// Game is running — sub-states handle menus, runs, nodes.
    Game,
    /// Application teardown — reached via quit-from-menu path.
    Teardown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_app_state_is_loading() {
        assert_eq!(AppState::default(), AppState::Loading);
    }
}
