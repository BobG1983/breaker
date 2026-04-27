//! W2 Behavior 28 — `emit_rings` skips when target has `Invulnerable`.

use std::collections::HashSet;

use bevy::prelude::*;

use super::{
    super::super::system::{DiffusionInstances, PendingDiffusionEmissions, PendingEmission},
    helpers::{build_emit_app, spawn_cell_at},
};
use crate::{mutators::hazards::definition::HazardKind, prelude::*};

#[test]
fn emit_rings_skips_when_target_invulnerable() {
    let mut app = build_emit_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    app.world_mut().entity_mut(c0).insert(Invulnerable);
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
    app.world_mut()
        .resource_mut::<DiffusionInstances>()
        .visited
        .insert(0, HashSet::from([c0]));

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
        "no ring messages when PendingEmission.target has Invulnerable"
    );
    assert!(
        app.world()
            .resource::<PendingDiffusionEmissions>()
            .queue
            .is_empty(),
        "queue must still be drained"
    );

    // visited[0] should only contain c0 (the seed from reduce_primary).
    let visited = &app.world().resource::<DiffusionInstances>().visited[&0];
    assert_eq!(visited, &HashSet::from([c0]));
}
