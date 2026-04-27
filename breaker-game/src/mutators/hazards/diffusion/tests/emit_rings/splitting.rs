//! W2 Behavior 27 — `emit_rings` drains the queue and splits `shared` flat
//! across candidate neighbors.

use std::collections::HashSet;

use bevy::prelude::*;

use super::{
    super::super::system::{DiffusionInstances, PendingDiffusionEmissions, PendingEmission},
    helpers::{assert_f32_eq, build_emit_app, spawn_cell_at},
};
use crate::{mutators::hazards::definition::HazardKind, prelude::*};

#[test]
fn emit_rings_splits_shared_across_candidates() {
    let mut app = build_emit_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let c1 = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0)); // far — NOT adjacent to c0
    let c2 = spawn_cell_at(&mut app, Vec2::new(0.0, 200.0));
    let bolt = app.world_mut().spawn_empty().id();

    // Pre-populate the queue (simulating what reduce_primary would queue).
    app.world_mut()
        .resource_mut::<PendingDiffusionEmissions>()
        .queue
        .push(PendingEmission {
            target:              c0,
            instance_id:         0,
            shared:              50.0,
            candidate_neighbors: vec![c1, c2],
            attributed_to:       Some(bolt),
        });
    app.world_mut()
        .resource_mut::<DiffusionInstances>()
        .visited
        .insert(0, HashSet::from([c0]));

    tick(&mut app);

    // Queue drained.
    assert!(
        app.world()
            .resource::<PendingDiffusionEmissions>()
            .queue
            .is_empty()
    );

    // Two new ring messages: each target gets 25.0.
    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    let rings: Vec<&DamageDealt<Cell>> = drained
        .iter()
        .filter(|m| m.source == Some(SourceId::hazard(HazardKind::Diffusion).instance(0).build()))
        .collect();
    assert_eq!(
        rings.len(),
        2,
        "expected 2 ring messages, got {}",
        rings.len()
    );

    for r in &rings {
        assert_f32_eq(r.amount, 25.0);
        assert_eq!(r.dealer, None);
        assert_eq!(r.attributed_to, Some(bolt));
    }

    // Visited set now contains C0, C1, C2.
    let visited = &app.world().resource::<DiffusionInstances>().visited[&0];
    assert!(visited.contains(&c0));
    assert!(visited.contains(&c1));
    assert!(visited.contains(&c2));
}

#[test]
fn emit_rings_single_neighbor_gets_full_shared() {
    // shared: 50.0, candidate_neighbors: [C1] → C1 gets 50.0.
    let mut app = build_emit_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let c1 = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0));

    app.world_mut()
        .resource_mut::<PendingDiffusionEmissions>()
        .queue
        .push(PendingEmission {
            target:              c0,
            instance_id:         0,
            shared:              50.0,
            candidate_neighbors: vec![c1],
            attributed_to:       None,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    let ring = drained
        .iter()
        .find(|m| m.source == Some(SourceId::hazard(HazardKind::Diffusion).instance(0).build()))
        .expect("exactly one ring message expected");
    assert_f32_eq(ring.amount, 50.0);
    assert_eq!(ring.target, c1);
}

#[test]
fn emit_rings_none_attributed_to_passes_through_as_none() {
    // Edge case: queue entry with attributed_to: None → emitted rings carry None.
    let mut app = build_emit_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let c1 = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0));

    app.world_mut()
        .resource_mut::<PendingDiffusionEmissions>()
        .queue
        .push(PendingEmission {
            target:              c0,
            instance_id:         0,
            shared:              50.0,
            candidate_neighbors: vec![c1],
            attributed_to:       None,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    let ring = drained
        .iter()
        .find(|m| m.source == Some(SourceId::hazard(HazardKind::Diffusion).instance(0).build()))
        .expect("ring message");
    assert!(ring.attributed_to.is_none());
    assert!(ring.dealer.is_none());
}
