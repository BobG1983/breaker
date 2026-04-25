//! W2 Behaviors 24–26: `diffusion_reduce_primary` runs in
//! `DmgSystems::MutateDamage` with Diffusion-active + Playing run-if.

use std::{collections::HashSet, marker::PhantomData};

use bevy::prelude::*;

use super::{
    super::system::{DiffusionConfig, PendingDiffusionEmissions, diffusion_reduce_primary},
    helpers::canonical_config,
};
use crate::{
    hazard::{
        definition::HazardKind,
        hazards::diffusion::system::PendingEmission,
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

/// Builds a test app with `RantzDmgPlugin` + Cell registered, `ActiveHazards`
/// resource, and ONLY `diffusion_reduce_primary` wired.
///
/// `diffusion_emit_rings` (the `PostApplyDamage` pass) is deliberately NOT scheduled
/// here — its job is to drain `PendingDiffusionEmissions.queue`, which would
/// defeat the queue assertions these tests make. `emit_rings` has its own
/// dedicated test file.
fn build_diffusion_test_app(share_percent: f32) -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .with_resource::<PendingDiffusionEmissions>()
        .with_resource::<crate::hazard::hazards::diffusion::system::DiffusionInstances>()
        .build();

    app.world_mut().insert_resource(DiffusionConfig {
        base_share_percent:      share_percent,
        share_per_level_percent: 0.0,
    });
    // Add Diffusion stack so hazard_active(Diffusion) run-condition passes.
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Diffusion);

    app.add_systems(
        FixedUpdate,
        diffusion_reduce_primary
            .in_set(DmgSystems::MutateDamage)
            .run_if(hazard_active(HazardKind::Diffusion))
            .run_if(in_state(NodeState::Playing)),
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

// ── W2 Behavior 24: system runs in MutateDamage gated by Playing/active ──

#[test]
fn reduce_primary_with_diffusion_inactive_passes_through() {
    // When Diffusion is "inactive" via 0 stacks, the run-if gate blocks the
    // system; msg.amount stays at 100.0 and the queue is empty.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .with_resource::<PendingDiffusionEmissions>()
        .with_resource::<crate::hazard::hazards::diffusion::system::DiffusionInstances>()
        .build();
    // Canonical config inserted, but no stacks.
    app.world_mut().insert_resource(canonical_config());
    app.add_systems(
        FixedUpdate,
        diffusion_reduce_primary
            .in_set(DmgSystems::MutateDamage)
            .run_if(hazard_active(HazardKind::Diffusion))
            .run_if(in_state(NodeState::Playing)),
    );

    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c0,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    assert!(
        app.world()
            .resource::<PendingDiffusionEmissions>()
            .queue
            .is_empty(),
        "inactive Diffusion — queue must be empty"
    );
}

// ── W2 Behavior 25: reduce_primary reduces msg.amount, queues PendingEmission ──

#[test]
fn reduce_primary_reduces_amount_and_queues_pending_emission() {
    let mut app = build_diffusion_test_app(50.0); // share_frac = 0.5
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let c1 = spawn_cell_at(&mut app, Vec2::new(30.0, 0.0));
    let c2 = spawn_cell_at(&mut app, Vec2::new(0.0, 30.0));
    let bolt = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        c0,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    // After reduce_primary, the message's amount should be 100 * (1 - 0.5) = 50.
    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    assert!(
        drained
            .iter()
            .any(|m| m.target == c0 && (m.amount - 50.0).abs() < 1e-4),
        "primary msg.amount must be reduced to 50.0, got {:?}",
        drained.iter().map(|m| m.amount).collect::<Vec<_>>()
    );

    // The queue must contain one PendingEmission with shared=50 and
    // candidate_neighbors = {c1, c2}, attributed_to = Some(bolt).
    let queue = &app.world().resource::<PendingDiffusionEmissions>().queue;
    assert_eq!(
        queue.len(),
        1,
        "expected one PendingEmission, got {}",
        queue.len()
    );
    let pe: &PendingEmission = &queue[0];
    assert_eq!(pe.target, c0);
    assert_eq!(pe.instance_id, 0);
    assert_f32_eq(pe.shared, 50.0);
    let neighbors: HashSet<Entity> = pe.candidate_neighbors.iter().copied().collect();
    assert_eq!(neighbors, HashSet::from([c1, c2]));
    assert_eq!(pe.attributed_to, Some(bolt));
}

#[test]
fn reduce_primary_share_frac_0_2() {
    // Edge case: share_frac = 0.2 → msg.amount = 80.0, queue shared = 20.0.
    let mut app = build_diffusion_test_app(20.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let _c1 = spawn_cell_at(&mut app, Vec2::new(30.0, 0.0));

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c0,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let queue = &app.world().resource::<PendingDiffusionEmissions>().queue;
    assert_eq!(queue.len(), 1);
    assert_f32_eq(queue[0].shared, 20.0);
}

#[test]
fn reduce_primary_attributed_to_forwarded_from_attributed_to() {
    // Edge case: dealer: None, attributed_to: Some(source).
    let mut app = build_diffusion_test_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let _c1 = spawn_cell_at(&mut app, Vec2::new(30.0, 0.0));
    let source = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: Some(source),
            target:        c0,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let queue = &app.world().resource::<PendingDiffusionEmissions>().queue;
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].attributed_to, Some(source));
}

#[test]
fn reduce_primary_attributed_to_none_when_both_none() {
    // Edge case: dealer: None, attributed_to: None → attributed_to: None.
    let mut app = build_diffusion_test_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let _c1 = spawn_cell_at(&mut app, Vec2::new(30.0, 0.0));

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c0,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let queue = &app.world().resource::<PendingDiffusionEmissions>().queue;
    assert_eq!(queue.len(), 1);
    assert!(queue[0].attributed_to.is_none());
}

#[test]
fn reduce_primary_excludes_dead_neighbors() {
    // Edge case: dead neighbor is excluded from candidate_neighbors.
    let mut app = build_diffusion_test_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let _c1 = spawn_cell_at(&mut app, Vec2::new(30.0, 0.0));
    let c2 = spawn_cell_at(&mut app, Vec2::new(0.0, 30.0));
    app.world_mut().entity_mut(c2).insert(Dead);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c0,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let queue = &app.world().resource::<PendingDiffusionEmissions>().queue;
    assert_eq!(queue.len(), 1);
    // c2 is Dead → only c1 should be a candidate.
    assert_eq!(queue[0].candidate_neighbors.len(), 1);
    assert!(!queue[0].candidate_neighbors.contains(&c2));
}

#[test]
fn reduce_primary_includes_invulnerable_neighbors_in_candidate_list() {
    // W7: `DiffusionAdjacencyQuery` drops `Without<Invulnerable>`. Invulnerable
    // neighbors now enter the `candidate_neighbors` list. The pipeline's
    // `invulnerable_filter::<Cell>` zeroes the eventual ring amount targeting
    // the invulnerable cell; denominator dilution is the user-approved
    // consequence. See `.claude/specs/w7-drop-invulnerable-filter-tests.md`.
    let mut app = build_diffusion_test_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let c1 = spawn_cell_at(&mut app, Vec2::new(30.0, 0.0));
    let c2 = spawn_cell_at(&mut app, Vec2::new(0.0, 30.0));
    app.world_mut().entity_mut(c2).insert(Invulnerable);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c0,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let queue = &app.world().resource::<PendingDiffusionEmissions>().queue;
    assert_eq!(queue.len(), 1);
    assert_eq!(
        queue[0].candidate_neighbors.len(),
        2,
        "both c1 and c2 must be in candidate_neighbors under W7 (c2 is invulnerable)"
    );
    assert!(
        queue[0].candidate_neighbors.contains(&c1),
        "vulnerable c1 in candidate_neighbors"
    );
    assert!(
        queue[0].candidate_neighbors.contains(&c2),
        "invulnerable c2 in candidate_neighbors (W7)"
    );
}

// ── W2 Behavior 26: pass-through when no candidates exist ──

#[test]
fn reduce_primary_passes_through_with_no_neighbors() {
    let mut app = build_diffusion_test_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    // No neighbors in range.

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c0,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    // Queue remains empty.
    assert!(
        app.world()
            .resource::<PendingDiffusionEmissions>()
            .queue
            .is_empty()
    );
}

#[test]
fn reduce_primary_passes_through_when_all_neighbors_dead() {
    let mut app = build_diffusion_test_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let c1 = spawn_cell_at(&mut app, Vec2::new(30.0, 0.0));
    app.world_mut().entity_mut(c1).insert(Dead);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c0,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    assert!(
        app.world()
            .resource::<PendingDiffusionEmissions>()
            .queue
            .is_empty(),
        "when every neighbor is Dead, no emission is queued"
    );
}
