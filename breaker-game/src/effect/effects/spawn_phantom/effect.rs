use std::collections::HashMap;

use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    time::{Timer, TimerMode},
};
use rand::Rng;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::{
        components::{Bolt, BoltDefinitionRef, BoltLifespan, PiercingRemaining},
        registry::BoltRegistry,
    },
    shared::rng::GameRng,
};

/// Marker for phantom bolt entities.
#[derive(Component)]
pub(crate) struct PhantomBoltMarker;

/// Entity that spawned this phantom bolt.
#[derive(Component)]
pub(crate) struct PhantomOwner(pub(crate) Entity);

/// Monotonically increasing per-owner spawn order for FIFO despawn.
///
/// Lowest value = oldest phantom = first to be despawned when over cap.
#[derive(Component, Debug)]
pub(crate) struct PhantomSpawnOrder(pub(crate) u64);

/// Tracks the next spawn order value per owner entity.
///
/// Lazily initialized on first `fire()` call. Each owner's counter starts
/// at 0 and increments by 1 per phantom spawned.
#[derive(Resource, Default, Debug)]
pub(crate) struct PhantomSpawnCounter(pub(crate) HashMap<Entity, u64>);

pub(crate) fn fire(
    entity: Entity,
    duration: f32,
    max_active: u32,
    _source_chip: &str,
    world: &mut World,
) {
    if world.get_entity(entity).is_err() {
        return;
    }

    // Early return BEFORE accessing PhantomSpawnCounter — tests assert the
    // resource is NOT created when max_active == 0.
    if max_active == 0 {
        return;
    }

    // Enforce max_active cap — despawn oldest phantoms (lowest spawn order) first.
    let mut owned: Vec<(Entity, u64)> = Vec::new();
    {
        let mut query = world.query::<(Entity, &PhantomOwner, &PhantomSpawnOrder)>();
        for (phantom_entity, owner, order) in query.iter(world) {
            if owner.0 == entity {
                owned.push((phantom_entity, order.0));
            }
        }
    }

    // Sort by spawn order ascending — lowest = oldest = first to despawn.
    owned.sort_unstable_by_key(|&(_, order)| order);

    while owned.len() >= max_active as usize {
        if let Some(&(oldest, _)) = owned.first() {
            world.despawn(oldest);
            owned.remove(0);
        }
    }

    // Read-then-increment: get the current counter value, then bump it.
    let counter_value;
    {
        let counter = world.get_resource_or_insert_with(PhantomSpawnCounter::default);
        counter_value = counter.0.get(&entity).copied().unwrap_or(0);
    }
    {
        if let Some(mut counter) = world.get_resource_mut::<PhantomSpawnCounter>() {
            counter.0.insert(entity, counter_value + 1);
        }
    }

    let spawn_pos = super::super::entity_position(world, entity);

    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .map_or_else(|| "Bolt".to_owned(), |r| r.0.clone());
    let Some(bolt_def) = world
        .resource::<BoltRegistry>()
        .get(&def_ref)
        .cloned()
        .or_else(|| world.resource::<BoltRegistry>().get("Bolt").cloned())
    else {
        warn!("default Bolt definition missing");
        return;
    };

    let angle = {
        let mut rng = world.resource_mut::<GameRng>();
        rng.0.random_range(0.0..std::f32::consts::TAU)
    };
    let direction = Vec2::new(angle.cos(), angle.sin());
    let velocity = Velocity2D(direction * bolt_def.base_speed);
    let phantom = {
        let visual = super::super::bolt_visual_handles(world, bolt_def.color_rgb);

        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(spawn_pos)
                .definition(&bolt_def)
                .with_velocity(velocity)
                .extra()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        super::super::insert_bolt_visuals(world, entity, visual);

        entity
    };
    world.entity_mut(phantom).insert((
        PhantomBoltMarker,
        PhantomOwner(entity),
        PhantomSpawnOrder(counter_value),
        BoltLifespan(Timer::from_seconds(duration, TimerMode::Once)),
        PiercingRemaining(u32::MAX),
    ));
}

/// No-op — phantoms self-despawn via `BoltLifespan`/`tick_bolt_lifespan`.
pub(crate) const fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}

/// Registers systems for `SpawnPhantom` effect.
pub(crate) const fn register(_app: &mut App) {}
