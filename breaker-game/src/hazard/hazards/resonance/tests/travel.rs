//! Group E — `resonance_wave_travel`.

use std::time::Duration;

use bevy::prelude::*;

use super::{
    super::system::{ResonanceWave, resonance_wave_travel},
    helpers::{SpawnWaveParams, spawn_wave, test_app_playing, tick_with_dt},
};
use crate::prelude::*;

// ── E1 — wave moves toward target at speed * dt each tick ──────────────

#[test]
fn e1_wave_moves_toward_target_at_speed_times_dt() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, resonance_wave_travel);
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 300.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let pos = app.world().get::<Position2D>(wave).unwrap();
    let got = pos.0;
    let expected = Vec2::new(0.0, 100.0);
    assert!(
        (got - expected).length() < 1e-4,
        "Expected position {expected:?}, got {got:?}"
    );
    let age = app.world().get::<ResonanceWave>(wave).unwrap().age;
    assert!(
        (age - 1.0).abs() < 1e-5,
        "Age should tick to ~1.0, got {age}"
    );
}

// ── E2 — wave travels toward spawn-time target, not live breaker ───────

#[test]
fn e2_wave_travels_toward_stored_target_not_live_breaker() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, resonance_wave_travel);
    // Breaker far to the right (but travel system does NOT read breaker).
    app.world_mut().spawn((
        crate::breaker::components::Breaker,
        Position2D(Vec2::new(500.0, 0.0)),
    ));
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 300.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let pos = app.world().get::<Position2D>(wave).unwrap();
    assert!(
        (pos.0 - Vec2::new(0.0, 100.0)).length() < 1e-4,
        "Wave should head toward stored target, got {:?}",
        pos.0
    );
}

// ── E3 — diagonal movement follows unit direction to target ────────────

#[test]
fn e3_wave_diagonal_movement_follows_unit_direction() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, resonance_wave_travel);
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(300.0, 400.0),
            speed:             100.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let pos = app.world().get::<Position2D>(wave).unwrap();
    let expected = Vec2::new(240.0, 320.0);
    assert!(
        (pos.0 - expected).length() < 1e-3,
        "Expected diagonal move to {expected:?}, got {:?}",
        pos.0
    );
}

// ── E4 — age increments by dt ──────────────────────────────────────────

#[test]
fn e4_wave_age_increments_by_dt() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, resonance_wave_travel);
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 300.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               0.5,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.25));

    let age = app.world().get::<ResonanceWave>(wave).unwrap().age;
    assert!(
        (age - 0.75).abs() < 1e-5,
        "Age should be 0.5 + 0.25 = 0.75, got {age}"
    );
}

// ── E5 — travel does NOT despawn on lifetime ───────────────────────────

#[test]
fn e5_travel_does_not_despawn_on_age() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, resonance_wave_travel);
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(500.0, 500.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               1.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert!(
        app.world().get_entity(wave).is_ok(),
        "Travel must not despawn non-expired waves"
    );
    let age = app.world().get::<ResonanceWave>(wave).unwrap().age;
    assert!(
        (age - 1.1).abs() < 1e-5,
        "Age should advance to 1.1, got {age}"
    );
}

// ── E6 — wave at target stays put, ages ────────────────────────────────

#[test]
fn e6_wave_at_target_stays_put_but_ages() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, resonance_wave_travel);
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 0.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.5));

    let pos = app.world().get::<Position2D>(wave).unwrap();
    assert!(
        (pos.0 - Vec2::new(0.0, 0.0)).length() < f32::EPSILON,
        "Zero direction → no movement, got {:?}",
        pos.0
    );
    assert!(
        !pos.0.x.is_nan() && !pos.0.y.is_nan(),
        "Position must not be NaN"
    );
    let age = app.world().get::<ResonanceWave>(wave).unwrap().age;
    assert!(
        (age - 0.5).abs() < 1e-5,
        "Age should tick to ~0.5, got {age}"
    );
}

// ── E7 — no overshoot: clamps at target ─────────────────────────────────

#[test]
fn e7_wave_does_not_overshoot_target_in_one_tick() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, resonance_wave_travel);
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 10.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let pos = app.world().get::<Position2D>(wave).unwrap();
    assert!(
        (pos.0 - Vec2::new(0.0, 0.0)).length() < f32::EPSILON,
        "Wave must clamp at target (no overshoot), got {:?}",
        pos.0
    );
}
