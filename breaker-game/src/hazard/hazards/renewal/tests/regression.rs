//! Group I — Retrofit-specific regression guards.
//!
//! Tripwires that catch an incomplete retrofit. All tests wire ONLY
//! `renewal_tick` (NOT `apply_heal::<Cell>`). Any change to `Hp` observed
//! here proves the retrofit leaked direct HP mutation.

use std::time::Duration;

use bevy::prelude::*;

use super::{
    super::system::{RenewalConfig, renewal_tick},
    helpers::{
        add_renewal_stacks, attach_timer, canonical_config, heal_collector_len, heals_for_cell,
        install_renewal_config, spawn_cell, spawn_cell_with_max, test_app_playing, tick_with_dt,
    },
};
use crate::shared::death_pipeline::Hp;

// ── Behavior 36 — renewal_tick must NOT mutate Hp.current directly ────────

#[test]
fn regression_renewal_does_not_mutate_hp_directly() {
    let mut app = test_app_playing();
    // ONLY renewal_tick wired — apply_heal::<Cell> is deliberately absent.
    // Any Hp change observed here is a direct mutation.
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    // 1. Message WAS emitted.
    assert_eq!(
        heal_collector_len(&app),
        1,
        "renewal must emit exactly one HealDealt<Cell> message on expiry"
    );
    // 2. Hp was NOT directly mutated (apply_heal isn't wired).
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 30.0).abs() < f32::EPSILON,
        "Hp.current must remain 30.0 (apply_heal isn't wired); got {}. \
         A change here proves renewal_tick is directly mutating Hp.",
        hp.current
    );
}

#[test]
fn regression_renewal_does_not_mutate_hp_with_elevated_max() {
    // Edge: hp.max > starting; still no direct Hp mutation.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell_with_max(&mut app, 30.0, 100.0, 200.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 1);
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 30.0).abs() < f32::EPSILON,
        "Hp.current must remain 30.0 even with elevated max; got {}",
        hp.current
    );
}

// ── Behavior 37 — amount is raw starting - current, no pre-clamp ────────

#[test]
fn regression_amount_is_raw_starting_minus_current_even_when_max_below_starting() {
    // Deliberately suspicious max < starting: a buggy pre-clamp might
    // emit `50.0 - 5.0 = 45.0`. We expect the raw `95.0`.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell_with_max(&mut app, 5.0, 100.0, 50.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let msgs = heals_for_cell(&app, cell);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 95.0).abs() < f32::EPSILON,
        "amount must be starting - current = 95.0 (not 45.0); got {}. \
         Pre-clamp is apply_heal's job via HealCap::Starting.",
        msgs[0].amount
    );
}

#[test]
fn regression_amount_without_max_is_identical_to_with_max_below_starting() {
    // Edge: absence of max produces the same amount — confirms the formula
    // has no max-dependence.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 5.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let msgs = heals_for_cell(&app, cell);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 95.0).abs() < f32::EPSILON,
        "amount must be 95.0 regardless of max; got {}",
        msgs[0].amount
    );
}

// ── Behavior 38 — Exactly 1 message per expiring damaged cell per tick ────

#[test]
fn regression_exactly_one_message_per_expiring_cell_per_tick() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(
        heal_collector_len(&app),
        1,
        "expected exactly one message (not two — catches double-iterate or \
         double-schedule regressions)"
    );
}

#[test]
fn regression_ten_damaged_cells_emit_exactly_ten_messages() {
    // Edge: scale up — 10 damaged cells on one tick emit exactly 10.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    for _ in 0..10 {
        let cell = spawn_cell(&mut app, 30.0, 100.0);
        attach_timer(&mut app, cell, 0.05);
    }

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(
        heal_collector_len(&app),
        10,
        "10 damaged cells → exactly 10 messages; got {}",
        heal_collector_len(&app)
    );
}

// ── Zero-duration reset guard regression ──────────────────────────────────

#[test]
fn regression_zero_duration_does_not_spam_heals_every_tick() {
    // With `per_level_reduction_frac = 1.0` at stack 2, `duration_secs`
    // returns 0.0. Without the early-return guard in `renewal_tick`, every
    // tick would re-expire the timer and emit another HealDealt<Cell>.
    // This test pins the guard: across TWO consecutive ticks, the system
    // must emit at most ONE message per damaged cell (and ideally zero,
    // since Renewal is functionally inactive at duration == 0.0).
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(
        &mut app,
        RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 1.0,
        },
    );
    add_renewal_stacks(&mut app, 2); // duration_secs(2) == 10.0 * 0.0^1 == 0.0
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    // Tick 1: MessageCollector clears in `First`, runs systems, collects.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    let tick1_count = heal_collector_len(&app);

    // Tick 2: same pattern.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    let tick2_count = heal_collector_len(&app);

    // Neither tick should emit heals — duration == 0.0 means Renewal is
    // inactive. The bug would emit ≥1 per tick indefinitely.
    assert_eq!(
        tick1_count, 0,
        "tick 1 must not emit heals when duration == 0.0; got {tick1_count}"
    );
    assert_eq!(
        tick2_count, 0,
        "tick 2 must not emit heals when duration == 0.0 (no re-expiry spam); got {tick2_count}"
    );
}
