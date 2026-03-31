use bevy::{
    prelude::*,
    time::{Timer, TimerMode},
};
use rand::Rng;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::{
        components::{Bolt, BoltLifespan, ExtraBolt},
        resources::BoltConfig,
    },
    effect::BoundEffects,
    shared::rng::GameRng,
};

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
    let spawn_pos = super::super::entity_position(world, entity);
    let config = world.resource::<BoltConfig>().clone();

    let bound_effects = if inherit {
        let mut query = world.query_filtered::<&BoundEffects, (With<Bolt>, Without<ExtraBolt>)>();
        query.iter(world).next().cloned()
    } else {
        None
    };

    for _ in 0..count {
        let angle = {
            let mut rng = world.resource_mut::<GameRng>();
            rng.0.random_range(0.0..std::f32::consts::TAU)
        };
        let direction = Vec2::new(angle.cos(), angle.sin());
        let velocity = Velocity2D(direction * config.base_speed);
        let bolt_entity = Bolt::builder()
            .at_position(spawn_pos)
            .config(&config)
            .with_velocity(velocity)
            .extra()
            .spawn(world);

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
