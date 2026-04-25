//! `apply_piercing_beam_damage` — reads `PiercingBeamEmissionRequested`
//! and emits one `DamageDealt<Cell>` per cell along the beam.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::messages::PiercingBeamEmissionRequested;
use crate::{cells::components::Cell, prelude::*};

type LiveCellQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Position2D), (With<Cell>, Without<Dead>)>;

/// Reads every `PiercingBeamEmissionRequested` produced last tick and emits
/// one `DamageDealt<Cell>` per live cell whose center lies inside the beam's
/// half-width along the forward axis. Damage amount is the raw
/// `req.base_damage` — the pipeline applies boosts/vuln in
/// `ApplyDamageBoosts` / `ApplyVulnerable` later in the same tick.
///
/// Runs in `FixedUpdate` `.in_set(DmgSystems::EmitDamage)`.
///
/// Intentional asymmetry vs `apply_explode_damage`: this consumer iterates
/// the cells query directly (no quadtree). The piercing beam is unbounded
/// along its forward axis — any quadtree rectangle would degenerate to a
/// full-playfield scan with extra tree-traversal overhead.
pub(crate) fn apply_piercing_beam_damage(
    mut reader: MessageReader<PiercingBeamEmissionRequested>,
    cells: LiveCellQuery,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for req in reader.read() {
        // Hoist per-request locals OUT of the per-cell inner loop.
        let normal = Vec2::new(-req.direction.y, req.direction.x);
        let dir = req.direction;
        let origin = req.origin;
        let half_width = req.half_width;

        for (cell_entity, cell_pos) in &cells {
            let offset = cell_pos.0 - origin;
            let along = offset.dot(dir);
            let perp = offset.dot(normal).abs();
            if along < 0.0 || perp > half_width {
                continue;
            }
            writer.write(DamageDealt::<Cell> {
                dealer:        req.dealer,
                attributed_to: None,
                target:        cell_entity,
                amount:        req.base_damage,
                source:        req.source.clone(),
                _marker:       PhantomData,
            });
        }
    }
}
