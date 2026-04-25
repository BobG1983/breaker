//! W2 Behaviors 27–29: `diffusion_emit_rings` drains the pending queue in
//! `DmgSystems::PostApplyDamage`.

use std::collections::HashSet;

use bevy::prelude::*;

use super::super::system::{
    DiffusionConfig, DiffusionInstances, PendingDiffusionEmissions, PendingEmission,
    diffusion_emit_rings, diffusion_reduce_primary,
};
use crate::{
    hazard::{
        definition::HazardKind,
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

#[track_caller]
fn assert_f32_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "expected {expected}, got {actual}"
    );
}

fn build_emit_app(share_percent: f32) -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .with_resource::<PendingDiffusionEmissions>()
        .with_resource::<DiffusionInstances>()
        .build();
    app.world_mut().insert_resource(DiffusionConfig {
        base_share_percent:      share_percent,
        share_per_level_percent: 0.0,
    });
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Diffusion);
    app.add_systems(
        FixedUpdate,
        (
            diffusion_reduce_primary
                .in_set(DmgSystems::MutateDamage)
                .run_if(hazard_active(HazardKind::Diffusion))
                .run_if(in_state(NodeState::Playing)),
            diffusion_emit_rings
                .in_set(DmgSystems::PostApplyDamage)
                .run_if(hazard_active(HazardKind::Diffusion))
                .run_if(in_state(NodeState::Playing)),
        ),
    );
    app
}

fn spawn_cell_at(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id()
}

// ── W2 Behavior 27: emit_rings drains queue and splits `shared` flat ──

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

// ── W2 Behavior 28: emit_rings skips when target has Invulnerable ──

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

// ── W2 Behavior 29: attenuation floor — per-neighbor amount < 1.0 skipped ──

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

// ── B35: ring damage source equals builder-produced hazard:diffusion:<instance> ──

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

fn install_cell_hp(app: &mut App, cell: Entity, hp: f32) {
    app.world_mut().entity_mut(cell).insert(Hp::new(hp));
}
