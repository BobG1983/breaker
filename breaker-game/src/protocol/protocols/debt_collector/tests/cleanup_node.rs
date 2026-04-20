//! Group G — `debt_collector_cleanup_node` (Behaviors 32–35).
//!
//! Pins the `OnExit(NodeState::Playing)` cleanup:
//! - Removes ALL `DebtStack` and `DebtCashOut` from every entity.
//! - Runs unconditionally — even when `DebtCollector` is NOT active.
//! - Tolerates absent `DebtCollectorConfig`.
//! - Re-entry to Playing re-attaches via `attach_stack` on the first tick.

use bevy::prelude::NextState;

use super::{
    super::system::{DebtCashOut, DebtStack},
    helpers::{
        build_debt_collector_app, build_debt_collector_app_no_config, install_debt_cash_out,
        seed_active_protocols_with_debt_collector, spawn_bolt_with_stack,
    },
};
use crate::prelude::*;

// ── Behavior 32 — exit from Playing removes all DebtStack/DebtCashOut ──────-

#[test]
fn exit_playing_removes_all_debt_stack_and_debt_cash_out() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);

    let a = spawn_bolt_with_stack(&mut app, 1.0);
    install_debt_cash_out(&mut app, a, 0.5);
    let b = spawn_bolt_with_stack(&mut app, 0.0);
    let c = app.world_mut().spawn(Bolt).id();
    install_debt_cash_out(&mut app, c, 2.0); // orphan cashout (no DebtStack)

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    for bolt in [a, b, c] {
        assert!(
            app.world().get::<DebtStack>(bolt).is_none(),
            "entity {bolt:?} should have no DebtStack after cleanup"
        );
        assert!(
            app.world().get::<DebtCashOut>(bolt).is_none(),
            "entity {bolt:?} should have no DebtCashOut after cleanup"
        );
    }

    // Edge case: idempotent — another transition does not panic.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();
}

// ── Behavior 33 — cleanup runs even when DebtCollector NOT active ──────────-

#[test]
fn cleanup_runs_even_when_debt_collector_not_active() {
    let mut app = build_debt_collector_app();
    // Intentionally do NOT seed ActiveProtocols.

    let bolt = spawn_bolt_with_stack(&mut app, 1.5);
    install_debt_cash_out(&mut app, bolt, 2.0);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(
        app.world().get::<DebtStack>(bolt).is_none(),
        "cleanup must fire even without DebtCollector active"
    );
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "cleanup must fire even without DebtCollector active"
    );
}

// ── Behavior 34 — cleanup tolerates absent config ──────────────────────────-

#[test]
fn cleanup_tolerates_absent_config() {
    let mut app = build_debt_collector_app_no_config();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);

    let bolt = spawn_bolt_with_stack(&mut app, 1.0);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update(); // must not panic

    assert!(
        app.world().get::<DebtStack>(bolt).is_none(),
        "cleanup removes DebtStack even without config"
    );
    assert!(app.world().get::<DebtCashOut>(bolt).is_none());
}

// ── Behavior 35 — re-entry to Playing re-attaches via attach_stack ─────────-

#[test]
fn re_entry_to_playing_re_attaches_debt_stack() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);

    let bolt = spawn_bolt_with_stack(&mut app, 1.5);
    install_debt_cash_out(&mut app, bolt, 0.5);

    // Exit Playing → cleanup.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();
    assert!(app.world().get::<DebtStack>(bolt).is_none());
    assert!(app.world().get::<DebtCashOut>(bolt).is_none());

    // Re-enter Playing and tick once — attach_stack must re-install DebtStack(0.0).
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();
    tick(&mut app);

    let stack = app
        .world()
        .get::<DebtStack>(bolt)
        .expect("attach_stack should re-install DebtStack after re-entry");
    assert!(
        (stack.0 - 0.0).abs() < f32::EPSILON,
        "re-attached DebtStack should be 0.0, got {}",
        stack.0
    );
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "re-entry must not re-populate DebtCashOut"
    );
}
