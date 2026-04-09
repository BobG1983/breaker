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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::test_utils::TestAppBuilder;

    #[test]
    fn spawn_breaker_returns_entity_with_breaker_marker() {
        let mut app = TestAppBuilder::new().build();
        let entity = spawn_breaker(&mut app, 100.0, -200.0);
        assert!(app.world().entity(entity).contains::<Breaker>());
    }
}
