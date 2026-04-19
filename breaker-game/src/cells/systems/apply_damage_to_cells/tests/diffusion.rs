//! Section D — integration tests for `apply_damage_to_cells`.
//!
//! Pin the Diffusion redistribution branch inside the cells-domain Cell damage
//! system. The non-Diffusion semantics mirror the generic `apply_damage::<T>`
//! tests for `TestEntity` in `shared/death_pipeline/systems/tests/apply_damage.rs`;
//! these tests exercise the Cell-specific path plus the Diffusion BFS branch.
//!
//! Test helpers (`PendingCellDamage`, `enqueue_cell_damage`) are copied locally
//! from `shared/death_pipeline/systems/tests/helpers.rs` (Option A per the
//! spec): the upstream items are `pub(super)` and not importable from the
//! cells-domain test module. Low-duplication cost for the decoupling benefit.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::system::apply_damage_to_cells;
use crate::{
    cells::components::ADJACENCY_RADIUS_SQ,
    hazard::{
        definition::HazardKind, hazards::diffusion::DiffusionConfig, resources::ActiveHazards,
    },
    prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
};

// ── Test helpers (cells-domain copy of PendingCellDamage/enqueue) ─────────

/// Pending `DamageDealt<Cell>` messages enqueued before
/// `apply_damage_to_cells` each tick. Local copy of the shared
/// death-pipeline test helper — keeps cells-domain tests independent.
#[derive(Resource, Default)]
struct PendingCellDamage(Vec<DamageDealt<Cell>>);

/// System that writes `DamageDealt<Cell>` from `PendingCellDamage` each tick.
fn enqueue_cell_damage(
    pending: Res<PendingCellDamage>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

fn damage_msg(target: Entity, amount: f32, dealer: Option<Entity>) -> DamageDealt<Cell> {
    DamageDealt {
        dealer,
        target,
        amount,
        source_chip: None,
        _marker: PhantomData,
    }
}

/// Canonical builder: state hierarchy at `NodeState::Playing`,
/// `ActiveHazards`, `DamageDealt<Cell>` with capture, `PendingCellDamage` +
/// `enqueue_cell_damage` ordered before `DeathPipelineSystems::ApplyDamage`,
/// and `apply_damage_to_cells` in that set.
fn build_apply_damage_to_cells_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message_capture::<DamageDealt<Cell>>()
        .with_resource::<PendingCellDamage>()
        .build();

    // Order `enqueue_cell_damage` before the `ApplyDamage` set, then put
    // `apply_damage_to_cells` INTO the set. We don't add `DeathPipelinePlugin`
    // here because that plugin also registers the generic `apply_damage::<Cell>`
    // which we are replacing.
    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        apply_damage_to_cells.in_set(DeathPipelineSystems::ApplyDamage),
    );
    app
}

/// Variant builder that deliberately OMITS `ActiveHazards`. Used to pin
/// robustness against the resource-registration-order edge case.
fn build_apply_damage_to_cells_app_without_active_hazards() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_message_capture::<DamageDealt<Cell>>()
        .with_resource::<PendingCellDamage>()
        .build();

    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        apply_damage_to_cells.in_set(DeathPipelineSystems::ApplyDamage),
    );
    app
}

fn spawn_cell_at(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    app.world_mut()
        .spawn((Cell, Position2D(pos), Hp::new(hp), KilledBy::default()))
        .id()
}

fn spawn_cell_at_dead(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp::new(hp),
            KilledBy::default(),
            Dead,
        ))
        .id()
}

fn spawn_cell_at_invulnerable(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp::new(hp),
            KilledBy::default(),
            Invulnerable,
        ))
        .id()
}

fn install_diffusion_config(app: &mut App, cfg: DiffusionConfig) {
    app.world_mut().insert_resource(cfg);
}

fn add_diffusion_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Diffusion);
    }
}

const fn canonical_diffusion_config() -> DiffusionConfig {
    DiffusionConfig {
        base_share_percent:      20.0,
        share_per_level_percent: 10.0,
        depth_increase_interval: 5,
    }
}

fn hp_of(app: &App, entity: Entity) -> f32 {
    app.world().get::<Hp>(entity).unwrap().current
}

fn push_damage(app: &mut App, msg: DamageDealt<Cell>) {
    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push(msg);
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 24 — No DiffusionConfig + stacks → primary takes full damage
// ════════════════════════════════════════════════════════════════════════

#[test]
fn no_diffusion_config_with_stacks_applies_full_damage_to_primary() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let neighbor = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    add_diffusion_stacks(&mut app, 1);
    // DELIBERATELY omit install_diffusion_config.

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    assert!(
        (hp_of(&app, primary) - 50.0).abs() < f32::EPSILON,
        "primary should take FULL 50.0 damage when DiffusionConfig absent, got HP {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, neighbor) - 100.0).abs() < f32::EPSILON,
        "neighbor should be untouched when DiffusionConfig absent, got HP {}",
        hp_of(&app, neighbor)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 24a — No ActiveHazards resource → no panic, full damage
// ════════════════════════════════════════════════════════════════════════

#[test]
fn no_active_hazards_resource_does_not_panic_and_applies_full_damage() {
    let mut app = build_apply_damage_to_cells_app_without_active_hazards();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    assert!(
        (hp_of(&app, primary) - 50.0).abs() < f32::EPSILON,
        "primary should take 50.0 damage with no ActiveHazards, got HP {}",
        hp_of(&app, primary)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 25 — Config present, 0 stacks → short-circuit (full damage)
// ════════════════════════════════════════════════════════════════════════

#[test]
fn config_present_zero_stacks_short_circuits_to_full_damage() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let neighbor = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    // No stacks added.

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    assert!((hp_of(&app, primary) - 50.0).abs() < f32::EPSILON);
    assert!((hp_of(&app, neighbor) - 100.0).abs() < f32::EPSILON);
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 26 — Stack 1, one neighbor
// ════════════════════════════════════════════════════════════════════════

#[test]
fn stack_one_primary_and_one_neighbor_split_damage_80_20() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let neighbor = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    // Primary: 50.0 * (1 - 0.20) = 40.0 damage → HP 60.0.
    assert!(
        (hp_of(&app, primary) - 60.0).abs() < f32::EPSILON,
        "primary HP expected 60.0, got {}",
        hp_of(&app, primary)
    );
    // Neighbor: 50.0 * 0.20 / 1 = 10.0 damage → HP 90.0.
    assert!(
        (hp_of(&app, neighbor) - 90.0).abs() < f32::EPSILON,
        "neighbor HP expected 90.0, got {}",
        hp_of(&app, neighbor)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 27 — Stack 1, two neighbors (design-doc §Expected Behaviors #1)
// ════════════════════════════════════════════════════════════════════════

#[test]
fn stack_one_primary_and_two_neighbors_splits_neighbor_share_evenly() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    let n2 = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    // Primary: 50 * 0.80 = 40 → 60 HP.
    assert!((hp_of(&app, primary) - 60.0).abs() < f32::EPSILON);
    // Each neighbor: 50 * 0.20 / 2 = 5 → 95 HP.
    assert!(
        (hp_of(&app, n1) - 95.0).abs() < f32::EPSILON,
        "n1 HP expected 95.0, got {}",
        hp_of(&app, n1)
    );
    assert!(
        (hp_of(&app, n2) - 95.0).abs() < f32::EPSILON,
        "n2 HP expected 95.0, got {}",
        hp_of(&app, n2)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 28 — Stack 3, three neighbors (design-doc §Expected Behaviors #2)
// ════════════════════════════════════════════════════════════════════════

#[test]
fn stack_three_primary_and_three_neighbors_shares_forty_percent_over_three() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let b = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    let c = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 100.0);
    let d = spawn_cell_at(&mut app, Vec2::new(0.0, 50.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 3);

    push_damage(&mut app, damage_msg(primary, 60.0, None));
    tick(&mut app);

    // Primary: 60 * 0.60 = 36 damage → 64 HP.
    assert!(
        (hp_of(&app, primary) - 64.0).abs() < f32::EPSILON,
        "primary HP expected 64.0, got {}",
        hp_of(&app, primary)
    );
    // Neighbors b, c, d each take 60 * 0.40 / 3 = 8.0 → 92 HP.
    for (name, e) in [("b", b), ("c", c), ("d", d)] {
        assert!(
            (hp_of(&app, e) - 92.0).abs() < f32::EPSILON,
            "{name} HP expected 92.0, got {}",
            hp_of(&app, e)
        );
    }
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 29 — Primary takes REDUCED damage (D1 pin)
// ════════════════════════════════════════════════════════════════════════

#[test]
fn primary_takes_reduced_damage_not_full_not_zero() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let _neighbor = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    let primary_hp = hp_of(&app, primary);
    assert!(
        (primary_hp - 60.0).abs() < f32::EPSILON,
        "primary HP must be 60.0 (reduced), not 50.0 (full) or 100.0 (none). Got {primary_hp}"
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 30 — Isolated primary takes FULL damage
// ════════════════════════════════════════════════════════════════════════

#[test]
fn isolated_primary_takes_full_damage() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    // Far-away cell: distance² = 40 000 > ADJACENCY_RADIUS_SQ (4900).
    let far = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    assert!(
        (hp_of(&app, primary) - 50.0).abs() < f32::EPSILON,
        "isolated primary should take FULL 50.0 damage, got HP {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, far) - 100.0).abs() < f32::EPSILON,
        "far cell should be untouched, got HP {}",
        hp_of(&app, far)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 31 — Neighbor at exact ADJACENCY_RADIUS_SQ boundary is included
// ════════════════════════════════════════════════════════════════════════

#[test]
fn neighbor_at_exact_radius_boundary_is_included() {
    // At amount=100, stack=1 (share=20%): primary takes 80 damage (100 ×
    // (1 - 0.20)) → HP 20; neighbor takes 20 damage (100 × 0.20 / 1) → HP
    // 80. Matches design doc §Expected Behaviors #1.
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    // distance² == ADJACENCY_RADIUS_SQ exactly.
    let neighbor = spawn_cell_at(&mut app, Vec2::new(70.0, 0.0), 100.0);
    // Guards against ADJACENCY_RADIUS_SQ drift.
    assert!(
        (ADJACENCY_RADIUS_SQ - 4900.0).abs() < f32::EPSILON,
        "ADJACENCY_RADIUS_SQ drift: expected 4900.0, got {ADJACENCY_RADIUS_SQ}"
    );
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 100.0, None));
    tick(&mut app);

    assert!(
        (hp_of(&app, primary) - 20.0).abs() < f32::EPSILON,
        "primary HP expected 20.0 (took 80.0 damage = 100 × 0.80), got {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, neighbor) - 80.0).abs() < f32::EPSILON,
        "neighbor HP expected 80.0 (took 20.0 damage = 100 × 0.20 / 1), got {}",
        hp_of(&app, neighbor)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 32 — Neighbor just outside radius is excluded
// ════════════════════════════════════════════════════════════════════════

#[test]
fn neighbor_just_outside_radius_is_excluded() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    // distance² = 5041 > ADJACENCY_RADIUS_SQ (4900).
    let far = spawn_cell_at(&mut app, Vec2::new(71.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 100.0, None));
    tick(&mut app);

    // No neighbors → primary takes full 100, HP 0.
    assert!(
        (hp_of(&app, primary) - 0.0).abs() < f32::EPSILON,
        "isolated primary should take FULL 100 damage, got HP {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, far) - 100.0).abs() < f32::EPSILON,
        "out-of-radius cell should be untouched, got HP {}",
        hp_of(&app, far)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 33 — Primary never damages itself (visited-set self-exclusion)
// ════════════════════════════════════════════════════════════════════════

#[test]
fn primary_never_shares_damage_with_itself() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    // No neighbors → isolated rule applies → primary takes full 50.
    assert!(
        (hp_of(&app, primary) - 50.0).abs() < f32::EPSILON,
        "primary should take full 50.0 damage (no neighbors, no self-hit), got HP {}",
        hp_of(&app, primary)
    );

    // Additional assertion: collector must contain EXACTLY the one
    // pre-emitted DamageDealt<Cell>; no ring messages are emitted (design
    // doc: ring damage is applied via in-memory HP accumulation, not via
    // new messages).
    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        1,
        "collector should hold exactly 1 DamageDealt<Cell>, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].target, primary);
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 34 — Dead-marked neighbors are excluded from adjacency
// ════════════════════════════════════════════════════════════════════════

#[test]
fn dead_marked_neighbors_are_excluded_from_adjacency_count() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let alive = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    let dead = spawn_cell_at_dead(&mut app, Vec2::new(-50.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    // With only the live neighbor counted: share = 50 × 0.20 / 1 = 10.0.
    assert!(
        (hp_of(&app, primary) - 60.0).abs() < f32::EPSILON,
        "primary HP expected 60.0, got {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, alive) - 90.0).abs() < f32::EPSILON,
        "alive neighbor HP expected 90.0 (share over 1 live neighbor), got {}",
        hp_of(&app, alive)
    );
    assert!(
        (hp_of(&app, dead) - 100.0).abs() < f32::EPSILON,
        "dead-marked cell must be untouched, got {}",
        hp_of(&app, dead)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 35 — Depth 2 (stack 6) BFS spreads to neighbor-of-neighbor
// ════════════════════════════════════════════════════════════════════════

#[test]
fn depth_two_bfs_attenuates_to_ring_two_neighbor() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    // n2 adjacent to n1 (dist² = 2500), NOT adjacent to primary (dist² = 10000).
    let n2 = spawn_cell_at(&mut app, Vec2::new(100.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 6);

    push_damage(&mut app, damage_msg(primary, 100.0, None));
    tick(&mut app);

    // share = 70%, depth = 2.
    // Primary: 100 * 0.30 = 30 damage → 70 HP.
    // N1 (ring 1, only live neighbor of primary): 100 * 0.70 / 1 = 70 damage → 30 HP.
    // N2 (ring 2, from N1; N1's only unvisited neighbor is N2):
    //   N1's received = 70; N2 takes 70 * 0.70 / 1 = 49 damage → 51 HP.
    assert!(
        (hp_of(&app, primary) - 70.0).abs() < f32::EPSILON,
        "primary HP expected 70.0, got {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, n1) - 30.0).abs() < f32::EPSILON,
        "n1 HP expected 30.0, got {}",
        hp_of(&app, n1)
    );
    assert!(
        (hp_of(&app, n2) - 51.0).abs() < f32::EPSILON,
        "n2 HP expected 51.0, got {}",
        hp_of(&app, n2)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 36 — Depth 2 does NOT back-propagate to primary (cycle prevention)
// ════════════════════════════════════════════════════════════════════════

#[test]
fn depth_two_does_not_rehit_primary_via_neighbor_backedge() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 6);

    push_damage(&mut app, damage_msg(primary, 100.0, None));
    tick(&mut app);

    // share = 70%, depth = 2.
    // Primary: 100 * 0.30 = 30 damage → 70 HP.
    // N1 (ring 1): 100 * 0.70 / 1 = 70 damage → 30 HP.
    // Ring 2 from N1 would target primary — BUT primary is in visited set.
    // No ring-2 damage to primary.
    assert!(
        (hp_of(&app, primary) - 70.0).abs() < f32::EPSILON,
        "primary HP must stay 70.0 (no cycle-back damage), got {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, n1) - 30.0).abs() < f32::EPSILON,
        "n1 HP expected 30.0, got {}",
        hp_of(&app, n1)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 37 — Mutual adjacency: ring-1 siblings don't bounce ring-2 to each other
// ════════════════════════════════════════════════════════════════════════

#[test]
fn ring_one_neighbors_do_not_double_emit_to_each_other_via_ring_two() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let n1 = spawn_cell_at(&mut app, Vec2::new(40.0, 0.0), 100.0);
    let n2 = spawn_cell_at(&mut app, Vec2::new(0.0, 40.0), 100.0);
    // Distances: P↔N1 = 1600, P↔N2 = 1600, N1↔N2 = 1600+1600 = 3200 — all ≤ 4900.
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 6);

    push_damage(&mut app, damage_msg(primary, 100.0, None));
    tick(&mut app);

    // share = 70%, depth = 2.
    // Primary: 100 * 0.30 = 30 → 70 HP.
    // N1 and N2 (both ring-1): each takes 100 * 0.70 / 2 = 35 → 65 HP.
    // Ring-2: N1's unvisited neighbors = {} (N2 is visited); N2's = {} similarly.
    assert!(
        (hp_of(&app, primary) - 70.0).abs() < f32::EPSILON,
        "primary HP expected 70.0, got {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, n1) - 65.0).abs() < f32::EPSILON,
        "n1 HP expected 65.0, got {}",
        hp_of(&app, n1)
    );
    assert!(
        (hp_of(&app, n2) - 65.0).abs() < f32::EPSILON,
        "n2 HP expected 65.0, got {}",
        hp_of(&app, n2)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 38 — Depth 3 (stack 11) spreads three rings + cap engages
// ════════════════════════════════════════════════════════════════════════

#[test]
fn depth_three_bfs_with_cap_propagates_three_rings() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 1000.0);
    let n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 1000.0);
    let n2 = spawn_cell_at(&mut app, Vec2::new(100.0, 0.0), 1000.0);
    let n3 = spawn_cell_at(&mut app, Vec2::new(150.0, 0.0), 1000.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 11);

    push_damage(&mut app, damage_msg(primary, 1000.0, None));
    tick(&mut app);

    // share = min(20 + 10*10, 95) = 95.0%, depth = 3.
    // Primary: 1000 * 0.05 = 50.0 → 950 HP.
    // N1 (ring 1, 1 neighbor): 1000 * 0.95 / 1 = 950 → 50 HP.
    // N2 (ring 2, N1's only unvisited neighbor): 950 * 0.95 / 1 = 902.5 → 97.5 HP.
    // N3 (ring 3, N2's only unvisited neighbor): 902.5 * 0.95 / 1 = 857.375 → 142.625 HP.
    let tol = 1e-3;
    assert!(
        (hp_of(&app, primary) - 950.0).abs() < tol,
        "primary HP expected 950.0, got {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, n1) - 50.0).abs() < tol,
        "n1 HP expected 50.0, got {}",
        hp_of(&app, n1)
    );
    assert!(
        (hp_of(&app, n2) - 97.5).abs() < tol,
        "n2 HP expected 97.5, got {}",
        hp_of(&app, n2)
    );
    assert!(
        (hp_of(&app, n3) - 142.625).abs() < tol,
        "n3 HP expected 142.625, got {}",
        hp_of(&app, n3)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 39 — Multiple incoming messages each redistribute independently
// ════════════════════════════════════════════════════════════════════════

#[test]
fn two_non_overlapping_clusters_each_redistribute_independently() {
    let mut app = build_apply_damage_to_cells_app();
    let p1 = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    let p2 = spawn_cell_at(&mut app, Vec2::new(500.0, 0.0), 100.0);
    let n2 = spawn_cell_at(&mut app, Vec2::new(550.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(p1, 50.0, None));
    push_damage(&mut app, damage_msg(p2, 50.0, None));
    tick(&mut app);

    assert!((hp_of(&app, p1) - 60.0).abs() < f32::EPSILON);
    assert!((hp_of(&app, n1) - 90.0).abs() < f32::EPSILON);
    assert!((hp_of(&app, p2) - 60.0).abs() < f32::EPSILON);
    assert!((hp_of(&app, n2) - 90.0).abs() < f32::EPSILON);
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 40 — Same-primary multi-message accumulates damage correctly
// ════════════════════════════════════════════════════════════════════════

#[test]
fn two_messages_to_same_primary_each_redistribute_and_accumulate() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let neighbor = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    // Each message: primary takes 40.0, neighbor takes 10.0.
    // Totals: primary = 100 - 80 = 20; neighbor = 100 - 20 = 80.
    assert!(
        (hp_of(&app, primary) - 20.0).abs() < f32::EPSILON,
        "primary HP expected 20.0 (took 40 + 40), got {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, neighbor) - 80.0).abs() < f32::EPSILON,
        "neighbor HP expected 80.0 (took 10 + 10), got {}",
        hp_of(&app, neighbor)
    );

    // KilledBy.dealer stays None because neither message carries a dealer.
    let killed_by = app.world().get::<KilledBy>(primary).unwrap();
    assert_eq!(killed_by.dealer, None);
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 40b — Killing-blow attribution follows message order, not arrival
// ════════════════════════════════════════════════════════════════════════

#[test]
fn multi_dealer_killing_blow_attributes_to_order_resolved_dealer() {
    // Two messages hit the same isolated primary in one tick. The first
    // message carries no dealer; the second carries a dealer and pushes
    // running HP past zero. Attribution must follow the order the primary's
    // running HP crosses zero — i.e., the second message's dealer — rather
    // than first-write-wins semantics.
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 15.0);
    // No neighbors → isolated-primary branch → each message delivers full
    // `amount` to primary (pass-through regardless of diffusion).
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    let dealer = app.world_mut().spawn_empty().id();

    // Msg 1: no dealer, drops HP 15 → 5 (still positive).
    push_damage(&mut app, damage_msg(primary, 10.0, None));
    // Msg 2: carries dealer, pushes HP 5 → -15 (killing blow).
    push_damage(&mut app, damage_msg(primary, 20.0, Some(dealer)));
    tick(&mut app);

    assert!(hp_of(&app, primary) <= 0.0, "primary should be killed");
    let killed_by = app.world().get::<KilledBy>(primary).unwrap();
    assert_eq!(
        killed_by.dealer,
        Some(dealer),
        "killing-blow dealer must be the second message's dealer"
    );
}

#[test]
fn multi_dealer_first_message_kills_attributes_to_first() {
    // First message alone kills the primary. Subsequent messages should not
    // overwrite attribution.
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 15.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    let dealer_a = app.world_mut().spawn_empty().id();
    let dealer_b = app.world_mut().spawn_empty().id();

    // Msg 1: dealer_a, 20 dmg → kills (15 → -5).
    push_damage(&mut app, damage_msg(primary, 20.0, Some(dealer_a)));
    // Msg 2: dealer_b, 5 dmg → piles on after kill.
    push_damage(&mut app, damage_msg(primary, 5.0, Some(dealer_b)));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(primary).unwrap();
    assert_eq!(
        killed_by.dealer,
        Some(dealer_a),
        "killing-blow dealer must be the first message's dealer"
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 41 — Zero-amount message does nothing
// ════════════════════════════════════════════════════════════════════════

#[test]
fn zero_amount_message_applies_no_damage() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let neighbor = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 0.0, None));
    tick(&mut app);

    assert!((hp_of(&app, primary) - 100.0).abs() < f32::EPSILON);
    assert!((hp_of(&app, neighbor) - 100.0).abs() < f32::EPSILON);
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 42 — Invulnerable primary absorbs damage and does NOT redistribute
// ════════════════════════════════════════════════════════════════════════

#[test]
fn invulnerable_primary_absorbs_and_does_not_redistribute() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at_invulnerable(&mut app, Vec2::ZERO, 100.0);
    let neighbor = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    assert!(
        (hp_of(&app, primary) - 100.0).abs() < f32::EPSILON,
        "invulnerable primary HP must stay 100.0, got {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, neighbor) - 100.0).abs() < f32::EPSILON,
        "neighbor must be untouched when primary is invulnerable, got {}",
        hp_of(&app, neighbor)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 43 — Dead primary absorbs damage and does NOT redistribute
// ════════════════════════════════════════════════════════════════════════

#[test]
fn dead_primary_absorbs_and_does_not_redistribute() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at_dead(&mut app, Vec2::ZERO, 100.0);
    let neighbor = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    assert!(
        (hp_of(&app, primary) - 100.0).abs() < f32::EPSILON,
        "dead primary HP must stay 100.0, got {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, neighbor) - 100.0).abs() < f32::EPSILON,
        "neighbor must be untouched when primary is dead, got {}",
        hp_of(&app, neighbor)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 44 — KilledBy.dealer is set on reduced-damage killing blow
// ════════════════════════════════════════════════════════════════════════

#[test]
fn killed_by_dealer_set_on_reduced_damage_killing_blow() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 40.0);
    let neighbor = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    let dealer = app.world_mut().spawn_empty().id();
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 50.0, Some(dealer)));
    tick(&mut app);

    // Primary took 40.0 reduced damage (50 * 0.80), HP 40 - 40 = 0.0 — killed.
    assert!(
        (hp_of(&app, primary) - 0.0).abs() < f32::EPSILON,
        "primary HP expected 0.0 (killing blow at reduced damage), got {}",
        hp_of(&app, primary)
    );
    let killed_by = app.world().get::<KilledBy>(primary).unwrap();
    assert_eq!(
        killed_by.dealer,
        Some(dealer),
        "KilledBy.dealer must be the original dealer, got {:?}",
        killed_by.dealer
    );
    // Neighbor still receives share: 50 * 0.20 / 1 = 10 → HP 90.
    assert!(
        (hp_of(&app, neighbor) - 90.0).abs() < f32::EPSILON,
        "neighbor HP expected 90.0, got {}",
        hp_of(&app, neighbor)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 45 — Overkill on primary still redistributes to neighbor
// ════════════════════════════════════════════════════════════════════════

#[test]
fn overkill_on_primary_still_emits_share_to_neighbor() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 10.0);
    let neighbor = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    // Primary: 10 - 40 = -30 HP.
    assert!(
        (hp_of(&app, primary) - (-30.0)).abs() < f32::EPSILON,
        "primary HP expected -30.0 (overkill), got {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, neighbor) - 90.0).abs() < f32::EPSILON,
        "neighbor HP expected 90.0 (still takes share), got {}",
        hp_of(&app, neighbor)
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 46 — Sub-HP share damage applied without rounding
// ════════════════════════════════════════════════════════════════════════

#[test]
fn sub_hp_share_damage_applies_without_rounding() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 1.0);
    let n2 = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 1.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 5.0, None));
    tick(&mut app);

    // Primary: 5 * 0.80 = 4 damage → HP 96.0.
    // Each neighbor: 5 * 0.20 / 2 = 0.5 damage → HP 0.5.
    assert!(
        (hp_of(&app, primary) - 96.0).abs() < f32::EPSILON,
        "primary HP expected 96.0, got {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, n1) - 0.5).abs() < f32::EPSILON,
        "n1 HP expected 0.5, got {}",
        hp_of(&app, n1)
    );
    assert!(
        (hp_of(&app, n2) - 0.5).abs() < f32::EPSILON,
        "n2 HP expected 0.5, got {}",
        hp_of(&app, n2)
    );
}
