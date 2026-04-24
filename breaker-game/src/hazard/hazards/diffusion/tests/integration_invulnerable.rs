//! W7 — diffusion with invulnerable neighbors, asserted at the message layer.
//!
//! `DiffusionAdjacencyQuery` drops `Without<Invulnerable>`, so invulnerable
//! cells enter the `candidate_neighbors` list and inflate `per_neighbor =
//! shared / n`. These tests write a primary `DamageDealt<Cell>`, run one tick,
//! and drain the message buffer to inspect:
//!   - the primary's amount post-`diffusion_reduce_primary` (in `MutateDamage`)
//!   - the ring amounts written by `diffusion_emit_rings` (in `PostApplyDamage`)
//!
//! HP-side outcomes on invulnerable cells (ring zeroed by
//! `invulnerable_filter::<Cell>` → no damage applied) are already proven by
//! `rantzsoft_dmg`'s own `invulnerable_filter_zeroes_damage_within_apply_damage_set`
//! crate test. Re-asserting them here would just rehash crate behavior.
//!
//! Behaviors 7, 8, 9 from `.claude/specs/w7-drop-invulnerable-filter-tests.md`.

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

/// Same builder shape as `integration_chain::build_app` (share percent 50.0,
/// Diffusion 1-stack active, full effects pipeline).
fn build_invulnerable_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .with_resource::<PendingDiffusionEmissions>()
        .with_resource::<DiffusionInstances>()
        .build();
    app.world_mut().insert_resource(DiffusionConfig {
        base_share_percent:      50.0,
        share_per_level_percent: 0.0,
        depth_increase_interval: 5,
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

fn spawn_vuln_cell(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id()
}

fn spawn_invuln_cell(app: &mut App, pos: Vec2) -> Entity {
    let e = spawn_vuln_cell(app, pos);
    app.world_mut().entity_mut(e).insert(Invulnerable);
    e
}

fn write_primary(app: &mut App, bolt: Entity, target: Entity, amount: f32) {
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer: Some(bolt),
            attributed_to: None,
            target,
            amount,
            source: None,
            _marker: PhantomData,
        });
}

fn drain_damage(app: &mut App) -> Vec<DamageDealt<Cell>> {
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect()
}

fn amount_for(msgs: &[DamageDealt<Cell>], target: Entity) -> f32 {
    msgs.iter()
        .find(|m| m.target == target)
        .unwrap_or_else(|| panic!("no DamageDealt<Cell> targeting {target:?}"))
        .amount
}

// ════════════════════════════════════════════════════════════════════════════
// W7 Behavior 7 — invulnerable neighbor joins the dilution denominator.
// Layout: c0 at origin; c1/c2 vulnerable ±60; c3 invulnerable at (0, 60).
// Each within 4900 of c0; single-hop only.
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn invulnerable_neighbor_dilutes_diffusion_damage() {
    let mut app = build_invulnerable_app();
    let c0 = spawn_vuln_cell(&mut app, Vec2::ZERO);
    let c1 = spawn_vuln_cell(&mut app, Vec2::new(60.0, 0.0));
    let c2 = spawn_vuln_cell(&mut app, Vec2::new(-60.0, 0.0));
    let c3 = spawn_invuln_cell(&mut app, Vec2::new(0.0, 60.0));
    let bolt = app.world_mut().spawn_empty().id();

    write_primary(&mut app, bolt, c0, 60.0);
    tick(&mut app); // reduce_primary → apply → emit_rings

    let msgs = drain_damage(&mut app);

    // shared = 60 * 0.5 = 30; n = 3 (c1, c2, c3 — c3 invulnerable included);
    // per_neighbor = 30/3 = 10.0. Pre-W7: n=2, per_neighbor = 15.0.
    assert!((amount_for(&msgs, c0) - 30.0).abs() < 1e-5);
    assert!((amount_for(&msgs, c1) - 10.0).abs() < 1e-5);
    assert!((amount_for(&msgs, c2) - 10.0).abs() < 1e-5);
    // c3 ring is emitted (W7-specific); crate's invulnerable_filter zeroes
    // delivered damage downstream. At emit time the amount is the diluted
    // per_neighbor value.
    assert!((amount_for(&msgs, c3) - 10.0).abs() < 1e-5);
}

// ── W7 Behavior 7 edge — proportional dilution with smaller primary. ─────────-

#[test]
fn invulnerable_dilution_is_proportional_to_primary_amount() {
    let mut app = build_invulnerable_app();
    let c0 = spawn_vuln_cell(&mut app, Vec2::ZERO);
    let c1 = spawn_vuln_cell(&mut app, Vec2::new(60.0, 0.0));
    let c2 = spawn_vuln_cell(&mut app, Vec2::new(-60.0, 0.0));
    let c3 = spawn_invuln_cell(&mut app, Vec2::new(0.0, 60.0));
    let bolt = app.world_mut().spawn_empty().id();

    write_primary(&mut app, bolt, c0, 30.0);
    tick(&mut app);

    let msgs = drain_damage(&mut app);

    // shared = 30 * 0.5 = 15; per_neighbor = 15/3 = 5.0.
    assert!((amount_for(&msgs, c0) - 15.0).abs() < 1e-5);
    assert!((amount_for(&msgs, c1) - 5.0).abs() < 1e-5);
    assert!((amount_for(&msgs, c2) - 5.0).abs() < 1e-5);
    assert!((amount_for(&msgs, c3) - 5.0).abs() < 1e-5);
}

// ── W7 Behavior 7 edge — adding a FOURTH invulnerable neighbor further
//    dilutes per_neighbor (monotonic in adjacency count).

#[test]
fn additional_invulnerable_neighbors_further_dilute() {
    let mut app = build_invulnerable_app();
    let c0 = spawn_vuln_cell(&mut app, Vec2::ZERO);
    let c1 = spawn_vuln_cell(&mut app, Vec2::new(60.0, 0.0));
    let c2 = spawn_vuln_cell(&mut app, Vec2::new(-60.0, 0.0));
    let c3 = spawn_invuln_cell(&mut app, Vec2::new(0.0, 60.0));
    let c4 = spawn_invuln_cell(&mut app, Vec2::new(0.0, -60.0));
    let bolt = app.world_mut().spawn_empty().id();

    write_primary(&mut app, bolt, c0, 60.0);
    tick(&mut app);

    let msgs = drain_damage(&mut app);

    // shared = 30; n = 4; per_neighbor = 30/4 = 7.5.
    assert!((amount_for(&msgs, c0) - 30.0).abs() < 1e-5);
    assert!((amount_for(&msgs, c1) - 7.5).abs() < 1e-5);
    assert!((amount_for(&msgs, c2) - 7.5).abs() < 1e-5);
    assert!((amount_for(&msgs, c3) - 7.5).abs() < 1e-5);
    assert!((amount_for(&msgs, c4) - 7.5).abs() < 1e-5);
}

// ════════════════════════════════════════════════════════════════════════════
// W7 Behavior 8 — all-invulnerable neighborhood: candidate_neighbors non-empty
// so primary IS reduced. Pre-W7 the filter would have emptied the list and
// the primary would have passed through at its original amount (60).
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn all_invulnerable_neighborhood_still_reduces_primary() {
    let mut app = build_invulnerable_app();
    let c0 = spawn_vuln_cell(&mut app, Vec2::ZERO);
    let c1 = spawn_invuln_cell(&mut app, Vec2::new(60.0, 0.0));
    let c2 = spawn_invuln_cell(&mut app, Vec2::new(-60.0, 0.0));
    let bolt = app.world_mut().spawn_empty().id();

    write_primary(&mut app, bolt, c0, 60.0);
    tick(&mut app);

    let msgs = drain_damage(&mut app);

    // shared = 30; n = 2; per_neighbor = 15.0 each.
    // W7-specific: c0's primary was reduced (pre-W7 it would still be 60).
    assert!(
        (amount_for(&msgs, c0) - 30.0).abs() < 1e-5,
        "primary must be reduced even with all-invulnerable neighbors (W7)"
    );
    assert!((amount_for(&msgs, c1) - 15.0).abs() < 1e-5);
    assert!((amount_for(&msgs, c2) - 15.0).abs() < 1e-5);
}

// ── W7 Behavior 8 edge — single invulnerable neighbor branch.

#[test]
fn single_invulnerable_neighbor_reduces_primary() {
    let mut app = build_invulnerable_app();
    let c0 = spawn_vuln_cell(&mut app, Vec2::ZERO);
    let c1 = spawn_invuln_cell(&mut app, Vec2::new(60.0, 0.0));
    let bolt = app.world_mut().spawn_empty().id();

    write_primary(&mut app, bolt, c0, 60.0);
    tick(&mut app);

    let msgs = drain_damage(&mut app);

    // shared = 30; n = 1; per_neighbor = 30.0.
    assert!(
        (amount_for(&msgs, c0) - 30.0).abs() < 1e-5,
        "primary must be reduced with a lone invulnerable neighbor (W7)"
    );
    assert!((amount_for(&msgs, c1) - 30.0).abs() < 1e-5);
}

// ════════════════════════════════════════════════════════════════════════════
// W7 Behavior 9 — all-vulnerable regression pin: W7 must not regress the
// clean case. Messages identical in shape to pre-W7.
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn all_vulnerable_neighborhood_regression_pin() {
    let mut app = build_invulnerable_app();
    let c0 = spawn_vuln_cell(&mut app, Vec2::ZERO);
    let c1 = spawn_vuln_cell(&mut app, Vec2::new(60.0, 0.0));
    let c2 = spawn_vuln_cell(&mut app, Vec2::new(-60.0, 0.0));
    let bolt = app.world_mut().spawn_empty().id();

    write_primary(&mut app, bolt, c0, 60.0);
    tick(&mut app);

    let msgs = drain_damage(&mut app);

    // shared = 30; per_neighbor = 15.0 each. Total conservation: 30 + 15 + 15 = 60.
    assert!((amount_for(&msgs, c0) - 30.0).abs() < 1e-5);
    assert!((amount_for(&msgs, c1) - 15.0).abs() < 1e-5);
    assert!((amount_for(&msgs, c2) - 15.0).abs() < 1e-5);
}
