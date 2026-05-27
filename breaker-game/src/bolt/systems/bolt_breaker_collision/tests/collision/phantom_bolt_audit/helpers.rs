use bevy::prelude::*;

pub(super) use crate::bolt::systems::bolt_breaker_collision::tests::{
    CapturedHitPairs, collect_breaker_hit_pairs, default_bolt_radius, default_breaker_height,
    spawn_bolt, spawn_breaker_at, spawn_phantom_breaker_at, test_app, tick,
};

/// Spawns a normal bolt at `(x, y)` with velocity `(vx, vy)` and stamps `PhantomBolt`
/// on the entity. The bolt has no other phantom components — this is the minimal
/// marker insertion that flips its "phantom-ness" for collision-audit equivalence tests.
pub(super) fn spawn_phantom_bolt(app: &mut App, x: f32, y: f32, vx: f32, vy: f32) -> Entity {
    let entity = spawn_bolt(app, x, y, vx, vy);
    app.world_mut()
        .entity_mut(entity)
        .insert(crate::bolt::components::PhantomBolt);
    entity
}
