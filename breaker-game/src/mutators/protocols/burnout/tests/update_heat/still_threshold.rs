//! C6–C9 — `still_timer` reset on movement, still-threshold instant-drain
//! firing, and post-fire fill behavior.

use bevy::prelude::*;

use super::{
    super::{
        super::system::update_heat::BurnoutSpeedBoost,
        helpers::{
            build_burnout_app, read_heat, read_speed_boost_remaining, set_breaker_velocity,
            set_heat_state, spawn_breaker_stationary, tick_n, ticks_for_seconds,
        },
    },
    common::{MOVING, seed_canonical},
};
use crate::prelude::*;

// ── C6 — Moving resets still_timer to 0.0 on the first moving tick ─────────-

#[test]
fn moving_resets_still_timer_on_first_moving_tick() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    set_heat_state(&mut app, breaker, 0.5, 0.8, false);

    set_breaker_velocity(&mut app, breaker, MOVING);
    tick(&mut app);

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.still_timer - 0.0).abs() < f32::EPSILON,
        "still_timer must reset to 0.0 on first moving tick, got {}",
        h.still_timer
    );
    // Heat advanced by one-tick fill increment: (1/64) / 4.0 ≈ 0.00390625.
    let expected = 0.5 + (1.0 / 64.0) / 4.0;
    assert!(
        (h.heat - expected).abs() < 1e-4,
        "heat expected ≈ {expected} after one moving tick, got {}",
        h.heat
    );
}

// ── C7 — Still-threshold crossing triggers instant drain + speed boost ─────-

#[test]
fn still_threshold_crossing_triggers_instant_drain_and_speed_boost() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    set_heat_state(&mut app, breaker, 0.8, 0.0, false);

    tick_n(&mut app, ticks_for_seconds(1.5));

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.heat - 0.0).abs() < 1e-5,
        "heat must instant-drain to 0.0 after still_threshold, got {}",
        h.heat
    );
    assert!(
        (h.still_timer - 0.0).abs() < 1e-5,
        "still_timer must reset to 0.0 after still_threshold fire, got {}",
        h.still_timer
    );
    assert!(
        !h.mega_bump_charged,
        "mega_bump_charged must be cleared by still_threshold drain"
    );
    assert_eq!(
        read_speed_boost_remaining(&app, breaker),
        Some(2.0),
        "BurnoutSpeedBoost must be installed with remaining == speed_boost_duration"
    );
}

// ── C7b — mega_bump_charged clears on still-threshold fire even if true ────-

#[test]
fn still_threshold_clears_mega_bump_charged_even_if_armed() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    set_heat_state(&mut app, breaker, 0.8, 0.0, true);

    tick_n(&mut app, ticks_for_seconds(1.5));

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        !h.mega_bump_charged,
        "mega_bump_charged must be cleared by still_threshold drain"
    );
}

// ── C8 — Still-threshold fires on the exact tick that crosses ──────────────-

#[test]
fn still_threshold_does_not_fire_before_crossing_tick() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    set_heat_state(&mut app, breaker, 0.8, 1.0, false);

    // 29 ticks ≈ 0.453s; still_timer ≈ 1.453, just below 1.5.
    tick_n(&mut app, 29);

    assert!(
        app.world().get::<BurnoutSpeedBoost>(breaker).is_none(),
        "BurnoutSpeedBoost must NOT be installed before still_timer crosses threshold"
    );
    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        h.heat > 0.0 && h.heat <= 0.8,
        "heat must be partially drained (in (0.0, 0.8]), got {}",
        h.heat
    );
}

// ── C8b — One additional tick crosses the threshold ────────────────────────-

#[test]
fn still_threshold_fires_on_crossing_tick() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    set_heat_state(&mut app, breaker, 0.8, 1.0, false);

    // 32 ticks = 0.5s exactly; still_timer = 1.0 + 0.5 = 1.5 >= still_threshold.
    tick_n(&mut app, 32);

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.heat - 0.0).abs() < 1e-5,
        "heat must instant-drain to 0.0 on crossing tick, got {}",
        h.heat
    );
    assert!(
        (h.still_timer - 0.0).abs() < 1e-5,
        "still_timer must reset to 0.0, got {}",
        h.still_timer
    );
    assert!(!h.mega_bump_charged);
    assert_eq!(
        read_speed_boost_remaining(&app, breaker),
        Some(2.0),
        "BurnoutSpeedBoost must be installed on crossing tick"
    );
}

// ── C9 — Movement after still-threshold: timer stays 0, heat fills from 0 ──-

#[test]
fn movement_after_still_threshold_fills_from_zero_with_timer_at_zero() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    set_heat_state(&mut app, breaker, 0.8, 0.0, false);

    // Fire the still-threshold by standing still 1.5s.
    tick_n(&mut app, ticks_for_seconds(1.5));
    // Now flip to moving.
    set_breaker_velocity(&mut app, breaker, MOVING);
    tick_n(&mut app, ticks_for_seconds(0.5));

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.still_timer - 0.0).abs() < f32::EPSILON,
        "still_timer must be 0.0 after moving again, got {}",
        h.still_timer
    );
    assert!(
        (h.heat - 0.125).abs() < 1e-3,
        "heat must fill from 0 at fill_duration rate, got {}",
        h.heat
    );
}
