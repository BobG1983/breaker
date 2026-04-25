//! Group C — `burnout_update_heat` system (Behaviors C1–C14).
//!
//! Pins the per-tick heat-gauge update:
//! - Fill at rate `1/fill_duration` while moving.
//! - Drain at rate `1/drain_duration` while stationary.
//! - Clamp to `[0.0, 1.0]`.
//! - Arm `mega_bump_charged` when `heat` reaches `1.0`.
//! - Track `still_timer`; reset to `0.0` on a moving tick.
//! - On `still_timer` crossing `still_threshold`, instant full drain +
//!   insert `BurnoutSpeedBoost { remaining: speed_boost_duration }` + clear
//!   `mega_bump_charged`.
//! - Harness-safe: no-op on absent `BurnoutConfig`.
//! - Run-if gated on `protocol_active(Burnout)` + `in_state(Playing)`.
//! - Lazy-insert `BurnoutHeat::default()` on breakers lacking it.

use bevy::prelude::*;

use super::{
    super::system::{BurnoutHeat, config::BurnoutConfig, update_heat::BurnoutSpeedBoost},
    helpers::{
        build_burnout_app, build_burnout_app_in_chip_selecting, build_burnout_app_no_config,
        canonical_burnout_config, install_burnout_config, read_heat, read_speed_boost_remaining,
        seed_active_protocols_with_burnout, set_breaker_velocity, set_heat_state,
        spawn_breaker_moving, spawn_breaker_stationary, tick_n, ticks_for_seconds,
    },
};
use crate::prelude::*;

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_burnout(app, 4.0, 2.0, 1.5, 4.0, 2.0);
}

const MOVING: Vec2 = Vec2::new(200.0, 0.0);

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

// ── C11 — Harness-safe — no panic when BurnoutConfig absent ────────────────-

#[test]
fn update_heat_tolerates_absent_burnout_config() {
    let mut app = build_burnout_app_no_config();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_moving(&mut app, Vec2::ZERO, MOVING);
    // Note: BurnoutHeat::default() is already present from spawn_breaker_moving.

    tick_n(&mut app, 3);

    let h = read_heat(&app, breaker).expect("BurnoutHeat should still be present");
    assert_eq!(
        h,
        BurnoutHeat::default(),
        "BurnoutHeat must be unchanged when config absent"
    );
}

// ── C12 — Run-if gated OFF when Burnout NOT in ActiveProtocols ─────────────-

#[test]
fn update_heat_gated_off_when_burnout_not_active() {
    let mut app = build_burnout_app();
    // Do NOT seed ActiveProtocols.
    let breaker = spawn_breaker_moving(&mut app, Vec2::ZERO, MOVING);

    tick_n(&mut app, ticks_for_seconds(1.0));

    let h = read_heat(&app, breaker).expect("BurnoutHeat should still be present");
    assert_eq!(
        h,
        BurnoutHeat::default(),
        "BurnoutHeat must remain default — update system never ran"
    );
}

// ── C13 — Run-if gated OFF when NodeState != Playing ───────────────────────-

#[test]
fn update_heat_gated_off_when_not_in_playing_state() {
    let mut app = build_burnout_app_in_chip_selecting();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_moving(&mut app, Vec2::ZERO, MOVING);

    tick_n(&mut app, ticks_for_seconds(1.0));

    let h = read_heat(&app, breaker).expect("BurnoutHeat should still be present");
    assert_eq!(
        h,
        BurnoutHeat::default(),
        "BurnoutHeat must remain default — update system never ran in ChipSelecting"
    );
}

// ── C14 — Lazy-insert BurnoutHeat::default() on first tick ─────────────────-

#[test]
fn update_heat_lazy_inserts_burnout_heat_on_breakers_lacking_it() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    // Spawn a breaker with NO BurnoutHeat (note: zero velocity, stationary).
    let breaker = app
        .world_mut()
        .spawn((Breaker, Position2D(Vec2::ZERO), Velocity2D(Vec2::ZERO)))
        .id();

    tick(&mut app);

    let h = app
        .world()
        .get::<BurnoutHeat>(breaker)
        .expect("BurnoutHeat should have been lazy-inserted after first tick");
    assert!(
        (h.heat - 0.0).abs() < f32::EPSILON,
        "lazy-inserted heat must be 0.0, got {}",
        h.heat
    );
    assert!(!h.mega_bump_charged);
    assert!(
        h.still_timer >= 0.0,
        "still_timer must be ≥ 0.0, got {}",
        h.still_timer
    );
}

// ── C14b — Second tick does NOT re-insert BurnoutHeat::default() ───────────-

#[test]
fn update_heat_does_not_reset_existing_burnout_heat_on_subsequent_ticks() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = app
        .world_mut()
        .spawn((Breaker, Position2D(Vec2::ZERO), Velocity2D(Vec2::ZERO)))
        .id();
    tick(&mut app);

    set_heat_state(&mut app, breaker, 0.42, 0.11, false);
    set_breaker_velocity(&mut app, breaker, MOVING);
    tick(&mut app);

    let h = read_heat(&app, breaker).expect("BurnoutHeat should still be present");
    // Expected: heat advanced by one-tick fill increment on top of 0.42, proving
    // the existing value was preserved (not reset to default).
    let expected = 0.42 + (1.0 / 64.0) / 4.0;
    assert!(
        (h.heat - expected).abs() < 1e-3,
        "second tick must mutate the existing BurnoutHeat — expected heat ≈ {expected}, got {}",
        h.heat
    );
}
