//! Detects AABB overlap between salvos and bolts. Despawns the salvo on
//! contact. The bolt is unaffected.

use bevy::prelude::*;

use crate::{cells::behaviors::survival::salvo::components::Salvo, prelude::*};

/// Checks salvo-bolt AABB overlap and despawns absorbed salvos.
pub(crate) fn salvo_bolt_collision(
    salvo_query: Query<(Entity, &Position2D, &Aabb2D), With<Salvo>>,
    bolt_query: Query<(Entity, &Position2D, &Aabb2D), With<Bolt>>,
    mut commands: Commands,
) {
    for (salvo_entity, salvo_pos, salvo_aabb) in &salvo_query {
        let salvo_world_aabb = Aabb2D::new(salvo_pos.0, salvo_aabb.half_extents);

        for (_bolt_entity, bolt_pos, bolt_aabb) in &bolt_query {
            let bolt_world_aabb = Aabb2D::new(bolt_pos.0, bolt_aabb.half_extents);

            if salvo_world_aabb.overlaps(&bolt_world_aabb) {
                commands.entity(salvo_entity).try_despawn();
                break;
            }
        }
    }
}
