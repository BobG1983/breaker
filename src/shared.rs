//! Shared types used across all domain plugins.
//!
//! Contains passive types only: state enums, cleanup markers, and playfield
//! configuration. No systems or plugins — those live in domain plugins.

use bevy::prelude::*;

/// Configuration for the playfield dimensions.
///
/// All playfield boundary queries should go through this resource
/// rather than using raw constants.
#[derive(Resource, Debug, Clone)]
pub struct PlayfieldConfig {
    /// Width of the playfield in world units.
    pub width: f32,
    /// Height of the playfield in world units.
    pub height: f32,
    /// RGB values for the background clear color.
    pub background_color_rgb: [f32; 3],
}

impl Default for PlayfieldConfig {
    fn default() -> Self {
        crate::screen::defaults::PlayfieldDefaults::default().into()
    }
}

impl PlayfieldConfig {
    /// Left boundary x coordinate.
    #[must_use]
    pub fn left(&self) -> f32 {
        -self.width / 2.0
    }

    /// Right boundary x coordinate.
    #[must_use]
    pub fn right(&self) -> f32 {
        self.width / 2.0
    }

    /// Bottom boundary y coordinate.
    #[must_use]
    pub fn bottom(&self) -> f32 {
        -self.height / 2.0
    }

    /// Top boundary y coordinate.
    #[must_use]
    pub fn top(&self) -> f32 {
        self.height / 2.0
    }

    /// Background clear color as a Bevy [`Color`].
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn background_color(&self) -> Color {
        Color::srgb(
            self.background_color_rgb[0],
            self.background_color_rgb[1],
            self.background_color_rgb[2],
        )
    }
}

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

    #[test]
    fn playfield_boundaries_are_symmetric() {
        let config = PlayfieldConfig::default();
        assert!((config.left() + config.right()).abs() < f32::EPSILON);
        assert!((config.bottom() + config.top()).abs() < f32::EPSILON);
    }

    #[test]
    fn playfield_dimensions_match_boundaries() {
        let config = PlayfieldConfig::default();
        assert!((config.right() - config.left() - config.width).abs() < f32::EPSILON);
        assert!((config.top() - config.bottom() - config.height).abs() < f32::EPSILON);
    }
}
