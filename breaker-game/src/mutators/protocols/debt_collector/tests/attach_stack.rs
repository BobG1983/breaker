//! Group F — `debt_collector_attach_stack` (Behaviors 28–31).
//!
//! Pins the attach system:
//! - Inserts `DebtStack(0.0)` on bolts missing it; preserves existing stacks.
//! - Runs even outside `NodeState::Playing` (no `NodeState` gate).
//! - Gated off when `DebtCollector` is not active.
//! - Tolerates absent `DebtCollectorConfig`.

use super::{
    super::system::DebtStack,
    helpers::{
        build_debt_collector_app, build_debt_collector_app_in_chip_selecting,
        build_debt_collector_app_no_config, seed_active_protocols_with_debt_collector,
    },
};
use crate::prelude::*;

// ── Behavior 28 — attach inserts DebtStack(0.0) on bolts missing it ────────-

#[test]
fn attach_inserts_debt_stack_on_bolts_missing_it() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);

    // Bolt A without DebtStack; bolt B with DebtStack(0.7).
    let a = app.world_mut().spawn(Bolt).id();
    let b = app.world_mut().spawn((Bolt, DebtStack(0.7))).id();

    tick(&mut app);

    let stack_a = app
        .world()
        .get::<DebtStack>(a)
        .expect("bolt A should have DebtStack(0.0) attached");
    assert!(
        (stack_a.0 - 0.0).abs() < f32::EPSILON,
        "bolt A DebtStack expected 0.0, got {}",
        stack_a.0
    );
    let stack_b = app
        .world()
        .get::<DebtStack>(b)
        .expect("bolt B should still have DebtStack");
    assert!(
        (stack_b.0 - 0.7).abs() < f32::EPSILON,
        "bolt B DebtStack must be preserved at 0.7 (NOT overwritten), got {}",
        stack_b.0
    );

    // Edge case: second tick with no new bolts is a no-op — no duplicate
    // insert panic, no value change.
    tick(&mut app);
    let stack_a = app.world().get::<DebtStack>(a).expect("stack retained");
    let stack_b = app.world().get::<DebtStack>(b).expect("stack retained");
    assert!((stack_a.0 - 0.0).abs() < f32::EPSILON);
    assert!((stack_b.0 - 0.7).abs() < f32::EPSILON);
}

// ── Behavior 29 — attach runs even when NodeState is not Playing ───────────-

#[test]
fn attach_runs_even_when_node_state_is_not_playing() {
    let mut app = build_debt_collector_app_in_chip_selecting();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);

    let bolt = app.world_mut().spawn(Bolt).id();

    tick(&mut app);

    let stack = app
        .world()
        .get::<DebtStack>(bolt)
        .expect("attach must run even in ChipSelecting state");
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "DebtStack(0.0) expected, got {}",
        stack.0
    );
}

// ── Behavior 30 — attach does NOT run when DebtCollector is not active ─────-

#[test]
fn attach_does_not_run_when_debt_collector_not_active() {
    let mut app = build_debt_collector_app();
    // Intentionally do NOT seed ActiveProtocols.

    let bolt = app.world_mut().spawn(Bolt).id();

    tick(&mut app);

    assert!(
        app.world().get::<DebtStack>(bolt).is_none(),
        "attach must NOT run when DebtCollector is not in ActiveProtocols"
    );
}

// ── Behavior 31 — attach tolerates absent config ───────────────────────────-

#[test]
fn attach_tolerates_absent_config_and_still_inserts_debt_stack() {
    let mut app = build_debt_collector_app_no_config();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);

    let bolt = app.world_mut().spawn(Bolt).id();

    tick(&mut app); // must not panic

    let stack = app
        .world()
        .get::<DebtStack>(bolt)
        .expect("attach must still insert DebtStack(0.0) without config");
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "DebtStack expected 0.0, got {}",
        stack.0
    );
}
