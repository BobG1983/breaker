//! Group B — `renewal_attach_timers` idempotence.
//!
//! Wires only `renewal_attach_timers` (not `renewal_tick`). The retrofit
//! must not change attach semantics: cells without a timer receive one at
//! the current stack's duration; cells that already have a timer are left
//! alone.

use std::time::Duration;

use bevy::prelude::*;

use super::{
    super::system::{RenewalConfig, RenewalTimer, renewal_attach_timers},
    helpers::{
        add_renewal_stacks, attach_timer, canonical_config, install_renewal_config, spawn_cell,
        test_app_playing, tick_with_dt,
    },
};

// ── Behavior 7 — Cell without timer receives one at current-stack duration ─

#[test]
fn attach_adds_timer_to_cells_without_one() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_attach_timers);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 50.0, 100.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 10.0).abs() < 1e-5,
        "stack-1 duration should be 10.0, got {}",
        timer.remaining
    );
}

#[test]
fn attach_uses_stack_three_duration_when_three_stacks_active() {
    // Edge: three stacks gives duration 6.4 (10.0 * 0.8^2).
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_attach_timers);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 3);
    let cell = spawn_cell(&mut app, 50.0, 100.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 6.4).abs() < 1e-4,
        "stack-3 duration should be 6.4, got {}",
        timer.remaining
    );
}

// ── Behavior 8 — Existing timer is not re-inserted or reset ───────────────

#[test]
fn attach_does_not_duplicate_existing_timers() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_attach_timers);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 50.0, 100.0);
    attach_timer(&mut app, cell, 2.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 2.0).abs() < 1e-5,
        "existing timer must be preserved, got {}",
        timer.remaining
    );
}

#[test]
fn attach_adds_timer_to_dynamically_spawned_cell_on_later_tick() {
    // Edge (Echo-Cells style): a cell spawned AFTER the first tick still
    // receives a fresh timer on the next tick.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_attach_timers);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let _first = spawn_cell(&mut app, 50.0, 100.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // Now spawn a second cell (post-first-tick) and run another tick.
    let second = spawn_cell(&mut app, 50.0, 100.0);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer = app.world().get::<RenewalTimer>(second).unwrap();
    assert!(
        (timer.remaining - 10.0).abs() < 1e-5,
        "dynamically-spawned cell should receive a fresh stack-1 timer, got {}",
        timer.remaining
    );
}

// ── Behavior 9 — No timer inserted when RenewalConfig is missing ──────────

#[test]
fn attach_does_nothing_without_config_resource() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_attach_timers);
    // NOTE: install_renewal_config NOT called.
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 50.0, 100.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(
        app.world().get::<RenewalTimer>(cell).is_none(),
        "no timer should be attached when RenewalConfig is absent"
    );
}

#[test]
fn attach_does_nothing_when_config_present_but_zero_stacks() {
    // Edge: config present, zero stacks → duration_secs(0) == 0.0 → early
    // return in the attach system body. This edge hand-wires only the
    // attach system (not via `register`) so the outer run-condition gate
    // is NOT in play; the guard being exercised is the internal
    // `duration <= 0.0` early return.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_attach_timers);
    install_renewal_config(&mut app, canonical_config());
    // No add_renewal_stacks call — ActiveHazards has 0 Renewal stacks.
    let cell = spawn_cell(&mut app, 50.0, 100.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(
        app.world().get::<RenewalTimer>(cell).is_none(),
        "duration_secs(0) == 0.0 should gate the attach"
    );
}

// ── Behavior 10 — No timer when duration_secs returns 0 (frac ≥ 1.0 at stack 2+) ─

#[test]
fn attach_does_nothing_when_frac_one_collapses_duration_to_zero() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_attach_timers);
    install_renewal_config(
        &mut app,
        RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 1.0,
        },
    );
    add_renewal_stacks(&mut app, 2);
    let cell = spawn_cell(&mut app, 50.0, 100.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(
        app.world().get::<RenewalTimer>(cell).is_none(),
        "duration <= 0.0 early return must skip attach"
    );
}

#[test]
fn attach_with_zero_duration_preserves_existing_timer() {
    // Edge: a pre-attached timer must not be overwritten when the system
    // early-returns because duration <= 0.0.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_attach_timers);
    install_renewal_config(
        &mut app,
        RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 1.0,
        },
    );
    add_renewal_stacks(&mut app, 2);
    let cell = spawn_cell(&mut app, 50.0, 100.0);
    attach_timer(&mut app, cell, 5.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 5.0).abs() < 1e-5,
        "pre-attached timer must be preserved, got {}",
        timer.remaining
    );
}
