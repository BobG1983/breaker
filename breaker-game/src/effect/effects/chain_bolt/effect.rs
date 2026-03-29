use bevy::prelude::*;
use rantzsoft_physics2d::constraint::DistanceConstraint;
use rantzsoft_spatial2d::components::Position2D;

/// Marker on a chain bolt entity, pointing to its anchor entity.
#[derive(Component)]
pub struct ChainBoltMarker(pub Entity);

/// Marker on an entity that serves as the anchor for a chain bolt.
#[derive(Component)]
pub struct ChainBoltAnchor;

/// Points to the `DistanceConstraint` entity for this chain bolt.
/// Used during `reverse()` to clean up constraint entities.
#[derive(Component, Debug)]
pub struct ChainBoltConstraint(pub Entity);

pub fn fire(entity: Entity, tether_distance: f32, world: &mut World) {
    let spawn_pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);

    let chain_bolt = super::super::spawn_extra_bolt(world, spawn_pos);
    world.entity_mut(chain_bolt).insert(ChainBoltMarker(entity));

    let constraint = world
        .spawn(DistanceConstraint {
            entity_a: entity,
            entity_b: chain_bolt,
            max_distance: tether_distance,
        })
        .id();

    world
        .entity_mut(chain_bolt)
        .insert(ChainBoltConstraint(constraint));

    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.insert(ChainBoltAnchor);
    }
}

pub fn reverse(entity: Entity, _tether_distance: f32, world: &mut World) {
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
pub fn register(_app: &mut App) {}
