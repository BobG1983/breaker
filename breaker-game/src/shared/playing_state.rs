//! Sub-state of `GameState::Playing`.

use bevy::prelude::*;

use super::game_state::GameState;

/// Sub-state of [`GameState::Playing`].
///
/// Only exists when `GameState::Playing` is active. Systems that should
/// freeze during pause use `run_if(in_state(PlayingState::Active))`.
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(GameState = GameState::Playing)]
pub enum PlayingState {
    /// Normal gameplay — physics, timers, and input all active.
    #[default]
    Active,
    /// Game paused — all gameplay systems frozen.
    Paused,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_playing_state_is_active() {
        assert_eq!(PlayingState::default(), PlayingState::Active);
    }
}
