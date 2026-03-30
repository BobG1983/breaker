use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{
    collision_layers::CollisionLayers, plugin::PhysicsSystems, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::messages::DamageCell,
    effect::{EffectiveDamageMultiplier, core::EffectSourceChip},
    shared::{CELL_LAYER, CleanupOnNodeExit, playing_state::PlayingState},
};

/// Marker component for shockwave entities.
#[derive(Component)]
pub(crate) struct ShockwaveSource;

/// Current radius of the expanding shockwave.
#[derive(Component)]
pub(crate) struct ShockwaveRadius(pub(crate) f32);

/// Maximum radius the shockwave expands to before despawning.
#[derive(Component)]
pub(crate) struct ShockwaveMaxRadius(pub(crate) f32);

/// Expansion speed of the shockwave in world units per second.
#[derive(Component)]
pub(crate) struct ShockwaveSpeed(pub(crate) f32);

/// Tracks which cell entities have already been damaged by this specific
/// shockwave instance to enforce at-most-once damage.
#[derive(Component, Default)]
pub(crate) struct ShockwaveDamaged(pub(crate) HashSet<Entity>);

/// Damage multiplier snapshotted from the source entity's
/// `EffectiveDamageMultiplier` at fire-time. Default `1.0`.
#[derive(Component)]
pub(crate) struct ShockwaveDamageMultiplier(pub(crate) f32);

/// Query data for [`apply_shockwave_damage`].
type ShockwaveDamageQuery = (
    &'static Position2D,
    &'static ShockwaveRadius,
    &'static mut ShockwaveDamaged,
    Option<&'static ShockwaveDamageMultiplier>,
    Option<&'static EffectSourceChip>,
);

pub(crate) fn fire(
    entity: Entity,
    base_range: f32,
    range_per_level: f32,
    stacks: u32,
    speed: f32,
    source_chip: &str,
    world: &mut World,
) {
    let effective_range = super::super::effective_range(base_range, range_per_level, stacks);

    let position = super::super::entity_position(world, entity);

    let edm = world
        .get::<EffectiveDamageMultiplier>(entity)
        .map_or(1.0, |e| e.0);

    world.spawn((
        ShockwaveSource,
        ShockwaveRadius(0.0),
        ShockwaveMaxRadius(effective_range),
        ShockwaveSpeed(speed),
        ShockwaveDamaged::default(),
        ShockwaveDamageMultiplier(edm),
        EffectSourceChip::new(source_chip),
        Position2D(position),
        CleanupOnNodeExit,
    ));
}

pub(crate) const fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}

/// Expand shockwave radius by speed * delta time each fixed tick.
pub(crate) fn tick_shockwave(
    time: Res<Time>,
    mut query: Query<(&mut ShockwaveRadius, &ShockwaveSpeed)>,
) {
    let dt = time.delta_secs();
    for (mut radius, speed) in &mut query {
        radius.0 = speed.0.mul_add(dt, radius.0);
    }
}

/// Despawn shockwaves that have reached their maximum radius.
pub(crate) fn despawn_finished_shockwave(
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
pub(crate) fn apply_shockwave_damage(
    quadtree: Res<CollisionQuadtree>,
    mut shockwaves: Query<ShockwaveDamageQuery, With<ShockwaveSource>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);
    for (position, radius, mut damaged, damage_mult, esc) in &mut shockwaves {
        if radius.0 <= 0.0 {
            continue;
        }
        let center = position.0;
        let multiplier = damage_mult.map_or(1.0, |m| m.0);
        let source_chip = esc.and_then(EffectSourceChip::source_chip);
        let candidates = quadtree
            .quadtree
            .query_circle_filtered(center, radius.0, query_layers);
        for cell in candidates {
            if damaged.0.insert(cell) {
                damage_writer.write(DamageCell {
                    cell,
                    damage: BASE_BOLT_DAMAGE * multiplier,
                    source_chip: source_chip.clone(),
                });
            }
        }
    }
}

pub(crate) fn register(app: &mut App) {
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
