//! Game phase state — sub-state of [`AppState::Game`].
//!
//! Temporary name `GamePhase` to avoid conflict with the existing 9-variant
//! `GameState`. Will be renamed to `GameState` in Wave 4e when the old enum
//! is deleted.

use bevy::prelude::*;

use super::AppState;

/// Game phase within the application.
///
/// Sub-state of [`AppState::Game`]. Controls whether the player is in
/// menus, an active run, or tearing down.
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(AppState = AppState::Game)]
pub enum GamePhase {
    /// Registry stuffing and second-phase loading.
    #[default]
    Loading,
    /// Menu screens (main menu, breaker select, options, meta).
    Menu,
    /// Active run — nodes, chip select, run end.
    Run,
    /// Game-level teardown (not used in normal flow).
    Teardown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_game_phase_is_loading() {
        assert_eq!(GamePhase::default(), GamePhase::Loading);
    }
}
