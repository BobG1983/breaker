//! Fast-expanding beam rectangle in the bolt's velocity direction.

use bevy::prelude::*;

pub(crate) fn fire(entity: Entity, damage_mult: f32, width: f32, world: &mut World) {
    let position = world
        .get::<Transform>(entity)
        .map_or(Vec3::ZERO, |t| t.translation);

    // Placeholder: beam cast along entity velocity direction, damaging all
    // intersecting cells, is not yet implemented.
    debug!(
        "PiercingBeam fired from {:?} at ({}, {}) — damage_mult: {}, width: {}",
        entity, position.x, position.y, damage_mult, width
    );
}

pub(crate) fn reverse(_entity: Entity, world: &mut World) {
    let _ = world;
}

pub(crate) fn register(_app: &mut App) {}
