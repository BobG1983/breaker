//! Breaker domain test infrastructure.
//!
//! Shared building blocks for breaker entity tests. All functions are
//! `pub(crate)` so other domains' tests can spawn breaker entities when
//! needed (e.g., collision system tests).

use bevy::prelude::*;

use crate::{breaker::definition::BreakerDefinition, prelude::*};

/// Standard test definition using `BreakerDefinition::default()` values.
///
/// Prefer this over calling `BreakerDefinition::default()` directly in tests
/// so all test suites share a single canonical factory.
pub(crate) fn default_breaker_definition() -> BreakerDefinition {
    BreakerDefinition::default()
}

/// Spawns a headless primary breaker at the given position via `Breaker::builder()`.
///
/// Uses `world.commands()` + `world.flush()` to apply the spawn immediately.
/// Returns the spawned `Entity` for assertions.
pub(crate) fn spawn_breaker(app: &mut App, x: f32, y: f32) -> Entity {
    let def = default_breaker_definition();
    let world = app.world_mut();
    let entity = Breaker::builder()
        .definition(&def)
        .at_position(Vec2::new(x, y))
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();
    entity
}
