//! Game state — sub-state of [`AppState::Game`].

use bevy::prelude::*;

use super::AppState;

/// Game state within the application.
///
/// Sub-state of [`AppState::Game`]. Controls whether the player is in
/// menus, an active run, or tearing down.
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(AppState = AppState::Game)]
pub enum GameState {
    /// Registry stuffing and second-phase loading.
    #[default]
    Loading,
    /// Menu screens (main menu, breaker select, options, meta).
    Menu,
    /// Active run — nodes, chip select, run end.
    Run,
    /// Game-level teardown — reached via quit-from-menu path.
    Teardown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_game_state_is_loading() {
        assert_eq!(GameState::default(), GameState::Loading);
    }
}
