use bevy::prelude::*;

use super::shockwave::{ShockwaveMaxRadius, ShockwaveRadius, ShockwaveSource, ShockwaveSpeed};
use crate::bolt::components::Bolt;

pub(crate) fn fire(
    entity: Entity,
    base_range: f32,
    range_per_level: f32,
    stacks: u32,
    speed: f32,
    world: &mut World,
) {
    let extra_stacks = u16::try_from(stacks.saturating_sub(1)).unwrap_or(u16::MAX);
    let effective_range = base_range + f32::from(extra_stacks) * range_per_level;

    // Collect bolt positions first to avoid borrow conflicts.
    let bolt_positions: Vec<Vec3> = {
        let mut query = world.query_filtered::<&Transform, With<Bolt>>();
        query.iter(world).map(|t| t.translation).collect()
    };

    for position in bolt_positions {
        world.spawn((
            ShockwaveSource(entity),
            ShockwaveRadius(0.0),
            ShockwaveMaxRadius(effective_range),
            ShockwaveSpeed(speed),
            Transform::from_translation(position),
        ));
    }
}

pub(crate) fn reverse(_entity: Entity, world: &mut World) {
    let _ = world;
}

pub(crate) fn register(_app: &mut App) {}
