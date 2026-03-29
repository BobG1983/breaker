use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{
    collision_layers::CollisionLayers, plugin::PhysicsSystems, resources::CollisionQuadtree,
};

use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::messages::DamageCell,
    shared::{CELL_LAYER, CleanupOnNodeExit, playing_state::PlayingState},
};

/// Marks the entity that spawned this shockwave.
#[derive(Component)]
pub struct ShockwaveSource(pub Entity);

/// Current radius of the expanding shockwave.
#[derive(Component)]
pub struct ShockwaveRadius(pub f32);

/// Maximum radius the shockwave expands to before despawning.
#[derive(Component)]
pub struct ShockwaveMaxRadius(pub f32);

/// Expansion speed of the shockwave in world units per second.
#[derive(Component)]
pub struct ShockwaveSpeed(pub f32);

/// Tracks which cell entities have already been damaged by this specific
/// shockwave instance to enforce at-most-once damage.
#[derive(Component, Default)]
pub struct ShockwaveDamaged(pub HashSet<Entity>);

pub fn fire(
    entity: Entity,
    base_range: f32,
    range_per_level: f32,
    stacks: u32,
    speed: f32,
    world: &mut World,
) {
    let effective_range = super::super::effective_range(base_range, range_per_level, stacks);

    let position = world
        .get::<Transform>(entity)
        .map_or(Vec3::ZERO, |t| t.translation);

    world.spawn((
        ShockwaveSource(entity),
        ShockwaveRadius(0.0),
        ShockwaveMaxRadius(effective_range),
        ShockwaveSpeed(speed),
        ShockwaveDamaged::default(),
        Transform::from_translation(position),
        CleanupOnNodeExit,
    ));
}

pub fn reverse(_entity: Entity, world: &mut World) {
    let _ = world;
}

/// Expand shockwave radius by speed * delta time each fixed tick.
pub fn tick_shockwave(time: Res<Time>, mut query: Query<(&mut ShockwaveRadius, &ShockwaveSpeed)>) {
    let dt = time.delta_secs();
    for (mut radius, speed) in &mut query {
        radius.0 += speed.0 * dt;
    }
}

/// Despawn shockwaves that have reached their maximum radius.
pub fn despawn_finished_shockwave(
    mut commands: Commands,
    query: Query<(Entity, &ShockwaveRadius, &ShockwaveMaxRadius)>,
) {
    for (entity, radius, max_radius) in &query {
        if radius.0 >= max_radius.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Damage cells within the expanding shockwave ring.
///
/// For each shockwave, queries the quadtree for cells within the current radius
/// and sends [`DamageCell`] for any cell not already in the [`ShockwaveDamaged`] set.
pub fn apply_shockwave_damage(
    quadtree: Res<CollisionQuadtree>,
    mut shockwaves: Query<
        (&Transform, &ShockwaveRadius, &mut ShockwaveDamaged),
        With<ShockwaveSource>,
    >,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);
    for (transform, radius, mut damaged) in &mut shockwaves {
        if radius.0 <= 0.0 {
            continue;
        }
        let center = transform.translation.truncate();
        let candidates = quadtree
            .quadtree
            .query_circle_filtered(center, radius.0, query_layers);
        for cell in candidates {
            if damaged.0.insert(cell) {
                damage_writer.write(DamageCell {
                    cell,
                    damage: BASE_BOLT_DAMAGE,
                    source_chip: None,
                });
            }
        }
    }
}

pub fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            tick_shockwave,
            apply_shockwave_damage,
            despawn_finished_shockwave,
        )
            .chain()
            .after(PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(PlayingState::Active)),
    );
}
