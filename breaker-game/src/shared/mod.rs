//! Shared types used across all domain plugins.
//!
//! Contains passive types only: state enums, cleanup markers, and playfield
//! configuration. No systems or plugins — those live in domain plugins.

pub mod collision_layers;
pub mod color;
pub mod components;
pub mod draw_layer;
pub mod game_state;
pub mod playfield;
pub mod playing_state;
pub mod resources;
pub mod rng;

pub use collision_layers::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, WALL_LAYER};
pub use color::color_from_rgb;
pub use components::{CleanupOnNodeExit, CleanupOnRunEnd, NodeScalingFactor};
pub use draw_layer::GameDrawLayer;
pub use game_state::GameState;
pub use playfield::{PlayfieldConfig, PlayfieldDefaults};
pub use playing_state::PlayingState;
pub use resources::RunSeed;
pub use rng::GameRng;
