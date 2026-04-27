//! B35 — ring damage source equals builder-produced
//! `hazard:diffusion:<instance>` `SourceId`.

use bevy::prelude::*;

use super::{
    super::super::system::{DiffusionInstances, PendingDiffusionEmissions, PendingEmission},
    helpers::{build_emit_app, install_cell_hp},
};
use crate::{mutators::hazards::definition::HazardKind, prelude::*};

#[test]
fn diffusion_ring_damage_source_equals_builder_with_instance() {
    use crate::prelude::SourceIdExt;

    let mut app = build_emit_app(50.0);
    app.world_mut().resource_mut::<DiffusionInstances>().next_id = 0;

    let primary = app.world_mut().spawn_empty().id();
    let neighbor = app.world_mut().spawn_empty().id();
    install_cell_hp(&mut app, primary, 10.0);
    install_cell_hp(&mut app, neighbor, 10.0);

    app.world_mut()
        .resource_mut::<PendingDiffusionEmissions>()
        .queue
        .push(PendingEmission {
            target:              primary,
            instance_id:         0,
            shared:              5.0,
            candidate_neighbors: vec![neighbor],
            attributed_to:       None,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    let expected = SourceId::hazard(HazardKind::Diffusion).instance(0).build();
    assert!(
        drained.iter().any(|m| m.source.as_ref() == Some(&expected)),
        "diffusion ring damage must carry builder source with instance id"
    );
}
