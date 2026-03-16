//! Shared types used across all domain plugins.
//!
//! Contains passive types only: state enums, cleanup markers, and playfield
//! configuration. No systems or plugins — those live in domain plugins.

use bevy::prelude::*;
use brickbreaker_derive::GameConfig;
use serde::Deserialize;

/// Converts an `[f32; 3]` RGB triple into an sRGB [`Color`].
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn color_from_rgb(rgb: [f32; 3]) -> Color {
    Color::srgb(rgb[0], rgb[1], rgb[2])
}

/// Playfield defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
#[game_config(name = "PlayfieldConfig")]
pub struct PlayfieldDefaults {
    /// Width of the playfield in world units.
    pub width: f32,
    /// Height of the playfield in world units.
    pub height: f32,
    /// RGB values for the background clear color.
    pub background_color_rgb: [f32; 3],
    /// Thickness of boundary walls in world units.
    pub wall_thickness: f32,
}

impl Default for PlayfieldDefaults {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            background_color_rgb: [0.02, 0.01, 0.04],
            wall_thickness: 180.0,
        }
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

    /// Half the wall thickness.
    #[must_use]
    pub fn wall_half_thickness(&self) -> f32 {
        self.wall_thickness / 2.0
    }

    /// Background clear color as a Bevy [`Color`].
    #[must_use]
    pub fn background_color(&self) -> Color {
        color_from_rgb(self.background_color_rgb)
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
    /// Transient 1-frame state between nodes — exits `Playing` then re-enters it.
    NodeTransition,
    /// Timed chip selection between nodes.
    ChipSelect,
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

/// A fade-out animation timer. Entities with this component will have their
/// alpha reduced over `duration` seconds and be despawned when finished.
///
/// Used across domains (bolt-lost text, bump grade text) for floating feedback.
#[derive(Component, Debug)]
pub struct FadeOut {
    /// Remaining time in the fade animation (seconds).
    pub timer: f32,
    /// Total duration of the fade animation (seconds).
    pub duration: f32,
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
/// Added to breaker, run-scoped chips, and accumulated state.
#[derive(Component)]
pub struct CleanupOnRunEnd;

/// The archetype selected for the current run.
///
/// Set at run start; read by `init_archetype` to look up the archetype
/// definition from the registry.
#[derive(Resource, Debug, Clone)]
pub struct SelectedArchetype(pub String);

impl Default for SelectedArchetype {
    fn default() -> Self {
        Self("Aegis".to_owned())
    }
}

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

    #[test]
    fn playfield_defaults_ron_parses() {
        let ron_str = include_str!("../assets/config/defaults.playfield.ron");
        let result: PlayfieldDefaults =
            ron::de::from_str(ron_str).expect("playfield RON should parse");
        assert!(result.width > 0.0);
    }
}
