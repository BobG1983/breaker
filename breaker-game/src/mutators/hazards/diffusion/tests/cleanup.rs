//! W2 Behaviors 22–23: `DiffusionInstances` and `PendingDiffusionEmissions`
//! default + cleanup on `OnExit(NodeState::Playing)`.

use std::collections::HashSet;

use bevy::prelude::*;

use super::super::system::{
    DiffusionInstances, PendingDiffusionEmissions, PendingEmission, reset_diffusion_state,
};
use crate::{mutators::hazards::resources::ActiveHazards, prelude::*};

// ── W2 Behavior 23: PendingDiffusionEmissions defaults to empty queue ──

#[test]
fn pending_diffusion_emissions_defaults_empty() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<PendingDiffusionEmissions>();

    assert!(
        app.world()
            .resource::<PendingDiffusionEmissions>()
            .queue
            .is_empty()
    );
}

#[test]
fn pending_diffusion_emissions_idempotent_init() {
    // Edge case: init_resource calls are idempotent.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<PendingDiffusionEmissions>();
    app.init_resource::<PendingDiffusionEmissions>();
    assert!(
        app.world()
            .resource::<PendingDiffusionEmissions>()
            .queue
            .is_empty()
    );
}

// ── W2 Behavior 22: both resources reset on OnExit(Playing) ──

#[test]
fn on_exit_playing_resets_diffusion_state() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_resource::<DiffusionInstances>()
        .with_resource::<PendingDiffusionEmissions>()
        .build();
    app.add_systems(OnExit(NodeState::Playing), reset_diffusion_state);

    // Spawn three bare entities (no components) — the test only needs valid
    // Entity handles to seed the resources; behavior isn't under test here.
    let target = app.world_mut().spawn_empty().id();
    let n1 = app.world_mut().spawn_empty().id();
    let n2 = app.world_mut().spawn_empty().id();

    // Seed both resources with non-default state.
    {
        let mut inst = app.world_mut().resource_mut::<DiffusionInstances>();
        inst.next_id = 5;
        inst.visited.insert(0, HashSet::from([n1, n2]));
    }
    {
        let mut pending = app.world_mut().resource_mut::<PendingDiffusionEmissions>();
        pending.queue.push(PendingEmission {
            target,
            instance_id: 0,
            shared: 50.0,
            candidate_neighbors: vec![n1, n2],
            attributed_to: None,
        });
    }

    // Transition out of Playing → OnExit fires reset.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Teardown);
    app.update();

    let inst = app.world().resource::<DiffusionInstances>();
    assert!(
        inst.visited.is_empty(),
        "visited must be cleared on OnExit(Playing)"
    );
    assert_eq!(inst.next_id, 0, "next_id must be reset to 0");

    let pending = app.world().resource::<PendingDiffusionEmissions>();
    assert!(
        pending.queue.is_empty(),
        "PendingDiffusionEmissions queue must be drained on OnExit(Playing)"
    );
}
