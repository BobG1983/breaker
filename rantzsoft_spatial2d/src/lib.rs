//! Game-agnostic 2D spatial transform plugin for Bevy.
//!
//! Provides canonical `Position2D`, `Rotation2D`, `Scale2D` components,
//! a `DrawLayer` trait for Z ordering, fixed-timestep interpolation,
//! parent/child propagation with Absolute/Relative modes, and visual offsets.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "test assertions use unwrap/expect/panic"
    )
)]

pub mod components;
pub mod draw_layer;
pub mod plugin;
pub mod propagation;
pub mod systems;
