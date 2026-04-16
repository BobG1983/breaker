//! Detects AABB overlap between salvos and cells. Writes `DamageDealt<Cell>`
//! on overlap. Salvos pass through cells (not despawned).

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    cells::behaviors::survival::salvo::components::{Salvo, SalvoDamage, SalvoSource},
    prelude::*,
};

type AliveCellQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Position2D, &'static Aabb2D), (With<Cell>, Without<Dead>)>;

/// Checks salvo-cell AABB overlap and writes damage messages.
pub(crate) fn salvo_cell_collision(
    salvo_query: Query<(Entity, &Position2D, &Aabb2D, &SalvoDamage, &SalvoSource), With<Salvo>>,
    cell_query: AliveCellQuery,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    for (salvo_entity, salvo_pos, salvo_aabb, salvo_damage, salvo_source) in &salvo_query {
        let salvo_world_aabb = Aabb2D::new(salvo_pos.0, salvo_aabb.half_extents);

        for (cell_entity, cell_pos, cell_aabb) in &cell_query {
            // Skip the source turret
            if cell_entity == salvo_source.0 {
                continue;
            }

            let cell_world_aabb = Aabb2D::new(cell_pos.0, cell_aabb.half_extents);

            if salvo_world_aabb.overlaps(&cell_world_aabb) {
                damage_writer.write(DamageDealt {
                    dealer:      Some(salvo_entity),
                    target:      cell_entity,
                    amount:      salvo_damage.0,
                    source_chip: None,
                    _marker:     PhantomData,
                });
            }
        }
    }
}
