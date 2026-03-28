//! Dispatches wall-defined effects to wall entities when spawned.
//!
//! Walls don't currently have RON definitions with effects, but this stub
//! exists for future extensibility and to complete the dispatch pattern.
//! Stub — real implementation in Wave 6.

use bevy::prelude::*;

/// Dispatches effects to wall entities.
/// Stub — real implementation in Wave 6.
pub(crate) fn dispatch_wall_effects(
    mut _commands: Commands,
) {
    // TODO: Wave 6 — if walls gain optional effects, push to BoundEffects
}
