//! Group D — `renewal_tick` timer expiry emits `HealDealt<Cell>`.
//!
//! These tests pin message shape and fields. `apply_heal::<Cell>` is NOT
//! wired; we observe only the emitted message via
//! `MessageCollector<HealDealt<Cell>>`.
//!
//! Includes 15A (second-expiry stack-up reset recomputes duration).

use std::time::Duration;

use bevy::prelude::*;

use super::{
    super::system::{RenewalTimer, renewal_tick},
    helpers::{
        add_renewal_stacks, attach_timer, canonical_config, heal_collector_len, heals_for_cell,
        install_renewal_config, spawn_cell, spawn_cell_with_max, test_app_playing, tick_with_dt,
    },
};
use crate::{
    hazard::{definition::HazardKind, resources::ActiveHazards},
    shared::death_pipeline::{HealCap, Hp},
};

// ── Behavior 13 — Damaged cell: timer expiry emits one HealDealt<Cell> ────

#[test]
fn damaged_cell_expiry_emits_one_heal_with_correct_fields() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 1);
    let msgs = heals_for_cell(&app, cell);
    assert_eq!(msgs.len(), 1);
    let msg = &msgs[0];
    assert_eq!(msg.target, cell);
    assert!(
        (msg.amount - 70.0).abs() < f32::EPSILON,
        "amount should be starting - current = 70.0, got {}",
        msg.amount
    );
    assert!(matches!(msg.cap, HealCap::Starting));
    assert_eq!(msg.healer, None);
    assert_eq!(msg.source, Some("hazard:renewal".to_string()));
}

#[test]
fn damaged_cell_at_one_hp_emits_amount_ninety_nine() {
    // Edge: hp 1 / 100 → amount 99.0.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 1.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let msgs = heals_for_cell(&app, cell);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 99.0).abs() < f32::EPSILON,
        "amount should be 99.0, got {}",
        msgs[0].amount
    );
}

// ── Behavior 14 — Timer resets to current-stack duration after expiry ────

#[test]
fn timer_resets_to_stack_one_duration_after_expiry() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 10.0).abs() < 1e-5,
        "timer should reset to 10.0 after expiry, got {}",
        timer.remaining
    );
}

#[test]
fn post_reset_second_tick_decrements_from_fresh_value() {
    // Edge: second tick after reset → 10.0 - 0.1 = 9.9.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1)); // expires, resets to 10.0
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1)); // 10.0 → 9.9

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 9.9).abs() < 1e-5,
        "second tick should produce remaining 9.9, got {}",
        timer.remaining
    );
}

// ── Behavior 15 — Timer reset uses current stack count, not attach-time count ─

#[test]
fn timer_reset_uses_current_stack_count_not_attach_count() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    // Stack up BEFORE the tick — final count is 3.
    add_renewal_stacks(&mut app, 2);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 6.4).abs() < 1e-4,
        "reset should use stack-3 duration (6.4), got {}",
        timer.remaining
    );
}

#[test]
fn timer_reset_after_stacking_to_ten_uses_design_doc_duration() {
    // Edge: stack up to 10 — reset duration is ≈ 1.342.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);
    add_renewal_stacks(&mut app, 9); // now 10 stacks total

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        timer.remaining > 1.34 && timer.remaining < 1.35,
        "reset should use stack-10 duration (~1.342), got {}",
        timer.remaining
    );
}

// ── Behavior 15A — Second expiry uses new stack-count duration ────────────

#[test]
fn second_expiry_uses_new_stack_count_duration() {
    // Recomputation happens on EVERY expiry: after the first expiry resets
    // the timer, stacking up to 3 must cause the NEXT expiry's reset to
    // use the stack-3 duration (6.4), not the stack-1 duration (10.0).
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    // First expiry at stack 1 → resets to 10.0.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    assert!(
        (app.world().get::<RenewalTimer>(cell).unwrap().remaining - 10.0).abs() < 1e-5,
        "first expiry should reset to stack-1 duration (10.0)"
    );

    // Stack up to 3. Re-damage so another heal is emitted on second expiry.
    add_renewal_stacks(&mut app, 2);
    app.world_mut().get_mut::<Hp>(cell).unwrap().current = 30.0;

    // Force a second expiry immediately by resetting the timer near zero.
    attach_timer(&mut app, cell, 0.05);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 6.4).abs() < 1e-4,
        "second expiry must reset to stack-3 duration (6.4), got {}",
        timer.remaining
    );
}

#[test]
fn third_expiry_after_dropping_stacks_uses_stack_one_duration() {
    // Edge: after the second expiry at stack 3 (reset to 6.4), drop stacks
    // back to 1 and trigger a third expiry — the reset-duration MUST use
    // the new stack count (1 → 10.0), not the previous (3 → 6.4).
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    // First expiry → resets to 10.0.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    assert!((app.world().get::<RenewalTimer>(cell).unwrap().remaining - 10.0).abs() < 1e-5);

    // Stack up to 3. Re-damage cell so next expiry re-emits.
    add_renewal_stacks(&mut app, 2);
    app.world_mut().get_mut::<Hp>(cell).unwrap().current = 30.0;

    // Force a second expiry immediately.
    attach_timer(&mut app, cell, 0.05);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    assert!(
        (app.world().get::<RenewalTimer>(cell).unwrap().remaining - 6.4).abs() < 1e-4,
        "second expiry should reset to stack-3 duration (6.4), got {}",
        app.world().get::<RenewalTimer>(cell).unwrap().remaining
    );

    // Drop stacks back to 1 via the force_insert_entry backdoor.
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .force_insert_entry(HazardKind::Renewal, 1);

    // Re-damage + force a third expiry immediately.
    app.world_mut().get_mut::<Hp>(cell).unwrap().current = 30.0;
    attach_timer(&mut app, cell, 0.05);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 10.0).abs() < 1e-4,
        "third expiry should reset to stack-1 duration (10.0) after dropping stacks, got {}",
        timer.remaining
    );
}

// ── Behavior 16 — Full-HP cell at expiry: no heal, timer still resets ────

#[test]
fn full_hp_cell_expiry_emits_no_heal_but_resets_timer() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 100.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 10.0).abs() < 1e-5,
        "timer should reset to 10.0, got {}",
        timer.remaining
    );
}

#[test]
fn over_hp_cell_expiry_emits_no_heal_and_resets_timer() {
    // Edge: hp.current > starting (Volatility over-heal); renewal should
    // still emit no heal and reset the timer.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell_with_max(&mut app, 120.0, 100.0, 150.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 10.0).abs() < 1e-5,
        "timer should reset even for over-HP cell, got {}",
        timer.remaining
    );
}

// ── Behavior 17 — Dead cell is skipped entirely ──────────────────────────

#[test]
fn dead_cell_is_skipped_entirely() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 0.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 0.05).abs() < 1e-5,
        "dead-cell timer should be UNTOUCHED, got {}",
        timer.remaining
    );
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(hp.current.abs() < f32::EPSILON);
}

#[test]
fn cell_with_negative_hp_is_skipped_entirely() {
    // Edge: hp.current = -5.0 — gate is `<= 0.0`, not `== 0.0`.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, -5.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 0.05).abs() < 1e-5,
        "negative-HP cell timer should be UNTOUCHED, got {}",
        timer.remaining
    );
}

// ── Behavior 18 — Multiple damaged cells: each emits its own heal ────────

#[test]
fn three_damaged_cells_each_emit_their_own_heal() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell_a = spawn_cell(&mut app, 30.0, 100.0);
    let cell_b = spawn_cell(&mut app, 50.0, 100.0);
    let cell_c = spawn_cell(&mut app, 10.0, 50.0);
    attach_timer(&mut app, cell_a, 0.05);
    attach_timer(&mut app, cell_b, 0.05);
    attach_timer(&mut app, cell_c, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 3);

    let msgs_a = heals_for_cell(&app, cell_a);
    assert_eq!(msgs_a.len(), 1);
    assert!((msgs_a[0].amount - 70.0).abs() < f32::EPSILON);
    assert!(matches!(msgs_a[0].cap, HealCap::Starting));
    assert_eq!(msgs_a[0].source, Some("hazard:renewal".to_string()));

    let msgs_b = heals_for_cell(&app, cell_b);
    assert_eq!(msgs_b.len(), 1);
    assert!((msgs_b[0].amount - 50.0).abs() < f32::EPSILON);
    assert!(matches!(msgs_b[0].cap, HealCap::Starting));

    let msgs_c = heals_for_cell(&app, cell_c);
    assert_eq!(msgs_c.len(), 1);
    assert!((msgs_c[0].amount - 40.0).abs() < f32::EPSILON);
    assert!(matches!(msgs_c[0].cap, HealCap::Starting));
}

#[test]
fn mixed_full_hp_and_damaged_cells_emits_only_for_damaged() {
    // Edge: one of the three is at full HP — only 2 messages emitted;
    // the full-HP timer still resets.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let damaged_a = spawn_cell(&mut app, 30.0, 100.0);
    let full = spawn_cell(&mut app, 100.0, 100.0);
    let damaged_b = spawn_cell(&mut app, 10.0, 50.0);
    attach_timer(&mut app, damaged_a, 0.05);
    attach_timer(&mut app, full, 0.05);
    attach_timer(&mut app, damaged_b, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 2);
    assert!(heals_for_cell(&app, full).is_empty());
    assert_eq!(heals_for_cell(&app, damaged_a).len(), 1);
    assert_eq!(heals_for_cell(&app, damaged_b).len(), 1);
    let full_timer = app.world().get::<RenewalTimer>(full).unwrap();
    assert!(
        (full_timer.remaining - 10.0).abs() < 1e-5,
        "full-HP cell timer should still reset, got {}",
        full_timer.remaining
    );
}

// ── Behavior 19 — Zero messages when no timer expired ────────────────────

#[test]
fn no_expiry_means_no_messages_and_all_timers_decrement() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, renewal_tick);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell_a = spawn_cell(&mut app, 30.0, 100.0);
    let cell_b = spawn_cell(&mut app, 50.0, 100.0);
    attach_timer(&mut app, cell_a, 5.0);
    attach_timer(&mut app, cell_b, 5.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 0);
    let t_a = app.world().get::<RenewalTimer>(cell_a).unwrap();
    let t_b = app.world().get::<RenewalTimer>(cell_b).unwrap();
    assert!((t_a.remaining - 4.9).abs() < 1e-5);
    assert!((t_b.remaining - 4.9).abs() < 1e-5);
}
