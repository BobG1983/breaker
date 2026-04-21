//! Diffusion hazard gating: `DiffusionConfig` absence, `ActiveHazards` absence,
//! zero stacks, and reduced-damage pin.

use bevy::prelude::*;

use super::helpers::*;

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
