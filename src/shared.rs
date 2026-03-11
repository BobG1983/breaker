//! Shared types used across all domain plugins.
//!
//! Contains passive types only: state enums, cleanup markers, and playfield
//! constants. No systems or plugins — those live in domain plugins.

use bevy::prelude::*;

/// Width of the playfield in world units.
pub const PLAYFIELD_WIDTH: f32 = 800.0;

/// Height of the playfield in world units.
pub const PLAYFIELD_HEIGHT: f32 = 600.0;

/// Left boundary of the playfield (x coordinate).
pub const PLAYFIELD_LEFT: f32 = -PLAYFIELD_WIDTH / 2.0;

/// Right boundary of the playfield (x coordinate).
pub const PLAYFIELD_RIGHT: f32 = PLAYFIELD_WIDTH / 2.0;

/// Bottom boundary of the playfield (y coordinate).
pub const PLAYFIELD_BOTTOM: f32 = -PLAYFIELD_HEIGHT / 2.0;

/// Top boundary of the playfield (y coordinate).
pub const PLAYFIELD_TOP: f32 = PLAYFIELD_HEIGHT / 2.0;

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
    /// Timed upgrade selection between nodes.
    UpgradeSelect,
    /// Run end screen — win or lose.
    RunEnd,
    /// Between-run Flux spending and meta-progression.
    MetaProgression,
}

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

/// Marker component for entities that should be despawned when exiting a node.
///
/// Added to bolt, cells, and other node-scoped entities. Node exit is modeled
/// as exiting [`GameState::Playing`] — any new transitions out of `Playing`
/// must account for the fact that all `CleanupOnNodeExit` entities will be
/// despawned.
#[derive(Component)]
pub struct CleanupOnNodeExit;

/// Marker component for entities that should be despawned when a run ends.
///
/// Added to breaker, run-scoped upgrades, and accumulated state.
#[derive(Component)]
pub struct CleanupOnRunEnd;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_game_state_is_loading() {
        assert_eq!(GameState::default(), GameState::Loading);
    }

    #[test]
    fn default_playing_state_is_active() {
        assert_eq!(PlayingState::default(), PlayingState::Active);
    }
}
