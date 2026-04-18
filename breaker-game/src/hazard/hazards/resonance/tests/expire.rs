//! MB2 — `resonance_wave_expire` in isolation.

use std::time::Duration;

use bevy::prelude::*;

use super::{
    super::system::{ResonanceWave, resonance_wave_expire},
    helpers::{SpawnWaveParams, spawn_wave, test_app_playing, tick_with_dt},
};

// ── MB2 — wave_expire in isolation ─────────────────────────────────────

#[test]
fn mb2_wave_expire_despawns_at_age_equal_to_max_lifetime_but_not_below() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, resonance_wave_expire);

    let wave_a = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::ZERO,
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::ZERO,
            age:               10.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );
    let wave_b = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::ZERO,
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::ZERO,
            age:               9.5,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(
        app.world().get_entity(wave_a).is_err(),
        "Wave A (age==max_lifetime) should be despawned (>=)"
    );
    assert!(
        app.world().get_entity(wave_b).is_ok(),
        "Wave B (age<max_lifetime) must not be despawned"
    );
    let age_b = app.world().get::<ResonanceWave>(wave_b).unwrap().age;
    assert!(
        (age_b - 9.5).abs() < 1e-5,
        "expire must not tick age; got {age_b}"
    );
}

// ── Sibling — wave strictly below max_lifetime is preserved ────────────

#[test]
fn wave_with_age_below_max_lifetime_is_not_despawned() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, resonance_wave_expire);

    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::ZERO,
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::ZERO,
            age:               9.9,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(
        app.world().get_entity(wave).is_ok(),
        "Wave with age<max_lifetime must survive expire"
    );
    let age = app.world().get::<ResonanceWave>(wave).unwrap().age;
    assert!(
        (age - 9.9).abs() < 1e-5,
        "expire must not tick age; got {age}"
    );
}
