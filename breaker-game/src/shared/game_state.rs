//! Top-level game state machine.

use bevy::prelude::*;

/// Top-level game state machine.
///
/// Controls which systems run and which UI is displayed.
/// Starts in [`GameState::Loading`] and transitions to [`GameState::MainMenu`]
/// once all assets are loaded.
#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameState {
    /// Initial state — preload all assets, build registries.
    #[default]
    Loading,
    /// Main menu screen.
    MainMenu,
    /// Pre-run setup — breaker and seed selection.
    RunSetup,
    /// Active gameplay within a node. See [`PlayingState`] for sub-states.
    Playing,
    /// Animated transition out of a completed node (clear animation).
    TransitionOut,
    /// Timed chip selection between nodes.
    ChipSelect,
    /// Animated transition into the next node (load animation).
    TransitionIn,
    /// Run end screen — win or lose.
    RunEnd,
    /// Between-run Flux spending and meta-progression.
    MetaProgression,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_game_state_is_loading() {
        assert_eq!(GameState::default(), GameState::Loading);
    }
}
