use bevy::prelude::*;

use super::{
    super::{
        super::system::{PhantomBreaker, PhantomBreakerLifetime},
        helpers::{
            build_afterimage_app, captured_bump_performed, seed_active_protocols_with_afterimage,
            spawn_breaker_with_bump_state, spawn_real_bolt,
        },
    },
    perfect_bump_state, spawn_canonical_phantom_breaker,
};
use crate::{
    breaker::components::{BaseHeight, BaseWidth, BumpState},
    effect_v3::effects::anchor::components::{AnchorActive, AnchorPlanted},
    prelude::*,
};

// ── CB9 — no PhantomBreaker in world → no-op ──────────────────────────────

#[test]
fn zero_phantom_breakers_in_world_is_noop() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.y - (-400.0)).abs() < 1.0,
        "no PhantomBreaker in world → bolt velocity unchanged, got y={}",
        velocity.0.y
    );
    assert!(
        captured_bump_performed(&app).is_empty(),
        "no PhantomBreaker in world → zero BumpPerformed emitted"
    );
}

// ── CB9 (edge case) — zero bolts AND zero phantoms = quiet tick ───────────

#[test]
fn zero_phantoms_and_zero_bolts_is_quiet() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);

    tick(&mut app);

    assert!(captured_bump_performed(&app).is_empty());
}

// ── CB10 — no real Breaker in world → early-return ────────────────────────

#[test]
fn zero_real_breakers_in_world_is_noop() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );
    // No real breaker spawned — `breaker_query.single()` should fail.

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.y - (-400.0)).abs() < 1.0,
        "no real Breaker → bolt velocity unchanged, got y={}",
        velocity.0.y
    );
    assert!(
        captured_bump_performed(&app).is_empty(),
        "no real Breaker → zero BumpPerformed emitted"
    );
}

// ── CB10 (edge case) — TWO real breakers still triggers early-return ──────

#[test]
fn two_real_breakers_is_noop() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker_a = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let _breaker_b = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let _bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    assert!(
        captured_bump_performed(&app).is_empty(),
        "two real breakers → breaker_query.single() fails → no BumpPerformed"
    );
}

// ── Anchor multiplier widens effective perfect window for phantom bounce ──

#[test]
fn anchor_planted_and_active_scales_perfect_window_for_phantom_bounce_grade() {
    // Given: a real breaker with BumpPerfectWindow(0.1), BumpLateWindow(0.2),
    // and BumpState { active: true, timer: 0.15 } — `timer` is JUST past the
    // raw perfect window (0.15 > 0.1), so the forward-grade would be `Early`
    // without anchor widening. Attach AnchorPlanted + AnchorActive with
    // perfect_window_multiplier: 2.0 so effective_pw = 0.1 * 2.0 = 0.2,
    // putting `timer` (0.15) back inside the widened perfect window.
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_canonical_phantom_breaker(&mut app);

    let widened_state = BumpState {
        active:         true,
        timer:          0.15, // > raw 0.1 (Early) but <= widened 0.2 (Perfect)
        post_hit_timer: 0.0,
        cooldown:       0.0,
        last_hit_bolt:  None,
    };
    let breaker = spawn_breaker_with_bump_state(&mut app, widened_state, 0.1, 0.2);
    app.world_mut().entity_mut(breaker).insert((
        AnchorPlanted,
        AnchorActive {
            bump_force_multiplier:     1.5,
            perfect_window_multiplier: 2.0,
            plant_delay:               0.3,
        },
    ));

    // Bolt overlapping the phantom AABB, moving DOWN into it.
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let bumps = captured_bump_performed(&app);
    assert_eq!(
        bumps.len(),
        1,
        "exactly one BumpPerformed expected on overlap, got {}",
        bumps.len()
    );
    assert_eq!(bumps[0].bolt, Some(bolt));
    assert_eq!(bumps[0].breaker, phantom);
    assert_eq!(
        bumps[0].grade,
        crate::breaker::messages::BumpGrade::Perfect,
        "AnchorPlanted + AnchorActive(perfect_window_multiplier: 2.0) must \
         widen effective_pw from 0.1 to 0.2, promoting timer=0.15 from Early \
         to Perfect"
    );
}

// ── Regression — horizontal bolt tangentially overlapping phantom no-ops ──
//
// Off-by-one regression in the entry guard: prior to the fix, the check
// read `velocity.y > 0.0` instead of `velocity.y >= 0.0`. A bolt with
// purely horizontal motion (velocity.y == 0.0) that happened to tangentially
// overlap a phantom AABB would fall through the guard, emit a spurious
// `BumpPerformed`, mirror velocity.y (a no-op for 0.0), and snap the bolt's
// y position to the top face + radius. Horizontal tangential contact is
// not a bump — the guard must treat `velocity.y == 0.0` the same as
// `velocity.y > 0.0`. This test pins the inclusive-zero behaviour.

#[test]
fn horizontal_velocity_tangentially_overlapping_phantom_does_not_emit_bump_performed() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);

    // PhantomBreaker at (0.0, -50.0) with half-extents (30.0, 5.0).
    // AABB: x ∈ [-30.0, 30.0], y ∈ [-55.0, -45.0] — top face at y = -45.0.
    app.world_mut().spawn((
        PhantomBreaker,
        PhantomBreakerLifetime(1.5),
        Position2D(Vec2::new(0.0, -50.0)),
        BaseWidth(60.0),
        BaseHeight(10.0),
    ));

    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);

    // Bolt center 1 unit above the phantom top face (y = -44.0 vs top = -45.0),
    // radius 8.0 → circle bottom at y = -52.0 tangentially overlaps the AABB.
    // Velocity is PURELY HORIZONTAL (velocity.y == 0.0).
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, -44.0),
        Vec2::new(300.0, 0.0),
        10.0,
        8.0,
    );

    tick(&mut app);

    // Guard must skip — no BumpPerformed.
    assert!(
        captured_bump_performed(&app).is_empty(),
        "horizontal bolt (velocity.y == 0.0) tangentially overlapping phantom must NOT emit \
         BumpPerformed — the entry guard must use `velocity.y >= 0.0` (inclusive), got {} emissions",
        captured_bump_performed(&app).len(),
    );

    // Velocity must be unchanged — no mirror, no snap.
    let velocity = app
        .world()
        .get::<Velocity2D>(bolt)
        .expect("bolt velocity must exist after tick");
    assert_eq!(
        velocity.0,
        Vec2::new(300.0, 0.0),
        "horizontal bolt velocity must be UNCHANGED (no reflection, no mirror), got {:?}",
        velocity.0,
    );

    // Position y must be unchanged — the system must not have snapped the
    // bolt to top-face + radius.
    let position = app
        .world()
        .get::<Position2D>(bolt)
        .expect("bolt position must exist after tick");
    assert!(
        (position.0.y - (-44.0)).abs() < f32::EPSILON,
        "bolt position y must be UNCHANGED (no snap), expected -44.0, got {}",
        position.0.y,
    );
}
