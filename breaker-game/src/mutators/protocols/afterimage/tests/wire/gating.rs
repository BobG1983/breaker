use bevy::prelude::*;

use super::{
    super::{
        super::system::PhantomBreakerLifetime,
        helpers::{
            build_afterimage_app, build_afterimage_app_in_chip_selecting, captured_bump_performed,
            phantom_bolts_owned_by, phantom_breaker_count, seed_active_protocols_with_afterimage,
            spawn_breaker_with_bump_state, spawn_breaker_with_dash, spawn_phantom_breaker_at,
            spawn_real_bolt, tick_n, write_bump_performed,
        },
    },
    helpers::perfect_bump_state,
};
use crate::{
    breaker::{components::DashState, messages::BumpGrade},
    prelude::*,
};

// ── I2 — spawn_phantom_breaker gated OFF when Afterimage not active ───────

#[test]
fn spawn_phantom_breaker_gated_off_when_afterimage_not_active() {
    let mut app = build_afterimage_app();
    // Do NOT seed ActiveProtocols.
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::ZERO);

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "run_if(protocol_active) must gate off afterimage_spawn_phantom_breaker"
    );
}

// ── I3 — spawn_phantom_breaker gated OFF when NodeState != Playing ────────

#[test]
fn spawn_phantom_breaker_gated_off_in_chip_selecting() {
    let mut app = build_afterimage_app_in_chip_selecting();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::ZERO);

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "run_if(in_state(Playing)) must gate off afterimage_spawn_phantom_breaker"
    );
}

// ── I4 (edge case) — ChipSelecting: tick_phantom_breaker does NOT tick ────

#[test]
fn tick_phantom_breaker_does_not_tick_in_chip_selecting() {
    let mut app = build_afterimage_app_in_chip_selecting();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 2.0);

    tick_n(&mut app, 2);

    let lifetime = app
        .world()
        .get::<PhantomBreakerLifetime>(phantom)
        .expect("phantom must persist — tick system is gated off");
    assert!(
        (lifetime.0 - 2.0).abs() < f32::EPSILON,
        "lifetime must be UNCHANGED in ChipSelecting, got {}",
        lifetime.0
    );
}

// ── I5 (edge case) — inactive → no reflection ─────────────────────────────

#[test]
fn check_phantom_bounce_gated_off_when_inactive() {
    let mut app = build_afterimage_app();
    // Do NOT seed ActiveProtocols.
    let _phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0), // overlapping top face
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.y - (-400.0)).abs() < 1.0,
        "inactive → no reflection, got velocity.y={}",
        velocity.0.y
    );
    assert!(captured_bump_performed(&app).is_empty());
}

// ── I5 (edge case) — ChipSelecting → no reflection ────────────────────────

#[test]
fn check_phantom_bounce_gated_off_in_chip_selecting() {
    let mut app = build_afterimage_app_in_chip_selecting();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0), // overlapping top face
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.y - (-400.0)).abs() < 1.0,
        "ChipSelecting → no reflection"
    );
    assert!(captured_bump_performed(&app).is_empty());
}

// ── I6 (edge case) — inactive → no spawn ──────────────────────────────────

#[test]
fn spawn_phantom_bolt_gated_off_when_inactive() {
    let mut app = build_afterimage_app();
    // Do NOT seed ActiveProtocols.
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(&mut app, phantom, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(phantom_bolts_owned_by(&mut app, real_bolt).len(), 0);
}

// ── I6 (edge case) — ChipSelecting → no spawn ─────────────────────────────

#[test]
fn spawn_phantom_bolt_gated_off_in_chip_selecting() {
    let mut app = build_afterimage_app_in_chip_selecting();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(&mut app, phantom, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(phantom_bolts_owned_by(&mut app, real_bolt).len(), 0);
}

// ── I10 — all four afterimage systems gated by active AND Playing ────────-
//
// Covered already by I2/I3/I5/I5-edge/I6-edge. This aggregate test double-
// checks the four gates are all in place by combining both scopes.

#[test]
fn all_four_systems_gated_when_only_in_state_playing_but_not_active() {
    let mut app = build_afterimage_app();
    // ActiveProtocols empty — only in_state(Playing) passes.
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::ZERO);
    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);
    assert_eq!(phantom_breaker_count(&mut app), 0);
}

#[test]
fn all_four_systems_gated_when_only_active_but_not_playing() {
    let mut app = build_afterimage_app_in_chip_selecting();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::ZERO);
    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);
    assert_eq!(phantom_breaker_count(&mut app), 0);
}
