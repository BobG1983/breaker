//! Group B — `drift_update_wind` system.
//!
//! Every test in this group wires ONLY `drift_update_wind` via
//! `wire_update_wind_only(&mut app)`. This bypasses the `wire`-installed
//! run conditions so the system runs unconditionally.

use std::time::Duration;

use bevy::prelude::*;

use super::{
    super::system::{DriftConfig, DriftWind, drift_update_wind},
    helpers::{
        canonical_config, insert_rng, install_drift_config, install_drift_wind, test_app_playing,
        tick_with_dt, wire_update_wind_only,
    },
};

// ── Behavior 11 — timer expiry → unit vector + timer = period_secs ───────

#[test]
fn wind_rolls_new_direction_when_timer_expires() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, drift_update_wind);
    insert_rng(&mut app, 42);
    app.world_mut().insert_resource(DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 33.3,
    });
    app.world_mut().insert_resource(DriftWind {
        direction: Vec2::X,
        timer:     0.0,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let wind = app.world().resource::<DriftWind>();
    assert!(
        (wind.direction.length() - 1.0).abs() < 1e-5,
        "direction should remain a unit vector, got len={}",
        wind.direction.length()
    );
    assert!(
        (wind.timer - 8.0).abs() < 1e-5,
        "timer should reset to period_secs, got {}",
        wind.timer
    );
}

#[test]
fn wind_reset_is_unconditional_on_prior_timer_overshoot() {
    // Edge: starting from timer=-3.5 (already expired by multiple frames)
    // still resets to period_secs exactly. Pins the assignment (`=`),
    // NOT an add (`+=`).
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    insert_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     -3.5,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let wind = app.world().resource::<DriftWind>();
    assert!(
        (wind.timer - 8.0).abs() < 1e-5,
        "timer should reset to period_secs regardless of prior overshoot"
    );
    assert!((wind.direction.length() - 1.0).abs() < 1e-5);
}

// ── Behavior 12 — timer still positive → direction unchanged, timer dec──

#[test]
fn wind_direction_stays_constant_within_interval() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, drift_update_wind);
    insert_rng(&mut app, 42);
    app.world_mut().insert_resource(DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 33.3,
    });
    let initial = Vec2::new(0.5, 0.5).normalize();
    app.world_mut().insert_resource(DriftWind {
        direction: initial,
        timer:     4.0,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let wind = app.world().resource::<DriftWind>();
    assert!((wind.direction - initial).length() < 1e-5);
    assert!((wind.timer - 3.9).abs() < 1e-5);
}

#[test]
fn wind_timer_decrement_accumulates_across_consecutive_ticks() {
    // Edge: two consecutive 0.1s ticks leave direction unchanged AND
    // timer ≈ 3.8.
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    insert_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    let initial = Vec2::new(0.5, 0.5).normalize();
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: initial,
            timer:     4.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let wind = app.world().resource::<DriftWind>();
    assert!((wind.direction - initial).length() < 1e-5);
    assert!((wind.timer - 3.8).abs() < 1e-5);
}

// ── Behavior 13 — seeded RNG produces deterministic direction at same seed ─

#[test]
fn seeded_rng_produces_deterministic_first_roll() {
    let mut app_a = test_app_playing();
    wire_update_wind_only(&mut app_a);
    insert_rng(&mut app_a, 42);
    install_drift_config(&mut app_a, canonical_config());
    install_drift_wind(
        &mut app_a,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );

    let mut app_b = test_app_playing();
    wire_update_wind_only(&mut app_b);
    insert_rng(&mut app_b, 42);
    install_drift_config(&mut app_b, canonical_config());
    install_drift_wind(
        &mut app_b,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );

    tick_with_dt(&mut app_a, Duration::from_secs_f32(0.1));
    tick_with_dt(&mut app_b, Duration::from_secs_f32(0.1));

    let dir_a = app_a.world().resource::<DriftWind>().direction;
    let dir_b = app_b.world().resource::<DriftWind>().direction;
    assert!(
        (dir_a - dir_b).length() < 1e-6,
        "same seed should produce same first-roll direction"
    );
}

#[test]
fn different_seed_produces_different_direction() {
    // Edge: a seed shift moves the angle noticeably.
    let mut app_a = test_app_playing();
    wire_update_wind_only(&mut app_a);
    insert_rng(&mut app_a, 42);
    install_drift_config(&mut app_a, canonical_config());
    install_drift_wind(
        &mut app_a,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );

    let mut app_b = test_app_playing();
    wire_update_wind_only(&mut app_b);
    insert_rng(&mut app_b, 43);
    install_drift_config(&mut app_b, canonical_config());
    install_drift_wind(
        &mut app_b,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );

    tick_with_dt(&mut app_a, Duration::from_secs_f32(0.1));
    tick_with_dt(&mut app_b, Duration::from_secs_f32(0.1));

    let dir_a = app_a.world().resource::<DriftWind>().direction;
    let dir_b = app_b.world().resource::<DriftWind>().direction;
    assert!(
        (dir_a - dir_b).length() > 1e-4,
        "different seeds should produce different directions"
    );
}

// ── Behavior 14 — deterministic direction across multiple rolls ──────────

#[test]
fn seeded_rng_deterministic_across_three_consecutive_rolls() {
    let mut app_a = test_app_playing();
    wire_update_wind_only(&mut app_a);
    insert_rng(&mut app_a, 42);
    install_drift_config(
        &mut app_a,
        DriftConfig {
            force:           100.0,
            period_secs:     1.0,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app_a,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );

    let mut app_b = test_app_playing();
    wire_update_wind_only(&mut app_b);
    insert_rng(&mut app_b, 42);
    install_drift_config(
        &mut app_b,
        DriftConfig {
            force:           100.0,
            period_secs:     1.0,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app_b,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );

    for _ in 0..3 {
        tick_with_dt(&mut app_a, Duration::from_secs_f32(1.5));
        tick_with_dt(&mut app_b, Duration::from_secs_f32(1.5));
        let dir_a = app_a.world().resource::<DriftWind>().direction;
        let dir_b = app_b.world().resource::<DriftWind>().direction;
        assert!(
            (dir_a - dir_b).length() < 1e-6,
            "same seed should produce identical roll sequences"
        );
    }
}

// ── Behavior 15 — direction is a unit vector after each roll ─────────────

#[test]
fn direction_is_unit_vector_across_five_consecutive_rolls() {
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    insert_rng(&mut app, 7);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     0.5,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );

    let mut rolled: Vec<Vec2> = Vec::new();
    for _ in 0..5 {
        tick_with_dt(&mut app, Duration::from_secs_f32(0.6));
        let dir = app.world().resource::<DriftWind>().direction;
        assert!(
            (dir.length() - 1.0).abs() < 1e-5,
            "direction must remain a unit vector, got len={}",
            dir.length()
        );
        rolled.push(dir);
    }

    // Edge: at least two of the five rolled directions differ noticeably.
    let first = rolled[0];
    let differs = rolled.iter().skip(1).any(|d| (*d - first).length() > 1e-4);
    assert!(
        differs,
        "at least one subsequent roll must differ from the first"
    );
}

// ── Behavior 16 — missing DriftConfig → no-op, no panic ──────────────────

#[test]
fn missing_drift_config_is_noop() {
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    insert_rng(&mut app, 42);
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );
    // No `DriftConfig` inserted.

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let wind = app.world().resource::<DriftWind>();
    assert_eq!(
        wind.timer.to_bits(),
        0.0_f32.to_bits(),
        "timer must be bitwise unchanged"
    );
    assert_eq!(wind.direction, Vec2::X);
}

#[test]
fn missing_drift_config_is_noop_across_two_ticks() {
    // Edge: second tick still leaves state unchanged.
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    insert_rng(&mut app, 42);
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let wind = app.world().resource::<DriftWind>();
    assert_eq!(wind.timer.to_bits(), 0.0_f32.to_bits());
    assert_eq!(wind.direction, Vec2::X);
}

// ── Behavior 17 — missing DriftWind → no-op, no panic ────────────────────

#[test]
fn missing_drift_wind_is_noop() {
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    insert_rng(&mut app, 42);
    // No `DriftWind` inserted.

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert!(
        app.world().get_resource::<DriftWind>().is_none(),
        "guard must not spontaneously insert DriftWind"
    );
}

#[test]
fn missing_drift_wind_is_noop_across_two_ticks() {
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    insert_rng(&mut app, 42);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert!(app.world().get_resource::<DriftWind>().is_none());
}

// ── Behavior 18 — missing GameRng → no-op, no panic ──────────────────────

#[test]
fn missing_game_rng_is_noop() {
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );
    // No `GameRng` inserted.

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let wind = app.world().resource::<DriftWind>();
    assert_eq!(
        wind.timer.to_bits(),
        0.0_f32.to_bits(),
        "timer must be bitwise unchanged when RNG is missing"
    );
}

#[test]
fn missing_game_rng_is_noop_across_two_ticks() {
    // Edge: second tick leaves direction == Vec2::X and timer bitwise 0.0.
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let wind = app.world().resource::<DriftWind>();
    assert_eq!(wind.direction, Vec2::X);
    assert_eq!(wind.timer.to_bits(), 0.0_f32.to_bits());
}

// ── Behavior 19 — multiple consecutive timer expiries reset cleanly ──────

#[test]
fn multiple_consecutive_timer_expiries_reset_cleanly() {
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    insert_rng(&mut app, 42);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     0.5,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );

    for _ in 0..3 {
        tick_with_dt(&mut app, Duration::from_secs_f32(0.6));
        let wind = app.world().resource::<DriftWind>();
        assert!(
            (wind.timer - 0.5).abs() < 1e-5,
            "timer should reset to period_secs=0.5, got {}",
            wind.timer
        );
        assert!(
            (wind.direction.length() - 1.0).abs() < 1e-5,
            "direction must be unit vector"
        );
    }
}

#[test]
fn four_consecutive_expiries_at_larger_dt_reset_cleanly() {
    // Edge: 4 ticks at dt=0.7s, period=0.5s — every tick resets to 0.5.
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    insert_rng(&mut app, 42);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     0.5,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );

    for _ in 0..4 {
        tick_with_dt(&mut app, Duration::from_secs_f32(0.7));
        let wind = app.world().resource::<DriftWind>();
        assert!(
            (wind.timer - 0.5).abs() < 1e-5,
            "timer should reset to 0.5 every tick"
        );
    }
}

// ── Behavior 20 — big-dt overshoot: timer = period_secs, no carry ────────

#[test]
fn big_dt_overshoot_sets_timer_to_period_no_carry_forward() {
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    insert_rng(&mut app, 42);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     2.0,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     1.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

    let wind = app.world().resource::<DriftWind>();
    assert!(
        (wind.timer - 2.0).abs() < 1e-5,
        "timer should be period_secs (2.0) regardless of overshoot, got {}",
        wind.timer
    );
    assert!((wind.direction.length() - 1.0).abs() < 1e-5);
}

#[test]
fn dt_equal_to_prior_timer_plus_period_still_resets_unconditionally() {
    // Edge: dt = period_secs exactly — timer decrements from 1.0 to -1.0,
    // rolls, resets to 2.0. Pins unconditional reset.
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    insert_rng(&mut app, 42);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     2.0,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     1.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(2.0));

    let wind = app.world().resource::<DriftWind>();
    assert!((wind.timer - 2.0).abs() < 1e-5);
}

// ── Behavior 21 — negative period_secs edge: re-rolls every tick ─────────

#[test]
fn negative_period_secs_causes_per_tick_reroll() {
    // Config-error state: period_secs=-1.0. Pins that negative causes per-tick
    // re-rolls rather than panicking or looping infinitely. This is a pinning
    // test — the production code intentionally does not validate the input.
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    insert_rng(&mut app, 42);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     -1.0,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let wind = app.world().resource::<DriftWind>();
    assert!(
        (wind.timer - (-1.0)).abs() < 1e-5,
        "negative period_secs is assigned verbatim, got {}",
        wind.timer
    );
    assert!((wind.direction.length() - 1.0).abs() < 1e-5);
}

#[test]
fn negative_period_secs_rerolls_again_on_second_tick() {
    // Edge: second tick decrements -1.0 to -1.1, re-rolls, resets to -1.0.
    let mut app = test_app_playing();
    wire_update_wind_only(&mut app);
    insert_rng(&mut app, 42);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     -1.0,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    let first_dir = app.world().resource::<DriftWind>().direction;
    assert!((first_dir.length() - 1.0).abs() < 1e-5);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let wind = app.world().resource::<DriftWind>();
    assert!((wind.timer - (-1.0)).abs() < 1e-5);
    assert!((wind.direction.length() - 1.0).abs() < 1e-5);
}
