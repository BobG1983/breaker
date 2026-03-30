use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_spatial2d::prelude::*;

use crate::{
    bolt::components::Bolt,
    shared::{CleanupOnNodeExit, playing_state::PlayingState},
};

/// Marker for gravity well entities.
#[derive(Component)]
pub struct GravityWellMarker;

/// Configuration and runtime state for a gravity well.
#[derive(Component)]
pub struct GravityWellConfig {
    /// Pull strength applied to bolts within radius.
    pub strength: f32,
    /// Attraction radius in world units.
    pub radius: f32,
    /// Remaining duration in seconds.
    pub remaining: f32,
    /// Entity that spawned this well.
    pub owner: Entity,
}

/// Monotonically increasing per-owner spawn order stamped on each gravity well.
/// Lower values are older. Used for deterministic FIFO despawn ordering.
#[derive(Component)]
pub struct GravityWellSpawnOrder(pub u64);

/// Per-owner counter tracking the next spawn order value to assign.
/// Lazily initialized in `fire()` on first use.
#[derive(Resource, Default)]
pub struct GravityWellSpawnCounter(pub HashMap<Entity, u64>);

pub(crate) fn fire(
    entity: Entity,
    strength: f32,
    duration: f32,
    radius: f32,
    max: u32,
    _source_chip: &str,
    world: &mut World,
) {
    if world.get_entity(entity).is_err() {
        return;
    }

    if max == 0 {
        return;
    }

    let position = super::super::entity_position(world, entity);

    // SCOPE A — read counter value from resource (copy out, drop borrow).
    let counter_value: u64 = {
        let counter_resource = world.get_resource_or_insert_with(GravityWellSpawnCounter::default);
        *counter_resource.0.get(&entity).unwrap_or(&0)
    };

    // SCOPE B — query owned wells with spawn order for deterministic FIFO despawn.
    let to_despawn: Vec<Entity> = {
        let mut query = world.query::<(Entity, &GravityWellConfig, &GravityWellSpawnOrder)>();
        let mut owned: Vec<(Entity, u64)> = query
            .iter(world)
            .filter(|(_, config, _)| config.owner == entity)
            .map(|(e, _, order)| (e, order.0))
            .collect();
        owned.sort_by_key(|(_, order)| *order); // ascending — lowest = oldest

        let mut despawn_list: Vec<Entity> = Vec::new();
        while owned.len() - despawn_list.len() >= max as usize {
            despawn_list.push(owned[despawn_list.len()].0);
        }
        despawn_list
    };

    // Despawn outside the query scope.
    for e in &to_despawn {
        world.despawn(*e);
    }

    // Spawn the new well with its spawn order stamp.
    world.spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength,
            radius,
            remaining: duration,
            owner: entity,
        },
        GravityWellSpawnOrder(counter_value),
        Position2D(position),
        CleanupOnNodeExit,
    ));

    // SCOPE C — re-borrow resource to store incremented counter.
    {
        if let Some(mut counter_resource) = world.get_resource_mut::<GravityWellSpawnCounter>() {
            counter_resource.0.insert(entity, counter_value + 1);
        }
    }
}

/// No-op — gravity wells self-despawn via their duration timer.
pub(crate) const fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}

/// Decrement well timers and despawn expired wells.
pub(crate) fn tick_gravity_well(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut GravityWellConfig), With<GravityWellMarker>>,
) {
    let dt = time.delta_secs();
    for (entity, mut config) in &mut query {
        config.remaining -= dt;
        if config.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Pull bolts toward active gravity wells.
pub(crate) fn apply_gravity_pull(
    time: Res<Time>,
    wells: Query<(&Position2D, &GravityWellConfig), With<GravityWellMarker>>,
    mut bolts: Query<(&Position2D, &mut Velocity2D), With<Bolt>>,
) {
    let dt = time.delta_secs();
    for (well_position, config) in &wells {
        let well_pos = well_position.0;
        for (bolt_position, mut velocity) in &mut bolts {
            let bolt_pos = bolt_position.0;
            let delta = well_pos - bolt_pos;
            let distance = delta.length();
            if distance > 0.0 && distance <= config.radius {
                let direction = delta / distance;
                let pull = config.strength * dt;
                velocity.x = direction.x.mul_add(pull, velocity.x);
                velocity.y = direction.y.mul_add(pull, velocity.y);
            }
        }
    }
}

pub(crate) fn register(app: &mut App) {
    use crate::bolt::BoltSystems;

    app.add_systems(
        FixedUpdate,
        (
            tick_gravity_well,
            apply_gravity_pull.before(BoltSystems::PrepareVelocity),
        )
            .run_if(in_state(PlayingState::Active)),
    );
}
