//! Group E — Per-message invariants (regression).
//!
//! Pin per-message fields across configurations. Every `HealDealt<Cell>`
//! emitted by Renewal must have: `cap == HealCap::Starting`,
//! `source == Some("hazard:renewal")`, `healer == None`,
//! `amount == starting - current`.

use std::time::Duration;

use bevy::prelude::*;

use super::{
    super::system::renewal_tick,
    helpers::{
        add_renewal_stacks, attach_timer, canonical_config, heal_collector_len, heals_for_cell,
        install_renewal_config, spawn_cell, spawn_cell_with_max, test_app_playing, tick_with_dt,
    },
};
use crate::{cells::components::Cell, prelude::*};

// ── Behavior 20 — Every emitted message has cap == HealCap::Starting ─────

#[test]
fn every_message_cap_is_starting() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 5);
    let cell = spawn_cell(&mut app, 10.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let all = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert_eq!(all.len(), 1);
    assert!(
        all.iter().all(|m| matches!(m.cap, HealCap::Starting)),
        "every emitted message must have cap == HealCap::Starting"
    );
}

#[test]
fn cap_is_starting_even_when_hp_max_is_elevated() {
    // Edge: hp.max > hp.starting (Volatility synergy). Cap is still
    // Starting, NOT Max — Renewal's cap choice is independent of hp.max.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell_with_max(&mut app, 30.0, 100.0, 200.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let msgs = heals_for_cell(&app, cell);
    assert_eq!(msgs.len(), 1);
    assert!(
        matches!(msgs[0].cap, HealCap::Starting),
        "cap must be Starting even when hp.max > hp.starting"
    );
}

// ── Behavior 21 — Every emitted message has source == "hazard:renewal" ────

#[test]
fn every_message_source_is_hazard_renewal() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell_a = spawn_cell(&mut app, 30.0, 100.0);
    let cell_b = spawn_cell(&mut app, 50.0, 100.0);
    attach_timer(&mut app, cell_a, 0.05);
    attach_timer(&mut app, cell_b, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let all = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert_eq!(all.len(), 2);
    assert!(
        all.iter()
            .all(|m| m.source == Some(SourceId::from("hazard:renewal"))),
        "every emitted message must have source == Some(\"hazard:renewal\")"
    );
}

#[test]
fn source_remains_hazard_renewal_after_second_expiry_cycle() {
    // Edge: after a second expiry cycle, the new messages also have the
    // "hazard:renewal" source (no cache/dedup bug).
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    // First expiry.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    // Re-damage + force-reset timer near zero so the next tick expires again.
    app.world_mut().get_mut::<Hp>(cell).unwrap().current = 20.0;
    attach_timer(&mut app, cell, 0.05);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let all = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    // MessageCollector clears each tick, so it holds only the second-tick
    // messages; verifying at least one is present proves the second cycle
    // fired AND its source is correct (the "no cache/dedup bug" guard).
    assert!(
        !all.is_empty(),
        "second expiry cycle must emit a HealDealt<Cell> message, got {}",
        all.len()
    );
    assert!(
        all.iter()
            .all(|m| m.source == Some(SourceId::from("hazard:renewal"))),
        "every emitted message must have source hazard:renewal"
    );
}

// ── Behavior 22 — Every emitted message has healer == None ───────────────

#[test]
fn every_message_healer_is_none() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell_a = spawn_cell(&mut app, 30.0, 100.0);
    let cell_b = spawn_cell(&mut app, 40.0, 100.0);
    let cell_c = spawn_cell(&mut app, 50.0, 100.0);
    attach_timer(&mut app, cell_a, 0.05);
    attach_timer(&mut app, cell_b, 0.05);
    attach_timer(&mut app, cell_c, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 3);
    let all = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert!(
        all.iter().all(|m| m.healer.is_none()),
        "every emitted message must have healer == None"
    );
}

// ── Behavior 23 — amount == starting - current, never pre-capped ─────────

#[test]
fn amount_is_starting_minus_current_not_pre_capped_by_max() {
    // hp.max = Some(20.0) > hp.starting = 10.0. amount MUST come from
    // starting, not max — a buggy impl using max.unwrap_or(starting) would
    // emit 15.0 (max - current).
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell_with_max(&mut app, 5.0, 10.0, 20.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let msgs = heals_for_cell(&app, cell);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 5.0).abs() < f32::EPSILON,
        "amount must be starting - current = 5.0 (not max - current = 15.0), got {}",
        msgs[0].amount
    );
}

#[test]
fn amount_ignores_max_even_when_max_below_starting() {
    // Edge: max < starting. amount is still `starting - current = 5.0`,
    // NOT `max - current = 2.0`. Cap enforcement is apply_heal's job.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell_with_max(&mut app, 5.0, 10.0, 7.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let msgs = heals_for_cell(&app, cell);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 5.0).abs() < f32::EPSILON,
        "amount must be 5.0 (starting - current), not 2.0 (max - current); got {}",
        msgs[0].amount
    );
}
