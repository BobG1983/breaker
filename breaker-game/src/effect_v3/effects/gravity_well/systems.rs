//! Gravity well systems — tick force application, despawn expired.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::components::*;
use crate::bolt::components::Bolt;

/// Applies gravitational pull to bolts within each well's radius.
pub fn tick_gravity_well(
    well_query: Query<
        (&Position2D, &GravityWellStrength, &GravityWellRadius),
        With<GravityWellSource>,
    >,
    mut bolt_query: Query<(&mut Velocity2D, &Position2D), With<Bolt>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (well_pos, strength, radius) in &well_query {
        for (mut velocity, bolt_pos) in &mut bolt_query {
            let to_well = well_pos.0 - bolt_pos.0;
            let dist = to_well.length();
            if dist > f32::EPSILON && dist <= radius.0 {
                let direction = to_well / dist;
                velocity.0 += direction * strength.0 * dt;
            }
        }
    }
}

/// Despawns gravity wells whose lifetime has expired.
pub fn despawn_expired_wells(
    mut query: Query<(Entity, &mut GravityWellLifetime), With<GravityWellSource>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut lifetime) in &mut query {
        lifetime.0 -= dt;
        if lifetime.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
