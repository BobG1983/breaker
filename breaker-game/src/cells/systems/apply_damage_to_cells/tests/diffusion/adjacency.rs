//! Adjacency filtering: radius boundaries, self-exclusion, and dead-neighbor
//! exclusion.

use bevy::prelude::*;

use super::helpers::*;
use crate::{cells::components::ADJACENCY_RADIUS_SQ, prelude::*};

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
