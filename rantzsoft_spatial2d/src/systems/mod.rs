//! Spatial systems: save previous state, compute globals, derive transform, apply velocity.

pub mod apply_velocity;
pub mod compute_globals;
pub mod derive_transform;
pub mod propagate_position;
pub mod propagate_rotation;
pub mod propagate_scale;
pub mod save_previous;
