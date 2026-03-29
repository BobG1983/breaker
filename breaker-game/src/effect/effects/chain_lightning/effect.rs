//! Arc damage jumping between nearby cells — chains between random targets in range.

use std::collections::HashSet;

use bevy::prelude::*;
use rand::prelude::IndexedRandom;
use rantzsoft_physics2d::{
    collision_layers::CollisionLayers, plugin::PhysicsSystems, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D};

use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::messages::DamageCell,
    shared::{CELL_LAYER, CleanupOnNodeExit, GameRng, PlayingState},
};

/// Deferred request for chain lightning arc damage.
///
/// Spawned by `fire()` with pre-computed targets from the quadtree walk,
/// consumed (and despawned) by `process_chain_lightning` in the same or next tick.
#[derive(Component)]
pub struct ChainLightningRequest {
    /// Pre-computed list of (cell entity, damage) pairs to apply.
    pub targets: Vec<(Entity, f32)>,
    /// Origin position for VFX.
    pub source: Vec2,
}

pub(crate) fn fire(entity: Entity, arcs: u32, range: f32, damage_mult: f32, world: &mut World) {
    let position = world
        .get::<Position2D>(entity)
        .map(|p| p.0)
        .or_else(|| {
            world
                .get::<Transform>(entity)
                .map(|t| t.translation.truncate())
        })
        .unwrap_or(Vec2::ZERO);

    if arcs == 0 {
        return;
    }

    let query_layers = CollisionLayers::new(0, CELL_LAYER);
    let damage = BASE_BOLT_DAMAGE * damage_mult;
    let mut targets = Vec::new();
    let mut hit_set = HashSet::new();
    let mut current_pos = position;

    // Zero or negative range cannot find any targets — skip the quadtree walk entirely.
    if range > 0.0 {
        for _ in 0..arcs {
            // Scope for immutable borrow of quadtree
            let candidates: Vec<Entity> = {
                let qt = world.resource::<CollisionQuadtree>();
                qt.quadtree
                    .query_circle_filtered(current_pos, range, query_layers)
            };

            let available: Vec<Entity> = candidates
                .into_iter()
                .filter(|e| !hit_set.contains(e))
                .collect();

            if available.is_empty() {
                break;
            }

            let target = {
                let mut rng = world.resource_mut::<GameRng>();
                // unwrap OK: `available` is checked non-empty above (early break on empty)
                *available.choose(&mut rng.0).unwrap()
            };

            hit_set.insert(target);
            targets.push((target, damage));

            // Get next position from the selected cell
            current_pos = world
                .get::<GlobalPosition2D>(target)
                .map_or(current_pos, |p| p.0);
        }
    }

    world.spawn((
        ChainLightningRequest {
            targets,
            source: position,
        },
        CleanupOnNodeExit,
    ));
}

pub(crate) fn reverse(_entity: Entity, world: &mut World) {
    let _ = world;
}

/// Process all pending chain lightning requests: send damage for each target, despawn request.
///
/// Iterates `ChainLightningRequest` entities, sends [`DamageCell`] for each
/// pre-computed target, then despawns the request entity.
pub fn process_chain_lightning(
    mut commands: Commands,
    requests: Query<(Entity, &ChainLightningRequest)>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    for (entity, request) in &requests {
        for &(cell, damage) in &request.targets {
            damage_writer.write(DamageCell {
                cell,
                damage,
                source_chip: None,
            });
        }
        commands.entity(entity).despawn();
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        process_chain_lightning
            .after(PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(PlayingState::Active)),
    );
}
