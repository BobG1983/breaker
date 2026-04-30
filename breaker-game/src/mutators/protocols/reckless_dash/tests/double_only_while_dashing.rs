//! Group B — End-to-end `BoltLost → reckless_dash_on_dash_transition →
//! handle_bolt_lost` chain (Behaviors 12–15).
//!
//! Proves that the `BoltLossBehavior` mutation produced by
//! `reckless_dash_on_dash_transition` flows through `handle_bolt_lost`
//! correctly: the live component value at message-processing time determines
//! the penalty applied.

use bevy::prelude::*;

use super::helpers::{
    build_reckless_dash_e2e_app, force_dash_state, seed_active_protocols_with_reckless_dash,
    spawn_breaker_for_e2e, write_bolt_lost,
};
use crate::{
    breaker::components::{BoltLossBehavior, DashState},
    prelude::*,
};

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_reckless_dash(app, 0.7, 4.0, true);
}

// ── Behavior 12 — BoltLost while Idle uses base LifeLoss(1) ─────────────────

#[test]
fn bolt_lost_while_idle_uses_base_lifeloss_one() {
    let mut app = build_reckless_dash_e2e_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_e2e(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        Some(Hp {
            current:  5.0,
            starting: 5.0,
            max:      None,
        }),
        DashState::Idle,
        DashState::Idle,
    );
    let bolt = app.world_mut().spawn_empty().id();

    // No DashState transition — breaker stays Idle.
    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker must have Hp")
        .current;
    assert!(
        (hp - 4.0).abs() < f32::EPSILON,
        "BoltLost while Idle must decrement Hp by 1 (LifeLoss(1)), got {hp}"
    );
}

// ── Behavior 13 — BoltLost during dash-enter tick uses doubled LifeLoss(2) ──

#[test]
fn bolt_lost_during_dash_enter_tick_uses_doubled_lifeloss_two() {
    let mut app = build_reckless_dash_e2e_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_e2e(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        Some(Hp {
            current:  5.0,
            starting: 5.0,
            max:      None,
        }),
        DashState::Idle,
        DashState::Idle,
    );
    let bolt = app.world_mut().spawn_empty().id();

    // Force dash enter AND write BoltLost in the same tick.
    // reckless_dash_on_dash_transition runs before handle_bolt_lost,
    // so the doubled BoltLossBehavior(LifeLoss(2)) is live when handle_bolt_lost reads.
    force_dash_state(&mut app, breaker, DashState::Dashing);
    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker must have Hp")
        .current;
    assert!(
        (hp - 3.0).abs() < f32::EPSILON,
        "BoltLost during dash-enter must decrement Hp by 2 (LifeLoss(2)), got {hp}"
    );

    // Overlay and live behavior confirm the transition happened.
    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(live, BoltLossBehavior::LifeLoss(2));
    assert!(
        app.world()
            .get::<super::super::system::OriginalBoltLossBehavior>(breaker)
            .is_some(),
        "OriginalBoltLossBehavior overlay must be present after dash enter"
    );
}

// ── Behavior 13 (edge case) — two BoltLost messages in dash-enter tick ───────

#[test]
fn two_bolt_lost_messages_during_dash_enter_tick_each_apply_doubled_penalty() {
    let mut app = build_reckless_dash_e2e_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_e2e(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        Some(Hp {
            current:  5.0,
            starting: 5.0,
            max:      None,
        }),
        DashState::Idle,
        DashState::Idle,
    );
    let bolt = app.world_mut().spawn_empty().id();

    force_dash_state(&mut app, breaker, DashState::Dashing);
    write_bolt_lost(&mut app, bolt, breaker);
    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker must have Hp")
        .current;
    assert!(
        (hp - 1.0).abs() < f32::EPSILON,
        "two BoltLost messages during dash-enter must each apply LifeLoss(2), Hp 5-2-2=1, got {hp}"
    );
}

// ── Behavior 14 — BoltLost after dash exit uses restored base behavior ────────

#[test]
fn bolt_lost_after_dash_exit_uses_restored_base_behavior() {
    let mut app = build_reckless_dash_e2e_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_e2e(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        Some(Hp {
            current:  5.0,
            starting: 5.0,
            max:      None,
        }),
        DashState::Idle,
        DashState::Idle,
    );
    let bolt = app.world_mut().spawn_empty().id();

    // Tick 1: enter dash (no BoltLost this tick).
    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    // Tick 2: exit dash + write BoltLost.
    force_dash_state(&mut app, breaker, DashState::Idle);
    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker must have Hp")
        .current;
    assert!(
        (hp - 4.0).abs() < f32::EPSILON,
        "BoltLost after dash exit must apply restored LifeLoss(1), Hp 5-1=4, got {hp}"
    );
    assert!(
        app.world()
            .get::<super::super::system::OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "OriginalBoltLossBehavior overlay must be removed after dash exit"
    );
}

// ── Behavior 14 (edge case) — enter-and-exit round-trip with concurrent BoltLost
//    each tick: Hp 5 - 2 (doubled on enter) - 1 (restored on exit) = 2 ────────

#[test]
fn bolt_lost_round_trip_doubled_on_enter_base_on_exit() {
    let mut app = build_reckless_dash_e2e_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_e2e(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        Some(Hp {
            current:  5.0,
            starting: 5.0,
            max:      None,
        }),
        DashState::Idle,
        DashState::Idle,
    );
    let bolt = app.world_mut().spawn_empty().id();

    // Tick 1: dash enter + BoltLost → Hp 5 - 2 = 3.
    force_dash_state(&mut app, breaker, DashState::Dashing);
    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp_after_tick1 = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker must have Hp")
        .current;
    assert!(
        (hp_after_tick1 - 3.0).abs() < f32::EPSILON,
        "after tick 1 (dash enter + BoltLost) Hp must be 3.0, got {hp_after_tick1}"
    );

    // Tick 2: dash exit + BoltLost → Hp 3 - 1 = 2.
    force_dash_state(&mut app, breaker, DashState::Idle);
    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp_after_tick2 = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker must have Hp")
        .current;
    assert!(
        (hp_after_tick2 - 2.0).abs() < f32::EPSILON,
        "after tick 2 (dash exit + BoltLost) Hp must be 2.0, got {hp_after_tick2}"
    );
}

// ── Behavior 14 (edge case) — Dashing → Braking exit restores base behavior ──

#[test]
fn bolt_lost_after_braking_exit_uses_restored_base_behavior() {
    let mut app = build_reckless_dash_e2e_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_e2e(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        Some(Hp {
            current:  5.0,
            starting: 5.0,
            max:      None,
        }),
        DashState::Idle,
        DashState::Idle,
    );
    let bolt = app.world_mut().spawn_empty().id();

    // Tick 1: enter dash + BoltLost → Hp 5 - 2 = 3.
    force_dash_state(&mut app, breaker, DashState::Dashing);
    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp_after_tick1 = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker must have Hp")
        .current;
    assert!(
        (hp_after_tick1 - 3.0).abs() < f32::EPSILON,
        "after tick 1 (enter + BoltLost) Hp must be 3.0, got {hp_after_tick1}"
    );

    // Tick 2: exit to Braking + BoltLost → Hp 3 - 1 = 2.
    force_dash_state(&mut app, breaker, DashState::Braking);
    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp_after_tick2 = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker must have Hp")
        .current;
    assert!(
        (hp_after_tick2 - 2.0).abs() < f32::EPSILON,
        "after Braking exit + BoltLost Hp must be 2.0 (restored LifeLoss(1)), got {hp_after_tick2}"
    );
    assert!(
        app.world()
            .get::<super::super::system::OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "OriginalBoltLossBehavior must be removed on Braking exit"
    );
}

// ── Behavior 15 — quiet ticks while Dashing keep live behavior doubled ────────

#[test]
fn quiet_ticks_while_dashing_keep_bolt_loss_behavior_doubled() {
    let mut app = build_reckless_dash_e2e_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_e2e(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        None,
        DashState::Idle,
        DashState::Idle,
    );

    // Enter dash.
    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    // 3 additional quiet ticks — no state mutation, no BoltLost.
    for _ in 0..3 {
        tick(&mut app);
    }

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(2),
        "live BoltLossBehavior must remain LifeLoss(2) after 3 quiet dashing ticks"
    );
    assert_eq!(
        app.world()
            .get::<super::super::system::OriginalBoltLossBehavior>(breaker)
            .map(|o| o.0),
        Some(BoltLossBehavior::LifeLoss(1)),
        "OriginalBoltLossBehavior overlay must remain present during quiet dashing ticks"
    );
}
