//! Group H — `register` wiring + run-condition gates (Behaviors 36–52).
//!
//! Pins that `register`-wired systems run under the correct schedules, gated
//! by `protocol_active(DebtCollector)` + `in_state(NodeState::Playing)` on the
//! three reader systems, that `debt_collector_attach_stack` is wired with the
//! `protocol_active` run-if ONLY (no `NodeState` gate), that cleanup runs on
//! `OnExit(NodeState::Playing)` unconditionally, and that the schedule is
//! harness-safe under missing resources + quiet ticks.

use bevy::prelude::NextState;

use super::{
    super::system::{DEBT_COLLECTOR_SENTINEL, DebtCashOut, DebtStack},
    helpers::{
        build_debt_collector_app, build_debt_collector_app_in_chip_selecting,
        build_debt_collector_app_no_config, collected_bonus_damage, install_debt_cash_out,
        seed_active_protocols_with_debt_collector, spawn_bolt_with_base_damage_and_cashout,
        spawn_bolt_with_stack, write_bolt_impact_cell, write_bolt_lost, write_bump_performed,
    },
};
use crate::{breaker::messages::BumpGrade, prelude::*};

// ── Behavior 36 — register wires on_bump gated on active + Playing ─────────-

#[test]
fn register_wires_on_bump_gated_on_active_and_playing() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 0.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Early);
    tick(&mut app);

    let stack = app.world().get::<DebtStack>(bolt).expect("stack retained");
    assert!(
        (stack.0 - 0.5).abs() < f32::EPSILON,
        "on_bump must run via register; stack expected 0.5, got {}",
        stack.0
    );
}

// ── Behavior 37 — on_bump gated off when DebtCollector NOT active ──────────-

#[test]
fn on_bump_gated_off_when_debt_collector_not_active() {
    let mut app = build_debt_collector_app();
    // Do NOT seed ActiveProtocols.
    let bolt = spawn_bolt_with_stack(&mut app, 0.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Early);
    tick(&mut app);

    let stack = app.world().get::<DebtStack>(bolt).expect("stack retained");
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "on_bump must not run when inactive; stack expected 0.0, got {}",
        stack.0
    );
}

// ── Behavior 38 — on_bump gated off when NodeState != Playing ──────────────-

#[test]
fn on_bump_gated_off_when_node_state_not_playing() {
    let mut app = build_debt_collector_app_in_chip_selecting();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 0.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Early);
    tick(&mut app);

    let stack = app.world().get::<DebtStack>(bolt).expect("stack retained");
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "on_bump must not run in ChipSelecting; stack expected 0.0, got {}",
        stack.0
    );
}

// ── Behavior 39 — register wires on_impact gated on active + Playing ───────-

#[test]
fn register_wires_on_impact_gated_on_active_and_playing() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_base_damage_and_cashout(&mut app, 10.0, 1.0);
    let cell = app.world_mut().spawn_empty().id();

    write_bolt_impact_cell(&mut app, bolt, cell);
    tick(&mut app);

    let bonuses = collected_bonus_damage(&app);
    assert_eq!(
        bonuses.len(),
        1,
        "one bonus expected, got {}",
        bonuses.len()
    );
    let msg = &bonuses[0];
    assert!(
        (msg.amount - 10.0).abs() < 1e-4,
        "amount expected 10.0 (= 10.0 × 1.0), got {}",
        msg.amount
    );
    assert_eq!(msg.source_chip.as_deref(), Some(DEBT_COLLECTOR_SENTINEL));
}

// ── Behavior 40 — on_impact gated off when NOT active ──────────────────────-

#[test]
fn on_impact_gated_off_when_debt_collector_not_active() {
    let mut app = build_debt_collector_app();
    // Do NOT seed ActiveProtocols.
    let bolt = spawn_bolt_with_base_damage_and_cashout(&mut app, 10.0, 1.5);
    let cell = app.world_mut().spawn_empty().id();

    write_bolt_impact_cell(&mut app, bolt, cell);
    tick(&mut app);

    assert!(
        collected_bonus_damage(&app).is_empty(),
        "no bonuses when DebtCollector inactive"
    );
    let cashout = app
        .world()
        .get::<DebtCashOut>(bolt)
        .expect("DebtCashOut must remain because on_impact did not run");
    assert!(
        (cashout.0 - 1.5).abs() < f32::EPSILON,
        "DebtCashOut must remain 1.5 when on_impact gated off, got {}",
        cashout.0
    );
}

// ── Behavior 41 — on_impact gated off when NodeState != Playing ────────────-

#[test]
fn on_impact_gated_off_when_node_state_not_playing() {
    let mut app = build_debt_collector_app_in_chip_selecting();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_base_damage_and_cashout(&mut app, 10.0, 1.5);
    let cell = app.world_mut().spawn_empty().id();

    write_bolt_impact_cell(&mut app, bolt, cell);
    tick(&mut app);

    assert!(
        collected_bonus_damage(&app).is_empty(),
        "no bonuses when NodeState != Playing"
    );
    let cashout = app
        .world()
        .get::<DebtCashOut>(bolt)
        .expect("DebtCashOut must remain");
    assert!(
        (cashout.0 - 1.5).abs() < f32::EPSILON,
        "DebtCashOut must remain 1.5 when gated off, got {}",
        cashout.0
    );
}

// ── Behavior 42 — register wires on_bolt_lost gated on active + Playing ────-

#[test]
fn register_wires_on_bolt_lost_gated_on_active_and_playing() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 1.0);
    install_debt_cash_out(&mut app, bolt, 0.5);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let stack = app.world().get::<DebtStack>(bolt).expect("stack retained");
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "on_bolt_lost must reset stack via register; got {}",
        stack.0
    );
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "DebtCashOut must be removed"
    );
}

// ── Behavior 43 — on_bolt_lost gated off when inactive ─────────────────────-

#[test]
fn on_bolt_lost_gated_off_when_debt_collector_not_active() {
    let mut app = build_debt_collector_app();
    // Do NOT seed ActiveProtocols.
    let bolt = spawn_bolt_with_stack(&mut app, 1.0);
    install_debt_cash_out(&mut app, bolt, 0.5);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let stack = app.world().get::<DebtStack>(bolt).expect("stack retained");
    assert!(
        (stack.0 - 1.0).abs() < f32::EPSILON,
        "DebtStack must remain 1.0 when gated off, got {}",
        stack.0
    );
    let cashout = app
        .world()
        .get::<DebtCashOut>(bolt)
        .expect("DebtCashOut must remain");
    assert!(
        (cashout.0 - 0.5).abs() < f32::EPSILON,
        "DebtCashOut must remain 0.5, got {}",
        cashout.0
    );
}

// ── Behavior 44 — on_bolt_lost gated off when NodeState != Playing ─────────-

#[test]
fn on_bolt_lost_gated_off_when_node_state_not_playing() {
    let mut app = build_debt_collector_app_in_chip_selecting();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 1.0);
    install_debt_cash_out(&mut app, bolt, 0.5);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let stack = app.world().get::<DebtStack>(bolt).expect("stack retained");
    assert!(
        (stack.0 - 1.0).abs() < f32::EPSILON,
        "DebtStack must remain 1.0 when ChipSelecting, got {}",
        stack.0
    );
    let cashout = app
        .world()
        .get::<DebtCashOut>(bolt)
        .expect("DebtCashOut must remain");
    assert!((cashout.0 - 0.5).abs() < f32::EPSILON);
}

// ── Behavior 45 — register wires attach_stack gated on active only ─────────-

#[test]
fn register_wires_attach_stack_gated_on_active_only() {
    let mut app = build_debt_collector_app_in_chip_selecting();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);

    let bolt = app.world_mut().spawn(Bolt).id();

    tick(&mut app);

    let stack = app.world().get::<DebtStack>(bolt).expect(
        "attach_stack MUST run outside NodeState::Playing (proves it is \
             wired in a separate add_systems call without the Playing gate)",
    );
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "DebtStack(0.0) expected from attach_stack in ChipSelecting, got {}",
        stack.0
    );
}

// ── Behavior 46 — attach_stack gated off when DebtCollector NOT active ─────-

#[test]
fn attach_stack_gated_off_when_debt_collector_not_active() {
    let mut app = build_debt_collector_app();
    // Do NOT seed ActiveProtocols.
    let bolt = app.world_mut().spawn(Bolt).id();

    tick(&mut app);

    assert!(
        app.world().get::<DebtStack>(bolt).is_none(),
        "attach_stack must NOT run when DebtCollector not active"
    );
}

// ── Behavior 47 — register wires cleanup_node unconditionally on OnExit ────-

#[test]
fn register_wires_cleanup_on_exit_node_state_playing() {
    let mut app = build_debt_collector_app();
    // Do NOT seed ActiveProtocols — cleanup must run regardless.
    let bolt = spawn_bolt_with_stack(&mut app, 1.5);
    install_debt_cash_out(&mut app, bolt, 0.5);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(
        app.world().get::<DebtStack>(bolt).is_none(),
        "cleanup must run on OnExit(Playing) even when protocol not active"
    );
    assert!(app.world().get::<DebtCashOut>(bolt).is_none());
}

// ── Behavior 48 — on_bump after BreakerSystems::GradeBump (same-tick) ──────-

#[test]
fn register_wires_on_bump_to_consume_bump_performed_same_tick() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 0.0);

    // Writer outside any specific set — at minimum, on_bump must read the
    // message in the SAME tick it was written (proving it runs downstream
    // of any prior writer in FixedUpdate).
    write_bump_performed(&mut app, Some(bolt), BumpGrade::Early);
    tick(&mut app);

    let stack = app.world().get::<DebtStack>(bolt).expect("stack retained");
    assert!(
        (stack.0 - 0.5).abs() < f32::EPSILON,
        "on_bump must consume same-tick BumpPerformed, got {}",
        stack.0
    );
}

// ── Behavior 49 — on_impact after BoltSystems::CellCollision (same-tick) ───-

#[test]
fn register_wires_on_impact_to_consume_bolt_impact_cell_same_tick() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_base_damage_and_cashout(&mut app, 10.0, 1.0);
    let cell = app.world_mut().spawn_empty().id();

    write_bolt_impact_cell(&mut app, bolt, cell);
    tick(&mut app);

    let bonuses = collected_bonus_damage(&app);
    assert_eq!(bonuses.len(), 1, "one bonus expected");
    assert!(
        (bonuses[0].amount - 10.0).abs() < 1e-4,
        "amount expected 10.0, got {}",
        bonuses[0].amount
    );
}

// ── Behavior 50 — on_bolt_lost after BoltSystems::BoltLost (same-tick) ─────-

#[test]
fn register_wires_on_bolt_lost_to_consume_bolt_lost_same_tick() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_stack(&mut app, 1.5);
    install_debt_cash_out(&mut app, bolt, 0.5);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let stack = app.world().get::<DebtStack>(bolt).expect("stack retained");
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "on_bolt_lost must reset stack to 0 same-tick, got {}",
        stack.0
    );
    assert!(app.world().get::<DebtCashOut>(bolt).is_none());
}

// ── Behavior 51 — register does not panic when resources absent ────────────-

#[test]
fn register_does_not_panic_when_config_absent() {
    let mut app = build_debt_collector_app_no_config();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);

    for _ in 0..3 {
        tick(&mut app);
    }

    // Resources remain absent; no bonuses generated.
    assert!(
        app.world()
            .get_resource::<super::super::system::DebtCollectorConfig>()
            .is_none(),
        "register must not side-effect-insert DebtCollectorConfig"
    );
    assert!(
        collected_bonus_damage(&app).is_empty(),
        "no bonuses emitted under quiet schedule with config absent"
    );
}

// ── Behavior 52 — schedule ticks cleanly with no messages and no bolts ─────-

#[test]
fn register_schedule_ticks_cleanly_with_no_messages_and_no_bolts() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);

    for _ in 0..3 {
        tick(&mut app);
    }

    assert!(
        collected_bonus_damage(&app).is_empty(),
        "quiet schedule must not produce bonus messages"
    );
}
