use bevy::prelude::*;
use rantzsoft_physics2d::{
    collision_layers::CollisionLayers, plugin::PhysicsSystems, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    cells::messages::DamageCell,
    effect::core::EffectSourceChip,
    shared::{CELL_LAYER, CleanupOnNodeExit, playing_state::PlayingState},
};

/// Deferred request for an instant area damage burst.
///
/// Spawned by `fire()` as a marker entity, consumed (and despawned) by
/// `process_explode_requests` in the same or next tick. Position is stored
/// in the entity's `Position2D` component.
#[derive(Component)]
pub(crate) struct ExplodeRequest {
    /// Damage radius in world units.
    pub range: f32,
    /// Flat damage dealt to each cell in range.
    pub damage: f32,
}

pub(crate) fn fire(entity: Entity, range: f32, damage: f32, source_chip: &str, world: &mut World) {
    let position = super::super::entity_position(world, entity);

    world.spawn((
        ExplodeRequest { range, damage },
        EffectSourceChip::new(source_chip),
        Position2D(position),
        CleanupOnNodeExit,
    ));
}

pub(crate) const fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}

/// Process all pending explode requests: query cells in range, send damage, despawn request.
///
/// For each request, queries the quadtree for cells within range, sends
/// [`DamageCell`] with the request's flat damage for each cell found,
/// then despawns the request entity.
pub(crate) fn process_explode_requests(
    mut commands: Commands,
    quadtree: Res<CollisionQuadtree>,
    requests: Query<(
        Entity,
        &Position2D,
        &ExplodeRequest,
        Option<&EffectSourceChip>,
    )>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);
    for (entity, position, request, esc) in &requests {
        let position = position.0;
        let damage = request.damage;
        let candidates =
            quadtree
                .quadtree
                .query_circle_filtered(position, request.range, query_layers);
        for cell in candidates {
            damage_writer.write(DamageCell {
                cell,
                damage,
                source_chip: esc.and_then(EffectSourceChip::source_chip),
            });
        }
        commands.entity(entity).despawn();
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        process_explode_requests
            .after(PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(PlayingState::Active)),
    );
}
