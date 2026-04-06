use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    time::{Timer, TimerMode},
};
use rand::Rng;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::{
        components::{Bolt, BoltDefinitionRef, BoltLifespan, ExtraBolt},
        registry::BoltRegistry,
    },
    effect::BoundEffects,
    shared::rng::GameRng,
};

/// Spawns additional bolts from an entity.
///
/// Each spawned bolt gets full physics components, a random velocity at
/// the definition's `base_speed`, and `CleanupOnExit<NodeState>`. If `inherit` is
/// true, `BoundEffects` from the source entity are cloned onto each spawned
/// bolt.
pub(crate) fn fire(
    entity: Entity,
    count: u32,
    lifespan: Option<f32>,
    inherit: bool,
    _source_chip: &str,
    world: &mut World,
) {
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

    let bound_effects = if inherit {
        let mut query = world.query_filtered::<&BoundEffects, (With<Bolt>, Without<ExtraBolt>)>();
        query.iter(world).next().cloned()
    } else {
        None
    };

    // Create visual handles once before the loop — Handle cloning is a cheap Arc increment
    let visual = super::super::bolt_visual_handles(world, bolt_def.color_rgb);

    for _ in 0..count {
        let angle = {
            let mut rng = world.resource_mut::<GameRng>();
            rng.0.random_range(0.0..std::f32::consts::TAU)
        };
        let direction = Vec2::new(angle.cos(), angle.sin());
        let velocity = Velocity2D(direction * bolt_def.base_speed);

        let bolt_entity = {
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
            entity
        };

        super::super::insert_bolt_visuals(
            world,
            bolt_entity,
            visual.as_ref().map(|(m, mat)| (m.clone(), mat.clone())),
        );

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
