//! Menu state — sub-state of [`GamePhase::Menu`].

use bevy::prelude::*;

use super::GamePhase;

/// Menu navigation state.
///
/// Sub-state of [`GamePhase::Menu`]. Controls which menu screen is active.
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(GamePhase = GamePhase::Menu)]
pub enum MenuState {
    /// Menu loading (pass-through).
    #[default]
    Loading,
    /// Main menu screen.
    Main,
    /// Breaker and seed selection screen.
    StartGame,
    /// Options screen (future).
    Options,
    /// Meta-progression / Flux spending screen (future).
    Meta,
    /// Menu teardown — parent `GamePhase` watches for this.
    Teardown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_menu_state_is_loading() {
        assert_eq!(MenuState::default(), MenuState::Loading);
    }
}
