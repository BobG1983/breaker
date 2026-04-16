//! Detects AABB overlap between salvos and the breaker. Writes
//! `SalvoImpactBreaker` and despawns the salvo on contact.

use bevy::prelude::*;

use crate::{
    cells::{behaviors::survival::salvo::components::Salvo, messages::SalvoImpactBreaker},
    prelude::*,
};

/// Checks salvo-breaker AABB overlap, writes impact message, despawns salvo.
pub(crate) fn salvo_breaker_collision(
    salvo_query: Query<(Entity, &Position2D, &Aabb2D), With<Salvo>>,
    breaker_query: Query<(Entity, &Position2D, &Aabb2D), With<Breaker>>,
    mut impact_writer: MessageWriter<SalvoImpactBreaker>,
    mut commands: Commands,
) {
    for (salvo_entity, salvo_pos, salvo_aabb) in &salvo_query {
        let salvo_world_aabb = Aabb2D::new(salvo_pos.0, salvo_aabb.half_extents);

        for (breaker_entity, breaker_pos, breaker_aabb) in &breaker_query {
            let breaker_world_aabb = Aabb2D::new(breaker_pos.0, breaker_aabb.half_extents);

            if salvo_world_aabb.overlaps(&breaker_world_aabb) {
                impact_writer.write(SalvoImpactBreaker {
                    salvo:   salvo_entity,
                    breaker: breaker_entity,
                });
                commands.entity(salvo_entity).try_despawn();
                break;
            }
        }
    }
}
