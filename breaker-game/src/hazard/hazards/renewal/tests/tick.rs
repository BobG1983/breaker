//! Group C — `renewal_tick` timer decrements (no expiry, no heal).
//!
//! Wires only `renewal_tick`. Config and stacks installed. These pin the
//! `timer.remaining > 0.0` branch: the timer counts down each tick and
//! emits no `HealDealt<Cell>` until it hits zero.

use std::time::Duration;

use bevy::prelude::*;

use super::{
    super::system::{RenewalTimer, renewal_tick},
    helpers::{
        add_renewal_stacks, attach_timer, canonical_config, heal_collector_len,
        install_renewal_config, spawn_cell, test_app_playing, tick_with_dt,
    },
};
use crate::shared::death_pipeline::Hp;

// ── Behavior 11 — Timer decrements by delta_secs, no heal while remaining > 0 ─

#[test]
fn timer_decrements_by_dt_and_emits_no_heal() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 5.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 4.9).abs() < 1e-5,
        "remaining should be 5.0 - 0.1 = 4.9, got {}",
        timer.remaining
    );
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!((hp.current - 30.0).abs() < f32::EPSILON);
    assert_eq!(heal_collector_len(&app), 0);
}

#[test]
fn timer_decrements_by_small_dt() {
    // Edge: dt = 0.016 → remaining 5.0 → 4.984.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 5.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 4.984).abs() < 1e-5,
        "remaining should be 4.984, got {}",
        timer.remaining
    );
    assert_eq!(heal_collector_len(&app), 0);
}

// ── Behavior 12 — Timer barely-above-zero does not expire ─────────────────

#[test]
fn timer_remaining_0_05_with_dt_0_04_does_not_expire() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.04));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 0.01).abs() < 1e-5,
        "remaining should be 0.05 - 0.04 = 0.01, got {}",
        timer.remaining
    );
    assert_eq!(heal_collector_len(&app), 0);
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!((hp.current - 30.0).abs() < f32::EPSILON);
}

#[test]
fn timer_remaining_0_05_with_dt_0_049_does_not_expire() {
    // Edge: still positive after the subtraction (0.001) — no heal.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.049));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 0.001).abs() < 1e-5,
        "remaining should be 0.05 - 0.049 = 0.001, got {}",
        timer.remaining
    );
    assert_eq!(heal_collector_len(&app), 0);
}

// ── Config-absent early-return guard ─────────────────────────────────────

#[test]
fn tick_without_config_resource_early_returns_no_heal_no_tick() {
    // Pins `renewal_tick`'s `let Some(config) = config else { return }`
    // branch: with no RenewalConfig installed, the system no-ops — timer
    // not decremented, no message emitted — even though a stack and timer
    // are present.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    // Note: intentionally NO install_renewal_config call.
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 0.05).abs() < 1e-5,
        "no-config early-return must not tick the timer; got {}",
        timer.remaining
    );
    assert_eq!(heal_collector_len(&app), 0);
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!((hp.current - 30.0).abs() < f32::EPSILON);
}
