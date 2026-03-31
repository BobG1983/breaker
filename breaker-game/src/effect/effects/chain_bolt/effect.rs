use bevy::prelude::*;
use rand::Rng;
use rantzsoft_physics2d::constraint::DistanceConstraint;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::{components::Bolt, resources::BoltConfig},
    shared::{CleanupOnNodeExit, rng::GameRng},
};

/// Marker on a chain bolt entity, pointing to its anchor entity.
#[derive(Component)]
pub(crate) struct ChainBoltMarker(pub(crate) Entity);

/// Marker on an entity that serves as the anchor for a chain bolt.
#[derive(Component)]
pub(crate) struct ChainBoltAnchor;

/// Points to the `DistanceConstraint` entity for this chain bolt.
/// Used during `reverse()` to clean up constraint entities.
#[derive(Component, Debug)]
pub(crate) struct ChainBoltConstraint(pub(crate) Entity);

pub(crate) fn fire(entity: Entity, tether_distance: f32, _source_chip: &str, world: &mut World) {
    let spawn_pos = super::super::entity_position(world, entity);

    let config = world.resource::<BoltConfig>().clone();
    let angle = {
        let mut rng = world.resource_mut::<GameRng>();
        rng.0.random_range(0.0..std::f32::consts::TAU)
    };
    let direction = Vec2::new(angle.cos(), angle.sin());
    let velocity = Velocity2D(direction * config.base_speed);
    let chain_bolt = Bolt::builder()
        .at_position(spawn_pos)
        .config(&config)
        .with_velocity(velocity)
        .extra()
        .spawn(world);
    world.entity_mut(chain_bolt).insert(ChainBoltMarker(entity));

    let constraint = world
        .spawn((
            DistanceConstraint {
                entity_a: entity,
                entity_b: chain_bolt,
                max_distance: tether_distance,
            },
            CleanupOnNodeExit,
        ))
        .id();

    world
        .entity_mut(chain_bolt)
        .insert(ChainBoltConstraint(constraint));

    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.insert(ChainBoltAnchor);
    }
}

pub(crate) fn reverse(
    entity: Entity,
    _tether_distance: f32,
    _source_chip: &str,
    world: &mut World,
) {
    let chain_bolts: Vec<(Entity, Option<Entity>)> = world
        .query::<(Entity, &ChainBoltMarker, Option<&ChainBoltConstraint>)>()
        .iter(world)
        .filter(|(_, marker, _)| marker.0 == entity)
        .map(|(e, _, constraint)| (e, constraint.map(|c| c.0)))
        .collect();

    for (chain_bolt_entity, constraint_entity) in &chain_bolts {
        if let Some(constraint) = constraint_entity {
            world.despawn(*constraint);
        }
        world.despawn(*chain_bolt_entity);
    }

    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.remove::<ChainBoltAnchor>();
    }
}

/// Registers systems for `ChainBolt` effect.
pub(crate) const fn register(_app: &mut App) {}
