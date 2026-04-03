//! Menu state — sub-state of [`GameState::Menu`].

use bevy::prelude::*;

use super::GameState;

/// Menu navigation state.
///
/// Sub-state of [`GameState::Menu`]. Controls which menu screen is active.
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(GameState = GameState::Menu)]
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
    /// Menu teardown — parent `GameState` watches for this.
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
