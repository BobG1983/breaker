use bevy::prelude::*;
use rantzsoft_physics2d::{
    collision_layers::CollisionLayers, plugin::PhysicsSystems, resources::CollisionQuadtree,
};

use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::messages::DamageCell,
    shared::{CELL_LAYER, CleanupOnNodeExit, playing_state::PlayingState},
};

/// Deferred request for an instant area damage burst.
///
/// Spawned by `fire()` as a marker entity, consumed (and despawned) by
/// `process_explode_requests` in the same or next tick. Position is stored
/// in the entity's `Transform` component.
#[derive(Component)]
pub struct ExplodeRequest {
    /// Damage radius in world units.
    pub range: f32,
    /// Multiplicative damage factor applied to `BASE_BOLT_DAMAGE`.
    pub damage_mult: f32,
}

pub fn fire(entity: Entity, range: f32, damage_mult: f32, world: &mut World) {
    let position = world
        .get::<Transform>(entity)
        .map_or(Vec3::ZERO, |t| t.translation);

    world.spawn((
        ExplodeRequest { range, damage_mult },
        Transform::from_translation(position),
        CleanupOnNodeExit,
    ));
}

pub fn reverse(_entity: Entity, world: &mut World) {
    let _ = world;
}

/// Process all pending explode requests: query cells in range, send damage, despawn request.
///
/// For each request, queries the quadtree for cells within range, computes
/// damage as `BASE_BOLT_DAMAGE * damage_mult`, sends [`DamageCell`] for each
/// cell found, then despawns the request entity.
pub fn process_explode_requests(
    mut commands: Commands,
    quadtree: Res<CollisionQuadtree>,
    requests: Query<(Entity, &Transform, &ExplodeRequest)>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);
    for (entity, transform, request) in &requests {
        let position = transform.translation.truncate();
        let damage = BASE_BOLT_DAMAGE * request.damage_mult;
        let candidates =
            quadtree
                .quadtree
                .query_circle_filtered(position, request.range, query_layers);
        for cell in candidates {
            damage_writer.write(DamageCell {
                cell,
                damage,
                source_chip: None,
            });
        }
        commands.entity(entity).despawn();
    }
}

pub fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        process_explode_requests
            .after(PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(PlayingState::Active)),
    );
}
