//! Shared types used across all domain plugins.
//!
//! Contains passive types only: state enums, cleanup markers, and playfield
//! configuration. No systems or plugins — those live in domain plugins.

pub mod draw_layer;
pub mod math;

use bevy::prelude::*;
pub use draw_layer::GameDrawLayer;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rantzsoft_defaults::GameConfig;
use serde::Deserialize;

/// Base damage dealt by a bolt hit. Fixed game-design constant.
pub const BASE_BOLT_DAMAGE: f32 = 10.0;

/// Collision layer bitmask: Bolt entities.
pub const BOLT_LAYER: u32 = 1 << 0;
/// Collision layer bitmask: Cell entities.
pub const CELL_LAYER: u32 = 1 << 1;
/// Collision layer bitmask: Wall entities.
pub const WALL_LAYER: u32 = 1 << 2;
/// Collision layer bitmask: Breaker entities.
pub const BREAKER_LAYER: u32 = 1 << 3;

/// Converts an `[f32; 3]` RGB triple into an sRGB [`Color`].
#[must_use]
pub const fn color_from_rgb(rgb: [f32; 3]) -> Color {
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
    /// Fraction of height reserved for the cell zone (0.0 to 1.0).
    #[serde(default = "default_zone_fraction")]
    pub zone_fraction: f32,
}

/// Default value for `zone_fraction` used by serde when the field is absent.
fn default_zone_fraction() -> f32 {
    0.667
}

impl Default for PlayfieldDefaults {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            background_color_rgb: [0.02, 0.01, 0.04],
            wall_thickness: 180.0,
            zone_fraction: default_zone_fraction(),
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
    pub const fn background_color(&self) -> Color {
        color_from_rgb(self.background_color_rgb)
    }

    /// Height of the cell zone in world units.
    #[must_use]
    pub fn cell_zone_height(&self) -> f32 {
        self.height * self.zone_fraction
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

/// Scale factor applied to breaker and bolt dimensions per layout.
///
/// Set at node entry from [`ActiveNodeLayout`]. Multiplies visual size and
/// collision hitboxes — speed is unaffected. Defaults to 1.0 (no scaling).
#[derive(Component, Debug, Clone, Copy)]
pub struct EntityScale(pub f32);

/// Marker component for entities that should be despawned when exiting a node.
///
/// Added to bolt, cells, and other node-scoped entities. Node exit is modeled
/// as exiting [`GameState::Playing`] — any new transitions out of `Playing`
/// must account for the fact that all `CleanupOnNodeExit` entities will be
/// despawned.
#[derive(Component, Default)]
pub struct CleanupOnNodeExit;

/// Marker component for entities that should be despawned when a run ends.
///
/// Added to breaker, run-scoped chips, and accumulated state.
#[derive(Component)]
pub struct CleanupOnRunEnd;

/// Deterministic RNG for gameplay randomness.
///
/// Initialized at app start with a fixed seed (deterministic for tests).
/// Reseeded at run start by `reset_run_state` using [`RunSeed`]
/// (user-controlled) or OS entropy when no seed is set.
#[derive(Resource)]
pub struct GameRng(pub ChaCha8Rng);

impl GameRng {
    /// Creates a `GameRng` with a specific seed. Useful for tests.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::from_seed(0)
    }
}

/// Optional seed for deterministic RNG at run start.
///
/// `None` means random (OS entropy). `Some(n)` seeds the [`GameRng`] with
/// the given value for deterministic replays.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSeed(pub Option<u64>);

/// The breaker selected for the current run.
///
/// Set at run start; read by `init_breaker` to look up the breaker
/// definition from the registry.
#[derive(Resource, Debug, Clone)]
pub struct SelectedBreaker(pub String);

impl Default for SelectedBreaker {
    fn default() -> Self {
        Self("Aegis".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_rng_from_seed_is_deterministic() {
        use rand::Rng;
        let mut rng1 = GameRng::from_seed(42);
        let mut rng2 = GameRng::from_seed(42);
        let v1: f32 = rng1.0.random();
        let v2: f32 = rng2.0.random();
        assert!((v1 - v2).abs() < f32::EPSILON);
    }

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
    fn run_seed_default_is_none() {
        let seed = RunSeed::default();
        assert_eq!(seed.0, None);
    }

    #[test]
    fn run_seed_some_holds_value() {
        let seed = RunSeed(Some(12345));
        assert_eq!(seed.0, Some(12345));
    }

    #[test]
    fn playfield_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.playfield.ron");
        let result: PlayfieldDefaults =
            ron::de::from_str(ron_str).expect("playfield RON should parse");
        assert!(result.width > 0.0);
        assert!((result.zone_fraction - 0.667).abs() < f32::EPSILON);
    }

    #[test]
    fn base_bolt_damage_equals_10() {
        assert!((BASE_BOLT_DAMAGE - 10.0_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn playfield_config_default_includes_zone_fraction() {
        let config = PlayfieldConfig::default();
        assert!(
            (config.zone_fraction - 0.667).abs() < f32::EPSILON,
            "expected zone_fraction ~0.667, got {}",
            config.zone_fraction,
        );
    }

    #[test]
    fn cell_zone_height_computes_fraction_of_height() {
        let config = PlayfieldConfig {
            height: 1080.0,
            zone_fraction: 0.667,
            ..Default::default()
        };
        let expected = 1080.0 * 0.667;
        assert!(
            (config.cell_zone_height() - expected).abs() < 0.01,
            "expected cell_zone_height ~{expected}, got {}",
            config.cell_zone_height(),
        );
    }

    #[test]
    fn collision_layer_constants_are_distinct_powers_of_two() {
        // Each layer constant is a distinct power of 2 (single bit set)
        let layers = [BOLT_LAYER, CELL_LAYER, WALL_LAYER, BREAKER_LAYER];

        // Each is a power of 2
        for &layer in &layers {
            assert!(
                layer.is_power_of_two(),
                "layer 0x{layer:02X} is not a power of 2"
            );
        }

        // All are distinct
        for (i, &a) in layers.iter().enumerate() {
            for (j, &b) in layers.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "layers at index {i} and {j} are not distinct");
                }
            }
        }

        // Specific values
        assert_eq!(BOLT_LAYER, 0x01);
        assert_eq!(CELL_LAYER, 0x02);
        assert_eq!(WALL_LAYER, 0x04);
        assert_eq!(BREAKER_LAYER, 0x08);

        // No overlap: bitwise OR of all equals bitwise sum (no shared bits)
        let combined = BOLT_LAYER | CELL_LAYER | WALL_LAYER | BREAKER_LAYER;
        assert_eq!(combined, 0x0F, "combined layers should be 0x0F");
    }

    #[test]
    fn cell_zone_height_with_zero_fraction_returns_zero() {
        let config = PlayfieldConfig {
            height: 1080.0,
            zone_fraction: 0.0,
            ..Default::default()
        };
        assert!(
            config.cell_zone_height().abs() < f32::EPSILON,
            "expected cell_zone_height 0.0, got {}",
            config.cell_zone_height(),
        );
    }
}
