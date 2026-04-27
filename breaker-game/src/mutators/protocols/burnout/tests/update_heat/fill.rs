//! C1–C3 — heat fills while moving, clamps at 1.0, arms mega-bump-charged.

use bevy::prelude::*;

use super::{
    super::helpers::{
        build_burnout_app, read_heat, set_heat_state, spawn_breaker_moving, tick_n,
        ticks_for_seconds,
    },
    common::{MOVING, seed_canonical},
};
use crate::prelude::*;

// ── C1 — Heat fills while moving — 0.5 s → heat ≈ 0.125 ────────────────────-

#[test]
fn heat_fills_while_moving_half_second_reaches_one_eighth() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_moving(&mut app, Vec2::ZERO, MOVING);
    set_heat_state(&mut app, breaker, 0.0, 0.0, false);

    tick_n(&mut app, ticks_for_seconds(0.5));

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.heat - 0.125).abs() < 1e-3,
        "heat expected ≈ 0.125 after 0.5s at fill_duration 4.0, got {}",
        h.heat
    );
    assert!(
        (h.still_timer - 0.0).abs() < f32::EPSILON,
        "still_timer must be 0.0 while moving, got {}",
        h.still_timer
    );
    assert!(
        !h.mega_bump_charged,
        "mega_bump_charged must be false at heat 0.125"
    );
}

// ── C1b — Heat fills 2.0 s → heat ≈ 0.5 (design-doc worked example) ────────-

#[test]
fn heat_fills_while_moving_two_seconds_reaches_one_half() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_moving(&mut app, Vec2::ZERO, MOVING);
    set_heat_state(&mut app, breaker, 0.0, 0.0, false);

    tick_n(&mut app, ticks_for_seconds(2.0));

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.heat - 0.5).abs() < 1e-3,
        "heat expected ≈ 0.5 after 2.0s at fill_duration 4.0, got {}",
        h.heat
    );
}

// ── C2 — Heat reaches full and arms mega_bump_charged ──────────────────────-

#[test]
fn heat_reaches_full_arms_mega_bump_charged() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_moving(&mut app, Vec2::ZERO, MOVING);
    set_heat_state(&mut app, breaker, 0.9, 0.0, false);

    tick_n(&mut app, ticks_for_seconds(0.5));

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.heat - 1.0).abs() < 1e-5,
        "heat expected 1.0 (clamped), got {}",
        h.heat
    );
    assert!(
        h.mega_bump_charged,
        "mega_bump_charged must be true at full heat"
    );
    assert!(
        (h.still_timer - 0.0).abs() < f32::EPSILON,
        "still_timer must remain 0.0 while moving"
    );
}

// ── C2b — Exactly at threshold → mega_bump_charged true on first move tick ─-

#[test]
fn heat_exactly_at_one_arms_mega_bump_on_first_moving_tick() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_moving(&mut app, Vec2::ZERO, MOVING);
    set_heat_state(&mut app, breaker, 1.0, 0.0, false);

    tick(&mut app);

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        h.mega_bump_charged,
        "mega_bump_charged must flip true at heat 1.0"
    );
}

// ── C3 — Heat clamps at 1.0 — extra moving time does not accumulate ────────-

#[test]
fn heat_clamps_at_one_under_continuing_movement() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_moving(&mut app, Vec2::ZERO, MOVING);
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);

    tick_n(&mut app, ticks_for_seconds(1.0));

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.heat - 1.0).abs() < 1e-5,
        "heat must remain clamped at 1.0 after extra tick, got {}",
        h.heat
    );
    assert!(h.mega_bump_charged, "mega_bump_charged must remain true");
}
