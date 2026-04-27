//! C4–C5, C10 — heat drains while stationary, clamps at 0.0,
//! and treats below-epsilon velocity as stationary.

use bevy::prelude::*;

use super::{
    super::{
        super::system::config::BurnoutConfig,
        helpers::{
            build_burnout_app, canonical_burnout_config, install_burnout_config, read_heat,
            seed_active_protocols_with_burnout, set_heat_state, spawn_breaker_moving,
            spawn_breaker_stationary, tick_n, ticks_for_seconds,
        },
    },
    common::seed_canonical,
};
use crate::prelude::*;

// ── C4 — Heat drains while stationary — 1.0 s at drain_duration 2.0 → 0.5 ──-

#[test]
fn heat_drains_while_stationary_one_second_reaches_one_half() {
    let mut app = build_burnout_app();
    // Use a still_threshold large enough that the instant-drain does NOT fire
    // during a 1.0s stillness window (canonical 1.5 is fine, but override to
    // make the intent explicit).
    install_burnout_config(
        &mut app,
        BurnoutConfig {
            still_threshold: 3.0,
            ..canonical_burnout_config()
        },
    );
    seed_active_protocols_with_burnout(&mut app, 4.0, 2.0, 3.0, 4.0, 2.0);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);

    tick_n(&mut app, ticks_for_seconds(1.0));

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.heat - 0.5).abs() < 1e-3,
        "heat expected ≈ 0.5 after 1.0s drain at drain_duration 2.0, got {}",
        h.heat
    );
    assert!(
        (h.still_timer - 1.0).abs() < 1e-3,
        "still_timer expected ≈ 1.0, got {}",
        h.still_timer
    );
    assert!(
        h.mega_bump_charged,
        "drain alone does NOT clear mega_bump_charged (only still-threshold does)"
    );
}

// ── C4b — Full drain over 2.0 s with still_threshold overridden ────────────-

#[test]
fn heat_fully_drains_over_drain_duration_without_still_threshold() {
    let mut app = build_burnout_app();
    install_burnout_config(
        &mut app,
        BurnoutConfig {
            still_threshold: 3.0,
            ..canonical_burnout_config()
        },
    );
    seed_active_protocols_with_burnout(&mut app, 4.0, 2.0, 3.0, 4.0, 2.0);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);

    tick_n(&mut app, ticks_for_seconds(2.0));

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.heat - 0.0).abs() < 1e-3,
        "heat expected ≈ 0.0 after 2.0s drain at drain_duration 2.0, got {}",
        h.heat
    );
}

// ── C5 — Heat clamps at 0.0 — stationary while already empty ───────────────-

#[test]
fn heat_clamps_at_zero_when_stationary_while_empty() {
    let mut app = build_burnout_app();
    install_burnout_config(
        &mut app,
        BurnoutConfig {
            still_threshold: 3.0,
            ..canonical_burnout_config()
        },
    );
    seed_active_protocols_with_burnout(&mut app, 4.0, 2.0, 3.0, 4.0, 2.0);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    set_heat_state(&mut app, breaker, 0.0, 0.0, false);

    tick_n(&mut app, ticks_for_seconds(1.0));

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.heat - 0.0).abs() < 1e-5,
        "heat must remain clamped at 0.0 (non-negative), got {}",
        h.heat
    );
    assert!(
        (h.still_timer - 1.0).abs() < 1e-3,
        "still_timer expected ≈ 1.0, got {}",
        h.still_timer
    );
    assert!(!h.mega_bump_charged);
}

// ── C10 — Tiny velocity below epsilon counts as stationary ─────────────────-

#[test]
fn below_epsilon_velocity_counts_as_stationary() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_moving(&mut app, Vec2::ZERO, Vec2::new(f32::EPSILON * 0.1, 0.0));
    set_heat_state(&mut app, breaker, 0.5, 0.0, false);

    tick(&mut app);

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        h.still_timer > 0.0,
        "below-epsilon velocity must be treated as stationary (still_timer > 0), got {}",
        h.still_timer
    );
    assert!(
        h.heat < 0.5,
        "heat must decrease (drain), not increase, got {}",
        h.heat
    );
}
