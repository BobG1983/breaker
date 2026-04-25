//! Group F — `burnout_tick_speed_boost` system (Behaviors F1–F5).
//!
//! Pins:
//! - `BurnoutSpeedBoost.remaining` decrements by `delta_secs` each tick.
//! - The component is removed when `remaining` reaches `0.0`.
//! - Still-threshold re-trigger resets `remaining` to `speed_boost_duration`
//!   (no stacking — the existing boost is overwritten, not accumulated).
//! - Harness-safe on absent `BurnoutConfig` — either unchanged or decremented
//!   by `delta_secs × n`.
//! - Run-if gated on `protocol_active(Burnout)` + `in_state(Playing)`.

use bevy::prelude::*;

use super::{
    super::system::update_heat::BurnoutSpeedBoost,
    helpers::{
        build_burnout_app, build_burnout_app_in_chip_selecting, build_burnout_app_no_config,
        install_burnout_speed_boost, read_speed_boost_remaining,
        seed_active_protocols_with_burnout, set_heat_state, spawn_breaker_stationary, tick_n,
        ticks_for_seconds,
    },
};
use crate::prelude::*;

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_burnout(app, 4.0, 2.0, 1.5, 4.0, 2.0);
}

// ── F1 — Speed boost decrements by delta_secs each tick ────────────────────-

#[test]
fn speed_boost_decrements_by_delta_secs_per_tick() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    install_burnout_speed_boost(&mut app, breaker, 2.0);

    tick(&mut app);

    let remaining =
        read_speed_boost_remaining(&app, breaker).expect("BurnoutSpeedBoost should be present");
    let expected = 2.0 - 1.0 / 64.0;
    assert!(
        (remaining - expected).abs() < 1e-5,
        "remaining expected ≈ {expected} after one tick (1/64s), got {remaining}"
    );
}

// ── F1b — 0.5 s → remaining ≈ 1.5 ──────────────────────────────────────────-

#[test]
fn speed_boost_half_second_reaches_one_and_a_half_remaining() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    install_burnout_speed_boost(&mut app, breaker, 2.0);

    tick_n(&mut app, 32);

    let remaining =
        read_speed_boost_remaining(&app, breaker).expect("BurnoutSpeedBoost should be present");
    assert!(
        (remaining - 1.5).abs() < 1e-3,
        "remaining expected ≈ 1.5 after 0.5s, got {remaining}"
    );
}

// ── F2 — Speed boost removed on expiry ─────────────────────────────────────-

#[test]
fn speed_boost_removed_when_remaining_reaches_zero() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    install_burnout_speed_boost(&mut app, breaker, 2.0);

    tick_n(&mut app, ticks_for_seconds(2.0));

    assert!(
        app.world().get::<BurnoutSpeedBoost>(breaker).is_none(),
        "BurnoutSpeedBoost must be removed after 2.0s expiry"
    );
}

// ── F2b — 0.5s duration expires after 32 ticks ─────────────────────────────-

#[test]
fn short_speed_boost_expires_after_exact_duration() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    install_burnout_speed_boost(&mut app, breaker, 0.5);

    tick_n(&mut app, 32);

    assert!(
        app.world().get::<BurnoutSpeedBoost>(breaker).is_none(),
        "BurnoutSpeedBoost must be removed after 0.5s (32 ticks) expiry"
    );
}

// ── F3 — Still-threshold re-trigger resets remaining (no stacking) ─────────-

#[test]
fn still_threshold_retrigger_resets_remaining_no_stacking() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    // Precondition: almost-expired boost + heat ready to drain on threshold.
    install_burnout_speed_boost(&mut app, breaker, 0.3);
    set_heat_state(&mut app, breaker, 0.8, 0.0, false);

    // 1.5 s of stillness — fires still-threshold again.
    tick_n(&mut app, ticks_for_seconds(1.5));

    let remaining =
        read_speed_boost_remaining(&app, breaker).expect("BurnoutSpeedBoost should be present");
    assert!(
        remaining > 0.3 + 1.5,
        "remaining must be RESET to speed_boost_duration, not stacked \
         (expected > 1.8 to prove reset vs accumulate), got {remaining}"
    );
    assert!(
        (1.95..=2.0).contains(&remaining),
        "remaining must be close to 2.0 (speed_boost_duration) after reset, got {remaining}"
    );
}

// ── F4 — Harness-safe — no panic when BurnoutConfig absent ─────────────────-

#[test]
fn tick_speed_boost_tolerates_absent_burnout_config() {
    let mut app = build_burnout_app_no_config();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    install_burnout_speed_boost(&mut app, breaker, 2.0);

    tick_n(&mut app, 3);

    let remaining =
        read_speed_boost_remaining(&app, breaker).expect("BurnoutSpeedBoost should be present");
    assert!(
        (remaining - 2.0).abs() < f32::EPSILON,
        "remaining must be unchanged (early-return on absent BurnoutConfig), got {remaining}"
    );
}

// ── F5a — Gated OFF when Burnout NOT in ActiveProtocols ────────────────────-

#[test]
fn tick_speed_boost_gated_off_when_burnout_not_active() {
    let mut app = build_burnout_app();
    // Do NOT seed ActiveProtocols.
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    install_burnout_speed_boost(&mut app, breaker, 2.0);

    tick_n(&mut app, 10);

    let remaining =
        read_speed_boost_remaining(&app, breaker).expect("BurnoutSpeedBoost should be present");
    assert!(
        (remaining - 2.0).abs() < f32::EPSILON,
        "remaining must be 2.0 exactly (system did not run), got {remaining}"
    );
}

// ── F5b — Gated OFF when NodeState != Playing ──────────────────────────────-

#[test]
fn tick_speed_boost_gated_off_when_not_in_playing_state() {
    let mut app = build_burnout_app_in_chip_selecting();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    install_burnout_speed_boost(&mut app, breaker, 2.0);

    tick_n(&mut app, 10);

    let remaining =
        read_speed_boost_remaining(&app, breaker).expect("BurnoutSpeedBoost should be present");
    assert!(
        (remaining - 2.0).abs() < f32::EPSILON,
        "remaining must be 2.0 exactly in ChipSelecting, got {remaining}"
    );
}
