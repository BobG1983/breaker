//! Game-agnostic 2D physics primitives for Bevy.
//!
//! Provides AABB types, continuous collision detection (ray-vs-AABB),
//! and an incremental quadtree spatial index.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "test assertions use unwrap/expect/panic"
    )
)]

pub mod aabb;
pub mod ccd;
pub mod quadtree;
