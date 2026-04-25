//! W2 Behaviors 30–31: multi-frame diffusion propagation + `instance_id` tracking.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::system::{
    DiffusionConfig, DiffusionInstances, PendingDiffusionEmissions, diffusion_emit_rings,
    diffusion_reduce_primary,
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

fn build_app(share_percent: f32) -> App {
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

// ── W2 Behavior 30: multi-frame diffusion propagation with 3 cells in a line ──

#[test]
fn multi_frame_diffusion_3_inline_cells() {
    // TODO(spec-deviation): spec §diffusion.Behavior30 specifies positions
    // (0, 30, 60) for the three cells, but those positions violate the
    // spec's own topology prose ("C0 adjacent to C1 only, not C2") because
    // ADJACENCY_RADIUS_SQ = 70 * 70 = 4900 and |C0 - C2|^2 = 3600 < 4900 —
    // so C0 WOULD be adjacent to C2 at spacing 30. The spec's numeric
    // positions and topology description are inconsistent.
    //
    // Deviation: this test uses spacing 50/100 (C0=0, C1=50, C2=100) so
    // that |C0 - C2| = 100 > 70 and the "linear chain C0→C1→C2" topology
    // matches the spec's prose intent. The expected HP values (C0=50,
    // C1=75, C2=75) are still correct under the 50/100 geometry because
    // the share math is neighborhood-count-independent at 50% — C1 gets
    // 25 of C0's 50 shared damage, and C2 gets 25 of C1's 25 shared
    // damage (second-frame ring from C1). Total damage conserved at 100.
    //
    // Do NOT change this to (0, 30, 60) without first fixing the spec's
    // topology description. guard-docs / reviewer-tests should flag this
    // TODO on the next sweep so we can either (a) make the spec
    // consistent by switching its positions to match the topology, or
    // (b) redesign the expected HP values to match (0, 30, 60) geometry.
    let mut app = build_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let c1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0));
    let c2 = spawn_cell_at(&mut app, Vec2::new(100.0, 0.0)); // 100 from c0: NOT adjacent
    let bolt = app.world_mut().spawn_empty().id();

    // Frame N: primary on C0 for 100 damage.
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

    tick(&mut app); // Frame N
    tick(&mut app); // Frame N+1
    tick(&mut app); // Frame N+2

    // After 3 frames: C0=50, C1=75, C2=75. Total damage = 100 (conserved).
    let hp0 = app.world().get::<Hp>(c0).unwrap().current;
    let hp1 = app.world().get::<Hp>(c1).unwrap().current;
    let hp2 = app.world().get::<Hp>(c2).unwrap().current;

    assert_f32_eq(hp0, 50.0);
    assert_f32_eq(hp1, 75.0);
    assert_f32_eq(hp2, 75.0);
    // Total absorbed: 50+25+25 = 100.
    assert_f32_eq((100.0 - hp0) + (100.0 - hp1) + (100.0 - hp2), 100.0);
}

#[test]
fn multi_frame_diffusion_triangle_cycle_no_infinite_loop() {
    // Triangular cycle: 3 cells mutually adjacent.
    let mut app = build_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let c1 = spawn_cell_at(&mut app, Vec2::new(30.0, 0.0));
    let c2 = spawn_cell_at(&mut app, Vec2::new(15.0, 26.0));
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

    // Run a few frames to allow propagation to stabilize.
    for _ in 0..5 {
        tick(&mut app);
    }

    let hp0 = app.world().get::<Hp>(c0).unwrap().current;
    let hp1 = app.world().get::<Hp>(c1).unwrap().current;
    let hp2 = app.world().get::<Hp>(c2).unwrap().current;

    // Frame N: C0 takes 50, emits 25 to C1 and 25 to C2. visited = {C0,C1,C2}.
    // Frame N+1: both ring messages hit, all neighbors already visited → pass-through.
    // Expected: C0=50, C1=75, C2=75.
    assert_f32_eq(hp0, 50.0);
    assert_f32_eq(hp1, 75.0);
    assert_f32_eq(hp2, 75.0);
}

// ── W2 Behavior 31: reduce_primary seeds fresh instance for non-diffusion source ──

#[test]
fn reduce_primary_seeds_fresh_instance_for_plain_source() {
    let mut app = build_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let _c1 = spawn_cell_at(&mut app, Vec2::new(30.0, 0.0));

    // First primary: instance_id 0 is allocated.
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

    let instances = app.world().resource::<DiffusionInstances>();
    assert_eq!(
        instances.next_id, 1,
        "next_id increments to 1 after seeding instance 0"
    );
    assert!(instances.visited.contains_key(&0));
}

#[test]
fn reduce_primary_orphan_instance_id_resurrects() {
    // Edge case: message with source="hazard:diffusion:999" where 999 is not
    // in visited. Writer-code resurrects instance 999 and seeds visited.
    let mut app = build_app(50.0);
    let c3 = spawn_cell_at(&mut app, Vec2::ZERO);
    let _c4 = spawn_cell_at(&mut app, Vec2::new(30.0, 0.0));

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c3,
            amount:        80.0,
            source:        Some(
                SourceId::hazard(HazardKind::Diffusion)
                    .instance(999)
                    .build(),
            ),
            _marker:       PhantomData,
        });

    tick(&mut app);

    let instances = app.world().resource::<DiffusionInstances>();
    assert!(
        instances.visited.contains_key(&999),
        "orphan instance 999 must be resurrected and seeded"
    );
    assert!(instances.visited[&999].contains(&c3));
}

#[test]
fn reduce_primary_non_diffusion_source_gets_fresh_instance() {
    // Edge case: another hazard's sentinel (not "hazard:diffusion:") is
    // treated as a fresh primary — seeded with a new instance_id.
    let mut app = build_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let _c1 = spawn_cell_at(&mut app, Vec2::new(30.0, 0.0));

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c0,
            amount:        100.0,
            source:        Some(SourceId::hazard(HazardKind::Cascade).build()),
            _marker:       PhantomData,
        });

    tick(&mut app);

    let instances = app.world().resource::<DiffusionInstances>();
    // A fresh instance 0 should be seeded (next_id = 1).
    assert_eq!(instances.next_id, 1);
    assert!(instances.visited.contains_key(&0));
}

// Regression: after an orphan-instance resurrection (`instance_id == 999`,
// which is greater than the allocator's `next_id`), the resource's `next_id`
// must be bumped past that orphan so a subsequent fresh primary does not
// collide with the resurrected instance.
#[test]
fn reduce_primary_orphan_resurrection_bumps_next_id() {
    let mut app = build_app(50.0);
    let c0 = spawn_cell_at(&mut app, Vec2::ZERO);
    let c3 = spawn_cell_at(&mut app, Vec2::new(30.0, 0.0));

    // Tick 1: seed orphan 999 via a hazard:diffusion:999-sourced message.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c3,
            amount:        80.0,
            source:        Some(
                SourceId::hazard(HazardKind::Diffusion)
                    .instance(999)
                    .build(),
            ),
            _marker:       PhantomData,
        });
    tick(&mut app);

    {
        let instances = app.world().resource::<DiffusionInstances>();
        assert!(
            instances.visited.contains_key(&999),
            "orphan 999 must be resurrected before the next_id bump check runs"
        );
        assert!(
            instances.next_id > 999,
            "next_id must be bumped past resurrected orphan, got {}",
            instances.next_id
        );
    }

    // Tick 2: send a fresh primary (no source). Allocator must hand out an
    // instance id >= the post-bump next_id — NOT 0, which would collide
    // against some hypothetical future instance if the bump were missing.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c0,
            amount:        50.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    let instances = app.world().resource::<DiffusionInstances>();
    // The fresh primary should have been allocated an id >= 1000, not 0
    // (which would be the untracked-bump bug).
    let fresh_ids: Vec<&u64> = instances.visited.keys().filter(|&&k| k != 999).collect();
    assert_eq!(
        fresh_ids.len(),
        1,
        "exactly one fresh id alongside orphan 999"
    );
    assert!(
        *fresh_ids[0] >= 1000,
        "fresh id must be >= 1000 (post-bump), got {}",
        fresh_ids[0]
    );
}
