use bevy::prelude::*;

use super::{
    super::helpers::{
        build_afterimage_app, captured_bump_performed, seed_active_protocols_with_afterimage,
        spawn_breaker_with_bump_state, spawn_real_bolt,
    },
    perfect_bump_state, spawn_canonical_phantom_breaker,
};
use crate::{breaker::messages::BumpGrade, prelude::*};

// ── CB1 — bolt overlapping + down → mirror, reposition, emit one bump ─────

#[test]
fn bolt_overlapping_and_moving_down_reflects_and_emits_one_bump_performed() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        // Bolt center at (0, 14) — 4 units inside the top face at y=10.
        // With radius 6 the bolt overlaps the AABB (bottom of bolt at y=8).
        // Harness has no velocity integration, so the bolt must be placed
        // already overlapping at spawn time for the overlap check to fire.
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app
        .world()
        .get::<Velocity2D>(bolt)
        .expect("bolt velocity must exist after tick");
    assert!(
        velocity.0.y > 0.0,
        "bolt velocity.y must be reflected upward after overlap, got {}",
        velocity.0.y
    );
    assert!(
        (velocity.0.y - 400.0).abs() < 1.0,
        "reflected |velocity.y| must be ~400.0, got {}",
        velocity.0.y
    );
    assert!(
        velocity.0.x.abs() < f32::EPSILON,
        "x component must be preserved at 0.0, got {}",
        velocity.0.x
    );

    let position = app
        .world()
        .get::<Position2D>(bolt)
        .expect("bolt position must exist after tick");
    assert!(
        position.0.y <= 10.0 + 6.0 + 0.01 && position.0.y >= 10.0 + 6.0 - 0.01,
        "bolt must be repositioned to top face (10.0) + radius (6.0) = 16.0, got y={}",
        position.0.y
    );

    let bumps = captured_bump_performed(&app);
    assert_eq!(
        bumps.len(),
        1,
        "exactly ONE BumpPerformed must be emitted on overlap, got {}",
        bumps.len()
    );
    assert_eq!(bumps[0].bolt, Some(bolt));
    assert_eq!(
        bumps[0].breaker, phantom,
        "BumpPerformed.breaker must be the PHANTOM entity (not the real breaker)"
    );
    assert_eq!(bumps[0].grade, BumpGrade::Perfect);
}

// ── CB1 (edge case) — deeply overlapping bolt still produces one reflect ──

#[test]
fn deeply_overlapping_bolt_still_reflects_exactly_once() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 5.0), // deeply inside the AABB
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let bumps = captured_bump_performed(&app);
    assert_eq!(
        bumps.len(),
        1,
        "a deeply overlapping bolt moving down must still emit EXACTLY one BumpPerformed, got {}",
        bumps.len()
    );
    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "bolt must be reflected upward, got velocity.y = {}",
        velocity.0.y
    );
}

// ── CB2 — bolt NOT overlapping → no reflection, no BumpPerformed ──────────

#[test]
fn bolt_not_overlapping_phantom_does_not_reflect_or_emit() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(500.0, 500.0), // well outside AABB
        Vec2::new(0.0, 400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.y - 400.0).abs() < 1.0,
        "bolt velocity must be UNCHANGED when no overlap, got y={}",
        velocity.0.y
    );
    assert!(
        captured_bump_performed(&app).is_empty(),
        "zero BumpPerformed messages expected when bolt does not overlap"
    );
}

// ── CB2 (edge case) — bolt aligned on y but x-outside does not overlap ────

#[test]
fn bolt_aligned_on_y_but_x_outside_does_not_overlap() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let _bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(200.0, 0.0), // x outside half-width 50
        Vec2::new(0.0, 400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    assert!(
        captured_bump_performed(&app).is_empty(),
        "bolt aligned on y but x-outside AABB must NOT emit BumpPerformed"
    );
}
