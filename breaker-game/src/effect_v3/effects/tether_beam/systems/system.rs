//! Tether beam systems — tick damage, cleanup dead targets.

use bevy::{ecs::entity::Entities, prelude::*};
use rantzsoft_spatial2d::components::Position2D;

use super::super::components::*;
use crate::{
    cells::components::Cell,
    effect_v3::components::EffectSourceChip,
    shared::death_pipeline::{DamageDealt, Dead},
};

/// Alive cell lookup — entity + position, excludes dead cells.
type AliveCellQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Position2D), (With<Cell>, Without<Dead>)>;

/// Applies damage to cells intersecting the beam line between the two bolts.
pub(crate) fn tick_tether_beam(
    beam_query: Query<(
        Entity,
        &TetherBeamSource,
        &TetherBeamDamage,
        &TetherBeamWidth,
        Option<&EffectSourceChip>,
    )>,
    position_query: Query<&Position2D>,
    cell_query: AliveCellQuery,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    for (beam_entity, source, beam_damage, &TetherBeamWidth(beam_width), chip) in &beam_query {
        let pos_a = position_query.get(source.bolt_a).map(|p| p.0);
        let pos_b = position_query.get(source.bolt_b).map(|p| p.0);

        let (Ok(a), Ok(b)) = (pos_a, pos_b) else {
            continue;
        };

        let beam_dir = b - a;
        let beam_len = beam_dir.length();
        if beam_len < f32::EPSILON {
            continue;
        }
        let beam_norm = beam_dir / beam_len;
        let perp = Vec2::new(-beam_norm.y, beam_norm.x);

        for (cell_entity, cell_pos) in &cell_query {
            let offset = cell_pos.0 - a;
            let along = offset.dot(beam_norm);
            let across = offset.dot(perp).abs();

            if along >= 0.0 && along <= beam_len && across <= beam_width {
                damage_writer.write(DamageDealt {
                    dealer:      Some(beam_entity),
                    target:      cell_entity,
                    amount:      beam_damage.0,
                    source_chip: chip.and_then(|c| c.0.clone()),
                    _marker:     std::marker::PhantomData,
                });
            }
        }
    }
}

/// Removes tether beam entities when either endpoint bolt no longer exists.
pub(crate) fn cleanup_tether_beams(
    beam_query: Query<(Entity, &TetherBeamSource)>,
    entities: &Entities,
    mut commands: Commands,
) {
    for (beam_entity, source) in &beam_query {
        if !entities.contains(source.bolt_a) || !entities.contains(source.bolt_b) {
            commands.entity(beam_entity).despawn();
        }
    }
}
