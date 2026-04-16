//! Despawns salvos that exit playfield boundaries.

use bevy::prelude::*;

use crate::{cells::behaviors::survival::salvo::components::Salvo, prelude::*};

/// Checks salvo positions against `PlayfieldConfig` boundaries and despawns
/// any that have left the playfield.
pub(crate) fn salvo_wall_collision(
    salvo_query: Query<(Entity, &Position2D, &Aabb2D), With<Salvo>>,
    playfield: Res<PlayfieldConfig>,
    mut commands: Commands,
) {
    let half_w = playfield.width / 2.0;
    let half_h = playfield.height / 2.0;

    for (salvo_entity, salvo_pos, salvo_aabb) in &salvo_query {
        let min_x = salvo_pos.0.x - salvo_aabb.half_extents.x;
        let max_x = salvo_pos.0.x + salvo_aabb.half_extents.x;
        let min_y = salvo_pos.0.y - salvo_aabb.half_extents.y;
        let max_y = salvo_pos.0.y + salvo_aabb.half_extents.y;

        if min_y <= -half_h || max_y >= half_h || min_x <= -half_w || max_x >= half_w {
            commands.entity(salvo_entity).try_despawn();
        }
    }
}
