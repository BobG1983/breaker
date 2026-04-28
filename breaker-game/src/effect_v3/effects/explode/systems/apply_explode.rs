//! `apply_explode_damage` — reads `ExplodeEmissionRequested` and emits one
//! `DamageDealt<Cell>` per cell within the request's radius.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_physics2d::resources::CollisionQuadtree;

use super::super::messages::ExplodeEmissionRequested;
use crate::{cells::components::Cell, prelude::*};

type LiveCellQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Position2D), (With<Cell>, Without<Dead>)>;

/// Reads every `ExplodeEmissionRequested` produced last tick and emits one
/// `DamageDealt<Cell>` per live cell whose center lies within `req.radius` of
/// `req.center`. Damage amount is the raw `req.base_damage` — the pipeline
/// applies boosts/vuln in `ApplyDamageBoosts` / `ApplyVulnerable` later in
/// the same tick. Forwarding `req.source` into `DamageDealt.source` is what
/// enables `DamageBoostStack` and `VulnerableStack` entries registered with
/// `add_filtered` / `add_one_shot_filtered` to scope their contribution to
/// this emission type.
///
/// Runs in `FixedUpdate` `.in_set(DmgSystems::EmitDamage)` so the emitted
/// damage flows through the full pipeline (boosts → mutators → vuln →
/// apply → ripple) on the same tick.
///
/// `quadtree` is `Option<Res<...>>` so test apps that register `EffectV3Plugin`
/// without inserting `CollisionQuadtree` (i.e. apps that don't run
/// `RantzPhysics2dPlugin`) don't panic — the system simply drains the reader
/// and returns. Mirrors the defensive pattern in
/// `mutators::hazards::volatility::system::reset_volatility_on_damage`.
pub(in crate::effect_v3) fn apply_explode_damage(
    mut reader: MessageReader<ExplodeEmissionRequested>,
    quadtree: Option<Res<CollisionQuadtree>>,
    cells: LiveCellQuery,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    let Some(quadtree) = quadtree else {
        // No quadtree — drain the reader so buffered requests can't leak
        // retroactively if `CollisionQuadtree` lands on a later frame.
        reader.clear();
        return;
    };

    // Hoisted above ALL loops — one allocation per system invocation.
    // `CELL_LAYER` is a const, so `query_layers` does not depend on any
    // per-request value.
    let query_layers = CollisionLayers::new(0, CELL_LAYER);

    for req in reader.read() {
        let candidates =
            quadtree
                .quadtree
                .query_circle_filtered(req.center, req.radius, query_layers);

        for candidate in candidates {
            let Ok((cell_entity, cell_pos)) = cells.get(candidate) else {
                continue;
            };
            if req.center.distance(cell_pos.0) > req.radius {
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
