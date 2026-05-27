//! Shared test helpers for phantom bolt cell-collision tests.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{BaseSpeed, GlobalPosition2D, Spatial2D};

use crate::{
    bolt::components::{BoltBaseDamage, PhantomBolt, PhantomDamagedCells, PhantomDedupKey},
    prelude::*,
    shared::size::BaseRadius,
};

// ── Phantom spawner (Wave-1 vocabulary) ────────────────────────────────────

/// Spawns an entity that is itself a full bolt, then stamps it as a phantom
/// by directly inserting the phantom triple via `world_mut()`. Direct insertion
/// is synchronous — the components are present before any `app.update()` runs,
/// so `bolt_cell_collision` (`FixedUpdate`) sees the marker on the first tick.
///
/// `owner` parameter removed — legacy components removed in Wave 5; they
/// are not part of the Wave 1 component vocabulary and must not be inserted
/// here.
pub(super) fn spawn_phantom_bolt_for_cell_collision(
    app: &mut App,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    radius: f32,
) -> Entity {
    let pos = Vec2::new(x, y);
    let velocity = Vec2::new(vx, vy);
    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            Velocity2D(velocity),
            BaseSpeed(velocity.length()),
            BoltBaseDamage(10.0),
            BaseRadius(radius),
            CollisionLayers::new(
                BOLT_LAYER,
                BOLT_LAYER | WALL_LAYER | BREAKER_LAYER | CELL_LAYER,
            ),
        ))
        .id();
    app.world_mut().entity_mut(entity).insert((
        PhantomBolt,
        PhantomDedupKey::Bolt(entity),
        PhantomDamagedCells::default(),
    ));
    entity
}
