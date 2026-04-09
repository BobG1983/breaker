//! Breaker domain test infrastructure.
//!
//! Shared building blocks for breaker entity tests. All functions are
//! `pub(crate)` so other domains' tests can spawn breaker entities when
//! needed (e.g., collision system tests).

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition};

use crate::breaker::{components::Breaker, definition::BreakerDefinition};

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
/// The builder places the entity at `(0.0, y)`, so the x coordinate is set
/// via a post-spawn `Position2D` + `PreviousPosition` override.
///
/// Returns the spawned `Entity` for assertions.
#[expect(
    dead_code,
    reason = "will be used when remaining domains migrate to breaker::test_utils"
)]
pub(crate) fn spawn_breaker(app: &mut App, x: f32, y: f32) -> Entity {
    let def = default_breaker_definition();
    let world = app.world_mut();
    let entity = Breaker::builder()
        .definition(&def)
        .with_y_position(y)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    // Builder hardcodes x=0.0; override to the requested position.
    let pos = Vec2::new(x, y);
    app.world_mut()
        .entity_mut(entity)
        .insert((Position2D(pos), PreviousPosition(pos)));
    entity
}
