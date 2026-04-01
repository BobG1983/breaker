use bevy::prelude::*;

use super::helpers::*;
use crate::bolt::components::PiercingRemaining;

// ── Behavior 9: bolt_wall_collision resets PiercingRemaining to ActivePiercings.total() on wall overlap ──

#[test]
fn bolt_overlapping_wall_resets_piercing_remaining() {
    // Given: Bolt overlapping a wall, with ActivePiercings(vec![3]) and PiercingRemaining(1)
    // When: bolt_wall_collision detects wall overlap
    // Then: PiercingRemaining resets to 3 (matching ActivePiercings.total())
    let mut app = test_app();

    // Wall at x=-5 with half_width=5, bolt at x=-2 with radius 8 => overlap
    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_piercing_bolt(
        &mut app,
        -2.0,
        200.0, // position: inside wall's expanded AABB
        -400.0,
        0.0,     // velocity: moving left
        vec![3], // ActivePiercings(vec![3])
        1,       // PiercingRemaining(1) — partially spent
    );

    tick(&mut app);

    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("PiercingRemaining should still be present on bolt");
    assert_eq!(
        pr.0, 3,
        "PiercingRemaining should reset to ActivePiercings.total() (3) on wall overlap, got {}",
        pr.0
    );
}

/// Edge case: `PiercingRemaining` without `ActivePiercings` stays unchanged.
#[test]
fn bolt_with_piercing_remaining_but_no_active_piercings_unchanged_on_wall_hit() {
    let mut app = test_app();

    // Wall at x=-5 with half_width=5, bolt at x=-2 with radius 8 => overlap
    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);

    // Spawn bolt with PiercingRemaining but NO ActivePiercings
    let bolt_entity = spawn_bolt(&mut app, -2.0, 200.0, -400.0, 0.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(PiercingRemaining(1));

    tick(&mut app);

    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("PiercingRemaining should still be present on bolt");
    assert_eq!(
        pr.0, 1,
        "PiercingRemaining without ActivePiercings should stay at 1 on wall overlap, got {}",
        pr.0
    );
}

// ── Behavior 5: bolt_wall_collision resets PiercingRemaining from ActivePiercings.total() ──

/// Given: Bolt with `ActivePiercings(vec![2, 1])`, `PiercingRemaining(0)`, NO `EffectivePiercing`.
/// When: bolt hits wall.
/// Then: `PiercingRemaining` = 3 (2 + 1).
///
/// Fails at RED because production reads `EffectivePiercing` (absent -> no reset).
#[test]
fn bolt_wall_collision_resets_piercing_remaining_from_active_piercings_total() {
    let mut app = test_app();

    // Wall at x=-5 with half_width=5, bolt at x=-2 with radius 8 => overlap
    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_piercing_bolt(
        &mut app,
        -2.0,
        200.0, // position: inside wall's expanded AABB
        -400.0,
        0.0,        // velocity: moving left
        vec![2, 1], // ActivePiercings(vec![2, 1]) -> total = 3
        0,          // PiercingRemaining(0)
    );

    tick(&mut app);

    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("PiercingRemaining should still be present on bolt");
    assert_eq!(
        pr.0, 3,
        "PiercingRemaining should reset to ActivePiercings.total() (2 + 1 = 3) on wall overlap, got {}",
        pr.0
    );
}
