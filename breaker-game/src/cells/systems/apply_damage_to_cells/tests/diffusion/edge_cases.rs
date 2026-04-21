//! Edge-case behaviours: zero amount, invulnerable/dead primary absorption,
//! overkill, and sub-HP share precision.

use bevy::prelude::*;

use super::helpers::*;

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
