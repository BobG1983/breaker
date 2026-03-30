use bevy::{
    prelude::*,
    time::{Timer, TimerMode},
};

use crate::bolt::components::{BoltLifespan, PiercingRemaining};

/// Marker for phantom bolt entities.
#[derive(Component)]
pub(crate) struct PhantomBoltMarker;

/// Entity that spawned this phantom bolt.
#[derive(Component)]
pub(crate) struct PhantomOwner(pub(crate) Entity);

pub(crate) fn fire(
    entity: Entity,
    duration: f32,
    max_active: u32,
    _source_chip: &str,
    world: &mut World,
) {
    if max_active == 0 {
        return;
    }

    // Enforce max_active cap — despawn oldest phantoms for this owner if at cap.
    let mut owned: Vec<Entity> = Vec::new();
    {
        let mut query = world.query::<(Entity, &PhantomOwner)>();
        for (phantom_entity, owner) in query.iter(world) {
            if owner.0 == entity {
                owned.push(phantom_entity);
            }
        }
    }

    while owned.len() >= max_active as usize {
        if let Some(oldest) = owned.first().copied() {
            world.despawn(oldest);
            owned.remove(0);
        }
    }

    let spawn_pos = super::super::entity_position(world, entity);

    let phantom = super::super::spawn_extra_bolt(world, spawn_pos);
    world.entity_mut(phantom).insert((
        PhantomBoltMarker,
        PhantomOwner(entity),
        BoltLifespan(Timer::from_seconds(duration, TimerMode::Once)),
        PiercingRemaining(u32::MAX),
    ));
}

/// No-op — phantoms self-despawn via `BoltLifespan`/`tick_bolt_lifespan`.
pub(crate) const fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}

/// Registers systems for `SpawnPhantom` effect.
pub(crate) const fn register(_app: &mut App) {}
