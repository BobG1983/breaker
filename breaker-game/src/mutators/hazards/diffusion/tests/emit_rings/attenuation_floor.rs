//! W2 Behavior 29 — per-neighbor attenuation floor at 1.0 (inclusive).

use bevy::prelude::*;

use super::{
    super::super::system::{PendingDiffusionEmissions, PendingEmission},
    helpers::{build_emit_app, spawn_cell_at},
};
use crate::{mutators::hazards::definition::HazardKind, prelude::*};

#[test]
fn emit_rings_attenuation_floor_skips_small_shares() {
    // shared: 0.1, candidate_neighbors: [C1, C2] → per-neighbor 0.05 → skip.
    let mut app = build_emit_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let c1 = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0));
    let c2 = spawn_cell_at(&mut app, Vec2::new(0.0, 200.0));

    app.world_mut()
        .resource_mut::<PendingDiffusionEmissions>()
        .queue
        .push(PendingEmission {
            target:              c0,
            instance_id:         0,
            shared:              0.1,
            candidate_neighbors: vec![c1, c2],
            attributed_to:       None,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    assert!(
        !drained
            .iter()
            .any(|m| m.source == Some(SourceId::hazard(HazardKind::Diffusion).instance(0).build())),
        "below 1.0 per-neighbor floor, no rings emitted"
    );
    assert!(
        app.world()
            .resource::<PendingDiffusionEmissions>()
            .queue
            .is_empty()
    );
}

#[test]
fn emit_rings_attenuation_floor_inclusive_at_1_0() {
    // shared: 2.0, 2 neighbors → per-neighbor 1.0 → IS emitted (inclusive floor).
    let mut app = build_emit_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let c1 = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0));
    let c2 = spawn_cell_at(&mut app, Vec2::new(0.0, 200.0));

    app.world_mut()
        .resource_mut::<PendingDiffusionEmissions>()
        .queue
        .push(PendingEmission {
            target:              c0,
            instance_id:         0,
            shared:              2.0,
            candidate_neighbors: vec![c1, c2],
            attributed_to:       None,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    assert_eq!(
        drained
            .iter()
            .filter(
                |m| m.source == Some(SourceId::hazard(HazardKind::Diffusion).instance(0).build())
            )
            .count(),
        2,
        "exactly 1.0 per-neighbor is inclusive → both emitted"
    );
}

#[test]
fn emit_rings_attenuation_floor_excludes_just_under() {
    // shared: 1.98, 2 neighbors → per-neighbor 0.99 → NOT emitted.
    let mut app = build_emit_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let c1 = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0));
    let c2 = spawn_cell_at(&mut app, Vec2::new(0.0, 200.0));

    app.world_mut()
        .resource_mut::<PendingDiffusionEmissions>()
        .queue
        .push(PendingEmission {
            target:              c0,
            instance_id:         0,
            shared:              1.98,
            candidate_neighbors: vec![c1, c2],
            attributed_to:       None,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    assert!(
        !drained
            .iter()
            .any(|m| m.source == Some(SourceId::hazard(HazardKind::Diffusion).instance(0).build())),
        "0.99 per-neighbor is below floor → no rings"
    );
}
