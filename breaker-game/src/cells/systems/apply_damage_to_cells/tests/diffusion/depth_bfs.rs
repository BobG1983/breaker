//! Multi-ring BFS spread: depth 2, depth 3, cycle prevention, mutual
//! adjacency.

use bevy::prelude::*;

use super::helpers::*;

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
