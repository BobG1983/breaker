//! Arc damage jumping between nearby cells — chains between random targets in range.

use bevy::prelude::*;

pub(crate) fn fire(entity: Entity, arcs: u32, range: f32, damage_mult: f32, world: &mut World) {
    let position = world
        .get::<Transform>(entity)
        .map_or(Vec3::ZERO, |t| t.translation);

    // Placeholder: spatial query for nearest cells within range and chaining
    // through `arcs` cells is not yet implemented.
    debug!(
        "ChainLightning fired from {:?} at ({}, {}) — arcs: {}, range: {}, damage_mult: {}",
        entity, position.x, position.y, arcs, range, damage_mult
    );
}

pub(crate) fn reverse(_entity: Entity, world: &mut World) {
    let _ = world;
}

pub(crate) fn register(_app: &mut App) {}
