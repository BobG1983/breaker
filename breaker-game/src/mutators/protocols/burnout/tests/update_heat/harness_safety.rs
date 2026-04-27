//! C11–C13 — harness-safety when `BurnoutConfig` is absent and run-if
//! gating off `protocol_active(Burnout)` + `in_state(Playing)`.

use bevy::prelude::*;

use super::{
    super::{
        super::system::BurnoutHeat,
        helpers::{
            build_burnout_app, build_burnout_app_in_chip_selecting, build_burnout_app_no_config,
            read_heat, spawn_breaker_moving, tick_n, ticks_for_seconds,
        },
    },
    common::{MOVING, seed_canonical},
};

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
