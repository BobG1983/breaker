//! Core share-split distribution: primary vs neighbor share math for stacked
//! Diffusion.

use bevy::prelude::*;

use super::helpers::*;

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
