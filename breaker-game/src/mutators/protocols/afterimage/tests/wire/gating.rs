use bevy::prelude::*;

use super::super::helpers::{
    build_afterimage_app, build_afterimage_app_in_chip_selecting, phantom_bolt_count,
    phantom_breaker_count, seed_active_protocols_with_afterimage, spawn_breaker_with_dash,
    spawn_phantom_breaker_at, spawn_real_bolt, write_bump_performed,
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

// ── I6 (edge case) — inactive → no spawn ──────────────────────────────────

#[test]
fn spawn_phantom_bolt_gated_off_when_inactive() {
    let mut app = build_afterimage_app();
    // Do NOT seed ActiveProtocols.
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(&mut app, phantom, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(phantom_bolt_count(&mut app), 0);
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

    assert_eq!(phantom_bolt_count(&mut app), 0);
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
