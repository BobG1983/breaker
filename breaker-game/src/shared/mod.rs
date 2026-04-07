//! Shared types used across all domain plugins.
//!
//! Contains passive types only: state enums, dimension components, and playfield
//! configuration. No systems or plugins — those live in domain plugins.

pub mod birthing;
pub(crate) use birthing::Birthing;
pub mod collision_layers;
pub mod color;
pub mod components;
pub mod draw_layer;
pub mod playfield;
pub mod resources;
pub mod rng;
pub mod size;

pub use collision_layers::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, WALL_LAYER};
pub use color::color_from_rgb;
pub use components::{BaseHeight, BaseWidth, NodeScalingFactor};
pub use draw_layer::GameDrawLayer;
pub use playfield::{PlayfieldConfig, PlayfieldDefaults};
pub use resources::RunSeed;
pub use rng::GameRng;

pub use crate::state::types::GameState;
