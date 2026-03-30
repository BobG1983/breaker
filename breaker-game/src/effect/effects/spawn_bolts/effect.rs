use bevy::{
    prelude::*,
    time::{Timer, TimerMode},
};
use rantzsoft_spatial2d::components::Position2D;

use crate::{bolt::components::BoltLifespan, effect::BoundEffects};

/// Spawns additional bolts from an entity.
///
/// Each spawned bolt gets full physics components, a random velocity at
/// `BoltConfig.base_speed`, and `CleanupOnNodeExit`. If `inherit` is true,
/// `BoundEffects` from the source entity are cloned onto each spawned bolt.
pub(crate) fn fire(
    entity: Entity,
    count: u32,
    lifespan: Option<f32>,
    inherit: bool,
    _source_chip: &str,
    world: &mut World,
) {
    let spawn_pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);

    let bound_effects = if inherit {
        world.get::<BoundEffects>(entity).cloned()
    } else {
        None
    };

    for _ in 0..count {
        let bolt_entity = super::super::spawn_extra_bolt(world, spawn_pos);

        if let Some(duration) = lifespan {
            world
                .entity_mut(bolt_entity)
                .insert(BoltLifespan(Timer::from_seconds(duration, TimerMode::Once)));
        }

        if let Some(ref effects) = bound_effects {
            world.entity_mut(bolt_entity).insert(effects.clone());
        }
    }
}

/// No-op — bolts persist independently once spawned.
pub(crate) const fn reverse(
    _entity: Entity,
    _count: u32,
    _lifespan: Option<f32>,
    _inherit: bool,
    _source_chip: &str,
    _world: &mut World,
) {
}

/// Registers systems for `SpawnBolts` effect.
pub(crate) const fn register(_app: &mut App) {}
